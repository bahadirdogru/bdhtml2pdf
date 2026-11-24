#!/usr/bin/env bash
# Security suite: runs every HTML→PDF engine on hostile pages (tests/security/)
# with its *default* settings and reports what leaked or hung.
#
# Usage:
#   bench/security.sh [-t SECONDS] [-o OUT_DIR]
# Engines other than bdhtml2pdf are optional, from the environment:
#   SGH=/path/to/sghtmltopdf  WK=/path/to/wkhtmltopdf  CHROME=chromium-browser
#   WEASY=weasyprint
#
# How a leak is detected:
#   * file reads   — the PDF text contains a line of /etc/passwd ("root:x:0:0") or
#                    a LEAKED-* marker from a file placed *outside* the input directory;
#   * SSRF         — a throwaway HTTP server on 127.0.0.1 (standing in for an internal
#                    service or a cloud metadata endpoint) logs a request;
#   * DoS          — the engine is still running when the time limit hits, or the
#                    memory limit kills it.
# Each run is a transient systemd user service with MemoryMax=2G and
# RuntimeMaxSec=$LIMIT, so a hostile page cannot take the machine down.
#
# Needs: systemd (user instance), python3 (http.server), poppler-utils.
set -uo pipefail
export LC_ALL=C.UTF-8

ROOT=$(cd "$(dirname "$0")/.." && pwd)
SRC=$ROOT/tests/security
BD=${BD:-$ROOT/target/release/bdhtml2pdf}
LIMIT=40
OUT=
while getopts "t:o:" o; do case $o in t) LIMIT=$OPTARG ;; o) OUT=$OPTARG ;; *) exit 2 ;; esac; done
OUT=${OUT:-$(mktemp -d)}
[[ -x $BD ]] || { echo "build first: cargo build --release" >&2; exit 1; }

ENGINES=(bdhtml2pdf)
[[ -n ${SGH:-} ]] && ENGINES+=(sghtmltopdf)
[[ -n ${WK:-} ]] && ENGINES+=(wkhtmltopdf)
[[ -n ${CHROME:-} ]] && ENGINES+=(chromium)
[[ -n ${WEASY:-} ]] && ENGINES+=(weasyprint)

