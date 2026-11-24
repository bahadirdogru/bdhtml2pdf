//! Very deep HTML must neither crash nor lose its deepest content.

use std::path::{Path, PathBuf};
use std::process::Command;

use bdhtml2pdf::depth::{MAX_NESTING_DEPTH, flatten_html};

fn nested(levels: usize, inner: &str) -> String {
    format!(
        "<!DOCTYPE html><meta charset=utf-8><p>top</p>{}{inner}{}",
        "<div>".repeat(levels),
        "</div>".repeat(levels)
    )
}

/// Deepest element level below `<html>` in serialized HTML without void
/// elements or raw text.
fn max_div_depth(html: &str) -> usize {
    let (mut depth, mut max) = (0usize, 0usize);
    for (i, _) in html.match_indices('<') {
        let rest = &html[i..];
        if rest.starts_with("<div") {
            depth += 1;
            max = max.max(depth);
        } else if rest.starts_with("</div") {
            depth -= 1;
        }
    }
    max
}

#[test]
fn shallow_pages_are_returned_unchanged() {
    let html = nested(50, "x");
    assert_eq!(flatten_html(html.clone()), html);
}

#[test]
fn deep_content_moves_up_in_document_order() {
    let html = nested(
        2000,
        "a<b>b</b><style>p{}</style><i>c<span>d</span></i><img src=x.png>e",
    );
    let out = flatten_html(html);
    // The divs start at level 2, below <html> and <body>; the moved ones sit
    // one level below the limit.
    assert!(
        max_div_depth(&out) <= MAX_NESTING_DEPTH,
        "{}",
        max_div_depth(&out)
    );
    let tail = &out[out.find("a<b>").expect("text kept")..];
    assert!(
        tail.starts_with("a<b></b>b<style>p{}</style><i></i>c<span></span>d<img src=\"x.png\">e"),
        "{tail:.200}"
    );
}

fn tempdir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-deep-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn convert(input: &Path, output: &Path, args: &[&str]) {
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .args(args)
        .arg("-q")
        .arg(input)
        .arg(output)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "{args:?} → {:?}: {}",
        o.status,
        String::from_utf8_lossy(&o.stderr)
    );
}

#[test]
fn twenty_thousand_levels_render_with_and_without_javascript() {
    let dir = tempdir();
    let input = dir.join("deep.html");
    std::fs::write(&input, nested(20_000, "bottom")).unwrap();
    for args in [&[][..], &["-n"][..]] {
        convert(&input, &dir.join("deep.pdf"), args);
        let html = dir.join("deep.out.html");
        convert(&input, &html, args);
        let dom = std::fs::read_to_string(&html).unwrap();
        assert!(dom.contains("bottom"), "{args:?}");
        assert!(max_div_depth(&dom) <= MAX_NESTING_DEPTH, "{args:?}");
    }
}
