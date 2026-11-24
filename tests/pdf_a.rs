//! `--pdf-a`: PDF/A files for archiving, written through krilla's
//! validators, with the creation date PDF/A requires.

use std::process::{Command, Output};

fn run(args: &[&str], html: &str, out_name: &str, name: &str) -> (Output, Vec<u8>) {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-pdf-a-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.html");
    let output = dir.join(out_name);
    std::fs::write(&input, html).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .args(args)
        .arg(&input)
        .arg(&output)
        .env("SOURCE_DATE_EPOCH", "1795262400")
        .output()
        .unwrap();
    let bytes = std::fs::read(&output).unwrap_or_default();
    (o, bytes)
}

fn contains(haystack: &[u8], needle: &str) -> bool {
    haystack
        .windows(needle.len())
        .any(|w| w == needle.as_bytes())
}

const PAGE: &str = "<!DOCTYPE html><title>Invoice</title><p>Invoice 42</p>";

#[test]
fn pdf_a_2b_is_written_with_its_level_and_a_date() {
    let (o, pdf) = run(&["--pdf-a", "2b"], PAGE, "out.pdf", "2b");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(contains(&pdf, "<pdfaid:part>2</pdfaid:part>"));
    assert!(contains(&pdf, "<pdfaid:conformance>B</pdfaid:conformance>"));
    // SOURCE_DATE_EPOCH 1795262400 is 2026-11-21 12:00:00 UTC.
    assert!(contains(&pdf, "2026-11-21T12:00:00"));
}

#[test]
fn pdf_a_3u_is_written() {
    let (o, pdf) = run(&["--pdf-a", "A-3u"], PAGE, "out.pdf", "3u");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(contains(&pdf, "<pdfaid:part>3</pdfaid:part>"));
    assert!(contains(&pdf, "<pdfaid:conformance>U</pdfaid:conformance>"));
}

#[test]
fn pdf_a_2a_is_tagged() {
    let html = "<!DOCTYPE html><html lang='en'><title>Invoice</title><h1>Invoice</h1><p>42</p>";
    let (o, pdf) = run(&["--pdf-a", "2a"], html, "out.pdf", "2a");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(contains(&pdf, "<pdfaid:conformance>A</pdfaid:conformance>"));
    assert!(contains(&pdf, "/StructTreeRoot"));
}

#[test]
fn pdf_a_2a_without_a_language_says_so() {
    let (o, _) = run(&["--pdf-a", "2a"], PAGE, "out.pdf", "2a-nolang");
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(err.contains("no language"), "{err}");
}

#[test]
fn without_the_flag_no_pdf_a_metadata_is_written() {
    let (o, pdf) = run(&[], PAGE, "out.pdf", "none");
    assert!(o.status.success());
    assert!(!contains(&pdf, "pdfaid"));
}

#[test]
fn pdf_a_1b_refuses_transparency_and_says_why() {
    let html = "<!DOCTYPE html><div style='opacity:.5;background:red;height:20px'></div>";
    let (o, _) = run(&["--pdf-a", "1b"], html, "out.pdf", "1b");
    assert!(!o.status.success());
    let err = String::from_utf8_lossy(&o.stderr);
    assert!(
        err.contains("PDF/A-1b") && err.contains("transparency"),
        "{err}"
    );
}

#[test]
fn an_unknown_level_is_rejected() {
    let (o, _) = run(&["--pdf-a", "4b"], PAGE, "out.pdf", "unknown");
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("expected 1b, 2b, 2u, 2a, 3b, 3u or 3a"));
}

#[test]
fn pdf_a_is_refused_for_image_output() {
    let (o, _) = run(&["--pdf-a", "2b"], PAGE, "out.png", "png");
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("only to PDF output"));
}
