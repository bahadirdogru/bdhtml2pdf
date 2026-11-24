# Development history before 1.0.0

The repository history was squashed into a single commit when 1.0.0 was
released (2026-11-24). This file keeps the earlier commit messages, newest
first, so their notes are not lost. The short hashes are the old commit
ids; TODO.md and other notes still refer to some of them. Releases
0.1.0–0.1.3 were pre-releases built from the commits marked below.

## After 0.1.3 (part of 1.0.0)

### 2026-11-24 · `cd8836f` TODO: wait for upstream on inline borders and padding

### 2026-11-24 · `3114266` TODO: keep Boa as is, follow upstream for speed

### 2026-11-24 · `5fd89a0` TODO: a plan for floated ::first-letter (drop caps)

### 2026-11-24 · `cdadaa1` Make block-level links clickable over their whole box

```text
A display:block <a href> was a link only over its text line. Pin the
fulgur fork commit that makes such an anchor's whole box a link area.
```

### 2026-11-24 · `2299068` TODO: GC_DROPPING is not worth a Boa PR on its own

### 2026-11-24 · `ca0db45` TODO: qrcode.js profile in bare Boa; global var reported (boa#5686)

### 2026-11-24 · `9f53756` TODO: slow global var is Boa itself, but not the QR code's cost

### 2026-11-24 · `bd8f194` Make block-level images inside a link clickable

```text
<a href><img style="display:block"></a> had no link annotation. Pin
fulgur 509a1489, which makes such an image's box a link area (and, in
tagged PDFs, puts its annotation in a Link).
```

### 2026-11-24 · `0fc1b7d` TODO: block boxes inside a link get no link area

### 2026-11-24 · `96c4ad8` Put the text of an inline role="img" into its Figure

```text
A <span role="img" aria-label="..."> gave an empty Figure in tagged
PDFs, its text left in the paragraph. Pin fulgur f0ad75c3, which makes
that text the Figure's content, in its place.
```

### 2026-11-24 · `d867306` Take aria-labelledby as a text alternative in tagged PDFs

```text
An SVG labelled only with aria-labelledby had no alt text, so --pdf-ua
refused it. Pin fulgur 314812c6, which takes the text of the elements
aria-labelledby names before aria-label, for figures and links.
```

### 2026-11-24 · `3ba46cb` Tag table captions as Caption in tagged PDFs

```text
A <caption> was an anonymous P in the structure tree. Pin fulgur
c8870762, which tags it as a Caption element before its Table.
```

### 2026-11-24 · `d7e7956` TODO: inline SVG page CSS sent upstream (blitz#1741, #1742)

### 2026-11-24 · `22c175d` Apply page CSS to inline SVG

```text
Page CSS such as `.icon rect { fill: red }` had no effect on an inline
<svg>: its markup went to usvg without the cascade. Pin Blitz 4257684d,
which writes each SVG element's computed paint into the markup. Gradients
referenced from the SVG's own markup keep working.
```

### 2026-11-24 · `313b839` TODO: ::first-letter sent upstream (blitz#1739, #1740)

### 2026-11-24 · `67497db` TODO: ::first-letter works, drop caps are next

### 2026-11-24 · `b2136c5` Style the first letter with ::first-letter

```text
::first-letter rules had no effect. Pin Blitz cf4ca56e, which gives the
block's first letter (with the punctuation around it) the ::first-letter
style. A floated first letter (drop cap) is still laid out inline.
```

### 2026-11-24 · `cc95622` TODO: what blitz#1018 does for inline padding and what it needs

### 2026-11-24 · `ed593f7` TODO: ::first-letter is unsupported; blitz#1018 covers inline padding

### 2026-11-24 · `d585fc9` Widen auto-width tables for their percentage cells

```text
A table without a width ignored its cells' percentages and was as wide
as their content, so 60% + 40% cells wrapped text that Chromium keeps on
one line. Pin Blitz 20d28619, which sizes such a table as Chromium's auto
table layout does: wide enough for each percentage column to hold its
max-content width at its share.
```

### 2026-11-24 · `d3315f0` Add PDF/VT-3 support as a goal

### 2026-11-24 · `bc5e8bf` TODO: auto-width tables ignore percentage cells (the e-irsaliye hr)

### 2026-11-24 · `701c03a` TODO: inline borders and padding need span spacing in Parley

### 2026-11-24 · `5200962` Draw the backgrounds of inline elements

```text
A <span> with a background, or a <mark>, drew only its text
(fulgur-rs/fulgur#973). Pin fulgur d6bc3673, which paints each inline
element's background behind its runs, per line, over the fonts' ascent
and descent. Borders and padding of inline boxes are still to do.
```

### 2026-11-24 · `5b7b9d2` TODO: plain wording for review notes and the commit rule

### 2026-11-24 · `40250f7` Benchmark the 0.1.3 release binary against five engines on the i7-12800H

```text
The published linux-x86_64 archive, built with opt-level 3, on the same
machine and engines as the 0.1.2 run. JavaScript-heavy pages are about
10% faster (React 0.164 -> 0.148 s, the 1000-row invoice 0.163 -> 0.149 s)
for a 29 MB binary and ~3 MB more RSS; scores, PDFs and security verdicts
are unchanged. README, the site, why.md and security.md now quote it.
```

### 2026-11-24 · `29e4506` TODO: collapsed row lines sent upstream (blitz#504, fulgur#969 item 47)

### 2026-11-24 · `e381469` Give borderless collapsed rows no gap between them

