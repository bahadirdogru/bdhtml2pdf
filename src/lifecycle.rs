//! Render lifecycle: parse → run scripts → drain timers on a virtual clock → final DOM.
//!
//! Timers run on virtual time, so `setTimeout(fn, 1000)` costs no wall-clock
//! waiting. `delay` bounds how far virtual time may advance (wkhtmltopdf's
//! `--javascript-delay`); `timeout` bounds real time spent in JS.
//!
//! Hostile pages are bounded in four ways:
//! - Loops: Boa's loop-iteration limit stops `while (true) {}` with an error
//!   that JavaScript cannot catch, and rendering continues with the DOM as it is.
//! - Timer floods: as in browsers, timers nested more than
//!   [`TIMER_NESTING_LEVEL`] deep wait at least [`MIN_NESTED_TIMER_DELAY`]. A
//!   `setTimeout(f, 0)` chain therefore uses up the virtual-time budget
//!   instead of spinning at the same instant.
//! - Wall clock: [`run_scripts_bounded`] runs JavaScript on a worker thread.
//!   If a script is stuck in a way the loop limit does not see (endless
//!   microtasks, catastrophic regular expressions), the caller gets
//!   [`JsTimeout`] and can end the process.
//! - Memory: see [`crate::limits`].
//!
//! The final DOM is flattened below [`crate::depth::MAX_NESTING_DEPTH`].

use std::path::{Path, PathBuf};
use std::sync::mpsc;
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
    /// Maximum loop iterations in one JavaScript function call. `None` is unlimited.
    pub loop_limit: Option<u64>,
}

impl Default for JsOptions {
    fn default() -> Self {
        Self {
            delay: Duration::from_millis(200),
            window_status: None,
            timeout: Duration::from_secs(30),
            debug: false,
            loop_limit: Some(DEFAULT_LOOP_LIMIT),
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

/// Default for [`JsOptions::loop_limit`]. Boa counts iterations of every loop
/// in one function call together. Ten million is far more than page scripts
/// run in a single call, and Boa gets there in a few seconds.
pub const DEFAULT_LOOP_LIMIT: u64 = 10_000_000;

/// HTML timer nesting level after which timeouts are clamped
/// (HTML Living Standard, "timer initialisation steps").
pub const TIMER_NESTING_LEVEL: u32 = 5;
/// The clamped minimum timer delay.
pub const MIN_NESTED_TIMER_DELAY: Duration = Duration::from_millis(4);

/// Extra wall-clock time [`run_scripts_bounded`] waits after `timeout`, so
/// that the regular timeout check between timer callbacks can finish first.
const WATCHDOG_GRACE: Duration = Duration::from_secs(2);

/// JavaScript was still running when the wall-clock limit expired.
#[derive(Debug)]
pub struct JsTimeout(pub Duration);

impl std::fmt::Display for JsTimeout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "JavaScript did not finish within {:?} (see --javascript-timeout)",
            self.0
        )
    }
}

impl std::error::Error for JsTimeout {}

/// Runs [`run_scripts`] on a worker thread and waits at most
/// `opts.timeout` plus a short grace period.
///
/// On [`JsTimeout`] the worker thread is still running and cannot be
/// stopped. The CLI ends the process at that point. A long-running host
/// should run conversions in child processes instead.
pub fn run_scripts_bounded(
    html: String,
    base_dir: PathBuf,
    opts: JsOptions,
) -> Result<std::io::Result<JsResult>, JsTimeout> {
    let limit = opts.timeout;
    let (tx, rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("javascript".into())
        // Boa and deep DOM recursion need more than the 2 MiB default.
        .stack_size(crate::depth::LARGE_STACK)
        .spawn(move || {
            let _ = tx.send(run_scripts(&html, &base_dir, &opts));
        })
        .expect("spawn JavaScript thread");
    match rx.recv_timeout(limit + WATCHDOG_GRACE) {
        Ok(result) => Ok(result),
        Err(_) => Err(JsTimeout(limit)),
    }
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
    if let Some(limit) = opts.loop_limit {
        doc.runtime_limits_mut().set_loop_iteration_limit(limit);
    }

    let wall_start = Instant::now();
    doc.execute_scripts();
    doc.poll(None);

    let virtual_start = doc.clock_now();
    let budget = if opts.window_status.is_some() {
        opts.timeout
    } else {
        opts.delay
    };
    let virtual_end = virtual_start + budget;

    let mut status_reached = opts.window_status.is_none();
    let mut errors = Vec::new();
    // Consecutive rounds in which the next timer was already due, i.e. timers
    // that rescheduled themselves without letting time pass.
    let mut nested = 0u32;

    loop {
        if let Some(wanted) = &opts.window_status
            && read_window_status(&mut doc).as_deref() == Some(wanted.as_str())
        {
            status_reached = true;
            break;
        }
        if wall_start.elapsed() > opts.timeout {
            errors.push(format!("JavaScript timeout after {:?}", opts.timeout));
            break;
        }
        let Some(deadline) = doc.next_timer_deadline() else {
            break;
        };
        let now = doc.clock_now();
        let target = if deadline > now {
            nested = 0;
            deadline
        } else {
            nested += 1;
            if nested > TIMER_NESTING_LEVEL {
                now + MIN_NESTED_TIMER_DELAY
            } else {
                now
            }
        };
        if target > virtual_end {
            break;
        }
        doc.advance_clock_to(target);
        doc.poll(None);
    }

    errors.extend(doc.take_js_errors());
    if opts.debug {
        for e in &errors {
            eprintln!("[js] {e}");
        }
    }

    let html = {
        let mut guard = doc.inner_mut();
        crate::depth::flatten_deep_nesting(&mut guard);
        crate::depth::serialize(&guard)
    };

    Ok(JsResult {
        html,
        errors,
        status_reached,
    })
}

fn read_window_status(doc: &mut ScriptDocument) -> Option<String> {
    doc.take_messages();
    doc.eval("__blitz_send_message(String(window.status ?? ''))");
    doc.take_messages().pop()
}
