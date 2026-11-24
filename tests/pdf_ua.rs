//! `--pdf-ua`: tagged PDF/UA-1 files for accessibility, validated by
//! krilla, with readable errors for what a page is missing.

use std::process::{Command, Output};

fn run(args: &[&str], html: &str, out_name: &str, name: &str) -> (Output, Vec<u8>) {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-pdf-ua-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.html");
    let output = dir.join(out_name);
    std::fs::write(&input, html).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .args(args)
        .arg(&input)
        .arg(&output)
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

/// The decoded content stream of page 1, where marked content and
/// artifacts are (the file itself is compressed).
fn page_content(pdf: &[u8]) -> Vec<u8> {
    let pdf = hayro::hayro_syntax::Pdf::new(pdf.to_vec()).expect("PDF parses");
    let page = pdf.pages().first().expect("one page");
    page.page_stream().expect("page content").to_vec()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

const INVOICE: &str = r#"<!DOCTYPE html><html lang="tr"><title>Fatura 42</title>
<style>@page { @bottom-center { content: "Sayfa " counter(page) } }</style>
<h1>Fatura</h1>
<table><thead><tr><th>Kalem</th><th>Tutar</th></tr></thead>
<tbody><tr><td>Tasarım<small style="display:block">İki gün</small></td><td>100</td></tr></tbody></table>
<p id="total"></p>
<script>document.getElementById("total").textContent = "Toplam: 100";</script>"#;

#[test]
fn pdf_ua_is_written_tagged_with_its_identification() {
    let (o, pdf) = run(&["--pdf-ua"], INVOICE, "out.pdf", "ua1");
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(contains(&pdf, "<pdfuaid:part>1</pdfuaid:part>"));
    assert!(contains(&pdf, "/StructTreeRoot"));
    assert!(contains(&pdf, "/Lang (tr)"));
    assert!(contains(&pdf, "/DisplayDocTitle true"));
    // Table cells are structure elements, and the footer is a pagination
    // artifact.
    assert!(contains(&pdf, "/TD"));
    let content = page_content(&pdf);
    assert!(contains(&content, "/Artifact"));
    assert!(contains(&content, "/Pagination"));
    assert!(contains(&content, "/Footer"));
}

#[test]
fn without_the_flag_nothing_is_tagged() {
    let (o, pdf) = run(&[], INVOICE, "out.pdf", "none");
    assert!(o.status.success());
    assert!(!contains(&pdf, "pdfuaid"));
    assert!(!contains(&pdf, "/StructTreeRoot"));
}

#[test]
fn a_page_without_a_language_says_so() {
    let html = "<!DOCTYPE html><title>T</title><h1>Hi</h1>";
    let (o, _) = run(&["--pdf-ua"], html, "out.pdf", "nolang");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("language"), "{}", stderr(&o));
}

#[test]
fn a_page_without_a_title_says_so_and_title_fixes_it() {
    let html = "<!DOCTYPE html><html lang=en><h1>Hi</h1>";
    let (o, _) = run(&["--pdf-ua"], html, "out.pdf", "notitle");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("has no title"), "{}", stderr(&o));
    let (o, _) = run(&["--pdf-ua", "--title", "Hi"], html, "out.pdf", "title");
    assert!(o.status.success(), "{}", stderr(&o));
}

#[test]
fn an_image_without_alt_text_says_how_to_fix_it() {
    let html = r#"<!DOCTYPE html><html lang=en><title>T</title><h1>Chart</h1>
<div><svg width="20" height="20"><rect width="20" height="20"/></svg></div>"#;
    let (o, _) = run(&["--pdf-ua"], html, "out.pdf", "noalt");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(
        err.contains("PDF/UA-1") && err.contains("aria-label"),
        "{err}"
    );
    let labelled = html.replace("<svg ", r#"<svg role="img" aria-label="A square" "#);
    let (o, pdf) = run(&["--pdf-ua"], &labelled, "out.pdf", "alt");
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(contains(&pdf, "A square"));
}

#[test]
fn pdf_ua_and_pdf_a_cannot_be_combined() {
    let (o, _) = run(&["--pdf-ua", "--pdf-a", "2b"], INVOICE, "out.pdf", "both");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("cannot be used with"), "{}", stderr(&o));
}

#[test]
fn pdf_ua_is_refused_for_image_output() {
    let (o, _) = run(&["--pdf-ua"], INVOICE, "out.png", "png");
    assert!(!o.status.success());
    assert!(stderr(&o).contains("only to PDF output"));
}

#[test]
fn code_with_tabs_and_text_in_other_languages_is_written() {
    // A tab used to be drawn as the font's .notdef box, which PDF/UA
    // refuses; an element's own lang was left out of the structure.
    let html = "<!DOCTYPE html><html lang=en><title>T</title><h1>Code</h1>\
<pre>if ready {\n\tstart()\n}</pre>\
<p lang=fr>Bonjour</p><p>Say <span lang=de>Guten Tag</span>.</p>";
    let (o, pdf) = run(&["--pdf-ua"], html, "out.pdf", "tabs-lang");
    assert!(o.status.success(), "{}", stderr(&o));
    assert!(contains(&pdf, "/Lang (fr)"), "the paragraph's language");
    assert!(
        contains(&page_content(&pdf), "/Lang (de)"),
        "the span's language"
    );
}

#[test]
fn headings_that_skip_a_level_say_which() {
    let html = "<!DOCTYPE html><html lang=en><title>T</title><h1>Report</h1><h3>Totals</h3>";
    let (o, _) = run(&["--pdf-ua"], html, "out.pdf", "headings");
    assert!(!o.status.success());
    let err = stderr(&o);
    assert!(
        err.contains("h1 is followed by h3") && err.contains("Totals"),
        "{err}"
    );
}
