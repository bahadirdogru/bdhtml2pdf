//! `--enable-smart-shrinking` prints at 80% as wkhtmltopdf does, so pages
//! written for wkhtmltopdf keep their page count; `--disable-smart-shrinking`
//! conflicts with it.

use std::process::{Command, Output};

fn run(args: &[&str], html: &str, name: &str) -> (Output, Vec<u8>) {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-shrink-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.html");
    let output = dir.join("out.pdf");
    std::fs::write(&input, html).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .args(args)
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    let pdf = std::fs::read(&output).unwrap_or_default();
    (o, pdf)
}

fn pages(pdf: &[u8]) -> usize {
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    count(b"/Type /Page") - count(b"/Type /Pages")
}

#[test]
fn smart_shrinking_fits_more_on_a_page() {
    let html = format!(
        "<!DOCTYPE html><meta charset=utf-8><body>{}</body>",
        "<p>Lorem ipsum dolor sit amet, consectetur adipiscing elit.</p>".repeat(150)
    );
    let (o, normal) = run(&[], &html, "normal");
    assert!(o.status.success());
    let (o, smart) = run(&["--enable-smart-shrinking"], &html, "smart");
    assert!(o.status.success());
    let (normal, smart) = (pages(&normal), pages(&smart));
    assert!(
        normal >= 3 && smart < normal,
        "{normal} pages, {smart} with smart shrinking"
    );
}

#[test]
fn smart_shrinking_flags_conflict() {
    let (o, _) = run(
        &["--enable-smart-shrinking", "--disable-smart-shrinking"],
        "<p>x</p>",
        "conflict",
    );
    assert_eq!(o.status.code(), Some(2));
}
