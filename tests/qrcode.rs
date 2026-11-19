//! QR codes drawn by qrcode.js (davidshimjs) must scan.
//!
//! qrcode.js draws on a `<canvas>` and shows the result as an `<img>` when
//! the browser has `CanvasRenderingContext2D`. bdhtml2pdf has no canvas, so
//! the library falls back to a `<table>` with one `<td>` per module, sized in
//! whole pixels and coloured with `background-color`. Invoice templates (the
//! Turkish e-archive invoice among them) draw their QR codes with it. These
//! tests print such a code, rasterize the PDF page with hayro and decode it
//! with rqrr.

use std::path::Path;
use std::process::Command;

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};

/// A payload like the JSON of an e-archive invoice QR code: long enough for a
/// 61×61 code (version 11 at level M), the size those invoices print.
const PAYLOAD: &str = r#"{"vkntckn":"1234567890","avkntckn":"9876543210","senaryo":"EARSIVFATURA","tip":"SATIS","tarih":"2026-11-19","no":"ABC2026000000123","ettn":"3f2504e0-4f89-11d3-9a0c-0305e82c3301","vergidahil":"1200.00","odenecek":"1200.00"}"#;

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("bdhtml2pdf-qr-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Writes a page that draws `PAYLOAD` with qrcode.js into a table cell, as
/// e-archive templates do, and returns its path.
fn qr_page(dir: &Path, size_px: u32) -> std::path::PathBuf {
    let vendor = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/vendor/qrcode.min.js");
    std::fs::copy(vendor, dir.join("qrcode.min.js")).unwrap();
    let payload = PAYLOAD.replace('\\', "\\\\").replace('\'', "\\'");
    let html = format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8">
<script src="qrcode.min.js"></script></head><body>
<table width="100%"><tr><td>Invoice text on the left</td>
<td align="right"><div id="qrcode"></div></td></tr></table>
<script>
new QRCode(document.getElementById("qrcode"), {{
  text: '{payload}', width: {size_px}, height: {size_px},
  correctLevel: QRCode.CorrectLevel.M
}});
</script></body></html>"#
    );
    let input = dir.join("qr.html");
    std::fs::write(&input, html).unwrap();
    input
}

fn convert(input: &Path, output: &Path, args: &[&str]) {
    let o = Command::new(env!("CARGO_BIN_EXE_bdhtml2pdf"))
        .arg("-q")
        .args(args)
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

/// Rasterizes the first page at `px_per_pt` and returns every QR code found,
/// decoded.
fn decode_qr_codes(pdf: Vec<u8>, px_per_pt: f32) -> Vec<String> {
    let pdf = Pdf::new(pdf).expect("PDF parses");
    let page = pdf.pages().first().expect("one page");
    let pixmap = hayro::render(
        page,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: px_per_pt,
            y_scale: px_per_pt,
            bg_color: WHITE,
        },
    );
    let (w, h) = (pixmap.width() as usize, pixmap.height() as usize);
    let rgba = pixmap.data_as_u8_slice();
    let mut image = rqrr::PreparedImage::prepare_from_greyscale(w, h, |x, y| {
        let i = (y * w + x) * 4;
        // Opaque white background: premultiplied RGB is the colour.
        ((rgba[i] as u32 * 299 + rgba[i + 1] as u32 * 587 + rgba[i + 2] as u32 * 114) / 1000) as u8
    });
    image
        .detect_grids()
        .into_iter()
        .map(|grid| {
            grid.decode()
                .map(|(_, text)| text)
                .unwrap_or_else(|e| format!("<{e:?}>"))
        })
        .collect()
}

fn assert_scans(name: &str, size_px: u32, args: &[&str], px_per_pt: f32) {
    let dir = tempdir(name);
    let input = qr_page(&dir, size_px);
    let output = dir.join("qr.pdf");
    convert(&input, &output, args);
    let codes = decode_qr_codes(std::fs::read(&output).unwrap(), px_per_pt);
    assert_eq!(
        codes,
        vec![PAYLOAD.to_string()],
        "{name}: one code with the payload"
    );
}

/// The size e-archive templates use (128 px for 61 modules: 2 px each), at
/// CSS size, scanned at 300 dpi.
#[test]
fn table_qr_code_scans() {
    assert_scans("plain", 128, &[], 300.0 / 72.0);
}

/// Printed at 80% as wkhtmltopdf does: each 2 px module becomes 1.2 pt.
#[test]
fn table_qr_code_scans_with_smart_shrinking() {
    assert_scans("smart", 128, &["--enable-smart-shrinking"], 300.0 / 72.0);
}

/// The shrunk code at 200 dpi, about 3.3 pixels per module, where seams
/// between cells or blurred module edges would show first. wkhtmltopdf's
/// canvas image of the same code does not decode at this resolution (it
/// does at 300 dpi); at 150 dpi neither does.
#[test]
fn table_qr_code_scans_at_low_resolution() {
    assert_scans("lowres", 128, &["--enable-smart-shrinking"], 200.0 / 72.0);
}

/// A larger code with modules of a fractional page size (3 px × 0.8 × 0.75 pt).
#[test]
fn larger_table_qr_code_scans() {
    assert_scans("large", 200, &["--enable-smart-shrinking"], 300.0 / 72.0);
}
