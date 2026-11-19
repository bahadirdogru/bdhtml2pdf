//! Image output (`.png`, `.jpg`): format, size, zoom, JavaScript and local resources.

use std::path::{Path, PathBuf};
use std::process::Command;

use image::{GenericImageView, Rgba};

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-image-{name}-{}", std::process::id()));
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

fn page(dir: &Path, body: &str) -> PathBuf {
    let path = dir.join("page.html");
    std::fs::write(
        &path,
        format!("<!DOCTYPE html><meta charset=utf-8><style>body{{margin:0}}</style>{body}"),
    )
    .unwrap();
    path
}

fn near(a: Rgba<u8>, b: [u8; 3]) -> bool {
    a.0[..3].iter().zip(b).all(|(&x, y)| x.abs_diff(y) <= 8)
}

#[test]
fn png_covers_the_whole_page_at_the_viewport_width() {
    let dir = tempdir("full");
    let input = page(
        &dir,
        "<div style='height:2000px;background:rgb(255,0,0)'></div>\
         <div style='height:100px;background:rgb(0,0,255)'></div>",
    );
    let out = dir.join("out.png");
    convert(&input, &out, &["--width", "300"]);
    let img = image::open(&out).unwrap();
    assert_eq!(img.dimensions(), (300, 2100));
    assert!(near(img.get_pixel(10, 10), [255, 0, 0]));
    assert!(near(img.get_pixel(10, 2050), [0, 0, 255]));
}

#[test]
fn height_and_zoom_set_the_pixel_size() {
    let dir = tempdir("zoom");
    let input = page(
        &dir,
        "<div style='width:50px;height:50px;background:rgb(0,128,0)'></div>",
    );
    let out = dir.join("out.png");
    convert(
        &input,
        &out,
        &["--width", "200", "--height", "100", "--zoom", "2"],
    );
    let img = image::open(&out).unwrap();
    assert_eq!(img.dimensions(), (400, 200));
    assert!(
        near(img.get_pixel(90, 90), [0, 128, 0]),
        "inside the zoomed box"
    );
    assert!(near(img.get_pixel(110, 110), [255, 255, 255]), "outside it");
}

#[test]
fn jpeg_by_extension_or_format_flag() {
    let dir = tempdir("jpeg");
    let input = page(&dir, "<p>jpeg</p>");
    let out = dir.join("out.jpg");
    convert(&input, &out, &["--quality", "50"]);
    assert_eq!(&std::fs::read(&out).unwrap()[..3], &[0xFF, 0xD8, 0xFF]);

    let out = dir.join("no-extension");
    convert(&input, &out, &["--format", "jpeg"]);
    assert_eq!(&std::fs::read(&out).unwrap()[..3], &[0xFF, 0xD8, 0xFF]);
}

#[test]
fn javascript_runs_before_the_image_is_taken() {
    let dir = tempdir("js");
    let input = page(
        &dir,
        "<div id=box style='height:100px'></div>\
         <script>setTimeout(() => { box.style.background = 'rgb(255,0,255)'; }, 100);</script>",
    );
    let out = dir.join("out.png");
    convert(&input, &out, &["--width", "100", "--height", "100"]);
    assert!(near(
        image::open(&out).unwrap().get_pixel(50, 50),
        [255, 0, 255]
    ));
}

#[test]
fn javascript_sees_the_image_viewport() {
    let dir = tempdir("viewport");
    let input = page(
        &dir,
        "<div id=box style='height:20px'></div>\
         <script>if (innerWidth === 640) box.style.background = 'rgb(0,0,0)';</script>",
    );
    let out = dir.join("out.png");
    convert(&input, &out, &["--width", "640", "--height", "20"]);
    assert!(near(image::open(&out).unwrap().get_pixel(5, 5), [0, 0, 0]));
}

#[test]
fn local_images_load_and_outside_files_do_not() {
    let dir = tempdir("resources");
    let site = dir.join("site");
    std::fs::create_dir_all(&site).unwrap();
    let red = image::RgbImage::from_pixel(10, 10, image::Rgb([255, 0, 0]));
    red.save(site.join("red.png")).unwrap();
    red.save(dir.join("outside.png")).unwrap();
    let input = page(
        &site,
        "<img src=red.png style='display:block;width:40px;height:40px'>\
         <img src=../outside.png style='display:block;width:40px;height:40px'>",
    );
    let out = dir.join("out.png");
    convert(&input, &out, &["--width", "100", "--height", "80"]);
    let img = image::open(&out).unwrap();
    assert!(near(img.get_pixel(20, 20), [255, 0, 0]), "local image");
    assert!(!near(img.get_pixel(20, 60), [255, 0, 0]), "outside image");
}
