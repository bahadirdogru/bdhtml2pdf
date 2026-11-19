# Architecture

## Pipeline

```
             input.html (+ files under its directory)
                              │
                              ▼
 ┌────────────────────────────────────────────────────────────┐
 │ blitz-html (html5ever)  →  blitz-dom  ←→  Boa (JavaScript)  │   one DOM, one Stylo
 │   Stylo 0.24 (CSS cascade)   Taffy (layout)   Parley (text) │
 │                                                              │
 │   lifecycle.rs: execute <script>s → drain timers on a        │
 │   virtual clock (--javascript-delay) → optional              │
 │   window.status wait → wall-clock timeout                    │
 └────────────────────────────────────────────────────────────┘
                              │  final DOM
                              ▼
 ┌────────────────────────────────────────────────────────────┐
 │ fulgur (ported to Blitz master)                             │
 │   GCPM: @page, margin boxes, running elements, string-set,  │
 │   counters, bookmarks · pagination · fragmentation          │
 │   (tables, multi-column, break-*) · SVG via usvg            │
 └────────────────────────────────────────────────────────────┘
                              │  page tree
                              ▼
            Krilla → vector PDF (subset fonts, text as text)
```

Everything runs in **one process** and **one thread of control**. There is no
browser, no IPC and no network stack.

Today the final DOM is handed to fulgur as serialized HTML and parsed again
with the same Stylo. This costs a few milliseconds and loses only rules added
through the CSSOM (`insertRule`, constructed stylesheets). Passing the DOM over
directly is on the [roadmap](roadmap.md).

## Source layout

| File | Role |
|---|---|
| `src/main.rs` | wkhtmltopdf- and wkhtmltoimage-compatible CLI: `input output`, `--format`, `--width`, `--height`, `--zoom`, `--quality`, `--page-size`, `--orientation`, `--page-width/height`, `--margin-*`, `--title`, `--disable-javascript`, `--javascript-delay`, `--window-status`, `--javascript-timeout`, `--javascript-loop-limit`, `--max-memory`, `--debug-javascript`, `-q`. The output extension picks PDF, PNG, JPEG or the post-JavaScript DOM (`.html`). |
| `src/lifecycle.rs` | Render lifecycle on `ScriptDocument`: no timer thread, **virtual time**. A `setTimeout(fn, 5000)` costs no real waiting, and the result does not depend on machine speed. Also sets the loop limit, clamps nested zero-delay timers as browsers do, and runs JavaScript on a worker thread under a wall-clock watchdog. |
| `src/depth.rs` | Flattens the DOM below 500 nesting levels (as Blink and WebKit do), and runs work on a thread with a 256 MiB stack. |
| `src/limits.rs` | Counting global allocator: ends the process with exit status 4 above `--max-memory`. |
| `src/fetch.rs` | Script loader sandbox: `data:` URLs and `file:` URLs **inside the input directory** only. Paths are canonicalized first, so `..` and symlinks cannot escape. No network. |
| `src/image.rs` | PNG/JPEG output: lays the DOM out at `--width` with `screen` media, loads local resources through fulgur's sandboxed loader, paints with `blitz-paint` on `vello_cpu`, encodes with `image`. Pages taller than 32 768 pixels are cut off with a warning. |
| `src/pdf.rs` | Calls fulgur with page size, margins, title and a base path that confines `<link>`, `<img>` and `@import` to the input directory. |
| `src/page.rs` | Page sizes (A3–A5, B4, B5, Letter, Legal, Ledger) and length parsing (`mm`, `cm`, `in`, `pt`, `px`). |
| `src/prefs.rs` | Turns on Stylo features that Blitz leaves off: `:has()`, `:nth-child(… of S)`, container queries. |

## Components and dependencies

All of them are Rust. The only native library is the system **fontconfig**,
used to find installed fonts.

