//! PDF output through fulgur (vector PDF via Krilla).

use std::path::Path;

use crate::PageOptions;

/// CSS pixels per PDF point.
const PT_TO_PX: f32 = 96.0 / 72.0;

/// Options for the PDF document as a whole.
#[derive(Debug, Clone, Default)]
pub struct PdfOptions {
    /// The PDF title; `None` uses the page's `<title>`.
    pub title: Option<String>,
    /// Write a PDF/A file of this level (ISO 19005), for archiving. The
    /// conversion fails, saying why, when the page cannot be written as
    /// one: PDF/A-1 does not allow transparency, every character needs a
    /// glyph in some font, and the `a` levels need a document language and
    /// alt text on images.
    pub pdf_a: Option<fulgur::PdfA>,
    /// Write a tagged PDF/UA-1 file (ISO 14289-1), for accessibility. The
    /// conversion fails, saying why, when the page has no language, no
    /// title or an image without alt text. Cannot be combined with `pdf_a`.
    pub pdf_ua: bool,
}

/// The creation date written into a PDF/A file: `SOURCE_DATE_EPOCH` when
/// it is set, for reproducible builds, or else now, in UTC.
fn creation_date() -> String {
    let secs = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        });
    format_utc(secs)
}

/// `secs` since the Unix epoch as `YYYY-MM-DDThh:mm:ssZ`.
fn format_utc(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    // Howard Hinnant's days-to-civil algorithm.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        rem / 60 % 60,
        rem % 60
    )
}

/// Resolves generic font families as Chromium does, so a page prints in the
/// same fonts as in the browser. Runs once, before the first document.
///
/// Chromium's standard and `serif` font is Times New Roman and its
/// `sans-serif` font Arial; on Linux fulgur maps both to a metric-compatible
/// family (Liberation, Tinos and Arimo) when one is installed, so line breaks
/// are the same as on Windows and macOS. fontconfig's own choice (Noto Serif
/// and Noto Sans on many systems) has other widths. Without a font of their
/// own Chromium prints `cursive` and `fantasy` in the standard font too.
/// Text the page does not style is `serif`, and a family that is not
/// installed falls through to it.
///
/// Where Helvetica, Times and Courier are not installed (Windows), Chromium
/// prints them in Arial, Times New Roman and Courier New, as Windows'
/// font substitutes say. On Linux fulgur already aliases them to the
/// Liberation fonts, and on macOS they are installed.
fn init_font_substitutes() {
    static INIT: std::sync::Once = std::sync::Once::new();
    INIT.call_once(|| {
        let times = || vec!["Times New Roman".to_string()];
        let _ = fulgur::set_font_substitutes(fulgur::FontSubstitutes {
            families: vec![
                ("Helvetica".into(), vec!["Arial".to_string()]),
                ("Times".into(), times()),
                ("Courier".into(), vec!["Courier New".to_string()]),
            ],
            generic_families: vec![
                ("serif".into(), times()),
                ("sans-serif".into(), vec!["Arial".to_string()]),
                ("cursive".into(), times()),
                ("fantasy".into(), times()),
            ],
        });
    });
}

fn engine(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    pdf: &PdfOptions,
    viewport_height_px: Option<f32>,
) -> fulgur::Engine {
    init_font_substitutes();
    // Content overflowing into the page margins is cut off, and the page
    // area starts at a whole CSS px (38px for 10 mm margins, not 37.8), as
    // Chromium prints it.
    let mut builder = fulgur::Engine::builder()
        .clip_to_page_area(true)
        .snap_page_area_origin(true)
        .base_path(base_dir)
        .assets(crate::assets::collect_images(html, base_dir));
    let margin = fulgur::Margin {
        top: page.margin_top,
        right: page.margin_right,
        bottom: page.margin_bottom,
        left: page.margin_left,
    };
    // Without a size flag the size is A4 portrait, fulgur's default, which
    // a CSS @page size overrides.
    if page.size_override {
        builder = builder.page_size(fulgur::PageSize {
            width: page.width,
            height: page.height,
        });
    }
    builder = if page.margins_override {
        builder.margin(margin)
    } else {
        builder.default_margin(margin)
    };
    if let Some(t) = &pdf.title {
        builder = builder.title(t);
    }
    if let Some(level) = pdf.pdf_a {
        builder = builder.pdf_a(level).creation_date(creation_date());
    }
    if pdf.pdf_ua {
        builder = builder.pdf_ua(true);
    }
    if let Some(h) = viewport_height_px {
        builder = builder.viewport_height(h);
    }
    if page.scale != 1.0 {
        builder = builder.scale(page.scale);
    }
    if let Some(min) = page.shrink_to_fit {
        builder = builder.shrink_to_fit(min);
    }
    builder.build()
}

/// Render `html` (normally the DOM serialized after JavaScript ran) to PDF bytes.
///
/// `base_dir` confines local resource loading (`<link>`, `<img>`, `@import`)
/// to that directory, mirroring the JavaScript fetch sandbox.
pub fn render_pdf(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    pdf: &PdfOptions,
) -> Result<Vec<u8>, String> {
    render_pdf_with_viewport(html, base_dir, page, pdf, None)
}

/// [`render_pdf`] with `vh` units and root percentage heights resolved
/// against `viewport_height_px` instead of the page height.
pub fn render_pdf_with_viewport(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    pdf: &PdfOptions,
    viewport_height_px: Option<f32>,
) -> Result<Vec<u8>, String> {
    engine(html, base_dir, page, pdf, viewport_height_px)
        .render(html)
        .map_err(|e| format!("PDF rendering failed: {e}"))
}

/// Lays `html` out on `page` without writing a PDF. Returns the bottom edge
/// of the content on the first page, in CSS pixels from the top of the page
/// content area, and whether any content went on to a second page.
pub fn first_page_extent(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    viewport_height_px: Option<f32>,
) -> Result<(f32, bool), String> {
    let layout = engine(
        html,
        base_dir,
        page,
        &PdfOptions::default(),
        viewport_height_px,
    )
    .layout(html)
    .map_err(|e| format!("layout failed: {e}"))?;
    let drawables = &layout.drawables;
    // Fragments are relative to the body's position, which already includes
    // margins collapsed through it (a heading's top margin, for example).
    let body_y = drawables.body_offset_pt.1.to_f32() * PT_TO_PX;
    let mut bottom = 0.0f32;
    let mut more_pages = false;
    for fragment in layout.geometry.values().flat_map(|g| &g.fragments) {
        if fragment.page_index == 0 {
            bottom = bottom.max(body_y + (fragment.y + fragment.height).to_f32());
        } else {
            more_pages = true;
        }
    }
    // The root element's border box covers the body's bottom margin too.
    if let Some(size) = drawables
        .root_id
        .and_then(|id| drawables.block_styles.get(&id))
        .and_then(|entry| entry.layout_size)
    {
        bottom = bottom.max(size.height.to_f32() * PT_TO_PX);
    }
    Ok((bottom, more_pages))
}

#[cfg(test)]
mod tests {
    use super::format_utc;

    #[test]
    fn format_utc_writes_civil_dates() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_795_262_400), "2026-11-21T12:00:00Z");
        assert_eq!(format_utc(4_102_444_799), "2099-12-31T23:59:59Z");
    }
}
