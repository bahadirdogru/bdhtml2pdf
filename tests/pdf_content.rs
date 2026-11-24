//! What ends up inside the PDF: images from every kind of reference, and the
//! right font instances.

use std::path::{Path, PathBuf};
use std::process::Command;

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-pdf-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn convert(input: &Path, output: &Path) {
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .arg(input)
        .arg(output)
        .output()
        .unwrap();
    assert!(
        o.status.success(),
        "{:?}: {}",
        o.status,
        String::from_utf8_lossy(&o.stderr)
    );
}

fn count(haystack: &[u8], needle: &[u8]) -> usize {
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

fn png(path: &Path, rgb: [u8; 3]) {
    image::RgbImage::from_pixel(4, 4, image::Rgb(rgb))
        .save(path)
        .unwrap();
}

/// Renders `body` and returns the number of image XObjects in the PDF.
fn images_in_pdf(dir: &Path, body: &str) -> usize {
    let input = dir.join("page.html");
    std::fs::write(&input, format!("<!DOCTYPE html><meta charset=utf-8>{body}")).unwrap();
    let out = dir.join("page.pdf");
    convert(&input, &out);
    count(&std::fs::read(&out).unwrap(), b"/Subtype /Image")
}

#[test]
fn images_load_from_img_css_and_linked_stylesheets() {
    let dir = tempdir("images");
    let site = dir.join("site");
    std::fs::create_dir_all(site.join("img")).unwrap();
    std::fs::create_dir_all(site.join("css")).unwrap();
    for (name, rgb) in [
        ("a", [255, 0, 0]),
        ("b", [0, 255, 0]),
        ("c", [0, 0, 255]),
        ("d", [9, 9, 9]),
    ] {
        png(&site.join(format!("img/{name}.png")), rgb);
    }
    std::fs::write(
        site.join("css/s.css"),
        ".c{background:url(../img/c.png) no-repeat;background-size:100% 100%}",
    )
    .unwrap();
    let box_ = "width:20px;height:20px";
    let n = images_in_pdf(
        &site,
        &format!(
            "<link rel=stylesheet href=css/s.css>\
             <style>.b{{background:url('img/b.png') no-repeat;background-size:100% 100%}}</style>\
             <img src=img/a.png style='display:block;{box_}'>\
             <div class=b style='{box_}'></div><div class=c style='{box_}'></div>\
             <div style='{box_};background:url(./img/d.png) no-repeat;background-size:100% 100%'></div>"
        ),
    );
    assert_eq!(n, 4);
}

#[test]
fn data_url_images_are_decoded() {
    let dir = tempdir("data");
    // Two different images: identical ones are written once.
    png(&dir.join("x.png"), [255, 0, 0]);
    png(&dir.join("y.png"), [0, 0, 255]);
    let x = base64_encode(&std::fs::read(dir.join("x.png")).unwrap());
    let y = base64_encode(&std::fs::read(dir.join("y.png")).unwrap());
    let n = images_in_pdf(
        &dir,
        &format!(
            "<img src='data:image/png;base64,{x}' width=20>\
             <div style='width:20px;height:20px;background:url(data:image/png;base64,{y}) no-repeat;background-size:100% 100%'></div>"
        ),
    );
    assert_eq!(n, 2);
}

#[test]
fn images_outside_the_input_directory_are_not_read() {
    let dir = tempdir("outside");
    let site = dir.join("site");
    std::fs::create_dir_all(&site).unwrap();
    png(&dir.join("secret.png"), [255, 0, 0]);
    let n = images_in_pdf(
        &site,
        "<img src=../secret.png width=20>\
         <img src='file:///etc/hostname' width=20>\
         <div style='width:20px;height:20px;background:url(../secret.png)'></div>",
    );
    assert_eq!(n, 0);
}

#[test]
fn bold_text_gets_its_own_font_instance() {
    // With a variable system font, bold used to fall back to the regular
    // instance: one embedded font for both weights.
    let dir = tempdir("bold");
    let input = dir.join("bold.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><meta charset=utf-8>\
         <p style='font-family:sans-serif'>regular <b>bold</b></p>",
    )
    .unwrap();
    let out = dir.join("bold.pdf");
    convert(&input, &out);
    let pdf = std::fs::read(&out).unwrap();
    assert!(
        count(&pdf, b"/FontFile2") >= 2,
        "regular and bold instances"
    );
}

#[test]
fn arial_uses_a_metric_compatible_font() {
    // Without Arial installed, `font-family: Arial` fell through to the
    // default font, so lines were wider and documents gained pages.
    // fontconfig's metric aliases map it to Liberation Sans or Arimo, which
    // have Arial's glyph widths; CI installs Liberation on Linux, and macOS
    // and Windows have Arial itself.
    let dir = tempdir("arial");
    let input = dir.join("arial.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><meta charset=utf-8>\
         <p style='font-family:Arial'>regular <b>bold</b></p>",
    )
    .unwrap();
    let out = dir.join("arial.pdf");
    convert(&input, &out);
    let names = embedded_fonts(&std::fs::read(&out).unwrap());
    assert!(
        names.iter().any(|n| is_arial(n)),
        "embedded fonts: {names:?}"
    );
}

/// Turkish letters that Latin-1 lacks, and the dotted and dotless i.
const TURKISH: &str = "Ğğ Şş İı Üü Öö Çç ĞÜŞİÖÇ ğüşıöç";

#[test]
fn unstyled_turkish_text_uses_times_new_roman() {
    // As in Chromium: text the page does not style is set in Times New Roman
    // (or its metric-compatible substitute on Linux). Every Turkish letter
    // comes from that font: a missing glyph would embed a fallback font.
    let dir = tempdir("default-font");
    let input = dir.join("plain.html");
    std::fs::write(
        &input,
        format!(
            "<!DOCTYPE html><meta charset=utf-8>\
             <table border=1><tr><td>{TURKISH}<td><b>{TURKISH}</b></table>"
        ),
    )
    .unwrap();
    let out = dir.join("plain.pdf");
    convert(&input, &out);
    let names = embedded_fonts(&std::fs::read(&out).unwrap());
    assert!(
        names.len() >= 2 && names.iter().all(|n| is_times(n)),
        "embedded fonts: {names:?}"
    );
}

#[test]
fn missing_tahoma_falls_back_to_times_new_roman_with_turkish_letters() {
    // Turkish e-invoice templates ask for `'Tahoma', "Times New Roman",
    // serif`. Where Tahoma is missing Chromium skips it and prints in Times
    // New Roman; so do we, Turkish letters included.
    let dir = tempdir("tahoma");
    let input = dir.join("tahoma.html");
    std::fs::write(
        &input,
        format!(
            "<!DOCTYPE html><meta charset=utf-8>\
             <p style=\"font-family:'Tahoma', 'Times New Roman', serif\">{TURKISH} <b>{TURKISH}</b></p>"
        ),
    )
    .unwrap();
    let out = dir.join("tahoma.pdf");
    convert(&input, &out);
    let names = embedded_fonts(&std::fs::read(&out).unwrap());
    let tahoma_installed = names.iter().any(|n| n.starts_with("Tahoma"));
    assert!(
        !names.is_empty()
            && names
                .iter()
                .all(|n| is_times(n) || (tahoma_installed && n.starts_with("Tahoma"))),
        "embedded fonts: {names:?}"
    );
}

#[test]
fn font_families_resolve_as_in_chromium() {
    // Chromium prints `sans-serif` in Arial (Liberation Sans on Linux, not
    // fontconfig's Noto Sans), Helvetica in Arial unless it is installed (on
    // macOS it is; on Linux not Nimbus Sans), and `serif`, `cursive`,
    // `fantasy` and families that are not installed in its standard font,
    // Times New Roman.
    let cases = [
        ("sans-serif", true),
        ("'Helvetica'", true),
        ("serif", false),
        ("cursive", false),
        ("fantasy", false),
        ("'No Such Font'", false),
    ];
    let dir = tempdir("families");
    for (i, (family, sans)) in cases.iter().enumerate() {
        let input = dir.join(format!("{i}.html"));
        std::fs::write(
            &input,
            format!(
                "<!DOCTYPE html><meta charset=utf-8><p style=\"font-family:{family}\">{TURKISH}</p>"
            ),
        )
        .unwrap();
        let out = dir.join(format!("{i}.pdf"));
        convert(&input, &out);
        let names = embedded_fonts(&std::fs::read(&out).unwrap());
        let expected = |n: &str| {
            if !sans {
                is_times(n)
            } else {
                is_arial(n) || (family.contains("Helvetica") && n.starts_with("Helvetica"))
            }
        };
        assert!(
            !names.is_empty() && names.iter().all(|n| expected(n)),
            "{family}: embedded fonts {names:?}"
        );
    }
}

fn is_arial(name: &str) -> bool {
    ["LiberationSans", "Arimo", "Arial"]
        .iter()
        .any(|family| name.starts_with(family))
}

fn is_times(name: &str) -> bool {
    ["LiberationSerif", "Tinos", "TimesNewRoman"]
        .iter()
        .any(|family| name.starts_with(family))
}

/// The base font names of the embedded (subset) fonts, without the subset tag.
/// The names of the subset fonts, read from `/BaseFont` entries only: binary
/// font data can contain bytes that look like `/XXXXXX+` (it did on Windows).
fn embedded_fonts(pdf: &[u8]) -> Vec<String> {
    const KEY: &[u8] = b"/BaseFont";
    let mut names: Vec<String> = pdf
        .windows(KEY.len())
        .enumerate()
        .filter(|(_, w)| *w == KEY)
        .filter_map(|(i, _)| {
            let rest = &pdf[i + KEY.len()..];
            let start = rest.iter().position(|&b| b == b'/')? + 1;
            let len = rest[start..]
                .iter()
                .position(|b| b.is_ascii_whitespace() || b"/>[]<(".contains(b))?;
            let name = std::str::from_utf8(&rest[start..start + len]).ok()?;
            Some(name.split_once('+')?.1.to_string())
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

fn base64_encode(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

#[test]
fn unparsable_and_escaping_imports_do_not_crash() {
    let dir = tempdir("imports");
    let site = dir.join("site");
    std::fs::create_dir_all(&site).unwrap();
    std::fs::write(dir.join("out.css"), "p{color:red}").unwrap();
    let input = site.join("page.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><style>@import url(\"http://[::1\"); @import \"../out.css\";</style>\
         <link rel=stylesheet href=../out.css><p>x</p>",
    )
    .unwrap();
    convert(&input, &dir.join("page.pdf"));
}

#[test]
fn ligatures_keep_all_their_letters_in_the_text() {
    // Comfortaa has the fi, fl, ff, ffi and ffl ligatures. Each ligature is
    // one glyph for several letters; the PDF has to map it back to all of
    // them, or `office` extracts, copies and reads aloud as `ofce`.
    let dir = tempdir("ligatures");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts/Comfortaa-Regular.otf"),
        dir.join("Comfortaa-Regular.otf"),
    )
    .unwrap();
    let words = "office affluent fluff Workflow";
    let input = dir.join("ligatures.html");
    std::fs::write(
        &input,
        format!(
            "<!DOCTYPE html><meta charset=utf-8><style>\
             @font-face {{ font-family: Brand; src: url(Comfortaa-Regular.otf); }}\
             p {{ font-family: Brand; }}</style><p>{words}</p>"
        ),
    )
    .unwrap();
    let out = dir.join("ligatures.pdf");
    convert(&input, &out);
    let Ok(o) = Command::new("pdftotext").arg(&out).arg("-").output() else {
        eprintln!("pdftotext not installed; skipping");
        return;
    };
    let text = String::from_utf8_lossy(&o.stdout);
    assert!(text.contains(words), "{text:?}");
}

#[test]
fn running_elements_in_an_inline_style_are_drawn_once() {
    // `position: running()` takes an element out of the body and into the
    // margin box that names it. With the rule in the page's own <style>,
    // fulgur used to draw it in both places (fulgur-rs/fulgur#983).
    let dir = tempdir("running");
    let input = dir.join("running.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><style>@page { @top-right { content: element(head) } }\
         .head { position: running(head) }</style>\
         <div class=head>RUNHEAD</div><p>Body text</p>",
    )
    .unwrap();
    let out = dir.join("running.pdf");
    convert(&input, &out);
    let Ok(o) = Command::new("pdftotext").arg(&out).arg("-").output() else {
        eprintln!("pdftotext not installed; skipping");
        return;
    };
    let text = String::from_utf8_lossy(&o.stdout);
    assert_eq!(text.matches("RUNHEAD").count(), 1, "{text:?}");
    assert!(text.contains("Body text"), "{text:?}");
}

#[test]
fn keep_all_breaks_after_a_hyphen_as_normal_does() {
    // `word-break: keep-all` only keeps letters together (CSS Text 3 § 5.2),
    // so Latin text breaks where `normal` breaks, after a hyphen too. ICU4X
    // kept "Stone-pot" whole (unicode-org/icu4x#8772), which pushed a word
    // of the Korean menu sample onto the next line.
    let dir = tempdir("keep-all");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fonts/Comfortaa-Regular.otf"),
        dir.join("Comfortaa-Regular.otf"),
    )
    .unwrap();
    let lines = |word_break: &str| {
        let input = dir.join(format!("{word_break}.html"));
        std::fs::write(
            &input,
            format!(
                "<!DOCTYPE html><meta charset=utf-8><style>\
                 @font-face {{ font-family: Brand; src: url(Comfortaa-Regular.otf); }}\
                 p {{ font: 16px Brand; width: 110px; word-break: {word_break}; }}</style>\
                 <p>abc Stone-pot bibimbap</p>"
            ),
        )
        .unwrap();
        let out = dir.join(format!("{word_break}.pdf"));
        convert(&input, &out);
        let o = Command::new("pdftotext")
            .arg("-raw")
            .arg(&out)
            .arg("-")
            .output()
            .ok()?;
        // pdftotext ends lines with \r\n on Windows.
        Some(
            String::from_utf8_lossy(&o.stdout)
                .trim()
                .replace("\r\n", "\n"),
        )
    };
    let (Some(normal), Some(keep_all)) = (lines("normal"), lines("keep-all")) else {
        eprintln!("pdftotext not installed; skipping");
        return;
    };
    assert!(normal.contains("Stone-\npot"), "{normal:?}");
    assert_eq!(keep_all, normal);
}

#[test]
fn inline_backgrounds_are_drawn() {
    // A `<span>` with a background and a `<mark>` paint their background
    // behind the text, as in browsers (fulgur-rs/fulgur#973).
    let dir = tempdir("inline-bg");
    let input = dir.join("inline.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><p>a <span style='background: rgb(255,255,0)'>highlighted span</span> b</p>\
         <p>a <mark>mark element</mark> b</p>",
    )
    .unwrap();
    let out = dir.join("inline.png");
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .arg(&input)
        .arg(&out)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let img = image::open(&out).unwrap().to_rgb8();
    let count = |rgb: [u8; 3]| img.pixels().filter(|p| p.0 == rgb).count();
    assert!(count([255, 255, 0]) > 500, "the span's yellow");
    // The UA stylesheet's `Mark` system color, as Stylo resolves it.
    assert!(count([255, 235, 59]) > 500, "the mark's background");
}

#[test]
fn first_letter_gets_its_own_style() {
    // `::first-letter` gives the block's first letter its style, here a
    // color; the rest of the text keeps the block's.
    let dir = tempdir("first-letter");
    let input = dir.join("fl.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><style>p { font: 40px serif }\
         p::first-letter { color: rgb(255, 0, 0) }</style><p>\u{201C}Hello world</p>",
    )
    .unwrap();
    let out = dir.join("fl.png");
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .arg(&input)
        .arg(&out)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let img = image::open(&out).unwrap().to_rgb8();
    // The red pixels: the opening quote and the H, nothing past them.
    let red: Vec<u32> = img
        .enumerate_pixels()
        .filter(|(_, _, p)| p.0 == [255, 0, 0])
        .map(|(x, _, _)| x)
        .collect();
    assert!(red.len() > 100, "the first letter is red");
    let dark_after = img
        .enumerate_pixels()
        .filter(|(x, _, p)| *x > red.iter().max().copied().unwrap_or(0) && p.0 == [0, 0, 0])
        .count();
    assert!(dark_after > 100, "the rest stays black");
}

#[test]
fn page_css_styles_inline_svg() {
    // Page CSS reaches the shapes of an inline <svg>, as in browsers, while a
    // gradient referenced from the SVG's own markup keeps working.
    let dir = tempdir("inline-svg-css");
    let input = dir.join("svg.html");
    std::fs::write(
        &input,
        "<!DOCTYPE html><style>body{margin:0} svg{display:block}\
         .icon rect { fill: rgb(255, 0, 0) } #b { fill: rgb(0, 0, 255) }</style>\
         <svg class=icon width=40 height=40><rect width='40' height='40'/></svg>\
         <svg width=40 height=40><rect id=b width='40' height='40'/></svg>\
         <svg width=40 height=40><defs><linearGradient id=g>\
         <stop offset=0 stop-color='rgb(0,255,0)'/><stop offset=1 stop-color='rgb(0,255,0)'/>\
         </linearGradient></defs><rect width=40 height=40 fill='url(#g)'/></svg>",
    )
    .unwrap();
    let out = dir.join("svg.png");
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .args(["-q", "--width", "200"])
        .arg(&input)
        .arg(&out)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let img = image::open(&out).unwrap().to_rgb8();
    assert_eq!(img.get_pixel(20, 20).0, [255, 0, 0], "page CSS by class");
    assert_eq!(img.get_pixel(20, 60).0, [0, 0, 255], "page CSS by id");
    assert_eq!(
        img.get_pixel(20, 100).0,
        [0, 255, 0],
        "the SVG's own gradient"
    );
}
