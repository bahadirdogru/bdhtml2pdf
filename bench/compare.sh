#!/usr/bin/env bash
# Compares Blitz (bdhtml2pdf) with OxiBrowser on the same fixtures.
# Measured: time, peak RSS, and the expected content in the DOM after JS runs.
#
# Usage: bench/compare.sh <bdhtml2pdf-binary> <oxibrowser-binary> [output-dir]
#
# OxiBrowser notes:
# - It does not open file://, so the fixtures are served over HTTP on 127.0.0.1.
# - It does not resolve relative <script src> (the page shows up as about:page),
#   so OxiBrowser gets a copy with script addresses rewritten to absolute URLs.
# - `fetch --format html` returns the raw response, not the DOM after JS runs,
#   so the live DOM is read with `--eval` instead.
set -u

BD=${1:?bdhtml2pdf binary}
OXI=${2:?oxibrowser binary}
OUT=${3:-$(mktemp -d)}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
FIX=$ROOT/tests/fixtures
PORT=8765
mkdir -p "$OUT"

# Copy with absolute script URLs for OxiBrowser
ABS=$OUT/fixtures-abs
rm -rf "$ABS" && cp -r "$FIX" "$ABS"
sed -i -E "s#src=\"(\./)?vendor/#src=\"http://127.0.0.1:$PORT/vendor/#g" "$ABS"/*.html
sed -i -E "s#src=\"(\./)?vendor/#src=\"http://127.0.0.1:$PORT/preact/vendor/#g" "$ABS"/preact/*.html

python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$ABS" >/dev/null 2>&1 &
SRV=$!
trap 'kill $SRV 2>/dev/null' EXIT
sleep 0.5

FIXTURES=(modern-js.html react.html vue.html preact/index.html)

check() { # $1=fixture $2=html file
  local f=$1 h=$2
  case $f in
    modern-js.html) echo "$(grep -o '<li>PASS' "$h" | wc -l)/19 PASS; FAIL: $(grep -o '<li>FAIL[^<]*' "$h" | sed 's/<li>FAIL //' | paste -sd ';')" ;;
    react.html) printf '%s, %s rows' "$(grep -q 'Durum: ÖDENDİ' "$h" && echo 'useEffect+timer OK' || echo 'NO timer update')" "$(grep -o '<tr' "$h" | wc -l)" ;;
    vue.html) printf '%s, %s li' "$(grep -q 'güncellendi' "$h" && echo 'mounted+timer OK' || echo 'NO timer update')" "$(grep -o '<li' "$h" | wc -l)" ;;
    preact/index.html) grep -q '>todos<' "$h" && echo "render OK" || echo "NOT rendered" ;;
  esac
}

measure() { # command... → "time rss"
  /usr/bin/time -f '%e %M' -o "$OUT/.time" "$@" >/dev/null 2>&1
  tail -1 "$OUT/.time" | awk '{printf "%5.2fs %6.1fMB", $1, $2/1024}'
}

# In OxiBrowser document.documentElement.outerHTML / body.innerHTML are
# `undefined`, so the DOM is serialized by walking it with basic APIs
# (childNodes, tagName, textContent).
OXI_SERIALIZE='(function s(n){if(n.nodeType==3)return n.textContent;if(n.nodeType!=1)return "";var t=n.tagName.toLowerCase();if(t=="script"||t=="style")return "";var a=n.id?" id=\""+n.id+"\"":"";return "<"+t+a+">"+Array.from(n.childNodes).map(s).join("")+"</"+t+">";})(document.documentElement)'

oxi_dom() { # $1=fixture $2=output
  "$OXI" fetch "http://127.0.0.1:$PORT/$1" --allow-private-ips --json --timeout 10 \
    --eval "$OXI_SERIALIZE" 2>"$2.stderr" \
    | python3 -c 'import json,sys; d=json.load(sys.stdin); print(d.get("data",{}).get("value") or d.get("error",""))' > "$2"
}
export -f oxi_dom; export OXI PORT OXI_SERIALIZE
export LC_NUMERIC=C

printf '%-10s %-18s %-16s %s\n' Engine Fixture "Time / RSS" Result
printf '%s\n' "--------------------------------------------------------------------------------"
for f in "${FIXTURES[@]}"; do
  name=$(echo "$f" | tr / _)

  m=$(measure "$BD" -q --window-status ready --javascript-delay 1000 --javascript-timeout 10 "$FIX/$f" "$OUT/blitz-$name")
  printf '%-10s %-18s %-16s %s\n' blitz "$f" "$m" "$(check "$f" "$OUT/blitz-$name")"

  m=$(measure bash -c "oxi_dom '$f' '$OUT/oxi-$name'")
  printf '%-10s %-18s %-16s %s\n' oxibrowser "$f" "$m" "$(check "$f" "$OUT/oxi-$name")"
done

echo
for b in "$BD" "$OXI"; do printf 'binary %-12s %6.1f MB\n' "$(basename "$b")" "$(stat -c %s "$b" | awk '{print $1/1048576}')"; done
echo "outputs: $OUT"
