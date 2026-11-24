#!/usr/bin/env bash
# Release benchmark: bdhtml2pdf against other HTML→PDF engines on the same fixtures.
#
# Usage:
#   bench/run.sh [-n RUNS] [-o OUT_DIR]
# Engines other than bdhtml2pdf are optional and taken from the environment:
#   SGH=/path/to/sghtmltopdf  WK=/path/to/wkhtmltopdf  CHROME=chromium-browser
#   WEASY=weasyprint   TIME=/path/to/gnu-time (default /usr/bin/time)
#
# Every engine run is a transient systemd user service, so wall time
# ("Service runtime") and peak memory ("Memory peak", the cgroup high-water
# mark that includes every child process) are measured the same way for
# single-process tools and for multi-process Chromium.
#
# Needs: systemd (user instance), poppler-utils (pdftotext, pdfinfo, pdffonts, pdfimages).
# Output: a Markdown report on stdout; PDFs and logs in OUT_DIR.
set -uo pipefail
export LC_ALL=C.UTF-8

ROOT=$(cd "$(dirname "$0")/.." && pwd)
FIX=$ROOT/tests/fixtures
BD=${BD:-$ROOT/target/release/bdhtml2pdf}
RUNS=5
OUT=
while getopts "n:o:" o; do case $o in n) RUNS=$OPTARG ;; o) OUT=$OPTARG ;; *) exit 2 ;; esac; done
OUT=${OUT:-$(mktemp -d)}
mkdir -p "$OUT/pdf" "$OUT/gen"
[[ -x $BD ]] || { echo "build first: cargo build --release" >&2; exit 1; }

# ---------------------------------------------------------------- engines
ENGINES=(bdhtml2pdf)
[[ -n ${SGH:-} ]] && ENGINES+=(sghtmltopdf)
[[ -n ${WK:-} ]] && ENGINES+=(wkhtmltopdf)
[[ -n ${CHROME:-} ]] && ENGINES+=(chromium)
[[ -n ${WEASY:-} ]] && ENGINES+=(weasyprint)

# render ENGINE INPUT OUTPUT → prints the command line to run
cmdline() {
  local e=$1 in=$2 out=$3
  case $e in
    bdhtml2pdf)  echo "$BD" -q "$in" "$out" ;;
    sghtmltopdf) echo "$SGH" "$in" --allow-path "$FIX" --allow-path "$OUT/gen" -o "$out" ;;
    wkhtmltopdf) echo "$WK" -q --enable-local-file-access "$in" "$out" ;;
    chromium)    echo "$CHROME" --headless --disable-gpu --user-data-dir="$OUT/chrome-profile" \
                   --no-first-run --no-default-browser-check --password-store=basic \
                   --use-mock-keychain --disable-extensions --disable-background-networking \
                   --no-pdf-header-footer --allow-file-access-from-files \
                   --virtual-time-budget=1000 --print-to-pdf="$out" "file://$in" ;;
    weasyprint)  echo "$WEASY" -q "$in" "$out" ;;
  esac
}

version() {
  case $1 in
    bdhtml2pdf)  "$BD" --version ;;
    sghtmltopdf) "$SGH" --version 2>&1 | head -1 ;;
    wkhtmltopdf) "$WK" --version ;;
    chromium)    "$CHROME" --version ;;
    weasyprint)  "$WEASY" --version ;;
  esac
}

