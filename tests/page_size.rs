//! Page size: without a size or orientation flag the page's CSS `@page`
//! size wins, as in Chromium, and pages without one are A4 portrait; any
//! size or orientation flag overrides `@page`, as in wkhtmltopdf.

use std::process::Command;

/// The first page's MediaBox width and height, in points, rounded.
fn media_box(args: &[&str], page_css: &str, name: &str) -> (i64, i64) {
    let dir = std::env::temp_dir().join(format!(
        "bdhtml2pdf-page-size-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.html");
    let output = dir.join("out.pdf");
    std::fs::write(
        &input,
        format!("<!DOCTYPE html><style>{page_css}</style><p>Hello</p>"),
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .args(args)
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let pdf = String::from_utf8_lossy(&std::fs::read(&output).unwrap()).into_owned();
    let start = pdf.find("/MediaBox [").expect("MediaBox") + "/MediaBox [".len();
    let end = start + pdf[start..].find(']').unwrap();
    let v: Vec<f64> = pdf[start..end]
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    ((v[2] - v[0]).round() as i64, (v[3] - v[1]).round() as i64)
}

const A4: (i64, i64) = (595, 842);
const A4_LANDSCAPE: (i64, i64) = (842, 595);
const A5: (i64, i64) = (420, 595);

#[test]
fn a4_without_flags_or_page_size() {
    assert_eq!(media_box(&[], "", "none"), A4);
}

#[test]
fn page_size_applies_without_size_flags() {
    assert_eq!(media_box(&[], "@page { size: A5 }", "a5"), A5);
    assert_eq!(
        media_box(&[], "@page { size: A4 landscape }", "landscape"),
        A4_LANDSCAPE
    );
}

#[test]
fn size_flags_override_page_size() {
    assert_eq!(media_box(&["-s", "A4"], "@page { size: A5 }", "s"), A4);
    assert_eq!(
        media_box(&["-O", "Landscape"], "@page { size: A5 }", "o"),
        A4_LANDSCAPE
    );
}
