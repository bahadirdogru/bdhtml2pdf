# Goals and roadmap

## Goals

| Goal | Target | Status (2026-11-19) |
|---|---|---|
| Modern CSS | Everything Stylo supports; no hand-written CSS subset | 10/11 in our CSS suite; container queries are missing |
| Modern JavaScript | ES2024+ and the DOM APIs real apps use; React, Vue and Preact render | 18/19 in our JS suite; React, Vue and Preact render, timers included |
| Turkish and other languages | Correct glyphs, extractable text, language-sensitive case mapping | Full alphabet, `@font-face`, `text-transform` with `lang="tr"` correct; JS locale functions missing (no `Intl`) |
| Vector PDF | Text stays text (searchable, selectable), subset fonts, small files | Yes; 0 raster images on text pages |
| Paged media | `@page`, margin boxes, running headers/footers, page counters, `string-set`, bookmarks, repeated table headers | Inherited from fulgur |
| Small footprint | Binary < 50 MB, < 100 MB memory for typical documents | Binary 29 MB with image output (opt-level 3); 30–50 MB RSS for single-page documents, 34 MB with CJK fallback text |
| One process, no browser | No Chromium, WebKit or Qt; no helper processes | Yes |
| Safe defaults | No network; file access confined to the input directory; bounded time and memory | Network, file access, JavaScript time and memory, and deep nesting hold in our suite. |
| wkhtmltopdf drop-in | Same CLI flags for the common cases | Page size, orientation, margins, title, JavaScript delay, `--window-status`. No header/footer flags, by choice: headers and footers belong in the page's CSS (`@page` margin boxes). wkhtmltoimage's PNG/JPEG output with `--width`, `--height`, `--zoom`, `--quality`. |
| Measured, reproducible claims | Every release ships benchmark and security results | `bench/run.sh`, `bench/security.sh`, [benchmarks/](benchmarks/README.md) |

## Known issues