# run_once ENGINE INPUT OUTPUT → "seconds cgroup_peak_MB max_rss_MB exit"
# cgroup peak: memory charged to the whole process tree (anonymous memory plus
# page cache it caused; binary pages already cached are not charged).
# max RSS: resident set of the largest single process (GNU time %M), which
# counts shared pages too and, for Chromium, only its biggest process.
run_once() {
  local log=$OUT/.run.log
  # shellcheck disable=SC2046
  systemd-run --user --wait --pipe --collect --same-dir -p MemoryMax=4G -p RuntimeMaxSec=120 \
    -E DBUS_SESSION_BUS_ADDRESS=disabled: \
    "${TIME:-/usr/bin/time}" -f '%M' -o "$OUT/.rss" $(cmdline "$@") >"$log" 2>&1 </dev/null
  local rt mp st rss
  rt=$(sed -n 's/.*Service runtime: //p' "$log" | tail -1)
  mp=$(sed -n 's/.*Memory peak: \([^ ]*\).*/\1/p' "$log" | tail -1)
  st=$(sed -n 's/.*status=\([0-9]*\).*/\1/p' "$log" | tail -1)
  rss=$(tail -1 "$OUT/.rss" 2>/dev/null | awk '{printf "%.1f", $1/1024}')
  printf '%s %s %s %s\n' "$(to_seconds "$rt")" "$(to_mb "$mp")" "${rss:-0}" "${st:-?}"
}
to_seconds() { # "1min 2.5s" | "1.021s" | "593ms"
  awk -v s="$1" 'BEGIN{t=0; n=split(s,p," "); for(i=1;i<=n;i++){v=p[i];
    if(v~/min$/){sub(/min$/,"",v);t+=v*60} else if(v~/ms$/){sub(/ms$/,"",v);t+=v/1000}
    else if(v~/s$/){sub(/s$/,"",v);t+=v}} printf "%.3f", t}'
}
to_mb() { # "672.3M" | "1.2G" | "512K" | "800B"
  awk -v s="$1" 'BEGIN{u=substr(s,length(s)); v=substr(s,1,length(s)-1)+0;
    m=(u=="G")?v*1024:(u=="M")?v:(u=="K")?v/1024:(u=="B")?v/1048576:0; printf "%.1f", m}'
}
median() { sort -n | awk '{a[NR]=$1} END{if(NR==0){print "-";exit} printf "%.3f", (NR%2)?a[(NR+1)/2]:(a[NR/2]+a[NR/2+1])/2}'; }

# ---------------------------------------------------------------- generated fixture
# Long invoice: 1000 rows, repeated table header, totals. Stresses pagination.
gen_invoice() {
  {
    echo '<!DOCTYPE html><html lang="tr"><head><meta charset="utf-8"><title>Fatura 2026-0001</title>'
    echo '<style>body{font-family:sans-serif;font-size:10pt}table{width:100%;border-collapse:collapse}'
    echo 'th,td{border-bottom:1px solid #ccc;padding:2px 4px;text-align:right}th{background:#eee}'
    echo 'thead{display:table-header-group}td:first-child,th:first-child{text-align:left}</style></head><body>'
    echo '<h1>Fatura No: 2026-0001</h1><p>Sayın Müşterimiz, Ağustos dönemi hizmet bedeli aşağıdadır.</p>'
    echo '<table><thead><tr><th>Açıklama</th><th>Adet</th><th>Birim</th><th>Tutar</th></tr></thead><tbody>'
    local i total=0
    for ((i = 1; i <= 1000; i++)); do
      total=$((total + (i % 7 + 1) * 125))
      printf '<tr><td>Satır %d — eğitim modülü</td><td>%d</td><td>1,25 ₺</td><td>%d,%02d ₺</td></tr>\n' \
        "$i" $((i % 7 + 1)) $(((i % 7 + 1) * 125 / 100)) $(((i % 7 + 1) * 125 % 100))
    done
    printf '</tbody></table><p id="total">GENEL TOPLAM: %d,%02d ₺</p></body></html>\n' $((total / 100)) $((total % 100))
  } >"$OUT/gen/invoice-large.html"
}
gen_invoice

