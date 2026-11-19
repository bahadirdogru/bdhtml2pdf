//! PDF output through fulgur (vector PDF via Krilla).

use std::path::Path;

use crate::PageOptions;

/// CSS pixels per PDF point.
const PT_TO_PX: f32 = 96.0 / 72.0;

/// The font of text the page does not style: Arial, as in wkhtmltopdf
/// (QtWebKit's standard font) rather than the browsers' serif. Windows and
/// macOS ship Arial; on Linux fulgur maps it to a metric-compatible family
/// (Liberation Sans, Arimo) when one is installed, so line breaks are the
/// same on all three. `:where` gives the rule no specificity: any
/// `font-family` the page sets on the root or below wins.
const DEFAULT_FONT_CSS: &str = ":where(html) { font-family: Arial, sans-serif; }";

fn engine(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    title: Option<&str>,
    viewport_height_px: Option<f32>,
) -> fulgur::Engine {
    let mut builder = fulgur::Engine::builder()
        .page_size(fulgur::PageSize {
            width: page.width,
            height: page.height,
        })
        .margin(fulgur::Margin {
            top: page.margin_top,
            right: page.margin_right,
            bottom: page.margin_bottom,
            left: page.margin_left,
        })
        .base_path(base_dir)
        .assets({
            // After the page's own stylesheets; order does not matter for a
            // rule without specificity except against `*`, which styles
            // every element directly anyway.
            let mut assets = crate::assets::collect_images(html, base_dir);
            assets.add_css(DEFAULT_FONT_CSS);
            assets
        });
    if let Some(t) = title {
        builder = builder.title(t);
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
    title: Option<&str>,
) -> Result<Vec<u8>, String> {
    render_pdf_with_viewport(html, base_dir, page, title, None)
}

/// [`render_pdf`] with `vh` units and root percentage heights resolved
/// against `viewport_height_px` instead of the page height.
pub fn render_pdf_with_viewport(
    html: &str,
    base_dir: &Path,
    page: &PageOptions,
    title: Option<&str>,
    viewport_height_px: Option<f32>,
) -> Result<Vec<u8>, String> {
    engine(html, base_dir, page, title, viewport_height_px)
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
    let layout = engine(html, base_dir, page, None, viewport_height_px)
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