```text
With border-collapse, rows whose cells have no border got a 1px unpainted
gap between them when another row of the table had a 1px border: Blitz
used one row gap for the whole table. Pin Blitz f6a61067 (one row track
per line, as wide as the borders meeting it) and fulgur 2be81aa8 (cell
borders centred on their own lines). The Japanese sample's sum rows now
match Chromium.
```

### 2026-11-24 · `c3f81b1` TODO: plan per-line collapsed row borders in Blitz and fulgur

### 2026-11-24 · `e95b307` TODO: ICU4X CLA signed

### 2026-11-23 · `79c5753` TODO: borderless collapsed rows get a 1px gap (Japanese sample)

### 2026-11-23 · `f51f191` TODO: Japanese sample's sum rows are 0.9px taller than in Chromium

### 2026-11-23 · `363a0d3` Compare pdftotext lines with \n on Windows too

### 2026-11-23 · `1939cbe` Keep the break after a hyphen with word-break: keep-all

```text
icu_segmenter's keep-all rule held HY among the letter units, so
"Stone-pot" stayed whole and the Korean menu sample put "Stone-pot" on
the next line, where Chromium breaks after the hyphen
(unicode-org/icu4x#8772). Patch icu_segmenter 2.3.0 with the fix from our
ICU4X fork (unicode-org/icu4x#8780) until a release has it.
```

### 2026-11-23 · `6781a0c` TODO: ICU4X keep-all hyphen break fixed in the fork (icu4x#8780)

### 2026-11-23 · `43d8b0d` TODO: Chromium's RTL page area clips boxes; left as it is

### 2026-11-23 · `1fe33a9` TODO: Chromium pixel-snaps boxes in print too; next is the RTL page edge

### 2026-11-23 · `c287d2f` Draw running elements from an inline <style> once

```text
fulgur 65bb78cc takes running elements out of the body when their
`position: running()` rule is in the page's own <style>; before, they
were drawn in the margin box and in the body (fulgur-rs/fulgur#983).
```

### 2026-11-23 · `266212a` TODO: running elements fixed in the fulgur fork (fulgur#983)

## 0.1.3 (2026-11-23)

### 2026-11-23 · `0dc08b3` Release 0.1.3

```text
Built with opt-level 3 instead of "s": JavaScript-heavy pages convert
12-14% faster for a 29 MB binary instead of 22 MB. The prose says 29 MB;
the comparison tables still quote the 0.1.2 run until the next one.
```

### 2026-11-23 · `8378495` TODO: Boa is slow on qrcode.js; fix in our Boa fork

### 2026-11-23 · `228f6ab` Build release binaries with opt-level 3

```text
Against opt-level "s", JavaScript-heavy pages convert 12-14% faster
(react/vue 0.28 -> 0.25 s, a qrcode.js invoice 0.84 -> 0.72 s) for a
29 MB binary instead of 22 MB and ~3 MB more peak memory.

CPU-specific builds (target-cpu x86-64-v3 / native) were within +-5% of
the portable build on every fixture, so the release archives stay
portable. docs/architecture.md records both rules.
```

### 2026-11-23 · `c748663` TODO: running elements drawn twice is an upstream fulgur bug

### 2026-11-23 · `100195c` Quote the 0.1.2 benchmark in why.md and security.md

### 2026-11-23 · `40b17a1` Benchmark 0.1.2 against five engines on the i7-12800H

## 0.1.2 (2026-11-23)

### 2026-11-23 · `3a7b3b1` Release 0.1.2

```text
The install commands use the new archive names (linux-x86_64,
linux-aarch64, macos-aarch64, windows-x86_64). The benchmark tables still
quote the 2026-11-22 run; the 0.1.2 run will be made on another machine.
```

### 2026-11-23 · `c14c129` Add scripts/sync-forks.sh for the dependency forks

```text
The core dependencies are forked under github.com/bahadirdogru in case one
goes away. GitHub does not update a fork by itself: the script
fast-forwards each fork's default branch to its upstream with gh repo
sync, reports any that diverged, and never touches our work branches.
```

### 2026-11-23 · `166f30e` Do not panic on non-ASCII text before url() in a stylesheet

```text
css_urls stepped through the CSS one byte at a time and sliced it as a
str, which panics inside a multi-byte character (content: "→" before a
url() or @import). It now matches the keywords on bytes.
```

### 2026-11-23 · `55b5b5d` Read right-to-left lines in order, keep element languages and lay out tabs

```text
fulgur 31f572d2:
- The structure tree reads right-to-left lines in reading order; the
  content stream keeps the visual order, as Chromium writes it.
- An element's own lang is written as /Lang. Only language tags are
  taken, so en_US becomes en-US, and an invalid <html lang> says so.
- A tab in <pre> draws nothing and reaches its tab stop, so PDF/UA and
  PDF/A accept it.
- A link inside a ligature keeps its annotation, and a heading's /T holds
  its text once.

Upstream: fulgur-rs/fulgur#977, #978, #979 and PR #969 item 45; the krilla
side is LaurenzV/krilla#492.
```

### 2026-11-22 · `9f8272e` Add the five-engine benchmark on an i7-12800H

### 2026-11-22 · `b776e64` Rerun the five-engine benchmark after the ligature fix

```text
Same machine, packages and fonts, 10 runs each, built from 50b0b72.
The ligature-text check now draws its words in Comfortaa, which has
ligatures; every engine passes it. Scores are unchanged except
Chromium's requestAnimationFrame check, which passes in about half of
the runs, and the security verdicts are the same. Update the numbers in
the README, the website and security.md.

WeasyPrint is no longer the only choice for PDF/UA in why.md.
```

### 2026-11-22 · `50b0b72` Keep all the letters of ligature glyphs in the text

