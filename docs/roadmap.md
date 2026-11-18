# Goals and roadmap

## Goals

| Goal | Target | Status (2026-11-19) |
|---|---|---|
| Modern CSS | Everything Stylo supports; no hand-written CSS subset | 10/11 in our CSS suite; container queries are missing |
| Modern JavaScript | ES2024+ and the DOM APIs real apps use; React, Vue and Preact render | 18/19 in our JS suite; React, Vue and Preact render, timers included |
| Turkish and other languages | Correct glyphs, extractable text, language-sensitive case mapping | Full alphabet, `@font-face`, `text-transform` with `lang="tr"` correct; JS locale functions missing (no `Intl`) |
| Vector PDF | Text stays text (searchable, selectable), subset fonts, small files | Yes; 0 raster images on text pages |
| Paged media | `@page`, margin boxes, running headers/footers, page counters, `string-set`, bookmarks, repeated table headers | Inherited from fulgur |
| Small footprint | Binary < 50 MB, < 100 MB memory for typical documents | Binary 24.7 MB; 30–50 MB RSS for single-page documents |
| One process, no browser | No Chromium, WebKit or Qt; no helper processes | Yes |
| Safe defaults | No network; file access confined to the input directory; bounded time and memory | Network and file access hold in our suite. A synchronous infinite loop is **not** yet bounded (see Known issues). |
| wkhtmltopdf drop-in | Same CLI flags for the common cases | Page size, orientation, margins, title, JavaScript delay, `--window-status`. Header/footer flags not yet. |
| Measured, reproducible claims | Every release ships benchmark and security results | `bench/run.sh`, `bench/security.sh`, [benchmarks/](benchmarks/README.md) |

## Known issues

| Issue | Impact | Plan |
|---|---|---|
| A synchronous infinite loop (`while(true){}`) hangs the process. `--javascript-timeout` is only checked between timer callbacks. | DoS with untrusted HTML. The caller must enforce a timeout externally. | Use Boa's runtime limits (loop iteration / instruction budget) and interrupt from the wall-clock watchdog. |
| Unbounded JavaScript allocation runs until something external kills it. | DoS. | A heap limit for Boa and a `--max-memory` option. |
| A zero-delay timer flood (`setTimeout(f, 0)` forever) runs until the 30 s wall-clock timeout, because virtual time never advances. | 30 s of CPU per hostile page. | Cap the number of timer callbacks per render, as well as virtual time. |
| Very deep nesting. Between about 3 000 and 3 500 nested elements, rendering jumps from 0.04 s to 35–50 s. At 20 000 levels the process aborts, probably from stack exhaustion. | DoS; generated HTML is rarely this deep. | Find the non-linear step (style, layout or fulgur conversion). Add a depth limit with a clear error, and run the pipeline on a thread with a large stack. |
| Ligature glyphs lose their text in the PDF: `office` extracts as `ofce`. | Search and copy-paste break on words with fi/fl/ffl ligatures. | Map each ligature glyph to its whole cluster text in the ToUnicode CMap (fulgur → Krilla glyph-to-text ranges). Not caused by the port: fulgur's own committed upstream example PDFs show the same loss (`Ofce`, `Workfow`). |
| No `Intl` in Boa as built. | `Intl.NumberFormat`, `toLocaleUpperCase('tr-TR')` and `localeCompare(…, 'tr')` fail in JavaScript. CSS-side Turkish casing works. | Enable Boa's `intl` feature (ICU4X data) and measure the binary size cost. Possibly ship a reduced locale set. |
| Container queries | `@container` rules never match. | Blitz/Stylo integration: supply container sizes to Stylo during layout. |
| Subgrid | `grid-template-columns: subgrid` falls back. | Taffy support upstream. |
| Long documents are slower than the others. A 1000-row, 24-page invoice takes about 1 s. | Batch throughput on very long tables. | Profile pagination. Hand the DOM to fulgur directly instead of serializing and re-parsing it. |
| fulgur GCPM snapshot shifts after the port | 3 byte-level snapshots break pages one paragraph earlier or later than before | Find the line-height or metrics change in Parley 0.13. Accept new snapshots only once it is understood. |
| Builds need sibling checkouts of patched fulgur and Blitz | Not buildable from crates.io alone | Upstream pull requests (see [architecture.md](architecture.md#changes-to-upstream-projects)). |

## Roadmap

1. **Hardening.**
   - Bounded JavaScript execution: instruction budget and watchdog.
   - A memory limit option.
   - A `--allow` list for extra directories.
   - An optional, explicit network allow-list (off by default).
2. **Upstreaming.** Blitz: public `LayoutPassState`, public `svg_source`, and
   the serializer fix. fulgur: the Blitz-master port and the counter fix.
3. **Direct DOM hand-off.** Pass the post-JavaScript DOM to fulgur without
   serialization. This saves time and keeps CSSOM changes (`insertRule`,
   constructed stylesheets).
4. **wkhtmltopdf CLI parity.**
   - `--header-html`, `--footer-html`, `--header-center` and similar, mapped to
     GCPM margin boxes.
   - `--no-images`, `--print-media-type`, `--zoom`, `--dpi`.
   - TOC and outline options.
5. **Canvas 2D.** Charts (Chart.js, ECharts canvas renderer) rendered to
   vector PDF paths. The OxiBrowser implementation (MIT) is the starting point.
6. **`Intl`** with a measured size budget.
7. **Server and library modes.** A long-running HTTP mode (one process, many
   conversions, no browser pool). A C ABI and bindings for Ruby, Python, PHP
   and Node, so existing wkhtmltopdf wrappers can switch.
8. **PDF features.** PDF/A for archiving invoices. PDF/UA tagging (fulgur
   already has groundwork). Digital signatures through an external signer.
