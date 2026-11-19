# bdhtml2pdf

**A small, single-process HTML + JavaScript to vector PDF engine: a modern
successor to wkhtmltopdf.** No Chromium, no WebKit, no Qt.

bdhtml2pdf takes the HTML your application already renders and runs its
JavaScript (React, Vue, Preact, plain DOM code). It styles the page with
Firefox's CSS engine, paginates it with print CSS, and writes a vector PDF with
real, searchable text and subset fonts. All of this happens in one process
of about 25 MB, written in Rust.

> **Status: pre-release (0.1.0).** It works end to end and is measured below,
> but it still builds against patched sibling checkouts of fulgur and Blitz
> (see [Building](#building)). Read [Known issues](#known-issues) before using
> it on untrusted input.

```sh
bdhtml2pdf invoice.html invoice.pdf
bdhtml2pdf --page-size A4 --orientation Landscape --margin-top 15mm report.html report.pdf
bdhtml2pdf --window-status ready --javascript-delay 2000 dashboard.html dashboard.pdf
bdhtml2pdf app.html rendered-dom.html   # write the DOM after JavaScript instead of a PDF
```

## Why

PDF generation sits in the middle of a lot of everyday systems:

- e-learning certificates and reports;
- invoices and statements;
- analytics exports;
- worksheets and printed course material;
- tickets, labels and contracts.

The templates are almost always HTML, and increasingly that HTML is built by
JavaScript and styled with modern CSS.

The usual engines force a bad trade:

- **wkhtmltopdf** is archived (2023). Its 2012-era WebKit cannot run ES2015+
  (React, Vue and Preact render blank) or modern CSS. It has a known SSRF
  (CVE-2022-35583).
- **Headless Chromium** renders everything, but costs a 432 MB install, about
  200–340 MB per conversion across several processes, and browser pools to
  babysit. It also brings a full browser's attack surface: with default flags
  it read `/etc/passwd` and made internal network requests in our security
  suite.
- **Native PDF libraries** are fast, but every HTML template has to be
  rewritten in their API.
- **New native HTML engines** (e.g. sghtmltopdf) are small and fast, but have
  no JavaScript and only a hand-written subset of CSS.

We wanted **modern JS + modern CSS + vector PDF + small footprint** together.
Nobody shipped that, but every hard part already existed as a maintained Rust
library. So we wrote the glue instead of another engine.
→ [docs/why.md](docs/why.md) has the full reasoning and the alternatives we
measured.

**Turkish is a first-class requirement.** The full alphabet
**Ç ç Ğ ğ I ı İ i Ö ö Ş ş Ü ü** is tested in system fonts, web fonts, bold
and italic, and it round-trips through PDF text extraction. Language-sensitive
case mapping works: `text-transform: uppercase` with `lang="tr"` turns
`istanbul` into **İSTANBUL**, and `capitalize` turns `izmir` into **İzmir**.
These are the kinds of details engines get wrong.

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
```

| Component | Role |
|---|---|
| [Blitz](https://github.com/DioxusLabs/blitz) + `blitz-vibey-script` | DOM, style and layout glue, and the JavaScript ↔ DOM binding |
| [Stylo](https://github.com/servo/stylo) 0.24 | CSS cascade, the engine Firefox uses |
| [Taffy](https://github.com/DioxusLabs/taffy) / [Parley](https://github.com/linebender/parley) | block/flex/grid layout / text shaping and line breaking |
| [Boa](https://github.com/boa-dev/boa) 0.22 | JavaScript engine, pure Rust |
| [fulgur](https://github.com/fulgur-rs/fulgur) | paged media (GCPM), pagination, PDF layout (ported by us to current Blitz) |
| [Krilla](https://github.com/LaurenzV/krilla) | PDF writer |

All of it is Rust except the system fontconfig. → [docs/architecture.md](docs/architecture.md)
covers the pipeline, all dependencies with versions and licenses, and the
patches we send upstream.

## Results

Measured on 2026-11-19 on an i7-12800H with Fedora 45. Every engine ran the
same pages with its default settings (median of 5 runs; memory is the largest
process's RSS, plus the whole process tree for Chromium). The numbers below are
copied from [docs/benchmarks/2026-11-19.md](docs/benchmarks/2026-11-19.md).

| | **bdhtml2pdf** 0.1.0 | sghtmltopdf 0.8.0 | wkhtmltopdf 0.12.6.1 | Chromium 157 headless |
|---|---|---|---|---|
| Modern CSS (11 checks) | **10** | 8 | 1 | 11 |
| Modern JavaScript (19 checks) | **18** (no `Intl`) | 0 (no JS) | 0 (no ES2015+) | 18–19 |
| React 18 / Vue 3 / Preact apps | **3/3** | 0/3 | 0/3 | 3/3 |
| HTML parsing (7) | **7** | 7 | 6 | 7 |
| Turkish (15) | **12** (only JS locale functions fail) | 8 | 8 | 14 |
| Fonts (8) | **7** (ligature text) | 7 | 6 | 8 |
| 1000-row invoice: pagination and repeated headers | ✅ | ✅ | ✅ | ✅ |
| Vector output (raster images on text pages) | **0** | 0 | 0 | 0 |
| File reads and SSRF blocked (11 attacks) | **11/11** | 11/11 | 9/11 | 1/11 |
| Denial-of-service cases handled (5) | **4/5** (deep nesting aborts) | 4/5 (no JS) | 3/5 | 3/5 |
| Typical one-page document | **0.07 s · 31 MB** | 0.02 s · 10 MB | 0.24 s · 35 MB | 0.36 s · 250 MB RSS (190 MB tree) |
| React app with timers | **0.16 s · 48 MB** | — | — | 0.38 s · 249 MB |
| 24-page invoice | 1.07 s · 60 MB | 0.08 s · 16 MB | 0.37 s · 47 MB | 0.71 s · 338 MB (tree) |
| Install size | **25 MB**, one file | 13 MB | 39 MB | 432 MB |

In short:

- bdhtml2pdf is the only engine besides Chromium that runs modern JavaScript
  apps and modern CSS.
- It does so with about **1/8 of Chromium's memory and 1/17 of its install
  size**, in a single process.
- It does not leak local files or make network requests by default.
- It is still slower than the native, JavaScript-free sghtmltopdf on static
  pages, and slower than everyone on very long tables.
- Hostile JavaScript (endless loops, timer floods, memory bombs) is stopped
  by built-in limits. Very deep HTML nesting can still abort the process.

Full tables, method and raw output: [docs/benchmarks/](docs/benchmarks/README.md).

## What is tested

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

→ [docs/testing.md](docs/testing.md) · [docs/security.md](docs/security.md)

### Benchmarks are part of every release

Each release runs `bench/run.sh` (correctness, time, memory and PDF facts) and
`bench/security.sh` (hostile pages) against bdhtml2pdf, wkhtmltopdf, headless
Chromium and sghtmltopdf on the same machine. The unedited output is
committed as `docs/benchmarks/<version>.md`. Each engine runs in its own
systemd transient service, so time and memory, including all of Chromium's
processes, are measured the same way. A regression in correctness or a new
security finding blocks the release or is listed as a known issue.
→ [docs/benchmarks/README.md](docs/benchmarks/README.md#release-procedure)

## Goals

- **Modern CSS and JavaScript first.** Use Stylo and Boa as they are, not a
  hand-picked subset.
- **Vector PDF.** Text stays text, fonts are subset, files stay small.
- **Small and single-process.** Binary under 50 MB; tens of MB of memory per
  typical document; no helper processes.
- **Safe by default.** No network; file access confined to the input
  directory; bounded execution.
- **Drop-in for wkhtmltopdf.** Same CLI for the common cases, then a server
  mode and language bindings.
- **Reuse, don't rewrite.** Upstream every change to Blitz and fulgur.
- **Prove it.** Every claim comes from a script in `bench/` that anyone can
  rerun.

→ [docs/roadmap.md](docs/roadmap.md)

## Known issues

Found by our own test suites; details and plans are in [docs/roadmap.md](docs/roadmap.md#known-issues).

- **Very deep nesting.** More than about 3 000 nested elements makes rendering
  very slow (35–50 s), and 20 000 levels aborts the process.
- **Ligature text.** Words with fi/fl/ffl ligatures lose those letters in
  extracted text: `office` → `ofce`.
- **No `Intl` in JavaScript.** `toLocaleUpperCase('tr-TR')`,
  `localeCompare(…, 'tr')` and `Intl.NumberFormat` fail. Turkish case mapping
  in CSS works.
- **Not supported yet:** container queries, subgrid, Canvas 2D (`<canvas>`
  charts), and the wkhtmltopdf header/footer flags.
- **Long tables are slower than they should be:** about 1 s for 24 pages.
- **Build:** depends on patched sibling checkouts of fulgur and Blitz until the
  upstream pull requests are merged.

## Command line

| Option | Default | Meaning |
|---|---|---|
| `input` `output` | | Files, or `-` for stdin/stdout. An output ending in `.html` writes the post-JavaScript DOM. |
| `-s, --page-size` | `A4` | A3, A4, A5, B4, B5, Letter, Legal, Ledger |
| `-O, --orientation` | `Portrait` | `Portrait` or `Landscape` |
| `--page-width`, `--page-height` | | custom size, e.g. `210mm` |
| `-T/-B/-L/-R, --margin-*` | `10mm` | `mm`, `cm`, `in`, `pt`, `px` |
| `--title` | | PDF title |
| `-n, --disable-javascript` | | skip scripts |
| `--javascript-delay` | `200` | virtual milliseconds of timers to run after load |
| `--window-status` | | wait until `window.status` equals this value |
| `--javascript-timeout` | `30` | wall-clock seconds for JavaScript; a script still running after that ends the process with exit status 3 |
| `--javascript-loop-limit` | `10000000` | loop iterations per JavaScript function call; stops `while (true) {}` (`0` = unlimited) |
| `--max-memory` | `1024` | heap limit in MiB; exceeding it ends the process with exit status 4 (`0` = unlimited) |
| `--debug-javascript` | | print JavaScript errors |
| `-q, --quiet` | | no warnings |

Exit status: `0` success, `1` error, `2` invalid arguments, `3` JavaScript
time limit, `4` memory limit.

## Building

Requirements: Rust 1.89+ and fontconfig development files
(`fontconfig-devel` / `libfontconfig1-dev`).

Until the upstream pull requests land, the build expects patched checkouts
next to this repository:

```
../fulgur   fulgur-rs/fulgur, branch blitz-master (port to Blitz master + counter fix)
../blitz    DioxusLabs/blitz at b4ad543 + public LayoutPassState/svg_source/runtime limits, serializer and list-marker fixes
```

```sh
cargo build --release          # target/release/bdhtml2pdf
cargo test --release           # release: Boa is slow in debug builds
bench/run.sh -n 5              # optional engines: SGH=… WK=… CHROME=…
bench/security.sh
```

## Documentation

| Document | Contents |
|---|---|
| [docs/why.md](docs/why.md) | use cases, alternatives, and why we built this |
| [docs/architecture.md](docs/architecture.md) | pipeline, source layout, dependencies, upstream patches, resource policy |
| [docs/testing.md](docs/testing.md) | every test suite and how it is scored |
| [docs/security.md](docs/security.md) | threat model, defaults, security suite |
| [docs/benchmarks/](docs/benchmarks/README.md) | method, release procedure, results per release |
| [docs/roadmap.md](docs/roadmap.md) | goals, status, known issues, roadmap |
| [docs/arastirma.md](docs/arastirma.md) | original research notes (Turkish) |

## License

GPL-3.0-or-later; see [LICENSE](LICENSE). The bundled libraries keep their own
licenses (MIT, Apache-2.0, MPL-2.0); see [docs/architecture.md](docs/architecture.md#components-and-dependencies).
Test fonts are under the SIL Open Font License ([tests/fixtures/fonts](tests/fixtures/fonts/README.md)).