```text
fulgur 19aa3185 maps each ligature glyph to all of its letters; before,
"office" extracted, copied and read aloud as "ofce". Test it with
Comfortaa, which has the ff, fi, fl, ffi and ffl ligatures.

The benchmark's ligature-text check used font-family: serif, which on
this machine is a font without ligatures, so it passed without testing
anything. It now uses Comfortaa through @font-face.
```

### 2026-11-22 · `5c5850f` Note the fulgur PDF/UA issue and PR items

### 2026-11-22 · `915ec90` Add --pdf-ua for accessible PDF/UA-1 output

```text
fulgur now tags every piece of page content for PDF/UA: text in
cells, divs and anonymous blocks, figures for images and SVGs, links
(also image-only ones and links in margin boxes), inline boxes in
reading order, and marks backgrounds, borders and @page headers and
footers as artifacts. --pdf-ua turns it on, and the server takes
pdf-ua. A page without a language, a title, alt text on an image or
with headings that skip a level fails, saying what to change.

Every website sample passes veraPDF's PDF/UA-1 profile, and PDF/A-2b
and 2a. The newsletter sample's first heading is now an h2 with the
same look, and the Chinese sample's chart has a text alternative.
```

### 2026-11-22 · `01556bb` Name WeasyPrint's --allowed-protocols in security.md

### 2026-11-22 · `7971af3` Add WeasyPrint to the security suite

```text
bench/security.sh runs WeasyPrint with WEASY=, as bench/run.sh does. On
2026-11-22 it blocks 7 of 11 file and network attacks and handles 4 of 5
denial-of-service cases; the other engines' verdicts are unchanged.
```

### 2026-11-22 · `302b426` Compare all five engines on one machine, 10 runs each

```text
bdhtml2pdf, sghtmltopdf 0.8.0, wkhtmltopdf 0.12.6.1, Chromium 156 and
WeasyPrint 70.0 ran on the same machine with the same system packages
and fonts, median of 10 runs, plus the security suite. The README, the
website, why.md and security.md now quote only this run instead of
mixing runs from two machines.

The bench counts Chromium's install directory instead of /usr/bin on
Debian.
```

### 2026-11-22 · `d312a49` Start the page area at a whole CSS pixel, as Chromium prints

```text
Chromium rounds the top left corner of the page area to the nearest
whole CSS px and keeps its exact width, so 10 mm margins put the content
at 38px, not 37.8. fulgur's snap_page_area_origin does the same.

The low-resolution QR test decoded at exactly 200 dpi, which depends on
where module edges fall between pixels; it now asks for 8 of 195 to 205
dpi.
```

### 2026-11-22 · `48e8ece` Note the Blitz PR for set_logical_size

### 2026-11-22 · `0ce7c76` Defer publishing the binding packages

### 2026-11-22 · `a63978e` Note that the Gemfile install works

### 2026-11-22 · `5c7a65b` Note the local binding test results in TODO.md

### 2026-11-22 · `26f6f7c` Add a server mode and bindings for Python, Node.js, Ruby and PHP

```text
`bdhtml2pdf serve` takes pages over HTTP (POST /pdf, /png, /jpg,
/html; options in the query string) and runs every conversion in a
child process in an empty directory. The memory limit and the
JavaScript timeout are process-wide, so this is what lets a memory bomb
or an endless script fail alone: 507 and 504, and the server stays up.
Server-only settings (memory, timeout) cannot be set by a request.

The bindings have no native code and no dependencies: each runs the
binary or talks to the server, with the same option names, formats and
errors. A CI job tests all four against a release build.

The website gets a tabbed quick start for the command line, the four
languages and the server.
```

### 2026-11-22 · `e27543f` Add use cases, WeasyPrint to the comparison, and crates.io metadata

```text
The site gets a "Where it fits" section and three more feature cards
(PDF/A, byte-identical output, four platforms). WeasyPrint 70.0 joins
the benchmark script and both comparison tables, with its lead on PDF
standards stated. The font checks accept an @font-face font embedded
under its CSS family name, as WeasyPrint does. The fulgur column now
shows its real version, 0.41.

Cargo.toml has the crates.io metadata; publishing waits for fulgur and
Blitz to publish what we depend on (TODO.md item 11).
```

### 2026-11-21 · `1ede900` Add PDF/A-2a and 3a, a feature comparison, and the PDF standards plan

```text
--pdf-a takes 2a and 3a too: tagged PDF/A, which needs <html lang> and
alt text on images; 2b stays the usual choice. The website compares
features with fulgur, sghtmltopdf, wkhtmltopdf and Chromium, from each
project's documentation and our measurements. The roadmap plans PDF/UA,
PDF/X-4, PDF/VT, PDF/A-3 embedded files and PDF/A-4.
```

### 2026-11-21 · `434144f` Add --pdf-a, and settle Canvas 2D and the header/footer flags

```text
--pdf-a 1b|2b|2u|3b|3u writes a PDF/A file for archiving, through fulgur's
new pdf_a option; the creation date PDF/A needs is now, or
SOURCE_DATE_EPOCH. The conversion fails, saying why, when the page cannot
be written at that level.

Canvas 2D and wkhtmltopdf's header/footer flags are no longer goals: an
environment with a canvas can rasterize it, and a page sets its headers
and footers with @page margin boxes. The website and README no longer
compare speed with sghtmltopdf; it runs no JavaScript.

Release archives drop the target triple's vendor field from the next
release on (linux-x86_64, linux-aarch64, macos-aarch64, windows-x86_64).
The release notes said glibc 2.36; the binaries need 2.35, and RHEL 9
(glibc 2.34) is too old, so the README and website now say RHEL 10.
```

