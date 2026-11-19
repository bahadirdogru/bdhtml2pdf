//! wkhtmltopdf- and wkhtmltoimage-compatible command line.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use bdhtml2pdf::depth::with_large_stack;
use bdhtml2pdf::limits::{EXIT_JS_TIMEOUT, LimitedAlloc, set_memory_limit};
use bdhtml2pdf::page::{MIN_SHRINK_SCALE, NAMED_SIZES, SMART_SHRINK_SCALE, length_pt, named_size};
use bdhtml2pdf::{ImageFormat, ImageOptions, JsOptions, JsTimeout, PageOptions};
use clap::Parser;

#[global_allocator]
static ALLOC: LimitedAlloc = LimitedAlloc;

#[derive(Parser)]
#[command(
    name = "bdhtml2pdf",
    version,
    about = "HTML + JavaScript to PDF or image"
)]
struct Cli {
    /// Input HTML file ("-" for stdin)
    input: String,
    /// Output file ("-" for stdout). The extension picks the format: .pdf,
    /// .png, .jpg/.jpeg, or .html (the DOM after JavaScript ran).
    output: String,
    /// Output format, overriding the extension: pdf, png, jpg, html
    #[arg(short = 'f', long)]
    format: Option<String>,

    /// Page size: A3, A4, A5, B4, B5, Letter, Legal, Ledger
    #[arg(short = 's', long, default_value = "A4")]
    page_size: String,
    /// Portrait or Landscape
    #[arg(short = 'O', long, default_value = "Portrait")]
    orientation: String,
    /// Custom page width (e.g. 210mm); requires --page-height
    #[arg(long, requires = "page_height")]
    page_width: Option<String>,
    /// Custom page height (e.g. 297mm); requires --page-width
    #[arg(long, requires = "page_width")]
    page_height: Option<String>,

    #[arg(short = 'T', long, default_value = "10mm")]
    margin_top: String,
    #[arg(short = 'B', long, default_value = "10mm")]
    margin_bottom: String,
    #[arg(short = 'L', long, default_value = "10mm")]
    margin_left: String,
    #[arg(short = 'R', long, default_value = "10mm")]
    margin_right: String,

    /// PDF title
    #[arg(long)]
    title: Option<String>,
    /// Print every page at 80%, as wkhtmltopdf does by default, for pages
    /// written for wkhtmltopdf; content still wider is shrunk to fit
    #[arg(long, conflicts_with = "disable_smart_shrinking")]
    enable_smart_shrinking: bool,
    /// Never shrink content wider than the page (by default it is scaled
    /// down, to at most half size, to fit the page width)
    #[arg(long)]
    disable_smart_shrinking: bool,

    /// Image: viewport width in pixels
    #[arg(long, default_value_t = 1024)]
    width: u32,
    /// Image: height in pixels (0 = the whole page)
    #[arg(long, default_value_t = 0)]
    height: u32,
    /// Zoom factor. PDF: content scale (2 = twice as large, as in
    /// wkhtmltopdf); image: device pixels per CSS pixel
    #[arg(long, default_value_t = 1.0)]
    zoom: f32,
    /// Image: JPEG quality, 1–100
    #[arg(long, default_value_t = 94, value_parser = clap::value_parser!(u8).range(1..=100))]
    quality: u8,

    /// Do not run JavaScript
    #[arg(short = 'n', long)]
    disable_javascript: bool,
    /// Virtual milliseconds to let JS timers run after load
    #[arg(long, default_value_t = 200)]
    javascript_delay: u64,
    /// Wait until window.status equals this value
    #[arg(long)]
    window_status: Option<String>,
    /// Wall-clock limit for JavaScript, in seconds
    #[arg(long, default_value_t = 30)]
    javascript_timeout: u64,
    /// Maximum loop iterations in one JavaScript function call; stops
    /// `while (true) {}` with an error (0 = unlimited)
    #[arg(long, default_value_t = bdhtml2pdf::lifecycle::DEFAULT_LOOP_LIMIT)]
    javascript_loop_limit: u64,
    /// Heap limit in MiB for the whole conversion; exit status 4 when
    /// exceeded (0 = unlimited)
    #[arg(long, default_value_t = 1024)]
    max_memory: u64,
    /// Print JavaScript errors
    #[arg(long)]
    debug_javascript: bool,
    /// Suppress warnings
    #[arg(short = 'q', long)]
    quiet: bool,
}

