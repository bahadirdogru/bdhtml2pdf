//! `bdhtml2pdf serve`: a small HTTP server for long-running use.
//!
//! Every conversion runs in a child process of this same binary, fed through
//! stdin and stdout. The memory limit and the JavaScript timeout are
//! process-wide (see [`crate::limits`]), so a child is the only way to stop a
//! memory bomb or an endless script without taking the server down with it.
//! Starting a child costs a few milliseconds; a conversion takes tens.
//!
//! The protocol is deliberately small:
//!
//! - `POST /pdf`, `/png`, `/jpg` or `/html` with the page as the body
//!   (`Content-Length` required). Options go in the query string, named as the
//!   command-line flags without the dashes: `/pdf?page-size=A5&pdf-a=2b`.
//! - `GET /health` answers `ok` and the version.
//!
//! Children run in an empty directory, so a page cannot load files from the
//! server's disk. One connection carries one request (`Connection: close`).

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Query options a client may set, and whether each takes a value.
/// `max-memory` and `javascript-timeout` stay under the server's control.
pub const OPTIONS: &[(&str, bool)] = &[
    ("page-size", true),
    ("orientation", true),
    ("page-width", true),
    ("page-height", true),
    ("margin-top", true),
    ("margin-bottom", true),
    ("margin-left", true),
    ("margin-right", true),
    ("title", true),
    ("pdf-a", true),
    ("pdf-ua", false),
    ("enable-smart-shrinking", false),
    ("disable-smart-shrinking", false),
    ("width", true),
    ("height", true),
    ("zoom", true),
    ("quality", true),
    ("disable-javascript", false),
    ("javascript-delay", true),
    ("window-status", true),
    ("javascript-loop-limit", true),
];

/// Server settings, from `bdhtml2pdf serve` flags.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    pub listen: String,
    /// Conversions running at the same time; more requests wait.
    pub jobs: usize,
    /// Largest request body, in bytes.
    pub max_body: usize,
    /// Heap limit per conversion, in MiB.
    pub max_memory: u64,
    /// Wall-clock limit per conversion; the child is killed after it.
    pub timeout: Duration,
}

/// A counting semaphore: at most `jobs` conversions at once.
struct Slots {
    free: Mutex<usize>,
    cv: Condvar,
}

impl Slots {
    fn acquire(self: &Arc<Self>) -> SlotGuard {
        let mut free = self.free.lock().unwrap();
        while *free == 0 {
            free = self.cv.wait(free).unwrap();
        }
        *free -= 1;
        SlotGuard(self.clone())
    }
}

struct SlotGuard(Arc<Slots>);

impl Drop for SlotGuard {
    fn drop(&mut self) {
        *self.0.free.lock().unwrap() += 1;
        self.0.cv.notify_one();
    }
}

/// Connections handled at once, waiting ones included; more get 503.
const MAX_CONNECTIONS: usize = 256;
const MAX_HEADER_BYTES: usize = 16 * 1024;
/// How long a client may take to send its request.
const READ_TIMEOUT: Duration = Duration::from_secs(30);

/// Run the server until the process is stopped.
pub fn serve(opts: ServerOptions) -> std::io::Result<()> {
    let listener = TcpListener::bind(&opts.listen)?;
    let exe = std::env::current_exe()?;
    let workdir = std::env::temp_dir().join(format!("bdhtml2pdf-serve-{}", std::process::id()));
    std::fs::create_dir_all(&workdir)?;
    eprintln!(
        "bdhtml2pdf {} listening on http://{} ({} jobs)",
        env!("CARGO_PKG_VERSION"),
        listener.local_addr()?,
        opts.jobs
    );
    if !listener.local_addr()?.ip().is_loopback() {
        eprintln!("warning: the server has no authentication; keep it behind a firewall or proxy");
    }
    let slots = Arc::new(Slots {
        free: Mutex::new(opts.jobs.max(1)),
        cv: Condvar::new(),
    });
    let open = Arc::new(Mutex::new(0usize));
    let ctx = Arc::new((opts, exe, workdir));
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        {
            let mut n = open.lock().unwrap();
            if *n >= MAX_CONNECTIONS {
                let _ = respond(&mut stream, 503, "text/plain", b"busy\n");
                continue;
            }
            *n += 1;
        }
        let (slots, open, ctx) = (slots.clone(), open.clone(), ctx.clone());
        std::thread::spawn(move || {
            let (opts, exe, workdir) = &*ctx;
            handle(stream, opts, exe, workdir, &slots);
            *open.lock().unwrap() -= 1;
        });
    }
    Ok(())
}

