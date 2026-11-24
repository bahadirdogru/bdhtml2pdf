<div align="center">

# bdhtml2pdf

**HTML + JavaScript → vector PDF, in one small process.**<br>
A modern successor to wkhtmltopdf. No Chromium, no WebKit, no Qt.

[![CI](https://github.com/bahadirdogru/bdhtml2pdf/actions/workflows/ci.yml/badge.svg)](https://github.com/bahadirdogru/bdhtml2pdf/actions/workflows/ci.yml)
[![License: GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-blue)](LICENSE)
[![Rust 1.92+](https://img.shields.io/badge/rust-1.92%2B-orange)](#building)
[![Release: 1.0.0](https://img.shields.io/badge/release-1.0.0-brightgreen)](https://github.com/bahadirdogru/bdhtml2pdf/releases)
[![Security suite: 16/16](https://img.shields.io/badge/security%20suite-16%2F16-brightgreen)](docs/security.md)

[Website](https://bdhtml2pdf.bahadirdogru.com) · [Quick start](#quick-start) · [How it works](#how-it-works) · [Results](#results) · [Command line](#command-line) · [Docs](#documentation)

<br>

<a href="docs/showcase/showcase.pdf"><img src="docs/showcase/showcase.png" width="760" alt="A Turkish student report rendered by bdhtml2pdf: a gradient header, KPI cards, a table and an SVG bar chart built by JavaScript, and three-column text"></a>

<sub>Rendered by bdhtml2pdf itself from <a href="docs/showcase/showcase.html">showcase.html</a>:<br>
the cards, table and chart are built by JavaScript; the PNG is the same layout as the <a href="docs/showcase/showcase.pdf">vector PDF</a> (35 KB, text selectable).</sub>

</div>

---

bdhtml2pdf takes the HTML your application already renders and runs its
JavaScript: React, Vue, Preact or plain DOM code. It styles the page with
**Firefox's CSS engine**, paginates it with print CSS, and writes a **vector
PDF** with real, searchable text and subset fonts. It can also write **PNG or
JPEG** images of the same layout, like wkhtmltoimage. All of this happens in
one process of about 29 MB, written in Rust.

|  |  |
|---|---|
| ⚡ **Modern web** | ES2022+ JavaScript, React 18 / Vue 3 / Preact, CSS nesting, `:has()`, `@layer`, grid, flex, multi-column, `oklch()` and `color-mix()` |
| 📄 **Real PDF** | text stays text, fonts are subset, `@page` and margin boxes, running headers, counters, repeated table headers, PDF/A for archiving |
| 🪶 **Small** | one 29 MB binary, ~30–70 MB of memory per page, 0.07 s for a typical document on an i7-12800H |
| 🔒 **Safe by default** | no network, files confined to the input directory, bounded JavaScript |
| 🇹🇷 **Turkish first** | the whole alphabet in every face, `lang="tr"` case mapping: `istanbul` → **İSTANBUL** |
| 🖼️ **Images too** | PNG and JPEG from the PDF layout, so multi-column text and pagination look the same |

> [!NOTE]
> **Release 1.0.0.** Every claim below is measured. It still builds against
> patched sibling checkouts of fulgur and Blitz (see [Building](#building)).
> Read [Known issues](#known-issues) before using it on untrusted input.

## Quick start

### Install

Prebuilt binaries are on the
[Releases](https://github.com/bahadirdogru/bdhtml2pdf/releases) page: one
archive per platform, each a single executable with no runtime to install,
plus a `.sha256` file. To build from source instead, see
[Building](#building).

| Platform | Archive |
|---|---|
| Linux x86_64 (glibc 2.35+: Ubuntu 22.04, Debian 12, RHEL 10 or newer) | `bdhtml2pdf-<version>-linux-x86_64.tar.gz` |
| Linux aarch64 | `bdhtml2pdf-<version>-linux-aarch64.tar.gz` |
| macOS, Apple silicon | `bdhtml2pdf-<version>-macos-aarch64.tar.gz` |
| Windows x86_64 | `bdhtml2pdf-<version>-windows-x86_64.zip` |

**Linux** (use `linux-aarch64` on ARM):

```sh
VERSION=1.0.0
PLATFORM=linux-x86_64
NAME=bdhtml2pdf-$VERSION-$PLATFORM
curl -LO https://github.com/bahadirdogru/bdhtml2pdf/releases/download/v$VERSION/$NAME.tar.gz
curl -LO https://github.com/bahadirdogru/bdhtml2pdf/releases/download/v$VERSION/$NAME.tar.gz.sha256
sha256sum -c $NAME.tar.gz.sha256
tar xzf $NAME.tar.gz
sudo install $NAME/bdhtml2pdf /usr/local/bin/   # or keep it anywhere on PATH
bdhtml2pdf --version
```

It needs fontconfig (`libfontconfig1` on Debian and Ubuntu, `fontconfig` on
Fedora and RHEL), which desktop and most server installs already have; in a
slim container image, install it along with some fonts:

```sh
apt-get install -y libfontconfig1 fonts-dejavu-core fonts-liberation   # Debian, Ubuntu
dnf install -y fontconfig dejavu-sans-fonts liberation-fonts            # Fedora, RHEL
```

**macOS**: the same steps with `PLATFORM=macos-aarch64` and
`shasum -a 256 -c` in place of `sha256sum -c`. The binary is not signed, so
a copy downloaded with a browser needs `xattr -d com.apple.quarantine
bdhtml2pdf` once (`curl` downloads are not quarantined).

**Windows** (PowerShell):

```powershell
$Version = "1.0.0"
$Name = "bdhtml2pdf-$Version-windows-x86_64"
Invoke-WebRequest "https://github.com/bahadirdogru/bdhtml2pdf/releases/download/v$Version/$Name.zip" -OutFile "$Name.zip"
(Get-FileHash "$Name.zip" -Algorithm SHA256).Hash   # compare with $Name.zip.sha256
Expand-Archive "$Name.zip" -DestinationPath .
.\$Name\bdhtml2pdf.exe --version
```

**Fonts.** Pages written for Arial, Helvetica, Times New Roman, Courier New,
Calibri or Cambria keep their line breaks when a metric-compatible family is
installed (`fonts-liberation`, `fonts-crosextra-carlito`,
`fonts-crosextra-caladea` on Debian and Ubuntu; `liberation-fonts`,
`google-carlito-fonts`, `google-caladea-fonts` on Fedora), as with
fontconfig's metric aliases. Windows and macOS already have the originals.
Text the page does not style is set in Times New Roman, as in Chromium, so it
looks the same on all three systems (Liberation Serif on Linux).

**Coming from wkhtmltopdf?** wkhtmltopdf prints everything at 80% by default
("smart shrinking"). Pass `--enable-smart-shrinking` to get the same sizes and
page breaks for templates tuned to it.

### Use

```sh
bdhtml2pdf invoice.html invoice.pdf
```

<table>
<tr><th>PDF</th><th>JavaScript-heavy pages</th><th>Images</th></tr>
<tr valign="top"><td>

```sh
bdhtml2pdf \
  --page-size A4 \
  --orientation Landscape \
  --margin-top 15mm \
  report.html report.pdf
```

</td><td>

```sh
# wait for the page to say it is done
bdhtml2pdf \
  --window-status ready \
  --javascript-delay 2000 \
  dashboard.html dashboard.pdf
```

</td><td>

```sh
# the whole page, 1280 px wide
bdhtml2pdf --width 1280 page.html page.png

# fixed size, twice the pixels
bdhtml2pdf --height 630 --zoom 2 \
  card.html card.jpg
```

</td></tr>
</table>

Need to see what JavaScript produced? `bdhtml2pdf app.html dom.html` writes
the DOM after scripts ran instead of a PDF.

### From Python, Node.js, Ruby and PHP, or over HTTP

```python
import bdhtml2pdf                                   # Python
pdf = bdhtml2pdf.convert("<h1>Merhaba</h1>", page_size="A4", pdf_a="2b")
```

```js
const pdf = await require('bdhtml2pdf').convert(html, { pageSize: 'A4' });   // Node.js
```

```ruby
pdf = Bdhtml2pdf.convert(html, page_size: "A4")                               # Ruby
```

```php
$pdf = (new Bdhtml2pdf\Converter())->convert($html, ['page_size' => 'A4']);  // PHP
```

```sh
bdhtml2pdf serve          # http://127.0.0.1:7368
curl --data-binary @invoice.html 'http://127.0.0.1:7368/pdf?page-size=A4' -o invoice.pdf
```

The packages have no native code: they run the binary, or talk to the server,
which runs every conversion in its own child process so a failing page fails
alone. They are not on PyPI, npm, RubyGems or Packagist yet; install them from
[bindings/](bindings/README.md), which also documents the server.

## Why

PDF generation sits in the middle of a lot of everyday systems:
certificates and course reports, invoices and statements, analytics exports,
worksheets, tickets, labels and contracts. The templates are almost always
HTML, and increasingly that HTML is built by JavaScript and styled with modern
CSS. The usual engines force a bad trade:

| Engine | The catch |
|---|---|
| **wkhtmltopdf** | Archived in 2023. Its 2012-era WebKit cannot run ES2015+ (React, Vue and Preact render blank) or modern CSS. Known SSRF (CVE-2022-35583). |
| **Headless Chromium** | Renders everything, but costs a 432 MB install, about 190–350 MB per conversion across several processes, and browser pools to babysit. With default flags it read `/etc/passwd` and made internal network requests in our security suite. |
| **Native PDF libraries** | Fast, but every HTML template has to be rewritten in their API. |
| **New native HTML engines** (e.g. sghtmltopdf) | Small and fast, but no JavaScript and only a hand-written subset of CSS. |
| **WeasyPrint** | Strong paged media and PDF standards (PDF/A, PDF/UA, PDF/X), but no JavaScript, and it needs Python and Pango. In our run it was about 5× slower than bdhtml2pdf on one page and 13× on a 24-page invoice. |

We wanted **modern JS + modern CSS + vector PDF + small footprint** together.
Nobody shipped that, but every hard part already existed as a maintained Rust
library. So we wrote the glue instead of another engine.
→ [docs/why.md](docs/why.md)

**Turkish is a first-class requirement.** The full alphabet
**Ç ç Ğ ğ I ı İ i Ö ö Ş ş Ü ü** is tested in system fonts, web fonts, bold
and italic, and it round-trips through PDF text extraction. `text-transform:
uppercase` with `lang="tr"` turns `istanbul` into **İSTANBUL**, and
`capitalize` turns `izmir` into **İzmir**: the kind of detail engines get
wrong.

## How it works

```
HTML ─► html5ever ─► Blitz DOM ◄──► Boa JavaScript        (one DOM, one Stylo)
                     Stylo (Firefox CSS) · Taffy (flex/grid) · Parley (text)
                         │  scripts run, timers drained on a virtual clock
                         ▼
                     fulgur: @page, margin boxes, running headers, counters,
                         │   pagination, table/multicol fragmentation
                         ▼
                     Krilla: vector PDF, subset fonts, text as text
                         │
                         └─► (images) hayro rasterizes a page as tall as the content ─► PNG / JPEG
```

| Component | Role |
|---|---|
| [Blitz](https://github.com/DioxusLabs/blitz) + `blitz-vibey-script` | DOM, style and layout glue, and the JavaScript ↔ DOM binding |
| [Stylo](https://github.com/servo/stylo) 0.24 | CSS cascade: the engine Firefox uses |
| [Taffy](https://github.com/DioxusLabs/taffy) · [Parley](https://github.com/linebender/parley) | block/flex/grid layout · text shaping and line breaking |
| [Boa](https://github.com/boa-dev/boa) 0.22 | JavaScript engine, pure Rust |
| [fulgur](https://github.com/fulgur-rs/fulgur) | paged media (GCPM), pagination, PDF layout; ported by us to current Blitz |
| [Krilla](https://github.com/LaurenzV/krilla) | PDF writer |
| [hayro](https://github.com/LaurenzV/hayro) | image output: rasterizes the PDF page (pure Rust, on vello_cpu) |

Everything is Rust except the system fontconfig. Every change we need in Blitz
and fulgur goes upstream as a pull request.
→ [docs/architecture.md](docs/architecture.md)

## Results

Measured on 2026-11-24 on one machine (i7-12800H, Fedora 45) with the same
system packages and fonts for every engine. Every engine ran the same pages
with its default settings (median of 10 runs; memory is the largest
process's RSS, plus the whole process tree for Chromium). Copied from
[docs/benchmarks/2026-11-24-i7-12800h.md](docs/benchmarks/2026-11-24-i7-12800h.md).
On a slower i7-6700 every engine takes about twice as long
([0.1.1 results](docs/benchmarks/2026-11-22.md)).

| | **bdhtml2pdf** 0.1.3 | sghtmltopdf 0.8.0 | wkhtmltopdf 0.12.6.1 | Chromium 157 headless | WeasyPrint 70.0 |
|---|---|---|---|---|---|
| Modern CSS (11 checks) | **10** | 8 | 1 | 11 | 8 |
| Modern JavaScript (19 checks) | **18** (no `Intl`) | 0 (no JS) | 0 (no ES2015+) | 18–19 | 0 (no JS) |
| React 18 / Vue 3 / Preact apps | **3/3** | 0/3 | 0/3 | 3/3 | 0/3 |
| HTML parsing (7) | **7** | 7 | 6 | 7 | 7 |
| Turkish (15) | **12** (only JS locale functions fail) | 8 | 8 | 14 | 11 |
| Fonts (8) | **8** | 7 | 6 | 8 | 7 |
| 1000-row invoice: pagination and repeated headers | ✅ | ✅ | ✅ | ✅ | ✅ |
| Vector output (raster images on text pages) | **0** | 0 | 0 | 0 | 0 |
| File reads and SSRF blocked (11 attacks) | **11/11** | 11/11 | 9/11 | 1/11 | 7/11 |
| Denial-of-service cases handled (5) | **5/5** | 4/5 (no JS) | 3/5 | 3/5 | 4/5 (no JS) |
| Typical one-page document | **0.07 s · 35 MB** | 0.03 s · 13 MB | 0.25 s · 40 MB | 0.35 s · 251 MB RSS (198 MB tree) | 0.47 s · 71 MB |
| React app with timers | **0.15 s · 52 MB** | — | — | 0.36 s · 264 MB | — |
| 24-page invoice | **0.15 s · 69 MB** | 0.08 s · 19 MB | 0.37 s · 47 MB | 0.69 s · 339 MB (tree) | 2.11 s · 146 MB |
| Install size | **29 MB**, one file | 13 MB | 39 MB | 432 MB | 83 MB + Python, Pango |

**In short:** bdhtml2pdf is the only engine besides Chromium that runs modern
JavaScript apps and modern CSS, with about **1/7 of Chromium's memory and
1/15 of its install size**, in a single process. It does not leak local files
or make network requests by default, and hostile JavaScript (endless loops,
timer floods, memory bombs) is stopped by built-in limits. sghtmltopdf and
WeasyPrint do not support JavaScript.

<details>
<summary><b>What is tested</b></summary>

Every check is a plain HTML page that goes through all engines unchanged.
Results are read back from the PDF with poppler; nothing is judged by eye.

| Suite | What it checks |
|---|---|
| **CSS** (11) | custom properties, nesting, `:has()`, `:is()`/`:where()`, `:not()` lists, `@layer`, media range syntax, container queries, `:nth-child(of S)`, generated content, `@supports selector()`; plus a visual page with grid, subgrid, flex, multi-column, gradients in `oklch()`/`color-mix()`, tables |
| **JavaScript** (19 + 3 apps) | ES2022+ language features, DOM APIs, `async`/`await`, microtasks, `requestAnimationFrame`, local `fetch()`; **React 18, Vue 3, Preact** apps whose final state appears only after a timer |
| **HTML** (7) | WHATWG parsing of malformed markup: implied tbody, foster parenting, adoption agency, `<template>`, character references |
| **Turkish** (15) | alphabet in every face, `@font-face`, `lang="tr"` uppercase/lowercase/capitalize, ₺ and typographic marks, PDF title metadata, JS locale functions |
| **Fonts** (8) | `@font-face` OTF and WOFF2, monospace, CJK and Arabic fallback, ligatures, symbols, all fonts embedded |
| **Pagination** | a generated 1000-row invoice: last row, totals, table header repeated on every page |
| **PDF output** | page size, page count, file size, raster images (should be 0 for text), fonts embedded and subset |
| **Security** (16) | local file reads, directory and symlink escapes, SSRF from HTML/CSS/SVG/JS, infinite loops, timer floods, memory bombs, billion laughs, deep nesting |

Each release runs `bench/run.sh` and `bench/security.sh` against bdhtml2pdf,
wkhtmltopdf, headless Chromium and sghtmltopdf on the same machine, each engine
in its own systemd transient service, and commits the unedited output as
`docs/benchmarks/<version>.md`. A regression in correctness or a new security
finding blocks the release or is listed as a known issue.

→ [docs/testing.md](docs/testing.md) · [docs/security.md](docs/security.md) · [docs/benchmarks/](docs/benchmarks/README.md)

</details>

## Command line

```
bdhtml2pdf [OPTIONS] <INPUT> <OUTPUT>
```

`INPUT` and `OUTPUT` are files, or `-` for stdin/stdout. The output extension
picks the format: `.pdf`, `.png`, `.jpg`/`.jpeg`, or `.html` (the DOM after
JavaScript). Anything else is PDF; `-f, --format pdf|png|jpg|html` overrides it.

<table>
<tr><th>Option</th><th>Default</th><th>Meaning</th></tr>
<tr><th colspan="3" align="left">📄 Page (PDF)</th></tr>
<tr><td><code>-s, --page-size</code></td><td><code>A4</code></td><td>A3, A4, A5, B4, B5, Letter, Legal, Ledger. Without this, <code>-O</code> or <code>--page-width</code>, a page's CSS <code>@page</code> size wins, as in Chromium; any of them overrides it, as in wkhtmltopdf</td></tr>
<tr><td><code>-O, --orientation</code></td><td><code>Portrait</code></td><td><code>Portrait</code> or <code>Landscape</code></td></tr>
<tr><td><code>--page-width</code>, <code>--page-height</code></td><td></td><td>custom size, e.g. <code>210mm</code></td></tr>
<tr><td><code>-T/-B/-L/-R, --margin-*</code></td><td><code>10mm</code></td><td><code>mm</code>, <code>cm</code>, <code>in</code>, <code>pt</code>, <code>px</code>. Without these flags a page's CSS <code>@page</code> margins win, as in Chromium; any flag overrides them, as in wkhtmltopdf</td></tr>
<tr><td><code>--title</code></td><td></td><td>PDF title</td></tr>
<tr><td><code>--pdf-a</code></td><td></td><td>write a PDF/A file for archiving: <code>1b</code>, <code>2b</code>, <code>2u</code>, <code>2a</code>, <code>3b</code>, <code>3u</code> or <code>3a</code>. <code>2b</code> is the usual choice; <code>b</code> keeps the appearance, <code>u</code> also keeps the text extractable, <code>a</code> also tags the document structure for accessibility (needs <code>&lt;html lang&gt;</code> and alt text on images); PDF/A-1 allows no transparency. The conversion fails, saying why, when the page cannot be one. The creation date is now, or <code>SOURCE_DATE_EPOCH</code></td></tr>
<tr><td><code>--pdf-ua</code></td><td></td><td>write a tagged PDF/UA-1 file for accessibility (ISO 14289-1): headings, paragraphs, lists, tables, links and figures are tagged in reading order for screen readers, and backgrounds, borders and <code>@page</code> headers and footers are marked as artifacts. Needs <code>&lt;html lang&gt;</code>, a <code>&lt;title&gt;</code> (or <code>--title</code>) and a text alternative on every image: <code>alt</code> on <code>&lt;img&gt;</code>, <code>aria-label</code> or <code>&lt;title&gt;</code> on <code>&lt;svg&gt;</code>; <code>alt=""</code> or <code>aria-hidden="true"</code> marks a decorative one. The conversion fails, saying what is missing. Cannot be combined with <code>--pdf-a</code></td></tr>
<tr><td><code>--enable-smart-shrinking</code></td><td></td><td>print every page at 80%, as wkhtmltopdf does by default; for pages written for wkhtmltopdf</td></tr>
<tr><td><code>--disable-smart-shrinking</code></td><td></td><td>never shrink; by default content wider than the page is scaled down to fit, to at most half size</td></tr>
<tr><td><code>--zoom</code></td><td><code>1</code></td><td>content scale, 0.1–10, as in wkhtmltopdf (<code>2</code> = twice as large); multiplies smart shrinking</td></tr>
<tr><th colspan="3" align="left">🖼️ Image (PNG, JPEG)</th></tr>
<tr><td><code>--width</code></td><td><code>1024</code></td><td>viewport width in pixels</td></tr>
<tr><td><code>--height</code></td><td><code>0</code></td><td>image height in pixels; <code>0</code> captures the whole page (up to 32 768 pixels)</td></tr>
<tr><td><code>--zoom</code></td><td><code>1</code></td><td>device pixels per CSS pixel, 0.1–10</td></tr>
<tr><td><code>--quality</code></td><td><code>94</code></td><td>JPEG quality, 1–100</td></tr>
<tr><th colspan="3" align="left">⚙️ JavaScript</th></tr>
<tr><td><code>-n, --disable-javascript</code></td><td></td><td>skip scripts</td></tr>
<tr><td><code>--javascript-delay</code></td><td><code>200</code></td><td>virtual milliseconds of timers to run after load</td></tr>
<tr><td><code>--window-status</code></td><td></td><td>wait until <code>window.status</code> equals this value</td></tr>
<tr><td><code>--javascript-timeout</code></td><td><code>30</code></td><td>wall-clock seconds; a script still running after that ends the process (exit 3)</td></tr>
<tr><td><code>--javascript-loop-limit</code></td><td><code>10000000</code></td><td>loop iterations per function call; stops <code>while (true) {}</code> (<code>0</code> = unlimited)</td></tr>
<tr><td><code>--max-memory</code></td><td><code>1024</code></td><td>heap limit in MiB; exceeding it ends the process (exit 4; <code>0</code> = unlimited)</td></tr>
<tr><td><code>--debug-javascript</code></td><td></td><td>print JavaScript errors</td></tr>
<tr><th colspan="3" align="left">Other</th></tr>
<tr><td><code>-q, --quiet</code></td><td></td><td>no warnings</td></tr>
</table>

**Exit status:** `0` success · `1` error · `2` invalid arguments · `3`
JavaScript time limit · `4` memory limit.

**Resources:** `<link>`, `@import`, images and fonts load from the input
file's directory and below, plus `data:` URLs. Nothing is fetched from the
network. → [docs/security.md](docs/security.md)

## Known issues

Found by our own test suites; details and plans are in
[docs/roadmap.md](docs/roadmap.md#known-issues).

- **Multi-column layout has gaps.** Text, wrappers and bordered boxes flow
  across columns and pages, and floats move with the text beside them. Bare
  text mixed with blocks inside the columns does not break between lines,
  and a positioned child turns splitting off for its column set.
- **`oklch()` and other modern colors in SVG attributes draw black**
  (`<rect fill="oklch(…)">`). Use hex or `rgb()` there; CSS colors in HTML
  are fine.
- **No `Intl` in JavaScript.** `toLocaleUpperCase('tr-TR')`,
  `localeCompare(…, 'tr')` and `Intl.NumberFormat` fail. Turkish case mapping
  in CSS works.
- **Not supported yet:** container queries and subgrid.
- **No wkhtmltopdf header/footer flags, by choice.** Put headers and footers
  in the page itself, with CSS `@page` margin boxes
  (`@bottom-center { content: counter(page) " / " counter(pages) }`).
- **No Canvas 2D, by choice.** `<canvas>` drawings stay empty. An environment
  that has a canvas can rasterize it itself; SVG charts work.
- **Long tables are slower than they should be:** about 1 s for 24 pages.
- **Extremely deep nesting parses slowly.** 20 000 nested elements take about
  1 s, but parsing time grows with the square of the depth: 80 000 levels take
  about 27 s. With JavaScript on, `--javascript-timeout` bounds it.
- **Build:** depends on patched sibling checkouts of fulgur and Blitz until the
  fulgur port (fulgur-rs/fulgur#969) and the open Blitz pull requests
  (DioxusLabs/blitz#1665, #1681) are merged.

## Building

Requirements: Rust 1.92+ and the fontconfig development files
(`fontconfig-devel` on Fedora, `libfontconfig1-dev` on Debian/Ubuntu).

Until the remaining upstream pull requests land, the build expects patched
checkouts next to this repository, at the commits pinned in
[`.github/upstream.env`](.github/upstream.env).
`scripts/upstream.sh fetch` checks them out:

```
../fulgur   github.com/bahadirdogru/fulgur, branch blitz-master  (fulgur-rs/fulgur#969)
../blitz    github.com/bahadirdogru/blitz,  branch public-layout-pass
            (Blitz b4ad543 plus DioxusLabs/blitz#1664, #1666, #1671 and
            #1677, merged upstream, and #1665 and #1681, in review)
```

```sh
scripts/upstream.sh fetch      # ../fulgur and ../blitz at the pinned commits
cargo build --release          # → target/release/bdhtml2pdf
cargo test --release           # release: Boa is slow in debug builds
bench/run.sh -n 5              # optional engines: SGH=… WK=… CHROME=…
bench/security.sh
```

Release builds use `opt-level = 3`. The release archives target the baseline
CPU of each platform. `RUSTFLAGS="-C target-cpu=native" cargo build --release`
builds for your own CPU, but on an i7-12800H it was within ±5% of the
portable build, about the run-to-run noise.

After committing and pushing a change in `../fulgur` or `../blitz`, record the
new commits with `scripts/upstream.sh pin` so CI builds against them.

[CI](.github/workflows/ci.yml) runs `cargo fmt`, `clippy` and the tests on
Linux (x86_64, aarch64), macOS and Windows. Pushing a `vX.Y.Z` tag runs
[release.yml](.github/workflows/release.yml): it builds the binaries and opens
a draft GitHub release (see the
[release procedure](docs/benchmarks/README.md#release-procedure)).

## Goals

- **Modern CSS and JavaScript first.** Use Stylo and Boa as they are, not a
  hand-picked subset.
- **Vector PDF.** Text stays text, fonts are subset, files stay small.
- **Small and single-process.** Binary under 50 MB; tens of MB of memory per
  typical document; no helper processes.
- **Safe by default.** No network; file access confined to the input
  directory; bounded execution.
- **Drop-in for wkhtmltopdf.** Same CLI for the common cases, a server mode
  and bindings for Python, Node.js, Ruby and PHP.
- **Print-ready variable data: PDF/VT-3.** Many personalized records
  (invoices, statements) in one file, each a document part, with shared
  content reused instead of repeated (ISO 16612-3, on PDF 2.0 and PDF/X-6).
- **Reuse, don't rewrite.** Upstream every change to Blitz and fulgur.
- **Prove it.** Every claim comes from a script in `bench/` that anyone can
  rerun.

→ [docs/roadmap.md](docs/roadmap.md)

## Documentation

| Document | Contents |
|---|---|
| [why.md](docs/why.md) | use cases, alternatives, and why we built this |
| [architecture.md](docs/architecture.md) | pipeline, source layout, dependencies, upstream patches, resource policy |
| [testing.md](docs/testing.md) | every test suite and how it is scored |
| [security.md](docs/security.md) | threat model, defaults, security suite |
| [bindings/](bindings/README.md) | Python, Node.js, Ruby and PHP packages, and the HTTP server |
| [benchmarks/](docs/benchmarks/README.md) | method, release procedure, results per release |
| [roadmap.md](docs/roadmap.md) | goals, status, known issues, roadmap |
| [research.md](docs/research.md) | original research notes: candidates, the Stylo blocker, early measurements |
| [index.html](docs/index.html) | the website, served by GitHub Pages from `docs/`; [samples](docs/samples) and pictures are rebuilt by `scripts/site.sh` |

## License

GPL-3.0-or-later; see [LICENSE](LICENSE). The bundled libraries keep their own
licenses (MIT, Apache-2.0, MPL-2.0); see
[docs/architecture.md](docs/architecture.md#components-and-dependencies).
Test fonts are under the SIL Open Font License
([tests/fixtures/fonts](tests/fixtures/fonts/README.md)).
