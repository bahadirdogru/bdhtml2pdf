# Why bdhtml2pdf exists

## The job

A large share of business documents are born as HTML and shipped as PDF:

| Domain | Typical documents | What they need from the engine |
|---|---|---|
| **E-learning** | completion certificates, course handouts, quiz and exam sheets, progress and grade reports, SCORM/LMS exports | per-learner generation at volume (thousands per batch), learner names in any script, charts drawn by JavaScript, reliable page breaks |
| **Invoicing and finance** | invoices, e-invoice visual copies, receipts, statements, dunning letters | exact layout, long tables that break across pages with repeated headers, currency and locale formatting, selectable/searchable text, small files for archiving and e-mail |
| **Reporting** | dashboards frozen to PDF, monthly reports, analytics exports | charts rendered by JS libraries, modern CSS layout (grid, flex, multi-column), headers/footers with page numbers |
| **Publishing and education content** | worksheets, books and booklets, catalogues | typography (font embedding, ligatures, hyphenation), print CSS (`@page`, running headers, counters) |
| **Operations** | shipping labels, tickets, contracts, HR letters | fast single-document latency inside a request, predictable memory per request |

Most of these systems already have the HTML: it is the same template the web
application renders for the browser. Converting that HTML to PDF should not
require a browser farm, and it should not require rewriting the template in a
PDF DSL.