struct Request {
    method: String,
    path: String,
    query: String,
    body: Vec<u8>,
}

/// An HTTP error: status and message.
type Fail = (u16, String);

fn read_request(stream: &TcpStream, max_body: usize) -> Result<Request, Fail> {
    let bad = |m: &str| (400, m.to_string());
    let mut r = BufReader::new(stream.take((MAX_HEADER_BYTES + max_body) as u64 + 1));
    let mut line = String::new();
    r.read_line(&mut line)
        .map_err(|_| bad("cannot read request"))?;
    let mut parts = line.split_whitespace();
    let (Some(method), Some(target)) = (parts.next(), parts.next()) else {
        return Err(bad("malformed request line"));
    };
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let (method, path, query) = (method.to_string(), path.to_string(), query.to_string());

    let mut header_bytes = line.len();
    let mut length = None;
    loop {
        let mut h = String::new();
        let n = r
            .read_line(&mut h)
            .map_err(|_| bad("cannot read headers"))?;
        header_bytes += n;
        if header_bytes > MAX_HEADER_BYTES {
            return Err((431, "headers too large".into()));
        }
        let h = h.trim_end();
        if n == 0 || h.is_empty() {
            break;
        }
        if let Some((name, value)) = h.split_once(':') {
            let value = value.trim();
            match name.trim().to_ascii_lowercase().as_str() {
                "content-length" => {
                    length = Some(
                        value
                            .parse::<usize>()
                            .map_err(|_| bad("bad Content-Length"))?,
                    )
                }
                "transfer-encoding" if !value.eq_ignore_ascii_case("identity") => {
                    return Err((411, "send Content-Length, not chunked".into()));
                }
                _ => {}
            }
        }
    }
    let mut body = Vec::new();
    if method == "POST" {
        let len = length.ok_or((411, "Content-Length required".into()))?;
        if len > max_body {
            return Err((413, format!("body larger than {max_body} bytes")));
        }
        body.resize(len, 0);
        r.read_exact(&mut body)
            .map_err(|_| bad("body shorter than Content-Length"))?;
    }
    Ok(Request {
        method,
        path,
        query,
        body,
    })
}

fn handle(
    mut stream: TcpStream,
    opts: &ServerOptions,
    exe: &Path,
    workdir: &Path,
    slots: &Arc<Slots>,
) {
    let _ = stream.set_read_timeout(Some(READ_TIMEOUT));
    let req = match read_request(&stream, opts.max_body) {
        Ok(r) => r,
        Err((code, msg)) => {
            let _ = respond(
                &mut stream,
                code,
                "text/plain",
                format!("{msg}\n").as_bytes(),
            );
            discard_rest(&mut stream);
            return;
        }
    };
    let format = match (req.method.as_str(), req.path.as_str()) {
        ("GET", "/health") => {
            let body = format!("ok bdhtml2pdf {}\n", env!("CARGO_PKG_VERSION"));
            let _ = respond(&mut stream, 200, "text/plain", body.as_bytes());
            return;
        }
        ("POST", "/pdf") => "pdf",
        ("POST", "/png") => "png",
        ("POST", "/jpg" | "/jpeg") => "jpg",
        ("POST", "/html") => "html",
        (_, "/pdf" | "/png" | "/jpg" | "/jpeg" | "/html" | "/health") => {
            let _ = respond(&mut stream, 405, "text/plain", b"method not allowed\n");
            return;
        }
        _ => {
            let _ = respond(
                &mut stream,
                404,
                "text/plain",
                b"not found; POST /pdf, /png, /jpg or /html\n",
            );
            return;
        }
    };
    let args = match query_args(&req.query) {
        Ok(a) => a,
        Err(msg) => {
            let _ = respond(
                &mut stream,
                400,
                "text/plain",
                format!("{msg}\n").as_bytes(),
            );
            return;
        }
    };
    let result = {
        let _slot = slots.acquire();
        convert(exe, workdir, format, &args, &req.body, opts)
    };
    let _ = match result {
        Ok(bytes) => respond(&mut stream, 200, content_type(format), &bytes),
        Err((code, msg)) => respond(&mut stream, code, "text/plain", msg.as_bytes()),
    };
}

/// After an early answer, read what the client is still sending for a
/// moment. Closing a socket with unread data resets the connection, and the
/// client may then lose the answer.
fn discard_rest(stream: &mut TcpStream) {
    let _ = stream.shutdown(std::net::Shutdown::Write);
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut buf = [0u8; 64 * 1024];
    while Instant::now() < deadline {
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }
    }
}

fn content_type(format: &str) -> &'static str {
    match format {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        _ => "text/html; charset=utf-8",
    }
}

