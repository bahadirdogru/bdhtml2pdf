# Security

HTML-to-PDF services are a classic attack surface. The HTML often contains
user-controlled data: a learner's name on a certificate, an address on an
invoice, a free-text field in a report. An engine that behaves like a browser
will do what browsers do with that HTML:

- fetch URLs, which is **SSRF** against internal services and cloud metadata
  endpoints;
- read `file://` URLs, which **leaks local files** such as credentials,
  `/etc/passwd` or application config;
- run scripts without limit, which is **denial of service**.

wkhtmltopdf's SSRF (CVE-2022-35583) and the many write-ups on exploiting
headless-Chrome PDF services come from exactly this.

## Threat model

- **Attacker:** controls part or all of the HTML passed to the engine, and
  knows which engine is used.
- **Assets:**
  - files on the host outside the template directory;
  - services reachable from the host (localhost, the internal network,
    `169.254.169.254`);
  - CPU and memory of the host.
- **Out of scope:**
  - memory-safety bugs in the engines themselves. bdhtml2pdf is Rust with no
    C/C++ in the rendering path except fontconfig, but we do not fuzz yet;
  - the content of the PDF being misleading. An attacker who controls the HTML
    controls the document.

## bdhtml2pdf's defaults

| Surface | Default |
|---|---|
| Network | No HTTP client is linked into the binary. `http(s)://`, `ws://` and `ftp://` resources are not fetched by HTML, CSS, SVG or JavaScript. |
| Local files | Only files under the directory of the input HTML, after canonicalization, so `..` and symlinks cannot escape. `file:///etc/passwd` is refused. With stdin input, the current directory is the root. |
| Frames and navigation | `<iframe>`, `<object>` and `<embed>` do not load documents. Navigation (`location.href`, meta refresh) does not replace the document. |
| JavaScript loops | At most `--javascript-loop-limit` (10 million) loop iterations per function call. `while (true) {}` ends with an error that `try`/`catch` cannot intercept, and the page renders with the DOM as it is. |
| JavaScript timers | Timers run on a virtual clock bounded by `--javascript-delay`. As in browsers, timers nested more than 5 deep wait at least 4 ms, so a `setTimeout(f, 0)` flood uses up the budget instead of spinning. |
| JavaScript wall clock | Scripts run on a worker thread. If they are still running `--javascript-timeout` (30 s) + 2 s after the start, the process ends with exit status 3. This catches what the loop limit cannot see: endless promise chains, catastrophic regular expressions, slow built-ins. |
| Memory | Heap allocations are counted. Above `--max-memory` (1024 MiB) the process ends with exit status 4 and a clear message, before the operating system's OOM handling. |

## Test suite

`bench/security.sh` runs every engine with its **default** settings, without
hardening flags, against the pages in `tests/security/`:

- Files the page must not reach are placed in a sibling `outside/` directory,
  and each one prints a `LEAKED-*` marker if it is used.
- A throwaway HTTP server on `127.0.0.1` stands in for an internal service and
  logs every request.
- Each run is a systemd transient service limited to 2 GB of memory and a
  fixed wall time, so a hostile page cannot affect the host.

Pages that attack through JavaScript are written in ES5. An engine with an old
JavaScript engine, such as wkhtmltopdf, therefore runs the attack instead of
"passing" because of a syntax error.

A case **fails** for an engine when the PDF contains `/etc/passwd` content or
a `LEAKED-*` marker, when the HTTP server logs a request, or when the engine is
killed by the time or memory limit.

