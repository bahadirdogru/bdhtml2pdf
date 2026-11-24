#!/usr/bin/env bash
# bdhtml2pdf vs sghtmltopdf (waka/sghtmltopdf, crates.io) on the same fixtures.
#
# Usage: bench/compare-sgh.sh [path/to/sghtmltopdf]
#   Build sghtmltopdf with: cargo install sghtmltopdf --locked --root <dir>
# Needs: pdftotext, pdfinfo (poppler-utils), GNU time.
set -euo pipefail
export LC_NUMERIC=C

ROOT=$(cd "$(dirname "$0")/.." && pwd)
FIX=$ROOT/tests/fixtures
BD=$ROOT/target/release/bdhtml2pdf
SGH=${1:-sghtmltopdf}
OUT=$(mktemp -d)
trap 'rm -rf "$OUT"' EXIT

[[ -x $BD ]] || { echo "build first: cargo build --release" >&2; exit 1; }
command -v "$SGH" >/dev/null || [[ -x $SGH ]] || { echo "sghtmltopdf not found: $SGH" >&2; exit 1; }

# fixture | marker text the JavaScript is expected to produce (empty: none)
CASES=(
  "modern-css.html|"
  "modern-js.html|"
  "react.html|ÖDENDİ"
  "vue.html|güncellendi"
  "preact/index.html|todos"
)

printf '%-12s %-18s %8s %9s %6s %6s %6s %8s\n' tool fixture time_s rss_MB pages PASS FAIL marker
run() { # tool name cmd...
  local tool=$1 name=$2 marker=$3; shift 3
  local pdf=$OUT/$tool-${name//\//_}.pdf
  /usr/bin/time -o "$OUT/time" -f '%e %M' "$@" "$pdf" >/dev/null 2>&1 || true
  local t; t=$(tail -1 "$OUT/time")
  local txt=$OUT/$tool-${name//\//_}.txt
  pdftotext -layout "$pdf" "$txt" 2>/dev/null || : > "$txt"
  local pages; pages=$(pdfinfo "$pdf" 2>/dev/null | awk '/^Pages/{print $2}')
  local pass fail mk="-"
  pass=$(grep -o 'PASS' "$txt" | wc -l || true); fail=$(grep -o 'FAIL' "$txt" | wc -l || true)
  [[ -n $marker ]] && { grep -q "$marker" "$txt" && mk=yes || mk=NO; }
  printf '%-12s %-18s %8.2f %9.1f %6s %6s %6s %8s\n' "$tool" "$name" \
    "${t%% *}" "$(awk "BEGIN{print ${t##* }/1024}")" "${pages:-0}" "$pass" "$fail" "$mk"
}

for c in "${CASES[@]}"; do
  f=${c%%|*}; m=${c#*|}
  run bdhtml2pdf  "$f" "$m" "$BD" -q "$FIX/$f"
  run sghtmltopdf "$f" "$m" "$SGH" "$FIX/$f" --allow-path "$FIX" -o
done

echo
printf 'binary size: bdhtml2pdf %.1f MB, sghtmltopdf %.1f MB\n' \
  "$(awk "BEGIN{print $(stat -c %s "$BD")/1048576}")" \
  "$(awk "BEGIN{print $(stat -c %s "$(command -v "$SGH" || echo "$SGH")")/1048576}")"