/// The query string as command-line flags, checked against [`OPTIONS`].
pub fn query_args(query: &str) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
        let key = key.replace('_', "-");
        let Some(&(name, takes_value)) = OPTIONS.iter().find(|(n, _)| *n == key) else {
            return Err(format!("unknown option {key:?}"));
        };
        if takes_value {
            // `--flag=value`, so a value that starts with a dash stays a value.
            args.push(format!("--{name}={value}"));
        } else {
            match value.as_ref() {
                "" | "1" | "true" | "yes" => args.push(format!("--{name}")),
                "0" | "false" | "no" => {}
                v => return Err(format!("option {name:?} takes true or false, not {v:?}")),
            }
        }
    }
    Ok(args)
}

fn convert(
    exe: &Path,
    workdir: &Path,
    format: &str,
    args: &[String],
    html: &[u8],
    opts: &ServerOptions,
) -> Result<Vec<u8>, Fail> {
    let internal = |e: std::io::Error| (500, format!("cannot run the converter: {e}\n"));
    let mut child = Command::new(exe)
        .arg("-q")
        .arg(format!("--format={format}"))
        .arg(format!("--max-memory={}", opts.max_memory))
        .arg(format!(
            "--javascript-timeout={}",
            opts.timeout.as_secs().max(1)
        ))
        .args(args)
        .args(["-", "-"])
        .current_dir(workdir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(internal)?;
    let mut stdin = child.stdin.take().unwrap();
    let html = html.to_vec();
    let writer = std::thread::spawn(move || {
        let _ = stdin.write_all(&html);
    });
    let mut out = child.stdout.take().unwrap();
    let mut err = child.stderr.take().unwrap();
    let reader = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = out.read_to_end(&mut v);
        v
    });
    let err_reader = std::thread::spawn(move || {
        let mut v = Vec::new();
        let _ = (&mut err).take(64 * 1024).read_to_end(&mut v);
        v
    });

    // The child's own JavaScript limit fires first; this catches the rest
    // (layout of a huge page, a stuck child).
    let deadline = Instant::now() + opts.timeout + Duration::from_secs(5);
    let status = loop {
        match child.try_wait().map_err(internal)? {
            Some(s) => break Some(s),
            None if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                break None;
            }
            None => std::thread::sleep(Duration::from_millis(5)),
        }
    };
    let _ = writer.join();
    let stdout = reader.join().unwrap_or_default();
    let stderr = String::from_utf8_lossy(&err_reader.join().unwrap_or_default()).into_owned();
    match status {
        None => Err((504, "conversion took too long and was stopped\n".into())),
        Some(s) if s.success() => Ok(stdout),
        Some(s) => {
            let code = match s.code() {
                Some(crate::limits::EXIT_MEMORY_LIMIT) => 507,
                Some(crate::limits::EXIT_JS_TIMEOUT) => 504,
                _ => 422,
            };
            let msg = if stderr.trim().is_empty() {
                format!("conversion failed ({s})\n")
            } else {
                stderr
            };
            Err((code, msg))
        }
    }
}

fn respond(
    stream: &mut TcpStream,
    code: u16,
    content_type: &str,
    body: &[u8],
) -> std::io::Result<()> {
    let reason = match code {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        411 => "Length Required",
        413 => "Content Too Large",
        422 => "Unprocessable Content",
        431 => "Request Header Fields Too Large",
        500 => "Internal Server Error",
        503 => "Service Unavailable",
        504 => "Gateway Timeout",
        507 => "Insufficient Storage",
        _ => "",
    };
    let head = format!(
        "HTTP/1.1 {code} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

/// The default number of parallel conversions: one per CPU.
pub fn default_jobs() -> usize {
    std::thread::available_parallelism().map_or(2, |n| n.get())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_becomes_flags() {
        assert_eq!(
            query_args("page-size=A5&pdf_a=2b&disable-javascript=1&enable-smart-shrinking=false")
                .unwrap(),
            ["--page-size=A5", "--pdf-a=2b", "--disable-javascript"]
        );
        assert_eq!(query_args("title=-q%20x").unwrap(), ["--title=-q x"]);
        assert_eq!(query_args("pdf-ua=true").unwrap(), ["--pdf-ua"]);
    }

    #[test]
    fn query_refuses_server_settings_and_unknown_options() {
        assert!(query_args("max-memory=0").is_err());
        assert!(query_args("javascript-timeout=999").is_err());
        assert!(query_args("output=/etc/passwd").is_err());
        assert!(query_args("disable-javascript=maybe").is_err());
    }
}
