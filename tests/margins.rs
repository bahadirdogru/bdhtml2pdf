//! Page margins: without a margin flag the page's CSS `@page` margins win, as
//! in Chromium, and pages without them get 10 mm; any margin flag overrides
//! `@page`, as in wkhtmltopdf.

use std::process::Command;

fn pages(args: &[&str], html: &str, name: &str) -> usize {
    let dir =
        std::env::temp_dir().join(format!("bdhtml2pdf-margins-{name}-{}", std::process::id()));
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
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let pdf = std::fs::read(&output).unwrap();
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    count(b"/Type /Page") - count(b"/Type /Pages")
}

/// Sixty fixed-height 100px boxes. An A4 page with 10 mm margins holds 1047
/// CSS px, ten boxes: six pages. With 30 mm margins it holds 896 px, eight
/// boxes: eight pages.
fn boxes(page_css: &str) -> String {
    format!(
        "<!DOCTYPE html><style>{page_css}</style><body style='margin:0'>{}</body>",
        "<div style='height:100px'></div>".repeat(60)
    )
}

#[test]
fn page_margins_apply_without_margin_flags() {
    assert_eq!(pages(&[], &boxes("@page { margin: 30mm }"), "css"), 8);
}

#[test]
fn pages_without_page_margins_get_10mm() {
    assert_eq!(pages(&[], &boxes(""), "default"), 6);
}

#[test]
fn a_margin_flag_overrides_page_margins() {
    assert_eq!(
        pages(&["-T", "10mm"], &boxes("@page { margin: 30mm }"), "flag"),
        6
    );
}