# ---------------------------------------------------------------- checks
# Each check prints "passed/total" and the names of failed items.
txt() { pdftotext -enc UTF-8 "$1" - 2>/dev/null | tr -s ' \n\t' '   '; }
has() { grep -qF -- "$2" <<<"$1"; }
count_pf() { # text → "pass fail"; an item passes only when its PASS line is shown and its FAIL line is not
  local p f split
  split=$(split_pf "$1")
  p=$(grep -oE '^PASS [^ ]*' <<<"$split" | sed 's/PASS //' | sort -u)
  f=$(grep -oE '^FAIL [^ ]*' <<<"$split" | sed 's/FAIL //' | sort -u)
  printf '%s %s' "$(comm -23 <(echo "$p") <(echo "$f") | grep -c .)" "$(grep -c . <<<"$f")"
}
# Engines sometimes run two items together ("PASS xFAIL x"); start each item on its own line.
split_pf() { sed -E 's/(PASS|FAIL) /\n\1 /g' <<<"$1"; }
fails() { split_pf "$1" | grep -oE '^FAIL [^ ]*' | sed 's/FAIL //' | sort -u | paste -sd, -; }

check() { # FIXTURE PDF → "score|notes"
  local f=$1 pdf=$2 t
  [[ -s $pdf ]] || { echo "no PDF|"; return; }
  t=$(txt "$pdf")
  case $f in
    modern-css.html)   read -r p _ < <(count_pf "$t"); echo "$p/11|$(fails "$t")" ;;
    modern-js.html)    read -r p _ < <(count_pf "$t"); echo "$p/19|$(fails "$t")" ;;
    html-parsing.html)
      read -r p _ < <(count_pf "$t")
      has "$t" 'H7-entities: & < ö ı € ©' && p=$((p + 1))
      echo "$p/7|$(fails "$t")" ;;
    turkish.html)
      local ok=0 bad=() fonts
      fonts=$(pdffonts "$pdf" 2>/dev/null)
      local -A exp=(
        [alphabet]='T1-alfabe: Ç ç Ğ ğ I ı İ i Ö ö Ş ş Ü ü'
        [font-face]='T2-fontface: Ç ç Ğ ğ I ı İ i Ö ö Ş ş Ü ü'
        [bold]='T3-kalın: ÇĞİÖŞÜ çğıöşü'
        [italic]='T3-italik: ÇĞİÖŞÜ çğıöşü'
        [pangram]='Pijamalı hasta yağız şoföre çabucak güvendi.'
        [upper-tr]='İSTANBUL ILIK İĞDIR'
        [lower-tr]='istanbul ışık ığdır'
        [capitalize-tr]='İzmir Ispanak'
        [invoice-marks]='Tutar 1.234,50 ₺ — KDV %20 “dâhil”'
      )
      for k in alphabet font-face bold italic pangram upper-tr lower-tr capitalize-tr invoice-marks; do
        if has "$t" "${exp[$k]}"; then ok=$((ok + 1)); else bad+=("$k"); fi
      done
      # WeasyPrint names an embedded @font-face by its CSS family, the others by the font's own name.
      if grep -qE 'Comfortaa|\+Brand ' <<<"$fonts"; then ok=$((ok + 1)); else bad+=(font-face-embedded); fi
      if pdfinfo "$pdf" 2>/dev/null | grep -qF 'ÇĞİÖŞÜ çğıöşü'; then ok=$((ok + 1)); else bad+=(pdf-title); fi
      read -r p _ < <(count_pf "$t")
      ok=$((ok + p)); [[ -n $(fails "$t") ]] && bad+=("$(fails "$t")")
      grep -q 'tr-' <<<"$t" || bad+=(js-not-run)
      echo "$ok/15|$(IFS=,; echo "${bad[*]}")" ;;
    fonts.html)
      local ok=0 bad=() fonts
      fonts=$(pdffonts "$pdf" 2>/dev/null)
      if grep -qE 'Comfortaa|\+BrandOTF ' <<<"$fonts"; then ok=$((ok + 1)); else bad+=(otf); fi
      # The body font is serif, so a Noto *Sans* face can only come from the WOFF2 rule.
      if grep -qE '\+NotoSans-Regular |\+BrandWOFF2 ' <<<"$fonts"; then ok=$((ok + 1)); else bad+=(woff2); fi
      if has "$t" 'F3-mono: generic monospace 0123456789'; then ok=$((ok + 1)); else bad+=(mono); fi
      if has "$t" '日本語のテキスト'; then ok=$((ok + 1)); else bad+=(cjk); fi
      if has "$t" 'مرحبا' || has "$t" 'ابحرم'; then ok=$((ok + 1)); else bad+=(arabic); fi
      if has "$t" 'office affluent'; then ok=$((ok + 1)); else bad+=(ligature-text); fi
      if has "$t" '€ ₺ © ® ™ ½ → ✓'; then ok=$((ok + 1)); else bad+=(symbols); fi
      if ! tail -n +3 <<<"$fonts" | awk '$(NF-4)!="yes"{bad=1} END{exit !bad}'; then ok=$((ok + 1)); else bad+=(not-all-embedded); fi
      echo "$ok/8|$(IFS=,; echo "${bad[*]}")" ;;
    react.html)        has "$t" 'Durum: ÖDENDİ' && echo "1/1|" || echo "0/1|not rendered" ;;
    vue.html)          has "$t" 'güncellendi' && echo "1/1|" || echo "0/1|not rendered" ;;
    preact/index.html) has "$t" 'todos' && echo "1/1|" || echo "0/1|not rendered" ;;
    invoice-large.html)
      local ok=0 bad=()
      has "$t" 'Satır 1000 — eğitim modülü' && ok=$((ok + 1)) || bad+=(last-row)
      has "$t" 'GENEL TOPLAM:' && ok=$((ok + 1)) || bad+=(total)
      # header repeated on every page
      local pages hdr
      pages=$(pdfinfo "$pdf" 2>/dev/null | awk '/^Pages/{print $2}')
      hdr=$(pdftotext -enc UTF-8 "$pdf" - 2>/dev/null | grep -c 'Açıklama')
      [[ ${pages:-0} -gt 1 && $hdr -ge $pages ]] && ok=$((ok + 1)) || bad+=("thead-repeat $hdr/$pages")
      echo "$ok/3|$(IFS=,; echo "${bad[*]}")" ;;
  esac
}

