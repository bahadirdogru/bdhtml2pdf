//! Scaling the printed page: `--enable-smart-shrinking` prints at 80% as
//! wkhtmltopdf does, so pages written for wkhtmltopdf keep their page count;
//! `--disable-smart-shrinking` conflicts with it; `--zoom` scales the PDF as
//! in wkhtmltopdf.

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

/// Fixed-height boxes, so the page count does not depend on the fonts: an
/// A4 page with 10 mm margins holds 1047 CSS px, ten 100 px boxes; at 80% it
/// holds 1309 px, thirteen.
#[test]
fn smart_shrinking_fits_more_on_a_page() {
    let html = format!(
        "<!DOCTYPE html><body style='margin:0'>{}</body>",
        "<div style='height:100px'></div>".repeat(60)
    );
    let (o, normal) = run(&[], &html, "normal");
    assert!(o.status.success());
    let (o, smart) = run(&["--enable-smart-shrinking"], &html, "smart");
    assert!(o.status.success());
    assert_eq!((pages(&normal), pages(&smart)), (6, 5));
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

/// `--zoom 2` lays the page out half as wide and tall and prints it twice as
/// large: five 100 px boxes per page instead of ten. `--zoom 0.5` fits twenty.
#[test]
fn zoom_scales_the_pdf() {
    let html = format!(
        "<!DOCTYPE html><body style='margin:0'>{}</body>",
        "<div style='height:100px'></div>".repeat(60)
    );
    let (o, large) = run(&["--zoom", "2"], &html, "zoom2");
    assert!(o.status.success());
    let (o, small) = run(&["--zoom", "0.5"], &html, "zoom05");
    assert!(o.status.success());
    assert_eq!((pages(&large), pages(&small)), (12, 3));
}

#[test]
fn zoom_out_of_range_is_an_error() {
    for zoom in ["0", "-1", "100", "NaN"] {
        // `-1` is rejected by the argument parser (status 2), the others
        // by the range check (status 1).
        let (o, pdf) = run(&[&format!("--zoom={zoom}")], "<p>x</p>", "badzoom");
        assert!(!o.status.success() && pdf.is_empty(), "--zoom {zoom}");
    }
}