Turkish is a first-class requirement for us. Every engine is tested with the
full Turkish alphabet — **Ç ç Ğ ğ I ı İ i Ö ö Ş ş Ü ü** — in system fonts,
web fonts, bold and italic faces. The tests also cover language-sensitive case
mapping, where `i`/`İ` and `ı`/`I` are different letters
(`text-transform: uppercase` on `istanbul` must give `İSTANBUL`, not
`ISTANBUL`). The extracted PDF text has to round-trip, so that search and
copy-paste work. See [testing.md](testing.md#turkish).

## The options we had

### wkhtmltopdf

For a decade this was the default answer: one binary, one process, a CLI that
every language wraps. It is no longer viable:

- **Unmaintained.** The GitHub repository was archived in January 2023. The last
  release is 0.12.6.1 (2020/2021). There will be no more fixes.
- **Frozen engine.** It embeds a patched QtWebKit from the early 2010s.
  - No ES2015+: classes, arrow functions, `let`/`const` and modules do not
    run, so React, Vue and Preact pages render blank.
  - No CSS custom properties, `:has()`, `:is()`, nesting, `@layer` or grid.
  - In our CSS suite it passes 1 of 11 checks
    ([benchmarks](benchmarks/2026-11-24-i7-12800h-1.0.0.md)).
- **Security.** Server-side request forgery through HTML input is a known
  issue (CVE-2022-35583). In 0.12.6 local file access was switched off by
  default, but templates that need local images or fonts are told to turn it
  back on with `--enable-local-file-access`.

### Headless Chromium (Puppeteer, Playwright, `--print-to-pdf`, Gotenberg)

Chromium produces the best output of any engine and supports the whole web
platform. For document generation at volume it costs a lot:

- **Size.** The browser install measured here is 442 MB, and container images
  are usually larger.
- **Memory.** About 190–210 MB of cgroup memory per conversion for a one-page
  document, and about 340 MB for a 25-page invoice, spread across several
  processes. See [benchmarks](benchmarks/2026-11-24-i7-12800h-1.0.0.md).
- **Latency.** A warm launch took 0.33–0.69 s per document here. A cold start
  with a fresh profile took several seconds in our trials, so production
  setups keep warm browser pools. Those pools then need health
  checks, restarts and memory-leak handling.
- **Security.** It is a full browser: network, file access, WebSocket, service
  workers. Running it on untrusted HTML means hardening flags, sandboxes and
  egress filtering, all of which are the operator's job.
- **Print fidelity.** Paged-media CSS (running headers, `string-set`,
  `counter(page)` in margin boxes, footnotes) is limited. Most of it is
  approximated with JavaScript or with the `headerTemplate` API.

### Pure document generators (gpdf, Typst, ReportLab, PDFKit and others)

These are very fast and very small, but they are not HTML engines. Every
template has to be rewritten in their API or markup, and JavaScript charts
have to be reproduced server-side. That is the right trade for a new system
whose templates live in code. It does not fit an existing web application
whose templates are HTML.

### Newer native HTML→PDF engines

We built, ran and measured these candidates. The test pages and scripts are in
this repository.

| Engine | What it is | Why it was not enough on its own |
|---|---|---|
| [sghtmltopdf](https://github.com/waka/sghtmltopdf) | Pure Rust, its own HTML/CSS engine (html5ever + cssparser + Taffy + pdf-writer), aimed at replacing wkhtmltopdf/wicked_pdf | **No JavaScript.** Its CSS coverage is hand-written, so it lacks `@layer`, `:nth-child(of S)`, gradients, multi-column and language-sensitive case mapping. Very small and fast for static templates. |
| [WeasyPrint](https://weasyprint.org/) | Python, its own CSS engine, Pango for text. Mature paged media; PDF/A, PDF/UA and PDF/X | **No JavaScript.** Needs Python and Pango. Five to fourteen times slower than bdhtml2pdf on the static pages of [our run](benchmarks/2026-11-24-i7-12800h-1.0.0.md). The better choice for static HTML that has to be PDF/X today. |
| [fulgur](https://github.com/fulgur-rs/fulgur) | Rust: Blitz (Stylo + Taffy + Parley) → Krilla. Vector PDF, GCPM paged media, bookmarks, PDF/UA | **No JavaScript** (a stated non-goal). Pinned to an older Blitz/Stylo, so it cannot share a DOM with Blitz's JavaScript runtime. |
| [Blitz](https://github.com/DioxusLabs/blitz) | Rust browser engine: html5ever, Firefox's Stylo, Taffy, Parley. Has `blitz-vibey-script`, a Boa-based DOM binding | No PDF output. |
| [OxiBrowser](https://github.com/project-oxi/oxibrowser) | Headless browser for AI agents: Boa + Blitz | Its PDF is a raster screenshot (no selectable text, large files). Its separate JS DOM failed to run React, Vue and Preact in our tests, and the binary is 68.8 MB. |
| [litehtml](https://github.com/litehtml/litehtml) | C++ HTML/CSS layout | No JavaScript, no PDF. We would have had to write both. |

The detailed research log is in [research.md](research.md).

## The decision

None of the candidates covered **modern JavaScript + modern CSS + vector PDF +
small footprint** together. Between them, though, every hard part already
existed as a maintained Rust library:

- **CSS:** Stylo, the style engine from Firefox. Modern selectors, cascade
  layers, nesting, custom properties, `color-mix()`, `oklch()` and container
  queries come with it; we do not reimplement them.
- **Layout:** Taffy (block, flex, grid) and Parley (text shaping and line
  breaking, through HarfRust and Skrifa).
- **JavaScript:** Boa, an ES2024+ engine in pure Rust. Blitz's
  `blitz-vibey-script` binds it directly to the DOM that Stylo and Taffy use,
  so there is one DOM, not a JS DOM serialized into a layout DOM.
- **Paged media and PDF:** fulgur's pagination and GCPM implementation, and
  Krilla's vector PDF writer with font subsetting.

So we wrote **glue, not an engine**:

- the render lifecycle: run scripts, drain timers on a virtual clock, wait for
  a ready signal;
- a sandboxed resource loader;
- a wkhtmltopdf-compatible command line;
- a port of fulgur to the current Blitz, so that JavaScript and PDF layout
  share the same DOM and the same Stylo.

Each change to an upstream library is kept small and goes back as an upstream
pull request rather than living on as a fork.

The rule we hold ourselves to: **we do not ship a claim we have not measured.**
Every comparison in this repository comes from a script in `bench/` that
anyone can rerun on their own machine. The results are published for every
release (see [benchmarks/README.md](benchmarks/README.md)).
