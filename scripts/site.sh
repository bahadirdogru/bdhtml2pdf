#!/usr/bin/env bash
# Regenerates the website's documents and pictures with bdhtml2pdf:
# the sample PDFs (docs/samples), the showcase (docs/showcase) and the
# pictures in docs/assets. The website itself is docs/index.html, served by
# GitHub Pages from the docs/ folder.
#
# Needs a release build, pdftoppm (poppler-utils) for the sample previews and
# python3 with Pillow for resizing and WebP.
#
#   cargo build --release && scripts/site.sh
set -euo pipefail
cd "$(dirname "$0")/.."

B=${BDHTML2PDF:-target/release/bdhtml2pdf}
assets=docs/assets
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Documents. The certificate is landscape and edge to edge: the page size
# and margins come from the command line, as in wkhtmltopdf.
"$B" -q docs/samples/invoice.html docs/samples/invoice.pdf
"$B" -q docs/samples/newsletter.html docs/samples/newsletter.pdf
"$B" -q docs/samples/ticket.html docs/samples/ticket.pdf
"$B" -q -O Landscape -T 0mm -B 0mm -L 0mm -R 0mm docs/samples/certificate.html docs/samples/certificate.pdf
"$B" -q --page-size A4 docs/showcase/showcase.html docs/showcase/showcase.pdf
"$B" -q --width 960 --zoom 2 docs/showcase/showcase.html docs/showcase/showcase.png

# Previews of the first page of each PDF.
for s in invoice newsletter ticket certificate; do
  pdftoppm -f 1 -l 1 -r 110 -png -singlefile "docs/samples/$s.pdf" "$tmp/$s"
done
pdftoppm -f 1 -l 1 -r 110 -png -singlefile docs/showcase/showcase.pdf "$tmp/report"

# The social card reads showcase.png, which must sit in its directory.
cp scripts/site/og.html docs/showcase/showcase.png "$tmp/"
"$B" -q --width 1200 --height 630 "$tmp/og.html" "$assets/og.png"
"$B" -q --width 180 --height 180 scripts/site/icon.html "$assets/apple-touch-icon.png"

python3 scripts/site/images.py "$tmp" "$assets" docs/index.html