### 2026-11-21 · `9fde454` Note the fulgur PR body update in TODO.md

### 2026-11-21 · `c2125e2` Lay out the page area at its exact width

```text
Pin the fulgur and Blitz commits that give Blitz the page area as an exact,
fractional size: an A4 page with 18mm margins is 657.6px wide, as in
Chromium, not 657px. Two boxes as wide as that page area now share a line.
TODO.md notes what is left: the Korean line break, which comes from ICU4X
(unicode-org/icu4x#8772), inline backgrounds (fulgur-rs/fulgur#973), and
a Blitz PR for the new viewport API.
```

### 2026-11-21 · `0a1ea33` Render the website samples again and note the next steps

```text
The samples and the Chromium comparison are rendered with page area
clipping. TODO.md lists what is left: the layout width truncated to whole
pixels, the Korean line break that follows from it, and smaller items.
```

### 2026-11-21 · `60eaf37` Clip page content to the page area, as Chromium prints it

```text
Content that overflows into the page margins is cut off on every side,
as in Chromium; the fulgur pin adds clip_to_page_area for this. The hanko
of the Japanese sample no longer draws into the right margin.
```

### 2026-11-21 · `a68ce08` Add a Korean sample and a Chromium comparison to the website

```text
A Korean restaurant menu joins the language samples. A new section shows
the first page of each language sample as Chromium prints it and as
bdhtml2pdf prints it, side by side; scripts/site.sh renders both when
Chromium is installed and records its version.
```

### 2026-11-21 · `c729cc2` Break lines at a <br> inside a flex item

```text
The Blitz pin brings a <br> after text into the text's anonymous flex
item, so it breaks the line as in browsers (DioxusLabs/blitz#1709). The
Persian seal and the Japanese hanko no longer need an inner div, and the
website drops the limitation.
```

### 2026-11-21 · `a230647` Let a page's @page size win over the default A4

```text
Without -s, -O or --page-width the size of a CSS @page rule now applies,
as in Chromium, with A4 portrait where the page sets none; any of those
flags still overrides it, as in wkhtmltopdf.
```

### 2026-11-21 · `10664fc` Add Arabic, Persian, Japanese, Chinese and Russian samples

```text
Five one-file samples under Languages on the website: an Arabic tax
invoice and a Persian employment certificate, right to left; a Japanese
invoice; a Chinese quarterly report with a chart drawn by JavaScript; and
a Russian newsletter in two columns. The existing samples are rendered
again, now with their @page margins.
```

### 2026-11-21 · `a6b3ad5` Let a page's @page margins win over the default margins

```text
Without -T/-B/-L/-R the margins of a CSS @page rule now apply, as in
Chromium, with 10mm where the page sets none; any margin flag still
overrides them, as in wkhtmltopdf. The fulgur pin also brings Arabic marks
in their place, page margin boxes aligned and placed as css-page says and
laid out right to left in RTL documents, a color per border side, and 3px
double borders as two lines.
```

### 2026-11-21 · `2558fab` Print Helvetica, Times and Courier as Chromium does on Windows and macOS

```text
On Windows these families are not installed, and Chromium prints them in
Arial, Times New Roman and Courier New, as Windows' font substitutes say;
we printed Helvetica in Times New Roman. On macOS Helvetica is installed
and both use it, which the test now accepts.
```

### 2026-11-21 · `d92f02c` Lay out images, collapsed tables and floats as Chromium does

```text
Pin the fulgur and Blitz commits that map the align attribute of images,
size collapsed table borders and widths as Chromium does, fit boxes beside
floats by their padding and borders (Taffy), and stop splitting tables
whose rows fit the page. The four e-invoice samples now end within a pixel
of Chromium's rendering.
```

### 2026-11-21 · `eb77a3d` Resolve font families as Chromium does

```text
Chromium prints `serif`, `cursive`, `fantasy` and families that are not
installed in its standard font, Times New Roman, and `sans-serif` and
Helvetica in Arial; on Linux in their metric-compatible Liberation
families. bdhtml2pdf took fontconfig's choices instead (Noto Serif,
Noto Sans, Nimbus Sans), whose widths and line heights moved page breaks.

Text the page does not style is now set in Times New Roman, as in
Chromium, instead of Arial. Turkish e-invoice templates asking for
`'Tahoma', "Times New Roman", serif` print in Times New Roman where Tahoma
is missing, every Turkish letter from that font.
```

### 2026-11-21 · `2a63fb1` Lay out tables as Chromium does

```text
Update fulgur and Blitz:

- Auto-layout table columns are sized as in Chromium (CSS Tables 3 width
  distribution): a long description gets most of an invoice table, short
  columns wrap, percentage widths on cells are honoured, and a nested
  table of fixed-width cells shrinks to fit its column.
- A table cell's own `vertical-align` no longer makes its lines taller.
- Collapsed borders are counted once, not twice.
- Text in middle- and bottom-aligned cells is drawn where it belongs.
- An empty table column no longer makes the page shrink to fit.

tests/tables.rs checks each against the widths and heights Chromium gives.
```

### 2026-11-20 · `155f276` Add a ticket with QR codes to the website samples

```text
The QR codes come from qrcode.js, which draws them as tables when there is
no canvas, as e-invoice templates do. A test checks that all four codes on
the page scan. The other samples are rendered again with 0.1.1.
```

## 0.1.1 (2026-11-20)

### 2026-11-20 · `928822b` Release 0.1.1

### 2026-11-20 · `3a5fc43` Make long tables in pages with a lang attribute 7 times faster

