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
    let pdf = std::fs::read(&out).unwrap();
    let names: Vec<String> = pdf
        .split(|&b| b == b'/')
        .filter_map(|word| {
            let word = word
                .split(|b| b.is_ascii_whitespace() || *b == b'>')
                .next()?;
            let name = std::str::from_utf8(word).ok()?;
            Some(name.split_once('+')?.1.to_string())
        })
        .collect();
    assert!(
        names.iter().any(|n| n.starts_with("LiberationSans")
            || n.starts_with("Arimo")
            || n.starts_with("Arial")),
        "embedded fonts: {names:?}"
    );
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
