//! `bdhtml2pdf serve`: conversions over HTTP, each in its own child process.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

struct Server {
    child: Child,
    addr: String,
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn start(extra: &[&str]) -> Server {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .args(["serve", "--listen", "127.0.0.1:0", "--jobs", "2"])
        .args(extra)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stderr.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    // "bdhtml2pdf 0.1.1 listening on http://127.0.0.1:PORT (2 jobs)"
    let addr = line
        .split("http://")
        .nth(1)
        .and_then(|r| r.split_whitespace().next())
        .unwrap_or_else(|| panic!("no address in {line:?}"))
        .to_string();
    Server { child, addr }
}

/// Send one request; returns the status code and the body.
fn request(server: &Server, method: &str, target: &str, body: &str) -> (u16, Vec<u8>) {
    let mut s = TcpStream::connect(&server.addr).unwrap();
    // The server may answer and close before a refused body is sent, so
    // write and read errors are not failures; the status line decides.
    let _ = write!(
        s,
        "{method} {target} HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\n\r\n{body}",
        body.len()
    );
    let mut resp = Vec::new();
    let _ = s.read_to_end(&mut resp);
    let split = resp.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&resp[..split]).into_owned();
    let code = head.split_whitespace().nth(1).unwrap().parse().unwrap();
    (code, resp[split + 4..].to_vec())
}

fn text(b: &[u8]) -> String {
    String::from_utf8_lossy(b).into_owned()
}

#[test]
fn health_answers_with_the_version() {
    let s = start(&[]);
    let (code, body) = request(&s, "GET", "/health", "");
    assert_eq!(code, 200);
    assert!(text(&body).starts_with("ok bdhtml2pdf "), "{}", text(&body));
}

#[test]
fn a_page_becomes_a_pdf_with_its_options() {
    let s = start(&[]);
    let page = r#"<!DOCTYPE html><p id="p"></p><script>p.textContent = "Merhaba İğdır"</script>"#;
    let (code, pdf) = request(&s, "POST", "/pdf?page-size=A5", page);
    assert_eq!(code, 200, "{}", text(&pdf));
    assert!(pdf.starts_with(b"%PDF-"));
    // A5 is 419.53 x 595.28 pt.
    assert!(text(&pdf).contains("419.52"), "not A5");
    let (code, html) = request(&s, "POST", "/html", page);
    assert_eq!(code, 200);
    assert!(
        text(&html).contains(r#"<p id="p">Merhaba İğdır"#),
        "script did not run"
    );
    let (code, png) = request(&s, "POST", "/png?width=300&height=200", page);
    assert_eq!(code, 200);
    assert!(png.starts_with(b"\x89PNG"));
}

#[test]
fn server_settings_and_unknown_options_are_refused() {
    let s = start(&[]);
    for q in [
        "max-memory=0",
        "javascript-timeout=600",
        "output=x.pdf",
        "debug-javascript=1",
    ] {
        let (code, body) = request(&s, "POST", &format!("/pdf?{q}"), "<p>x</p>");
        assert_eq!(code, 400, "{q}: {}", text(&body));
    }
    assert_eq!(request(&s, "GET", "/pdf", "").0, 405);
    assert_eq!(request(&s, "POST", "/nope", "x").0, 404);
}

#[test]
fn a_page_cannot_read_the_servers_files() {
    let s = start(&[]);
    let page = r#"<iframe src="file:///etc/passwd"></iframe><img src="../../../../etc/passwd">"#;
    let (code, pdf) = request(&s, "POST", "/pdf", page);
    assert_eq!(code, 200);
    assert!(!text(&pdf).contains("root:"));
}

#[test]
fn a_memory_bomb_or_an_endless_script_fails_alone() {
    let s = start(&["--max-memory", "200", "--timeout", "3"]);
    let bomb = r#"<script>let s = "x"; for (;;) s = s + s;</script>"#;
    let (code, body) = request(&s, "POST", "/pdf", bomb);
    assert_eq!(code, 507, "{}", text(&body));
    assert!(text(&body).contains("memory limit"));
    let endless = "<script>while (true) {}</script>";
    let (code, _) = request(&s, "POST", "/pdf?javascript-loop-limit=0", endless);
    assert_eq!(code, 504);
    // The server is still there.
    assert_eq!(request(&s, "GET", "/health", "").0, 200);
}

#[test]
fn a_body_over_the_limit_is_refused() {
    let s = start(&["--max-body", "1"]);
    let big = "x".repeat((1 << 20) + 1);
    assert_eq!(request(&s, "POST", "/pdf", &big).0, 413);
}