```text
In a page with lang="tr" (or any language fontique has no fallback data
for), every table cell queried fontconfig for fallback fonts again. Blitz
now looks the default fallbacks up when a document is created, with the
same results. The 1000-row invoice-large takes 0.17 s instead of 1.07 s,
faster than wkhtmltopdf (0.37 s) and Chromium (0.71 s).

tests/long_tables.rs compares a 500-row table with and without lang="tr";
before the fix the Turkish one took 6 times as long. docs/benchmarks/
2026-11-20.md is rerun, and the README, the site and the roadmap use its
numbers.
```

### 2026-11-20 · `3dfb079` Read font names from /BaseFont entries in the font tests

```text
embedded_fonts took every /XXXXXX+Name in the PDF bytes. On Windows the
embedded Arial's binary data contained such a sequence, so
unstyled_text_uses_arial saw an empty font name next to ArialMT and failed.
```

### 2026-11-20 · `9e9b560` Test that QR codes drawn as tables scan

```text
Without a canvas, qrcode.js (davidshimjs), which invoice templates use,
draws a <table> with one cell per module. The new tests print an
e-archive-sized code (61 modules in 128 px), rasterize the page with hayro
and decode it with rqrr: at CSS size, with smart shrinking at 300 and
200 dpi, and at 200 px. wkhtmltopdf's canvas image of the same code does
not decode at 200 dpi.

qrcode.js is vendored under tests/fixtures/vendor with its MIT license.
rqrr is a dev-dependency only.
```

### 2026-11-20 · `31be263` Build for size, share font data, and trim dependencies

```text
- The release profile uses opt-level "s": the binary is 22 MB with image
  output (31.5 MB before), at the same speed on real pages; only hot
  JavaScript loops in Boa run some 10% slower.
- fulgur shares font files with Parley instead of copying them. A page
  that falls back to a system CJK font takes 34 MB instead of 147 MB.
- Blitz pins usvg to krilla's version (one SVG stack instead of two) and
  builds Boa without Temporal and boa_runtime's unused fetch.

docs/benchmarks/2026-11-20.md is the bdhtml2pdf rerun; the README, the site
and the roadmap use its numbers.
```

### 2026-11-20 · `add265f` Apply --zoom to PDF output and default to Arial

```text
--zoom now scales PDF output as in wkhtmltopdf: the page is laid out
1/zoom times as wide and printed zoom times as large. It multiplies smart
shrinking and the shrink-to-fit minimum, and scripts see the matching
viewport. Values outside 0.1 to 10 are an error, for images too.

Text the page does not style is set in Arial, as in wkhtmltopdf (QtWebKit's
standard font), so it looks the same on Windows, macOS and Linux, where
fontconfig maps Arial to Liberation Sans. The rule is `:where(html)`, so
a page's own font-family rules win, `*` included.
```

### 2026-11-20 · `49124e6` Draw absolutely positioned boxes where they belong

```text
Builds against fulgur 2e8fcdee. An absolute box inside a table cell or a
static <div> (the hidden QR payload of e-archive invoice templates) was
placed off the page, so the whole document was shrunk to fit it, and one
in an otherwise empty <div> was not drawn at all.
```

### 2026-11-19 · `3e888a7` Document installing from the release binaries

```text
README and website: download, checksum and install steps for Linux, macOS
and Windows, the fonts a slim Linux container needs, and the
--enable-smart-shrinking hint for wkhtmltopdf templates; building from
source moves below them on the website.

The website's install steps no longer overflow narrow screens: the step
column can shrink below its code blocks' width. The 2026-11-19 benchmark
notes no longer point at a temporary directory.
```

## 0.1.0 (2026-11-19)

### 2026-11-19 · `2598298` Count pages of fixed-height boxes in the smart shrinking test

```text
With DejaVu (the CI fonts) 150 paragraphs filled five pages at 80% as at
100%: a page break fell right at the edge. Boxes of a fixed height make
the page count independent of the fonts.
```

### 2026-11-19 · `84e3459` Shrink content wider than the page, and add wkhtmltopdf's smart shrinking

```text
By default content wider than the page is scaled down to fit it, to at
most half size, as browsers do when printing; everything else prints at
its CSS size. --enable-smart-shrinking prints every page at 80%, as
wkhtmltopdf does by default, so pages written for it keep their page
count (an 800px table ends at 480pt, as in wkhtmltopdf 0.12.6).
--disable-smart-shrinking never shrinks.
```

### 2026-11-19 · `d12f069` Exit with status 4 on the memory limit on Windows too

```text
The memory limit ended the process with abort() outside Unix, so on Windows it exited with 0xC0000409 instead of status 4, failing CI. The C runtime's _write and _exit, which libc binds on Windows, do the same as on Unix without allocating.
```

### 2026-11-19 · `9da1044` Draw table borders: the border attribute and collapsed borders

```text
Blitz now maps a table's border attribute (DioxusLabs/blitz#1681), and fulgur draws borders in the collapsing border model, so <table border="1"> and border-collapse: collapse tables print their lines as in wkhtmltopdf and browsers. The website's invoice and report samples regain their row separators.
```

### 2026-11-19 · `aa55395` Link to bahadirdogru.com from the site footer

```text
The main site and its subdomains now link to each other.
```

### 2026-11-19 · `50c9137` Use metric-compatible fonts for Arial and other missing families

```text
fulgur now maps a missing Arial, Helvetica, Times New Roman, Courier New, Calibri, Cambria or Georgia to an installed metric-compatible family (Liberation, Carlito, Caladea, ...), as fontconfig's metric aliases do. Before, such text fell back to the default font, so lines were wider and documents gained pages compared with wkhtmltopdf. CI installs Liberation for the new test.
```