fn page_options(cli: &Cli) -> Result<PageOptions, String> {
    // A PDF is laid out on a page `1 / zoom` times the paper: bound it so a
    // tiny zoom cannot ask for a huge layout.
    let zoom = cli.zoom;
    if !(0.1..=10.0).contains(&zoom) {
        return Err(format!("invalid zoom {zoom} (expected 0.1 to 10)"));
    }
    let (mut width, mut height) = match (&cli.page_width, &cli.page_height) {
        (Some(w), Some(h)) => (length_pt(w)?, length_pt(h)?),
        _ => named_size(&cli.page_size).ok_or_else(|| {
            format!(
                "unknown page size {:?} (expected one of {NAMED_SIZES})",
                cli.page_size
            )
        })?,
    };
    match cli.orientation.to_ascii_lowercase().as_str() {
        "portrait" => {}
        "landscape" => std::mem::swap(&mut width, &mut height),
        o => {
            return Err(format!(
                "unknown orientation {o:?} (expected Portrait or Landscape)"
            ));
        }
    }
    Ok(PageOptions {
        width,
        height,
        margin_top: length_pt(&cli.margin_top)?,
        margin_right: length_pt(&cli.margin_right)?,
        margin_bottom: length_pt(&cli.margin_bottom)?,
        margin_left: length_pt(&cli.margin_left)?,
        scale: zoom
            * if cli.enable_smart_shrinking {
                SMART_SHRINK_SCALE
            } else {
                1.0
            },
        shrink_to_fit: (!cli.disable_smart_shrinking).then_some(MIN_SHRINK_SCALE * zoom),
    })
}

enum Output {
    Pdf,
    Html,
    Image(ImageFormat),
}

fn output_format(cli: &Cli) -> Result<Output, String> {
    let name = match &cli.format {
        Some(f) => f.to_ascii_lowercase(),
        None => Path::new(&cli.output)
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default(),
    };
    Ok(match name.as_str() {
        "png" => Output::Image(ImageFormat::Png),
        "jpg" | "jpeg" => Output::Image(ImageFormat::Jpeg),
        "html" | "htm" => Output::Html,
        "pdf" => Output::Pdf,
        _ if cli.format.is_none() => Output::Pdf,
        f => {
            return Err(format!(
                "unknown format {f:?} (expected pdf, png, jpg or html)"
            ));
        }
    })
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    set_memory_limit(match cli.max_memory {
        0 => None,
        mib => Some(usize::try_from(mib.saturating_mul(1 << 20)).unwrap_or(usize::MAX)),
    });
    bdhtml2pdf::prefs::init();
    let page = page_options(&cli)?;
    let output = output_format(&cli)?;
    let image = ImageOptions {
        width: cli.width,
        height: (cli.height > 0).then_some(cli.height),
        zoom: cli.zoom,
        format: match output {
            Output::Image(f) => f,
            _ => ImageFormat::Png,
        },
        quality: cli.quality,
    };

    let (html, base_dir) = if cli.input == "-" {
        let mut s = String::new();
        std::io::Read::read_to_string(&mut std::io::stdin(), &mut s)?;
        (s, std::env::current_dir()?)
    } else {
        let path = PathBuf::from(&cli.input);
        let html = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        let dir = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        (html, dir.to_path_buf())
    };

    let dom_html = if cli.disable_javascript {
        with_large_stack(|| bdhtml2pdf::depth::flatten_html(html))
    } else {
        let js = JsOptions {
            delay: Duration::from_millis(cli.javascript_delay),
            window_status: cli.window_status.clone(),
            timeout: Duration::from_secs(cli.javascript_timeout),
            debug: cli.debug_javascript,
            loop_limit: (cli.javascript_loop_limit > 0).then_some(cli.javascript_loop_limit),
            viewport: match output {
                Output::Image(_) => (
                    image.width,
                    image
                        .height
                        .unwrap_or(bdhtml2pdf::image::DEFAULT_VIEWPORT_HEIGHT),
                ),
                // Scripts see the page as wide as it is laid out.
                _ => {
                    let (w, h) = bdhtml2pdf::lifecycle::A4_VIEWPORT;
                    (
                        (w as f32 / page.scale) as u32,
                        (h as f32 / page.scale) as u32,
                    )
                }
            },
        };
        let res = bdhtml2pdf::run_scripts_bounded(html, base_dir.clone(), js)??;
        if !cli.quiet {
            if !res.status_reached {
                eprintln!("warning: window.status was not reached before the timeout");
            }
            if !cli.debug_javascript && !res.errors.is_empty() {
                eprintln!(
                    "warning: {} JavaScript error(s); use --debug-javascript to show them",
                    res.errors.len()
                );
            }
        }
        res.html
    };

    let out = match output {
        Output::Html => dom_html.into_bytes(),
        Output::Pdf => with_large_stack(|| {
            bdhtml2pdf::render_pdf(&dom_html, &base_dir, &page, cli.title.as_deref())
        })?,
        Output::Image(_) => {
            let img = with_large_stack(|| bdhtml2pdf::render_image(&dom_html, &base_dir, &image))?;
            if img.truncated && !cli.quiet {
                eprintln!(
                    "warning: the page was cut off at {} pixels (use --height or a smaller --zoom)",
                    img.height
                );
            }
            img.bytes
        }
    };
    if cli.output == "-" {
        std::io::Write::write_all(&mut std::io::stdout(), &out)?;
    } else {
        std::fs::write(&cli.output, out)?;
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            if e.is::<JsTimeout>() {
                // The JavaScript thread is still running; end the process
                // without waiting for it.
                std::process::exit(EXIT_JS_TIMEOUT);
            }
            ExitCode::FAILURE
        }
    }
}