| Layer | Crate | Version | Upstream | License |
|---|---|---|---|---|
| HTML parsing | html5ever | 0.40 | Servo | MIT/Apache-2.0 |
| DOM, style and layout glue | blitz-dom, blitz-html, blitz-traits | master `b4ad543` + local patches | DioxusLabs/blitz | MIT/Apache-2.0 |
| CSS cascade | stylo | 0.24 | Servo/Firefox | MPL-2.0 |
| Selectors, CSS parsing | selectors 0.43, cssparser 0.38 | | Servo | MPL-2.0 |
| Box layout | taffy | 0.15 (git `a3997d5`) | DioxusLabs/taffy | MIT |
| Text layout | parley, fontique | 0.13 | linebender | MIT/Apache-2.0 |
| Shaping, font reading | harfrust 0.15, skrifa | | HarfBuzz port, Google Fonts | MIT/Apache-2.0 |
| JavaScript | boa_engine | 0.22 | boa-dev | MIT/Unlicense |
| JS ↔ DOM binding | blitz-vibey-script | master | DioxusLabs/blitz | MIT/Apache-2.0 |
| Pagination, GCPM, PDF layout | fulgur | 0.41 (`blitz-master` port) | fulgur-rs | MIT/Apache-2.0 |
| PDF writer | krilla, krilla-svg | 0.7 | LaurenzV | MIT/Apache-2.0 |
| Image output | blitz-paint, anyrender 0.15, vello_cpu 0.4, image 0.25 | | DioxusLabs, linebender, image-rs | MIT/Apache-2.0 |
| SVG | usvg / resvg | 0.45 (PDF), 0.50 (DOM) | linebender | Apache-2.0/MIT |
| CLI | clap | 4 | | MIT/Apache-2.0 |

`Cargo.lock` pins 446 crates. Release builds use `lto = "fat"`,
`codegen-units = 1`, `panic = "abort"` and `strip = true`.

### Why one Stylo matters

Stylo declares `links = "servo_style_crate"`, so **a binary can contain only one
Stylo**. fulgur upstream is on Blitz 0.2 / Stylo 0.8, and Blitz's JavaScript
runtime is on Blitz 0.3 / Stylo 0.24. Using both meant either two processes
or porting fulgur. We ported fulgur. The port took 680 compile errors to
zero, and all 2109 fulgur-blitz unit tests pass.

## Changes to upstream projects

These changes currently live in sibling checkouts (`../fulgur` on branch
`blitz-master`, `../blitz` on `public-layout-pass`). They are referenced from
`Cargo.toml` through `path` and `[patch]` entries, and will be sent upstream as
pull requests.

| Project | Change | Why |
|---|---|---|
| Blitz | `LayoutPassState` public | fulgur relays out fixed and paginated boxes with Blitz's Taffy tree. |
| Blitz | `Node::svg_source` public | fulgur re-parses inline SVG with the usvg version Krilla uses. |
| Blitz | `ScriptDocument::runtime_limits_mut` | Lets the embedder set Boa's loop-iteration limit, so `while (true) {}` ends instead of hanging. |
| Blitz | Serializer: HTML elements are never written as `<x />` | `<i></i>` was serialized as `<i />`, which HTML parses as an *open* `<i>`. After a JavaScript round trip, the rest of the page became italic or was swallowed. A regression test is added. |
| fulgur | Port to Blitz master / Stylo 0.24 / Parley 0.13 / Taffy git | New network-handler model, layout data only on element nodes, Stylo getter renames, Parley run indices, Taffy's new `LayoutOutput` fields. |
| fulgur | Counter attributes in the null namespace, set through the DOM mutator | Blitz 0.3 matches attribute namespaces, so `counter()` / `counters()` in `::before` rendered empty. |
| Blitz | Outside list markers keep `white-space: pre` (CSS Lists 3 UA style) | Parley 0.13 collapsed the trailing space of `"1. "`, so numbered markers touched the item text (`1.Author`). |
| fulgur | A captioned table's percentage width moves to the caption wrapper | The `fit-content` caption wrapper made `width:100%` tables shrink to their content width under Blitz 0.3. |

## Resource policy

| Resource | Default |
|---|---|
| `<script src>`, `fetch()` | `data:` and files under the input directory; everything else is refused. |
| `<link>`, `@import`, `<img>`, fonts | Files under the input directory (fulgur's `base_path`); remote URLs are not fetched. |
| Network | None. There is no HTTP client in the binary. |
| JavaScript time | Virtual-time budget (`--javascript-delay`, default 200 ms), nested-timer clamping, 10 million loop iterations per call, and a wall-clock watchdog (`--javascript-timeout`, default 30 s; exit status 3). |
| Memory | `--max-memory`, default 1024 MiB of heap; exit status 4. |

The [security suite](security.md) checks these defaults, and checks what
happens where they do not yet hold.