### 2026-11-19 · `94d9336` Add CI and release workflows

```text
CI runs fmt, clippy and the release tests on Linux (x86_64, aarch64), macOS and Windows. A vX.Y.Z tag builds Linux (in Debian 12), macOS and Windows binaries and opens a draft release with their SHA-256 sums.

The patched fulgur and Blitz commits are pinned in .github/upstream.env; scripts/upstream.sh fetches them next to the repository, and pins new ones after they are pushed.
```

### 2026-11-19 · `0dd4e13` Record which upstream pull requests are merged

```text
Four of the five Blitz pull requests (#1664, #1666, #1671, #1677) were merged; the serializer fix (#1665) is in review and the fulgur port (#969) is a draft. The architecture table gets an upstream status column and lists the fulgur multi-column and paragraph fixes.
```

### 2026-11-19 · `90d78ab` Drop the inline float known issue, fixed in fulgur

```text
fulgur now draws floats inside a line of text where Blitz places them, and keeps inline blocks and floats in paragraphs split across pages.
```

### 2026-11-19 · `3223346` Add the project website, served by GitHub Pages from docs/

```text
docs/index.html is a static page with no scripts, cookies or external
resources: features, sample documents, how it works, measured results,
security, Turkish support, install steps and known issues. The samples
(invoice, newsletter, certificate) and every picture are rendered by
bdhtml2pdf; scripts/site.sh regenerates them. CNAME points the site at
bdhtml2pdf.bahadirdogru.com and .nojekyll serves the folder as it is.

The known issues follow the latest fulgur changes: floats now flow
through multi-column layout, and two issues found while making the
samples are listed: floats inside a line of text are misplaced, and
oklch() and other CSS Color 4 values in SVG attributes draw black.
```

### 2026-11-19 · `fc073ac` Document multi-column layout continuing across pages

```text
fulgur now continues a multi-column block that is taller than the page on the next pages, and no longer panics on bare text between blocks inside the columns. The known issues list what still stays whole.
```

### 2026-11-19 · `0e28cc9` Document bordered boxes breaking across columns, and the page limit

```text
fulgur now breaks bordered boxes across columns. The known issues now
name the remaining multicol gap: a multi-column block taller than the
page loses its content after the first page.
```

### 2026-11-19 · `04a9a6c` Show multi-paragraph columns with a color-mix() rule in the showcase

```text
fulgur now flows multicol content through wrappers and between
paragraphs, and resolves column-rule colors with Stylo. The showcase
goes back to three paragraphs and takes its rule color from the
--line variable. The known issues narrow to boxes with decorations,
which still stay whole in one column.
```

### 2026-11-19 · `d85963f` Rewrite the README around a showcase rendered by bdhtml2pdf

```text
- docs/showcase: a Turkish report page (JavaScript-built cards, table and
  SVG chart, oklch/color-mix, grid, multi-column text) and its PNG and
  PDF, rendered by bdhtml2pdf itself. The README opens with it.
- README: highlights, grouped command-line options, quick start, current
  binary size and Rust version, fork and PR references for the build.
- A fixed --height now crops a page laid out at full length, as a browser
  window does. Before, a block taller than the image moved to a second
  page and the image came out empty.
- docs: multi-column limitations and the SVG font fix found while making
  the showcase.
```

### 2026-11-19 · `1643575` Render images from the PDF layout, so multi-column layout shows

```text
Image output now lays the page out with fulgur on a single page as wide
as the viewport and as tall as the content, then rasterizes that page
with hayro. The image shows exactly what the PDF shows: multi-column
layout, table fragmentation and GCPM. `vh` units follow the viewport
(`--height`, or 768), not the page length. blitz-paint, anyrender and
vello are no longer dependencies.

Found on the way, and fixed for PDF output too:

- Images were never drawn: fulgur only draws images registered in its
  asset bundle. src/assets.rs finds the images a page refers to (<img>,
  <style>, linked and @imported stylesheets, style attributes) and
  registers the files inside the input directory.
- Bold text printed as regular with variable system fonts, and data:
  images were dropped (fixed in fulgur).
- An @import with an unparsable URL aborted the process (fixed in Blitz).

New tests: tests/pdf_content.rs, and multi-column and vh cases in
tests/image_output.rs.
```

### 2026-11-19 · `284fd5c` Add PNG and JPEG output

```text
The output extension (or --format) picks the image format. The DOM after
JavaScript is laid out at --width with screen media, painted by
blitz-paint on vello_cpu and encoded with image. Local resources load
through fulgur's sandboxed loader. New flags: --width, --height, --zoom,
--quality, --format.
```

### 2026-11-19 · `f819167` Flatten very deep HTML and render on a large stack

```text
Content below 500 nesting levels moves up to that level in document
order, as Blink and WebKit do, so fulgur's 512-level cutoff no longer
drops text. Parsing, layout and PDF output run on a 256 MiB stack.
20 000 nested elements now render in about 1 s instead of aborting.
```

### 2026-11-19 · `26218da` bench: translate compare.sh comments and output to English

```text
The Turkish strings it searches for are fixture content and stay.
```

### 2026-11-19 · `68bc956` Docs: translate research notes and Cargo.toml comments to English

```text
docs/arastirma.md becomes docs/research.md. Links in README.md,
docs/why.md and docs/benchmarks/README.md are updated.
```

### 2026-11-19 · `528a54b` Bound hostile JavaScript: loop limit, timer clamping, watchdog, memory limit

