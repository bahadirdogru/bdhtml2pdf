//! The page area is laid out at its exact width, which is rarely a whole
//! number of CSS px, as in Chromium: an A4 page with 18 mm margins is
//! 657.6px wide, not 657px.

use std::process::Command;

fn pages(html: &str, name: &str) -> usize {
    let dir = std::env::temp_dir().join(format!(
        "bdhtml2pdf-layout-width-{name}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("in.html");
    let output = dir.join("out.pdf");
    std::fs::write(&input, html).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let pdf = std::fs::read(&output).unwrap();
    let count = |needle: &[u8]| pdf.windows(needle.len()).filter(|w| *w == needle).count();
    count(b"/Type /Page") - count(b"/Type /Pages")
}

/// Two 600px tall boxes, each `item` px wide, in a wrapping flex row on an
/// A4 page with 18 mm margins (657.6 x 986.5 px). Side by side they fill one
/// page; wrapped onto two rows they need two.
fn two_boxes(item: f32) -> String {
    let b = format!("<div style='width:{item}px;height:600px'></div>");
    format!(
        "<!DOCTYPE html><style>@page {{ size: A4; margin: 18mm }}</style>\
         <body style='margin:0'><div style='display:flex;flex-wrap:wrap'>{b}{b}</div></body>"
    )
}

#[test]
fn boxes_filling_the_fractional_page_width_share_a_line() {
    assert_eq!(pages(&two_boxes(328.75), "fits"), 1);
}

#[test]
fn boxes_wider_than_the_page_width_wrap() {
    assert_eq!(pages(&two_boxes(329.0), "wraps"), 2);
}

/// The left edge of a full-width red box on page 1, in CSS px, measured on
/// a raster at 10 px per CSS px.
fn red_box_left(margin: &str) -> f32 {
    use hayro::hayro_interpret::InterpreterSettings;
    use hayro::hayro_syntax::Pdf;
    use hayro::vello_cpu::color::palette::css::WHITE;
    use hayro::{PixmapSettings, RenderCache, RenderSettings};

    let dir = std::env::temp_dir().join(format!(
        "bdhtml2pdf-page-area-{margin}-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let (input, output) = (dir.join("in.html"), dir.join("out.pdf"));
    std::fs::write(
        &input,
        format!(
            "<!DOCTYPE html><style>@page {{ size: A4; margin: {margin} }}</style>\
             <body style='margin:0'><div style='height:50px;background:#f00'></div></body>"
        ),
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .arg(&input)
        .arg(&output)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let pdf = Pdf::new(std::fs::read(&output).unwrap()).unwrap();
    let scale = 10.0 / 0.75;
    let pixmap = hayro::render(
        &pdf.pages()[0],
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
        },
    );
    let w = pixmap.width() as usize;
    let rgba = pixmap.data_as_u8_slice();
    // A row through the middle of the box, 25px below a 0 to 1 inch margin.
    let y = (25.0 + 48.0) as usize * 10;
    let row = &rgba[y * w * 4..(y + 1) * w * 4];
    let x = row
        .chunks(4)
        .position(|p| p[0] > 200 && p[1] < 80)
        .expect("a red pixel");
    x as f32 / 10.0
}

/// Chromium starts the page area at the nearest whole CSS px and keeps its
/// exact width: 10 mm (37.8px) margins put the content at 38px, 16 mm
/// (60.47px) at 60px.
#[test]
fn the_page_area_starts_at_a_whole_pixel() {
    assert_eq!(red_box_left("10mm"), 38.0);
    assert_eq!(red_box_left("16mm"), 60.0);
    assert_eq!(red_box_left("18mm"), 68.0);
}
