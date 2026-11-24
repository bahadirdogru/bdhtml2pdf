# bdhtml2pdf – Phase 1 research notes

This is the research log kept while choosing the architecture. Entries are
dated and record what was known at the time. Later measurements are in
[benchmarks/](benchmarks/README.md).

Date: 2026-11-18
Method: the candidate repositories were shallow-cloned, and their README,
`Cargo.toml`, license and git history were read. **Nothing was built or
measured at this stage** (there was no Rust toolchain on the machine yet). The
size and memory figures below are the projects' own claims.

## Goal

A modern successor to wkhtmltopdf:
- one binary, one process, low memory;
- vector PDF;
- JavaScript support;
- no Chromium, WebKit or Qt.

Nothing that already exists gets rewritten. The work is to combine existing
parts.

## Findings

| Project | What it does | JS | PDF | License | Last commit |
|---|---|---|---|---|---|
| [fulgur](https://github.com/fulgur-rs/fulgur) | "Modern wkhtmltopdf alternative". Blitz 0.2 + Stylo 0.8 + Taffy 0.9 + Parley 0.6 → Krilla 0.7 | **None** (an explicit non-goal in its design document) | Vector, pagination, GCPM header/footer, page numbers, font subsetting, PDF/UA, bookmarks | MIT/Apache-2.0 | 2026-11-08, very active |
| [Blitz](https://github.com/DioxusLabs/blitz) | DOM, style and layout on html5ever + Stylo + Taffy + Parley | **Yes:** the `blitz-vibey-script` package binds Boa directly to blitz-dom (DOM, events, timers, fetch); it is said to run Preact. `publish = false`, about 9.3k lines | None (raster paint through vello) | MIT/Apache-2.0 | 2026-11-18, very active, 0.3.0-beta |
| [OxiBrowser](https://github.com/project-oxi/oxibrowser) | Headless browser for AI agents. Boa + Blitz 0.3 + Stylo + Parley, CDP | Yes (Boa, Shadow DOM, fetch/XHR, Canvas 2D) | **Raster:** PNG from vello_cpu embedded in a PDF with printpdf | MIT | 2026-09-30 |
| [TypePress](https://github.com/alitrack/typepress) | Markdown/Mermaid/KaTeX pre-processing on top of fulgur | None | fulgur | MIT | 2026-06-24 |
| [Krilla](https://github.com/LaurenzV/krilla) | High-level PDF writer | – | Vector, font subsetting | MIT/Apache-2.0 | 2026-10-04 |
| [litehtml](https://github.com/litehtml/litehtml) | Lightweight HTML/CSS layout in C++ | None | None (drawing is left to the host) | BSD-3 | 2026-10-25 |
| [waka/sghtmltopdf](https://github.com/waka/sghtmltopdf) | The "waka/sg" from the specification. Pure Rust with its own CSS engine (html5ever + cssparser + selectors + Taffy + pdf-writer), meant to replace wkhtmltopdf/wicked_pdf | **None** | Vector, streaming mode, server mode, Ruby gem | MIT | v0.8.0 (the first search missed it because the name was incomplete) |

Notes:
- **Raikiri.** fulgur is developing its own layout engine, Raikiri
  (`fulgur-raikiri`, "under development"), to replace Blitz. The Blitz backend
  is the default for now.
- **Blitz versions.** fulgur uses Blitz **0.2** / Stylo **0.8**, while Blitz's
  JS package is on Blitz **0.3**. Combining them requires moving fulgur's Blitz
  adapter to 0.3.
- **OxiBrowser** produces raster PDFs (no selectable text, large files), so
  it is not suitable for the PDF side. It can serve as a reference for
  JS–DOM integration.
- **litehtml** has neither JavaScript nor PDF output, so the C++ route would
  mean writing both. That conflicts with the "do not write from scratch" rule;
  it was ruled out.

## Conclusion: proposed architecture

Almost every part already exists. Only the **glue** is missing:

```
HTML ──► blitz-html/blitz-dom (Stylo + Taffy + Parley)
            │
            ├─► blitz-vibey-script (Boa): run <script>s, mutate the DOM,
            │    drain the timer/microtask queues, wait for a "ready" signal
            │
            └─► fulgur-blitz: pagination + GCPM → Krilla → vector PDF
                         │
                 bdhtml2pdf CLI (wkhtmltopdf-compatible arguments)
```

New code to write:
1. Move fulgur's Blitz adapter to Blitz 0.3, or contribute it to fulgur
   upstream.
2. The render lifecycle:
   - run scripts;
   - bound the event loop (timeout, heap limit);
   - wait for a ready signal like `window.status` / `--window-status`;
   - layout;
   - PDF.
3. A wkhtmltopdf CLI compatibility layer: `--page-size`, `--orientation`,
   `--margin-*`, `--header-*`, `--footer-*`, `--javascript-delay`,
   `--window-status`, `--no-images` and so on.

Strategy: where possible, **depend and contribute upstream instead of
forking**. If the changes go to fulgur and Blitz as pull requests, the
maintenance burden does not stay with us.

## Risks and open questions

- **`blitz-vibey-script` is unpublished** (`publish = false`), and its name
  suggests it is experimental. It can be used as a git dependency, but its
  API may change.
- **Boa's performance and memory** with React, Vue, Chart.js and ECharts need
  measuring. If they fall short, QuickJS-NG (rquickjs) is the alternative, but
  the Blitz bindings would then have to be rewritten.
- **Canvas 2D:** no `getContext` was found in Blitz/vibey-script. Chart.js and
  ECharts need it. OxiBrowser's Canvas 2D implementation and ECharts' SVG
  renderer mode should be looked at.
- **Raikiri:** fulgur's move to Raikiri could push the Blitz backend aside
  later.
- **Flex and grid:** Taffy supports both. The "no flex/grid" statement in
  TypePress's README looks out of date and should be verified by testing.

## Blocker: fulgur and Blitz master cannot share a binary (2026-11-18)

Stylo declares `links = "servo_style_crate"`. fulgur's Stylo 0.8 and Blitz
master's Stylo 0.24 therefore cannot be linked into the same binary, and the
"serialization bridge" (plan A) is not possible in one process.

fulgur's `prep/blitz-0.3` branch (2026-07, Blitz 0.3.0-alpha.6) was never
merged. Pointing fulgur main at Blitz master (b4ad543) gives **680 compile
errors** in `fulgur-blitz`. Most are mechanical:
- `final_layout` changed from a field to a method;
- lifetime parameters;
- cssparser's `new_error`;
- Stylo's `clone_*` getters.

## Blitz vs OxiBrowser (2026-11-18)

Method:
- Script: `bench/compare.sh`.
- Same fixtures (`tests/fixtures/`), release builds, peak RSS from
  `/usr/bin/time`.
- Blitz is this repository's prototype (blitz-vibey-script, rev b4ad543).
- OxiBrowser is main, built with `--features browser`, release.

| Fixture | Blitz time / RSS | Blitz result | OxiBrowser time / RSS | OxiBrowser result |
|---|---|---|---|---|
| modern-js | 0.08 s / 25 MB | 18/19 (no `Intl`) | 0.25 s / 51 MB | 17/19 (no `dataset`, no `DocumentFragment.append`) |
| React 18 | 0.22 s / 42 MB | renders, `useEffect`/timer update works, 50 rows | 0.28 s / 68 MB | **does not render**: `React is not defined` (the UMD global is not set up) |
| Vue 3 | 0.21 s / 45 MB | renders, `mounted`/timer works, 30 `li` | 0.30 s / 72 MB | **does not render**: `SVGElement is not defined` |
| Preact | 0.09 s / 26 MB | renders | 0.17 s / 51 MB | **does not render**: `preact is not defined` |

| | Blitz prototype | OxiBrowser |
|---|---|---|
| Binary | **19.5 MB** | **68.8 MB** (the README claims about 44 MB) |
| C/C++ dependencies | none | BoringSSL (`btls-sys`); needs `g++` |
| DOM | one DOM (Blitz): JavaScript and layout share the tree | a separate JS DOM (about 31k lines), serialized to HTML and re-parsed in Blitz |
| PDF | not yet | raster (a PNG screenshot cut into pages), only through CDP |
| Other | — | no `file://`; relative `<script src>` is not resolved; `outerHTML`/`innerHTML` are `undefined` |

Note: OxiBrowser's binary also contains CDP, networking and agent features,
which could be stripped to make it smaller. The React, Vue and Preact failures
are in its JS DOM core, however, and stripping features does not fix them.

## Next steps (as of 2026-11-18)

1. Install a Rust toolchain. Build the fulgur CLI in release mode, then
   measure the binary size and the peak RSS on sample invoice and table pages.
2. Run the `blitz-vibey-script` tests in the Blitz workspace. Try them with
   Preact and a simple Chart.js (SVG/Canvas) page.
3. A small prototype: `ScriptDocument` → run scripts → hand the serialized DOM
   to fulgur (a temporary path until the adapter is ported) → PDF.

## sghtmltopdf comparison (2026-11-19)

sghtmltopdf 0.8.0 was built with `cargo install` and measured on the same
fixtures (`bench/run.sh`, `bench/security.sh`; results in `docs/benchmarks/`).
Summary:

- **Fast on static HTML.** Very light: a 12.6 MB binary, a few MB of memory,
  tens of milliseconds per page.
- **No JavaScript.** React, Vue and Preact pages come out blank or as raw
  templates.
- **Hand-written CSS subset.** It has no `@layer`, `:nth-child(of S)`,
  gradients or multi-column layout. It also has no case mapping for
  `lang="tr"`: `istanbul` → `ISTANBUL`, `IŞIK` → `işik`.
- **No subsetting for CJK fonts.** It embeds CJK fonts whole: `fonts.html`
  produced a 19 MB PDF.
- **Good security defaults.** No network, and no files outside `--allow-path`.

Conclusion: sghtmltopdf is a good option for static templates, but it is not
enough for our goal of modern JavaScript and modern CSS. Ideas worth taking
(not code): its server mode and page-by-page streaming output.