pdf_facts() { # PDF → "pages size_KB images fonts(subset/embedded) page_size"
  local pdf=$1
  [[ -s $pdf ]] || { echo "- - - - -"; return; }
  local pages kb imgs fonts ps
  pages=$(pdfinfo "$pdf" 2>/dev/null | awk '/^Pages/{print $2}')
  ps=$(pdfinfo "$pdf" 2>/dev/null | awk '/^Page size/{printf "%.0fx%.0f", $3, $5}')
  kb=$(awk -v b="$(stat -c %s "$pdf")" 'BEGIN{printf "%.0f", b/1024}')
  imgs=$(pdfimages -list "$pdf" 2>/dev/null | tail -n +3 | wc -l)
  fonts=$(pdffonts "$pdf" 2>/dev/null | tail -n +3 | awk '{n++; if($(NF-4)=="yes")e++; if($(NF-3)=="yes")s++} END{printf "%d/%d/%d", n, e, s}')
  echo "$pages $kb $imgs $fonts $ps"
}

# ---------------------------------------------------------------- run
FIXTURES=(modern-css.html modern-js.html html-parsing.html turkish.html fonts.html
          react.html vue.html preact/index.html invoice-large.html)
fixture_path() { [[ $1 == invoice-large.html ]] && echo "$OUT/gen/$1" || echo "$FIX/$1"; }