```text
Three of the five denial-of-service cases in bench/security.sh failed:
- `while (true) {}` hung until killed. --javascript-timeout was checked
  only between timer callbacks.
- A JavaScript allocation loop ran until killed at the 2 GB cgroup limit.
- A zero-delay timer flood spun at a single virtual instant until the
  30 s wall-clock timeout.

Four layers now bound them.

1. Loop limit (`--javascript-loop-limit`, default 10 million per call).
   - It uses Boa's RuntimeLimits through the new
     ScriptDocument::runtime_limits_mut (DioxusLabs/blitz#1671).
   - The error is an engine error, so try/catch cannot swallow it.
   - The script stops, the page renders with the DOM as it is, and the
     error is reported with --debug-javascript.
   - while(true) now stops in 0.15 s. The JavaScript memory bomb stops
     in 3.8 s at 47 MB.

2. HTML timer clamping. As in browsers, once timers keep rescheduling
   without time passing for more than 5 rounds, each round advances
   virtual time by at least 4 ms. A setTimeout(f, 0) flood therefore uses
   up --javascript-delay and ends: 30.06 s -> 0.07 s.

3. Wall-clock watchdog. run_scripts_bounded runs JavaScript on a worker
   thread with a 256 MiB stack.
   - If the thread is still running after --javascript-timeout + 2 s,
     the CLI exits with status 3 and a clear message.
   - This covers what the loop limit cannot see: endless promise chains,
     catastrophic regular expressions, slow built-ins.
   - Boa has no interrupt hook, so the process ends instead of the script.

4. Memory limit (`--max-memory`, default 1024 MiB).
   - src/limits.rs installs a counting global allocator.
   - Above the limit it writes a message and exits with status 4,
     without allocating or unwinding.
   - The largest fixture (fonts.html) uses 147 MB RSS.

Docs: README (options, exit statuses, known issues), security.md (defaults,
results, a per-attack table), roadmap.md, architecture.md, testing.md. In
the 2026-11-19 security results, the three DoS rows show the rerun, with a
note giving the earlier values.

Verified:
- tests/js_limits.rs (`cargo test --release`), 6 cases:
  - the loop limit, including through try/catch;
  - the timer flood;
  - the watchdog (exit 3);
  - the memory limit (exit 4);
  - heavy finite scripts running unchanged.
- bench/run.sh -n 5: correctness unchanged (CSS 10/11, JS 18/19, Turkish
  12/15, fonts 7/8; React, Vue and Preact pass; invoice 3/3). Times and
  memory are within noise.
- bench/security.sh: 15/16. Only dos-deep-nesting still aborts; that is
  HTML layout, not JavaScript.
- `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` are
  clean.
```

### 2026-11-19 · `eeb34e1` Apply rustfmt to fetch.rs and page.rs

```text
Formatting only. The repository had never been run through rustfmt. The
next commit formats the files it touches; this one does the rest, so
`cargo fmt --check` passes.
```

### 2026-11-19 · `4774b1c` Docs: record list-marker and caption fixes, ligature bug is upstream

```text
architecture.md, "Changes to upstream projects", gets two new entries:
- Blitz: outside list markers keep `white-space: pre`, the CSS Lists 3
  UA style for ::marker. Parley 0.13 collapsed the trailing space of
  "1. ", so numbered markers touched the item text ("1.Author").
- fulgur: a captioned table's percentage width moves to the caption
  wrapper. The fit-content wrapper made `width:100%` tables shrink to
  their content width under Blitz 0.3.

roadmap.md: the ligature text loss ("office" -> "ofce") was not caused by
the Blitz-master port. fulgur's own committed upstream example PDFs show
the same loss ("Ofce", "Workfow").
```

### 2026-11-19 · `715f567` Document purpose, architecture, tests, benchmarks and security results

```text
README.md is rewritten in English. It covers:
- why the project exists: e-learning certificates, invoices, reports and
  publishing need modern JS and CSS without a browser farm;
- how the pipeline works and which components it uses;
- headline results against sghtmltopdf, wkhtmltopdf and headless
  Chromium;
- the test suites, goals, known issues, CLI reference and build
  instructions.

Turkish support (ÇçĞğIıİiÖöŞşÜü, lang="tr" case mapping) is called out
as a first-class requirement.

docs/ is split by topic:
- why.md: use cases, the alternatives (wkhtmltopdf, Chromium, PDF
  libraries, sghtmltopdf, fulgur, Blitz, OxiBrowser, litehtml) and why we
  wrote glue instead of an engine.
- architecture.md: the pipeline, source layout, every dependency with
  version and license, the upstream patches, and the resource policy.
- testing.md: each suite, every check, and how it is scored.
- security.md: threat model, defaults, the attack catalogue and a
  results summary.
- benchmarks/README.md: method (systemd isolation, median of N, cgroup
  peak and RSS) and the release procedure. Each release commits the
  unedited output of bench/run.sh and bench/security.sh.
- benchmarks/2026-11-19.md: the first published results.
- roadmap.md: goals with current status, known issues and roadmap.
- arastirma.md: the Turkish research notes, now with the sghtmltopdf
  findings.

Measured results (i7-12800H, Fedora 45, median of 5 runs):
- CSS: 10/11 (sghtmltopdf 8, wkhtmltopdf 1, Chromium 11).
- JS: 18/19, and React, Vue and Preact render (sghtmltopdf and
  wkhtmltopdf: 0).
- A one-page document takes 0.07 s and 31 MB RSS. Chromium takes 0.36 s
  and 250 MB RSS; its install is 432 MB, against 25 MB for bdhtml2pdf.
- All 11 file-read and SSRF attacks are blocked. Chromium with default
  flags leaked /etc/passwd and made internal requests.

Known issues found by the suites and documented, not hidden:
- JavaScript DoS is not bounded: while(true), memory bombs and zero-delay
  timer floods;
- 3000+ levels of nesting are slow, and 20 000 levels abort;
- ligature glyphs lose their text ("office" -> "ofce");
- no Intl in Boa;
- container queries and subgrid are missing;
- long tables are slow (24 pages in 1.07 s).
```

