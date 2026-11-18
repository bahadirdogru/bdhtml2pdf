//! Render lifecycle: parse → run scripts → drain timers on a virtual clock → final DOM.
//!
//! Timers run on virtual time, so `setTimeout(fn, 1000)` costs no wall-clock
//! waiting. `delay` bounds how far virtual time may advance (wkhtmltopdf's
//! `--javascript-delay`); `timeout` bounds real time spent in JS.

use std::path::Path;
use std::time::{Duration, Instant};

use blitz_dom::{Document, DocumentConfig};
use blitz_vibey_script::ScriptDocument;
use url::Url;

use crate::fetch::SandboxedFetcher;

#[derive(Debug, Clone)]
pub struct JsOptions {
    /// Virtual time budget for timers after scripts run.
    pub delay: Duration,
    /// If set, rendering waits (up to `timeout` of virtual time) until `window.status` equals this.
    pub window_status: Option<String>,
    /// Hard wall-clock limit for JS execution.
    pub timeout: Duration,
    /// Print uncaught JS errors to stderr.
    pub debug: bool,
}

impl Default for JsOptions {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(200),
            window_status: None,
            timeout: Duration::from_secs(30),
            debug: false,
        }
    }
}

pub struct JsResult {
    /// Serialized DOM after scripts ran.
    pub html: String,
    pub errors: Vec<String>,
    /// Whether `window_status` was reached (always true if none was requested).
    pub status_reached: bool,
}

/// Viewport used for JS layout queries (A4 width at 96 dpi).
const VIEWPORT_WIDTH: u32 = 794;
const VIEWPORT_HEIGHT: u32 = 1123;

pub fn run_scripts(html: &str, base_dir: &Path, opts: &JsOptions) -> std::io::Result<JsResult> {
    let base_url = Url::from_directory_path(base_dir.canonicalize()?)
        .map_err(|_| std::io::Error::other("invalid base directory"))?;

    let config = DocumentConfig {
        base_url: Some(base_url.to_string()),
        viewport: Some(blitz_traits::shell::Viewport::new(
            VIEWPORT_WIDTH,
            VIEWPORT_HEIGHT,
            1.0,
            blitz_traits::shell::ColorScheme::Light,
        )),
        ..DocumentConfig::default()
    };

    let mut doc = ScriptDocument::from_html(html, config)
        .without_timer_thread()
        .with_virtual_time()
        .with_fetcher(SandboxedFetcher::new(base_dir)?);

    let wall_start = Instant::now();
    doc.execute_scripts();
    doc.poll(None);

    let virtual_start = doc.clock_now();
    let budget = if opts.window_status.is_some() { opts.timeout } else { opts.delay };
    let virtual_end = virtual_start + budget;

    let mut status_reached = opts.window_status.is_none();
    let mut errors = Vec::new();

    loop {
        if let Some(wanted) = &opts.window_status {
            if read_window_status(&mut doc).as_deref() == Some(wanted.as_str()) {
                status_reached = true;
                break;
            }
        }
        if wall_start.elapsed() > opts.timeout {
            errors.push(format!("JavaScript timeout after {:?}", opts.timeout));
            break;
        }
        match doc.next_timer_deadline() {
            Some(deadline) if deadline <= virtual_end => {
                doc.advance_clock_to(deadline);
                doc.poll(None);
            }
            _ => break,
        }
    }

    errors.extend(doc.take_js_errors());
    if opts.debug {
        for e in &errors {
            eprintln!("[js] {e}");
        }
    }

    let html = {
        let guard = doc.inner();
        let root = guard.root_element();
        let mut out = String::from("<!DOCTYPE html>");
        root.write_outer_html(&mut out);
        out
    };

    Ok(JsResult { html, errors, status_reached })
}

fn read_window_status(doc: &mut ScriptDocument) -> Option<String> {
    doc.take_messages();
    doc.eval("__blitz_send_message(String(window.status ?? ''))");
    doc.take_messages().pop()
}