echo "## Environment"
echo
echo "- Date: $(date -u +%Y-%m-%dT%H:%MZ)"
echo "- CPU: $(grep -m1 'model name' /proc/cpuinfo | cut -d: -f2- | sed 's/^ //'), $(nproc) threads"
echo "- RAM: $(free -g | awk '/^Mem/{print $2}') GB; OS: $(. /etc/os-release; echo "$PRETTY_NAME"), kernel $(uname -r)"
echo "- Runs per fixture: $RUNS (median time, maximum memory; one warm-up run discarded)"
install_size() { # size of what has to be shipped to run the engine
  local bin; bin=$(readlink -f "$(command -v "$(cmdline "$1" x y | awk '{print $1}')")")
  case $1 in
    # Debian's and Fedora's launchers are scripts in /usr/bin; count the
    # directory the browser itself is installed in
    chromium)
      local d
      for d in /usr/lib/chromium /usr/lib64/chromium-browser /usr/lib/chromium-browser "$(dirname "$bin")"; do
        [[ -x $d/chromium || -x $d/chromium-browser || -x $d/chrome || $d == "$(dirname "$bin")" ]] && break
      done
      du -shL "$d" | cut -f1 ;;
    # a Python virtualenv (pipx); Python itself and Pango are not counted
    weasyprint) du -shL "$(dirname "$(dirname "$bin")")" | cut -f1 ;;
    *) du -shL "$bin" | cut -f1 ;;
  esac
}
for e in "${ENGINES[@]}"; do echo "- $e: $(version "$e") — install size $(install_size "$e")"; done
echo

declare -A RES
for f in "${FIXTURES[@]}"; do
  in=$(fixture_path "$f")
  for e in "${ENGINES[@]}"; do
    pdf=$OUT/pdf/$e-${f//\//_}.pdf
    rm -f "$pdf"
    run_once "$e" "$in" "$pdf" >/dev/null # warm-up (font caches, profile)
    times=() mems=() rsss=() st=
    for ((r = 0; r < RUNS; r++)); do
      rm -f "$pdf"
      read -r s m x st < <(run_once "$e" "$in" "$pdf")
      times+=("$s"); mems+=("$m"); rsss+=("$x")
    done
    t=$(printf '%s\n' "${times[@]}" | median)
    m=$(printf '%s\n' "${mems[@]}" | sort -n | tail -1)
    x=$(printf '%s\n' "${rsss[@]}" | sort -n | tail -1)
    RES[$e,$f]="$t|$m/$x|$st|$(check "$f" "$pdf")|$(pdf_facts "$pdf")"
  done
done

echo "## Correctness"
echo
printf '| Fixture |'; for e in "${ENGINES[@]}"; do printf ' %s |' "$e"; done; echo
printf '|---|'; for _ in "${ENGINES[@]}"; do printf '%s' '---|'; done; echo
for f in "${FIXTURES[@]}"; do
  printf '| %s |' "$f"
  for e in "${ENGINES[@]}"; do
    IFS='|' read -r _ _ _ score notes _ <<<"${RES[$e,$f]}"
    [[ -n $notes ]] && printf ' %s <br><sub>%s</sub> |' "$score" "$notes" || printf ' %s |' "$score"
  done
  echo
done
echo

echo "## Time and memory"
echo
echo "Median wall time in seconds · memory in MB as cgroup peak (whole process tree) / max RSS (largest process)."
echo
printf '| Fixture |'; for e in "${ENGINES[@]}"; do printf ' %s |' "$e"; done; echo
printf '|---|'; for _ in "${ENGINES[@]}"; do printf '%s' '---:|'; done; echo
for f in "${FIXTURES[@]}"; do
  printf '| %s |' "$f"
  for e in "${ENGINES[@]}"; do
    IFS='|' read -r t m _ <<<"${RES[$e,$f]}"
    printf ' %s s · %s MB |' "$t" "$m"
  done
  echo
done
echo

echo "## PDF output"
echo
echo "pages · size KB · raster images · fonts total/embedded/subset · page size (pt)"
echo
printf '| Fixture |'; for e in "${ENGINES[@]}"; do printf ' %s |' "$e"; done; echo
printf '|---|'; for _ in "${ENGINES[@]}"; do printf '%s' '---|'; done; echo
for f in "${FIXTURES[@]}"; do
  printf '| %s |' "$f"
  for e in "${ENGINES[@]}"; do
    IFS='|' read -r _ _ _ _ _ facts <<<"${RES[$e,$f]}"
    read -r pg kb im fo ps <<<"$facts"
    printf ' %s · %s · %s · %s · %s |' "$pg" "$kb" "$im" "$fo" "$ps"
  done
  echo
done
echo
echo "PDFs: $OUT/pdf" >&2