| Issue | Impact | Plan |
|---|---|---|
| A JavaScript watchdog timeout ends the whole process: Boa cannot interrupt a running script from outside. | A library user that embeds bdhtml2pdf in a long-running service loses the process on a stuck script. | Convert each document in a child process; ask Boa upstream for an interrupt hook. |
| HTML parsing is quadratic in nesting depth. html5ever looks for an open `<p>` through the whole stack of open elements on every block start tag. 20 000 levels parse in about 1 s, 40 000 in 4.5 s, 80 000 in 27 s. | DoS with extremely deep input. With JavaScript on, `--javascript-timeout` ends it (exit 3); with `-n` nothing does. | Upstream html5ever: count the open `<p>` elements so that the check is O(1) when there are none. |
| No `Intl` in Boa as built. | `Intl.NumberFormat`, `toLocaleUpperCase('tr-TR')` and `localeCompare(…, 'tr')` fail in JavaScript. CSS-side Turkish casing works. | Enable Boa's `intl` feature (ICU4X data) and measure the binary size cost. Possibly ship a reduced locale set. |
| Inside the columns, boxes break between their children: paragraphs between lines, wrappers and boxes with a border, padding or background (painted in slices, `box-decoration-break: slice`). Boxes with a shadow, transform, opacity, clipping, a fixed height or a min/max height, boxes with an edge on one side only, and bare text mixed with blocks (Blitz's anonymous block wrappers), stay whole. Floats move with the content beside them, which stays in their column. A positioned child turns splitting off for its column set. Columns taller than the page continue on the next page (filled page by page, the last page balanced). | A tall box of these kinds overflows its column instead of continuing in the next one. | Extend fulgur's flow units in `multicol_layout.rs` case by case. |
| SVG presentation attributes with `oklch()`, `color-mix()` and other CSS Color 4 values draw black. usvg parses SVG colors itself and knows only CSS Color 3. | Inline SVG charts and icons that reuse modern CSS colors lose them. | Resolve such attribute colors with Stylo before handing the SVG to usvg, or extend usvg's color parser upstream. |
| Container queries | `@container` rules never match. | Blitz/Stylo integration: supply container sizes to Stylo during layout. |
| Subgrid | `grid-template-columns: subgrid` falls back. | Taffy support upstream. |
| Long tables: a 1000-row, 24-page invoice takes 0.17 s. | Batch throughput on very long tables. | Profile again. Hand the DOM to fulgur directly instead of serializing and re-parsing it. |
| fulgur GCPM snapshot shifts after the port | 3 byte-level snapshots break pages one paragraph earlier or later than before | Find the line-height or metrics change in Parley 0.13. Accept new snapshots only once it is understood. |
| Builds need sibling checkouts of patched fulgur and Blitz | Not buildable from crates.io alone | Four of the six Blitz pull requests are merged; the serializer fix (DioxusLabs/blitz#1665), the table `border` attribute (#1681) and the fulgur port (fulgur-rs/fulgur#969) are open. See [architecture.md](architecture.md#changes-to-upstream-projects). |

## Roadmap

1. **Hardening.**
   - ✅ Bounded JavaScript execution: loop limit, HTML timer clamping, and a
     wall-clock watchdog.
   - ✅ A memory limit (`--max-memory`).
   - ✅ Very deep HTML: content below 500 levels moves up to that level, as
     in Blink and WebKit, and parsing, layout and PDF output run on a thread
     with a 256 MiB stack.
   - A `--allow` list for extra directories.
   - An optional, explicit network allow-list (off by default).
2. **Upstreaming.**
   - ✅ Blitz, merged on 2026-11-19: public `LayoutPassState` and
     `svg_source` (DioxusLabs/blitz#1664), the list-marker fix (#1666),
     `ScriptDocument::runtime_limits_mut` (#1671) and the `@import` abort
     fix (#1677).
   - Blitz serializer fix (#1665): review comments addressed, awaiting
     re-review. Blitz table `border` attribute (#1681): open. Once both
     merge, build against Blitz `main` instead of the `public-layout-pass`
     branch.
   - fulgur: the Blitz-master port and its fixes (fulgur-rs/fulgur#969,
     draft).
3. **Direct DOM hand-off.** Pass the post-JavaScript DOM to fulgur without
   serialization. This saves time and keeps CSSOM changes (`insertRule`,
   constructed stylesheets).
4. **wkhtmltopdf CLI parity.** Not the header/footer flags, by choice: the
   page sets its headers and footers with `@page` margin boxes.
   - `--no-images`, `--print-media-type`, `--zoom`, `--dpi`.
   - TOC and outline options.
5. **Canvas 2D: not planned, by choice.** An environment that has a canvas
   can rasterize it itself; we have no reason to support it. SVG charts
   work.
6. **`Intl`** with a measured size budget.
7. **Server and bindings: done.** `bdhtml2pdf serve` takes pages over HTTP
   and runs each conversion in a child process, because the memory limit and
   the JavaScript timeout are process-wide. The Python, Node.js, Ruby and PHP
   packages in `bindings/` run the binary or talk to the server; no native
   code. Next: publish them to PyPI, npm, RubyGems and Packagist; ship the
   binary inside the Python wheels and npm packages, per platform, so one
   install command is enough; wkhtmltopdf-wrapper compatibility (the
   `pdfkit`, `wicked_pdf` and `snappy` option names).
8. **PDF standards.**
   - **PDF/A** (ISO 19005, archiving): done, `--pdf-a 1b|2b|2u|2a|3b|3u|3a`;
     2b is the usual level. Every website sample passes veraPDF at 2b and
     2a. Next: embedded files for PDF/A-3 (e-invoice XML, as in
     Factur-X/ZUGFeRD), and PDF/A-4, which krilla already validates.
   - **PDF/UA** (ISO 14289, accessibility): done, `--pdf-ua` (PDF/UA-1).
     Every piece of page content is tagged in reading order or marked as an
     artifact (backgrounds, borders, `@page` headers and footers); tables
     get `/ColSpan` and `/RowSpan`, links and image links get `Link`
     elements, `aria-hidden`, `role="img"`, `role="presentation"` tables
     and decorative images (`alt=""`) are honoured. The website samples
     and a set of edge cases pass veraPDF's PDF/UA-1 profile. Next:
     PDF/UA together with PDF/A-2a (krilla 0.7 applies one validator per
     file), PDF/UA-2 (not in krilla yet), and keeping headings, lists and
     tables inside an element with fractional `opacity` as separate
     elements (krilla cannot tag inside a transparency group, so such an
     element's content is tagged as one sequence).
   - **PDF/X** (ISO 15930, print production): PDF/X-4 with an output intent
     ICC profile, CMYK colors and `TrimBox`/`BleedBox` from `@page` `bleed`
     and `marks`. krilla has no PDF/X validator yet, so this needs work
     there first.
   - **PDF/VT** (ISO 16612-2, variable data and transactional printing,
     built on PDF/X-4): many personalized records (invoices, statements) in
     one file, each marked as a document part, with shared content written
     once. Depends on PDF/X-4. Digital signatures through an external signer.
