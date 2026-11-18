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
| JavaScript time | Timers run on a virtual clock bounded by `--javascript-delay`. Wall clock is bounded by `--javascript-timeout`, **between** callbacks (see Known issues). |
| Memory | No internal limit yet. Run under a cgroup or container limit. |

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
| **bdhtml2pdf** | ✅ 8/8 blocked | ✅ 3/3 blocked | ❌ `while(true)` hangs · ❌ JS memory bomb runs until killed · ⚠️ 20 000-level nesting aborts (stack) · ⚠️ timer flood stops only at the 30 s wall-clock timeout · ✅ billion laughs |
| sghtmltopdf | ✅ 8/8 | ✅ 3/3 | ✅ for the JS cases, because it has no JavaScript · ⚠️ deep nesting exits with an error |
| wkhtmltopdf | ✅ 8/8 (local file access is off by default since 0.12.6) | ❌ HTML/CSS and JavaScript (image, XHR, WebSocket) reach the internal server | ❌ billion laughs: OOM at 2 GB · ❌ deep nesting: SIGSEGV · ✅ loops are stopped after about 10 s |
| headless Chromium | ❌ 6/8 leak: `/etc/passwd` through iframe, object, navigation and meta refresh; outside CSS, JS and symlinks load | ❌ HTML/CSS, JS (fetch, XHR, Image, WebSocket) and inline SVG | ❌ `while(true)` hangs · ❌ memory bomb: OOM at 2 GB · ✅ the rest |

What this means:

- **Chromium is not safe on untrusted HTML with default flags.** Opening a
  `file://` page lets it read any local file, and it makes every network request
  the page asks for. Production setups have to add their own isolation.
- **wkhtmltopdf** still has the SSRF class of CVE-2022-35583 with default
  settings.
- **bdhtml2pdf** holds the file and network boundaries. It is **not yet safe
  against JavaScript denial of service.** Until bounded execution lands (see
  [roadmap](roadmap.md)), run it under an external timeout and memory limit, as
  the hardening advice below says.

## Hardening advice for any engine

Even with good defaults, run converters as a separate low-privilege user or
container:

- with no network egress, or an egress allow-list;
- with a read-only filesystem except the output directory;
- with a memory limit and a wall-clock timeout per conversion.

This is defense in depth. The engine's defaults are the first line, not the
only one.