| Case | Attack |
|---|---|
| `file-iframe`, `file-object` | `<iframe>`/`<object>`/`<embed>` of `file:///etc/passwd` |
| `js-fetch-file` | `fetch()` and synchronous XHR of `file:///etc/passwd` and `../outside/secret.txt` |
| `js-navigate`, `meta-refresh` | replace the document with `file:///etc/passwd` |
| `css-escape` | `<link>` and `@import` of `../outside/*.css` |
| `script-escape` | `<script src="../outside/leak.js">` |
| `symlink-escape` | symlinks inside the input directory that point outside |
| `ssrf-html` | `<img>`, `<iframe>`, `<link>`, CSS `background-image`, `@font-face` to `127.0.0.1` |
| `ssrf-js` | `fetch`, XHR, `sendBeacon`, `new Image()`, `WebSocket` to `127.0.0.1` |
| `ssrf-svg` | SVG `<image href>` inline and inside a `data:` SVG |
| `dos-js-loop` | `while (true) {}` |
| `dos-timer-flood` | `setInterval(…, 0)` and self-rescheduling `setTimeout(…, 0)` forever |
| `dos-memory` | unbounded allocation from JavaScript |
| `dos-svg-entities` | "billion laughs" XML entity expansion in SVG |
| `dos-deep-nesting` | 20 000 nested `<div>` elements |

## Results

The latest results are in [benchmarks/2026-11-19.md](benchmarks/2026-11-19.md#security-results).
Summary:

| Engine (defaults) | File reads and escapes (8) | SSRF (3) | Denial of service (5) |
|---|---|---|---|
| **bdhtml2pdf** | ✅ 8/8 blocked | ✅ 3/3 blocked | ✅ `while(true)` stopped in 0.15 s · ✅ JS memory bomb stopped in 3.8 s at 47 MB · ✅ timer flood ends in 0.07 s · ✅ billion laughs · ⚠️ 20 000-level nesting aborts (stack) |
| sghtmltopdf | ✅ 8/8 | ✅ 3/3 | ✅ for the JS cases, because it has no JavaScript · ⚠️ deep nesting exits with an error |
| wkhtmltopdf | ✅ 8/8 (local file access is off by default since 0.12.6) | ❌ HTML/CSS and JavaScript (image, XHR, WebSocket) reach the internal server | ❌ billion laughs: OOM at 2 GB · ❌ deep nesting: SIGSEGV · ✅ loops are stopped after about 10 s |
| headless Chromium | ❌ 6/8 leak: `/etc/passwd` through iframe, object, navigation and meta refresh; outside CSS, JS and symlinks load | ❌ HTML/CSS, JS (fetch, XHR, Image, WebSocket) and inline SVG | ❌ `while(true)` hangs · ❌ memory bomb: OOM at 2 GB · ✅ the rest |

What this means:

- **Chromium is not safe on untrusted HTML with default flags.** Opening a
  `file://` page lets it read any local file, and it makes every network request
  the page asks for. Production setups have to add their own isolation.
- **wkhtmltopdf** still has the SSRF class of CVE-2022-35583 with default
  settings.
- **bdhtml2pdf** holds the file and network boundaries, and bounds hostile
  JavaScript in time and memory by default. Very deep HTML nesting can still
  abort the process (see [roadmap](roadmap.md#known-issues)), so keep the
  hardening advice below for untrusted input.

## JavaScript limits in detail

| Attack | What stops it | Result |
|---|---|---|
| `while (true) {}`, also inside `try`/`catch` | Boa's loop-iteration limit (an uncatchable engine error) | script ends, page renders, error reported with `--debug-javascript` |
| JavaScript allocation in a loop | the loop limit, usually first | as above |
| Allocation without loops (`new Array(4e8).fill(…)`) | `--max-memory` | exit status 4 |
| `setInterval(f, 0)`, self-rescheduling `setTimeout(f, 0)` | HTML timer clamping on the virtual clock | ends when `--javascript-delay` is used up |
| Endless microtasks (`Promise.resolve().then(f)` chains), catastrophic regular expressions | wall-clock watchdog | exit status 3 after `--javascript-timeout` + 2 s |

The limits are covered by `tests/js_limits.rs` (`cargo test --release`). Its
last case checks that heavy but finite scripts run unchanged: a million
iterations in one call, thousands of calls with loops, and short timer chains.

A watchdog timeout ends the whole process, because Boa cannot interrupt a
running script from outside. A long-running service that embeds the library
should therefore convert each document in a child process.

## Hardening advice for any engine

Even with good defaults, run converters as a separate low-privilege user or
container:

- with no network egress, or an egress allow-list;
- with a read-only filesystem except the output directory;
- with a memory limit and a wall-clock timeout per conversion.

This is defense in depth. The engine's defaults are the first line, not the
only one.