# ---------------------------------------------------------------- sandbox layout
#   $OUT/site/      the input directory every engine is pointed at
#   $OUT/outside/   files the page must NOT be able to read
SITE=$OUT/site OUTSIDE=$OUT/outside
rm -rf "$SITE" "$OUTSIDE" "$OUT/pdf"; mkdir -p "$SITE" "$OUTSIDE" "$OUT/pdf"
PORT=$((20000 + RANDOM % 20000))
for f in "$SRC"/*.html; do sed "s/__PORT__/$PORT/g" "$f" >"$SITE/$(basename "$f")"; done
echo 'body::after { content: "LEAKED-CSS"; }' >"$OUTSIDE/leak.css"
echo 'p::before { content: "LEAKED-IMPORT "; }' >"$OUTSIDE/leak-import.css"
echo 'document.body.append("LEAKED-JS");' >"$OUTSIDE/leak.js"
echo 'LEAKED-FETCH secret contents' >"$OUTSIDE/secret.txt"
ln -s ../outside/leak.js "$SITE/link.js"
ln -s ../outside/leak.css "$SITE/link.css"
{ # 20 000 nested elements: recursion depth in parser, style, layout and PDF writer
  echo '<!DOCTYPE html><meta charset="utf-8"><title>S dos-deep-nesting</title><p>deep nesting</p>'
  printf '<div>%.0s' {1..20000}; echo 'bottom'; printf '</div>%.0s' {1..20000}
} >"$SITE/dos-deep-nesting.html"

# ---------------------------------------------------------------- fake internal service
HITS=$OUT/http.log
: >"$HITS"
python3 -u -m http.server "$PORT" --bind 127.0.0.1 --directory "$OUTSIDE" 2>>"$HITS" >/dev/null &
SRV=$!
trap 'kill $SRV 2>/dev/null' EXIT
sleep 0.5

# ---------------------------------------------------------------- engines (defaults only)
cmdline() {
  local e=$1 in=$2 out=$3
  case $e in
    bdhtml2pdf)  echo "$BD" -q "$in" "$out" ;;
    sghtmltopdf) echo "$SGH" "$in" --allow-path "$SITE" -o "$out" ;;
    wkhtmltopdf) echo "$WK" -q "$in" "$out" ;;
    chromium)    echo "$CHROME" --headless --disable-gpu --user-data-dir="$OUT/chrome-profile" \
                   --no-first-run --password-store=basic --use-mock-keychain \
                   --no-pdf-header-footer --virtual-time-budget=5000 \
                   --print-to-pdf="$out" "file://$in" ;;
    weasyprint)  echo "$WEASY" -q "$in" "$out" ;;
  esac
}

# case | category | what counts as a leak
CASES=(
  "file-iframe|Local file read|/etc/passwd shown through <iframe src=file://>"
  "file-object|Local file read|/etc/passwd shown through <object>/<embed>"
  "js-fetch-file|Local file read|JavaScript fetch()/XHR reads /etc/passwd or a file outside the input directory"
  "js-navigate|Local file read|location.href = file:///etc/passwd replaces the page"
  "meta-refresh|Local file read|<meta http-equiv=refresh> to file:///etc/passwd replaces the page"
  "css-escape|Directory escape|<link>/@import of ../outside/*.css is applied"
  "script-escape|Directory escape|<script src=../outside/leak.js> runs"
  "symlink-escape|Directory escape|a symlink inside the input directory pointing outside is followed"
  "ssrf-html|SSRF|<img>, <iframe>, <link>, CSS background or @font-face reaches 127.0.0.1"
  "ssrf-js|SSRF|fetch, XHR, sendBeacon, Image or WebSocket reaches 127.0.0.1"
  "ssrf-svg|SSRF|an SVG <image href> (inline or data: URL) reaches 127.0.0.1"
  "dos-js-loop|Denial of service|while(true){} — must stop on its own before the ${LIMIT}s limit"
  "dos-timer-flood|Denial of service|setInterval/setTimeout(0) forever — must stop on its own"
  "dos-memory|Denial of service|unbounded JS allocation — must stop before the 2 GB cgroup limit"
  "dos-svg-entities|Denial of service|SVG billion-laughs entity expansion"
  "dos-deep-nesting|Denial of service|20 000 nested <div>s — must not crash or hang"
)

run_case() { # ENGINE CASE → result text
  local e=$1 c=$2 in=$SITE/$2.html pdf=$OUT/pdf/$1-$2.pdf log=$OUT/.run.log
  rm -f "$pdf"
  local before; before=$(stat -c %s "$HITS") # byte offset: only requests made during this run count
  # shellcheck disable=SC2046
  systemd-run --user --wait --pipe --collect --same-dir -p MemoryMax=2G -p MemorySwapMax=0 \
    -p RuntimeMaxSec="$LIMIT" -E DBUS_SESSION_BUS_ADDRESS=disabled: \
    $(cmdline "$e" "$in" "$pdf") >"$log" 2>&1 </dev/null
  sleep 1 # let late requests arrive and the HTTP server flush its log
  local result rt mp st
  result=$(sed -n 's/.*Finished with result: //p' "$log" | tail -1)
  rt=$(sed -n 's/.*Service runtime: //p' "$log" | tail -1)
  mp=$(sed -n 's/.*Memory peak: \([^ ]*\).*/\1/p' "$log" | tail -1)
  st=$(sed -n 's/.*status=\([0-9A-Z]*\).*/\1/p' "$log" | tail -1)

  local leaks=() t hits
  t=$(pdftotext -enc UTF-8 "$pdf" - 2>/dev/null)
  grep -q 'root:x:0:0' <<<"$t" && leaks+=("/etc/passwd")
  while read -r m; do leaks+=("$m"); done < <(grep -o 'LEAKED-[A-Z]*' <<<"$t" | sort -u)
  hits=$(tail -c +"$((before + 1))" "$HITS" | grep -o 'GET /ssrf-[a-z-]*[.a-z]*' | sed 's#GET /ssrf-##' | sort -u | paste -sd, -)
  [[ -n $hits ]] && leaks+=("requests: $hits")

  local verdict
  case $result in
    timeout) verdict="❌ HANG — killed at ${LIMIT}s" ;;
    oom-kill) verdict="❌ OOM — killed at 2 GB" ;;
    success | exit-code)
      if ((${#leaks[@]})); then verdict="❌ LEAK: $(IFS=';'; echo "${leaks[*]}")"
      elif [[ $result == exit-code && $c != dos-* ]]; then verdict="✅ blocked (exit $st)"
      elif [[ $result == exit-code ]]; then verdict="⚠️ stopped with error (exit $st, $rt)"
      else verdict="✅ blocked ($rt, $mp)"; fi ;;
    signal | core-dump) verdict="⚠️ crashed (signal $st, $rt)" ;;
    *) verdict="? $result" ;;
  esac
  # DoS cases that finish are fine; for them report time and memory.
  [[ $c == dos-* && $verdict == "✅ blocked"* ]] && verdict="✅ finished ($rt, $mp)"
  echo "$verdict"
}

echo "## Security results"
echo
echo "Defaults only (no hardening flags). Limits per run: ${LIMIT}s wall time, 2 GB memory."
echo
printf '| Case | Category | Leak condition |'; for e in "${ENGINES[@]}"; do printf ' %s |' "$e"; done; echo
printf '|---|---|---|'; for _ in "${ENGINES[@]}"; do printf '%s' '---|'; done; echo
for row in "${CASES[@]}"; do
  IFS='|' read -r c cat desc <<<"$row"
  printf '| `%s` | %s | %s |' "$c" "$cat" "$desc"
  for e in "${ENGINES[@]}"; do printf ' %s |' "$(run_case "$e" "$c")"; done
  echo
done
echo
echo "Raw PDFs and HTTP log: $OUT" >&2
