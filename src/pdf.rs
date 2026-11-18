//! PDF output through fulgur (vector PDF via Krilla).

use std::path::Path;

use crate::PageOptions;

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
        .base_path(base_dir);
    if let Some(t) = title {
        builder = builder.title(t);
    }
    builder
        .build()
        .render(html)
        .map_err(|e| format!("PDF rendering failed: {e}"))
}
