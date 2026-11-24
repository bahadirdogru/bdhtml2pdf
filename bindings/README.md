# Language bindings

Small packages for Python, Node.js, Ruby and PHP. None of them has a native
extension or a dependency: each one either **runs the `bdhtml2pdf` program**
(one process per page) or **talks to `bdhtml2pdf serve`** over HTTP. Install
bdhtml2pdf itself from the [releases](https://github.com/bahadirdogru/bdhtml2pdf/releases)
first.

Why a process and not a library inside your interpreter: the memory limit and
the JavaScript timeout that stop hostile pages are process-wide. Inside your
PHP or Python process, a memory bomb would end that process. As a child
process, only the conversion fails.

| Language | Package | Directory | Tests |
|---|---|---|---|
| Python ≥ 3.9 | `bdhtml2pdf` (PyPI) | [python](python) | `PYTHONPATH=src python3 -m unittest discover -s tests` |
| Node.js ≥ 18 | `bdhtml2pdf` (npm) | [node](node) | `npm test` |
| Ruby ≥ 2.7 | `bdhtml2pdf` (RubyGems) | [ruby](ruby) | `ruby -Ilib test/test_bdhtml2pdf.rb` |
| PHP ≥ 8.1 | `bahadirdogru/bdhtml2pdf` (Composer) | [php](php) | `php tests/run.php` |

The packages are not published yet; install them from this directory
(`pip install ./python`, `npm install ./node`, `gem build` in `ruby`, a
Composer `path` repository for `php`).

All four work the same way:

- **Options** are the command-line flags, in snake_case (or camelCase in
  Node and PHP): `page_size: "A5"`, `margin_top: "20mm"`, `pdf_a: "2b"`,
  `disable_javascript: true`. `true` is a bare flag; `false` and `null`
  leave it out. See `bdhtml2pdf --help`.
- **Format**: `pdf` (default), `png`, `jpg` or `html` (the DOM after
  JavaScript ran).
- **Program**: `binary` option, then `$BDHTML2PDF_BINARY`, then `PATH`.
- **Files**: `base_dir` is where relative images, styles and scripts load
  from (the current directory by default).
- **Errors** raise an error with bdhtml2pdf's message; its status is the
  exit status, or the HTTP status from a server.

Tests run against a built binary: set `BDHTML2PDF_BINARY=…/bdhtml2pdf`.

## The server

```sh
bdhtml2pdf serve                       # http://127.0.0.1:7368, one job per CPU
curl --data-binary @invoice.html 'http://127.0.0.1:7368/pdf?page-size=A4&pdf-a=2b' -o invoice.pdf
```

| Request | Answer |
|---|---|
| `POST /pdf`, `/png`, `/jpg`, `/html`, page as the body | 200 and the file |
| `GET /health` | `ok bdhtml2pdf <version>` |
| unknown or server-only option | 400 |
| body over `--max-body` (32 MiB) | 413 |
| conversion error (bad option value, PDF/A refused) | 422 and the message |
| JavaScript or the whole conversion over `--timeout` (30 s) | 504 |
| over `--max-memory` (1024 MiB) | 507 |

Every conversion runs in a child process in an empty directory, so a page
cannot read the server's files, and a failing page fails alone. `--jobs`
conversions run at once; the rest wait. The server has no authentication and
listens on 127.0.0.1; put it behind a proxy if other machines need it.
