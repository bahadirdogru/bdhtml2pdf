# Benchmarks

Every release publishes one results file in this directory, named after the
release (`v0.1.0.md`, …). A results file is the **unedited** Markdown output of
the two scripts below, plus a short "what changed" note. Numbers are never
copied in from elsewhere or rounded in our favour.

| Results | bdhtml2pdf | Date |
|---|---|---|
| [2026-11-19.md](2026-11-19.md) | 0.1.0 (pre-release, Blitz master `b4ad543`) | 2026-11-19 |

## What is compared

| Engine | Why it is in the comparison | How it is run |
|---|---|---|
| **bdhtml2pdf** | this project | `bdhtml2pdf -q in.html out.pdf` |
| **wkhtmltopdf 0.12.6.1** | the tool we are replacing | `wkhtmltopdf -q --enable-local-file-access in.html out.pdf`. The flag is needed for the local fonts and scripts every fixture uses; the security suite runs without it. |
| **headless Chromium** | the quality reference and the usual replacement | `chromium --headless --print-to-pdf --virtual-time-budget=1000 --allow-file-access-from-files …` with a dedicated profile. The security suite runs without the file-access flag. |
| **sghtmltopdf** | the closest native alternative | `sghtmltopdf in.html --allow-path <fixtures> -o out.pdf` |

Engines are optional: each one runs only when its environment variable points
to a binary.

## How it is measured

- **Isolation.** Every run is a transient systemd user service
  (`systemd-run --user --wait`). That gives every engine the same accounting,
  and a runaway page cannot affect the host (`MemoryMax`, `RuntimeMaxSec`).
- **Time.** The service's runtime, wall clock from exec to exit. We report the
  **median of N runs** (default 5) after one discarded warm-up run. The
  warm-up fills font caches and Chromium's profile, and sghtmltopdf builds its
  font index on the first run.
- **Memory.** Two numbers:
  - **cgroup peak**: the high-water mark of memory charged to the whole
    process tree, including all of Chromium's processes.
  - **max RSS**: the resident set of the largest single process (GNU time
    `%M`). RSS also counts shared library pages, and for Chromium it covers
    only the biggest process.

  We report the maximum over the N runs.
- **Correctness.** The checks described in [testing.md](../testing.md),
  evaluated on the last run's PDF.
- **PDF facts.** Pages, page size, file size, raster image count, and fonts
  as total/embedded/subset.
- **Environment.** CPU, RAM, OS, kernel and each engine's version and install
  size are printed at the top of every results file.

The scripts do not tune any engine beyond what is needed to load local files.
Page size is left at each engine's default: A4 everywhere except Chromium,
which uses Letter. This affects page counts slightly.

## Release procedure

Run on an idle machine: no builds or test suites in the background. Load
average shows up in the timings.

```sh
cargo build --release
export SGH=/path/to/sghtmltopdf WK=/path/to/wkhtmltopdf CHROME=chromium-browser
{
  echo "# $(target/release/bdhtml2pdf --version) — benchmark results"
  echo
  bench/run.sh -n 5
  bench/security.sh
} > docs/benchmarks/vX.Y.Z.md
```

Then:

1. Add a short **"Changes since the previous release"** section at the top of
   the new file: what improved, what regressed, and why.
2. Add the file to the table above and update the headline table in the
   top-level README.
3. A regression in a correctness check or a new security finding blocks the
   release, or is listed under *Known issues* in the README with an issue link.

### Getting the other engines

- **sghtmltopdf:** `cargo install sghtmltopdf --locked --root <dir>`.
- **wkhtmltopdf:** the official 0.12.6.1 packages at
  <https://github.com/wkhtmltopdf/packaging/releases>. They can be unpacked
  without installing, for example
  `rpm2cpio wkhtmltox-*.rpm | cpio -idm` into a scratch directory.
- **Chromium:** the distribution package (`chromium` / `chromium-browser`).

## Older comparison scripts

- `bench/compare.sh`: Blitz (bdhtml2pdf's JS stage) against OxiBrowser on DOM
  output. This was used to choose Blitz over OxiBrowser; see
  [../research.md](../research.md).
- `bench/compare-sgh.sh`: a quick two-engine run against sghtmltopdf. It is
  superseded by `bench/run.sh`.