### 2026-11-19 · `40ef485` Add self-checking test pages, benchmark and security suites

```text
Every test is a plain HTML page that runs unchanged through every engine.
Results are read back from the PDF with poppler (pdftotext, pdffonts,
pdfinfo, pdfimages); nothing is judged by eye.

Fixtures (tests/fixtures):
- modern-css.html: 11 PASS/FAIL checks:
  - custom properties, nesting, :has(), :is()/:where(), :not() lists;
  - @layer, media range syntax, container queries, :nth-child(of S);
  - generated content, @supports selector().
  Plus a visual section: grid, subgrid, flex, multi-column, oklch() and
  color-mix() gradients, tables.
- modern-js.html: 19 checks of ES2022+ language features, DOM APIs, the
  event loop and local fetch().
- react.html, vue.html, preact/: React 18, Vue 3 and Preact apps whose
  final text appears only after a timer. UMD builds are vendored, so no
  network is needed.
- html-parsing.html: the WHATWG tree for malformed markup:
  - implied tbody, foster parenting, the adoption agency algorithm;
  - "<div/>", inert <template>, character references.
- turkish.html: ÇçĞğIıİiÖöŞşÜü
  - in system fonts, @font-face, bold and italic;
  - lang="tr" uppercase/lowercase/capitalize (i->İ, I->ı);
  - ₺ and typographic marks;
  - the PDF title;
  - JS toLocaleUpperCase / localeCompare in tr-TR.
- fonts.html: @font-face OTF and WOFF2 under aliased names, monospace, CJK
  and Arabic fallback, ligatures, symbols, all fonts embedded.
- fonts/: Comfortaa (OTF) and Noto Sans (WOFF2), both OFL, with
  provenance notes.

bench/run.sh compares bdhtml2pdf with sghtmltopdf, wkhtmltopdf and
headless Chromium (each optional, set through SGH, WK and CHROME):
- Every run is a transient systemd user service, so wall time and memory
  are measured the same way for every engine. Memory is reported both as
  the cgroup peak of the whole process tree and as the RSS of the largest
  process.
- It reports the median of N runs after a warm-up, correctness scores,
  and PDF facts: pages, page size, size, raster images, and fonts
  embedded/subset.
- It also generates a 1000-row invoice to test pagination and repeated
  table headers.
- The output is Markdown, meant to be committed per release.

bench/security.sh runs every engine with default settings on the hostile
pages in tests/security/:
- file:// reads through iframe, object, fetch, XHR, navigation and meta
  refresh;
- ../ and symlink escapes for CSS, @import and scripts;
- SSRF from HTML, CSS, SVG and JavaScript, detected by a throwaway
  127.0.0.1 HTTP server;
- DoS: infinite loop, timer flood, memory bomb, billion laughs, and
  20 000-level nesting.
Pages that attack through JavaScript are ES5, so old engines run them too.
Each run is capped at 2 GB and 40 s.

bench/compare.sh (Blitz vs OxiBrowser) and bench/compare-sgh.sh are the
earlier comparison scripts used to choose the architecture.
```

### 2026-11-19 · `b09f1bb` Add bdhtml2pdf engine: HTML + JavaScript to vector PDF in one process

```text
A wkhtmltopdf-compatible command line that runs a page's JavaScript and
writes a vector PDF, without Chromium, WebKit or Qt.

Pipeline:
  html5ever -> Blitz DOM <-> Boa (blitz-vibey-script)   one DOM, one Stylo
            -> fulgur (pagination, GCPM)  -> Krilla (vector PDF)

- src/lifecycle.rs: runs scripts on a ScriptDocument with no timer thread
  and a virtual clock, so setTimeout() costs no real waiting and output
  does not depend on machine speed. Supports --javascript-delay (virtual
  time budget), --window-status (wait until window.status matches) and
  --javascript-timeout (wall-clock limit between callbacks).
- src/fetch.rs: script loader sandbox. Allows data: URLs and file: URLs
  inside the input directory, canonicalized so `..` and symlinks cannot
  escape. No network.
- src/pdf.rs: renders the final DOM with fulgur. Its base path confines
  <link>, <img> and @import to the input directory.
- src/prefs.rs: enables the Stylo features Blitz leaves off: :has(),
  :nth-child(of S) and container queries.
- src/page.rs: page sizes (A3-A5, B4, B5, Letter, Legal, Ledger) and
  lengths in mm, cm, in, pt and px.
- src/main.rs: CLI with input/output (or "-"), page size, orientation,
  custom size, margins, title, JavaScript options and quiet mode. An
  output name ending in .html writes the post-JavaScript DOM instead of
  a PDF.

Stylo can be linked only once per binary (links = "servo_style_crate").
fulgur is therefore used from a local port to Blitz master (../fulgur,
branch blitz-master), so JavaScript and PDF layout share Stylo 0.24.
Blitz comes from a local checkout (../blitz) through [patch], at rev
b4ad543 plus:
  - LayoutPassState and Node::svg_source made public;
  - a serializer fix: HTML elements are never written as <x />.
These patches will go upstream as pull requests.

Release profile: fat LTO, one codegen unit, panic=abort, stripped. The
binary is 24.7 MB.
```

### 2026-11-18 · `f9bd831` Initial commit
