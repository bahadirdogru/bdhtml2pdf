//! Raster output (PNG, JPEG), like wkhtmltoimage.
//!
//! The image is the PDF layout on a single page as wide as the viewport and
//! as tall as the content, rasterized by hayro. It therefore shows exactly
//! what the PDF shows: multi-column layout, table fragmentation and GCPM
//! included. fulgur lays the page out with `screen` media, as wkhtmltoimage
//! does, and loads local resources through the same sandbox as the PDF.

use std::io::Cursor;
use std::path::Path;

use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::color::palette::css::WHITE;
use hayro::{PixmapSettings, RenderCache, RenderSettings};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};

use crate::PageOptions;
use crate::pdf::{first_page_extent, render_pdf_with_viewport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
}

#[derive(Debug, Clone)]
pub struct ImageOptions {
    /// Viewport width in CSS pixels.
    pub width: u32,
    /// Image height in CSS pixels. `None` captures the whole page.
    pub height: Option<u32>,
    /// Device pixels per CSS pixel.
    pub zoom: f32,
    pub format: ImageFormat,
    /// JPEG quality, 1–100.
    pub quality: u8,
}

impl Default for ImageOptions {
    fn default() -> Self {
        Self {
            width: 1024,
            height: None,
            zoom: 1.0,
            format: ImageFormat::Png,
            quality: 94,
        }
    }
}

/// Viewport height for `vh` units, and the smallest image height when
/// capturing the whole page.
pub const DEFAULT_VIEWPORT_HEIGHT: u32 = 768;

/// The largest image side in device pixels. Longer pages are cut off at the
/// bottom (and reported); a 1024-pixel-wide image of this height takes
/// 128 MiB as RGBA.
pub const MAX_IMAGE_SIDE: u32 = 32_768;

/// CSS pixels to PDF points.
const PX_TO_PT: f32 = 0.75;

pub struct RenderedImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The page was longer than [`MAX_IMAGE_SIDE`] and was cut off.
    pub truncated: bool,
}

/// Renders `html` (normally the DOM after JavaScript) to an encoded image.
pub fn render_image(
    html: &str,
    base_dir: &Path,
    opts: &ImageOptions,
) -> Result<RenderedImage, String> {
    if opts.width == 0 {
        return Err("image width must be greater than 0".into());
    }
    if !(opts.zoom > 0.0 && opts.zoom.is_finite()) {
        return Err(format!("invalid zoom {}", opts.zoom));
    }
    let max_css = MAX_IMAGE_SIDE as f32 / opts.zoom;
    if opts.width as f32 > max_css {
        return Err(format!(
            "image would be {} pixels wide; the limit is {MAX_IMAGE_SIDE} (lower --width or --zoom)",
            (opts.width as f32 * opts.zoom).ceil()
        ));
    }
    let page = |height_px: f32| PageOptions {
        width: opts.width as f32 * PX_TO_PT,
        height: height_px * PX_TO_PT,
        margin_top: 0.0,
        margin_right: 0.0,
        margin_bottom: 0.0,
        margin_left: 0.0,
        size_override: true,
        margins_override: true,
        scale: 1.0,
        shrink_to_fit: None,
    };

    // `vh` units and `height: 100%` on the root follow the viewport, as in a
    // browser, not the page, which is only as tall as the image.
    let viewport = opts.height.unwrap_or(DEFAULT_VIEWPORT_HEIGHT) as f32;
    // Lay out on the tallest page an image may have, then size the page to
    // the content. A fixed `--height` crops that page, as a browser window
    // does; a page only that tall would push blocks that do not fit onto a
    // second page instead.
    let (bottom, more) = first_page_extent(html, base_dir, &page(max_css), Some(viewport))?;
    let page_px = bottom.max(viewport).min(max_css).ceil();
    let (crop_px, truncated) = match opts.height {
        Some(h) => ((h as f32).min(max_css), h as f32 > max_css),
        None => (page_px, more),
    };

    let pdf = render_pdf_with_viewport(
        html,
        base_dir,
        &page(page_px.max(crop_px)),
        &crate::pdf::PdfOptions::default(),
        Some(viewport),
    )?;
    let pdf = Pdf::new(pdf).map_err(|e| format!("cannot read the generated PDF: {e:?}"))?;
    let first = pdf
        .pages()
        .first()
        .ok_or("the generated PDF has no pages")?;
    // PDF points to device pixels.
    let scale = opts.zoom / PX_TO_PT;
    let pixmap = hayro::render(
        first,
        &RenderCache::new(),
        &InterpreterSettings::default(),
        &RenderSettings::default(),
        &PixmapSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
        },
    );
    let width = pixmap.width() as u32;
    let height = (pixmap.height() as u32).min((crop_px * opts.zoom).round() as u32);
    // The background is opaque, so premultiplied and straight RGBA agree.
    // Rows are stored top to bottom, so cropping keeps a prefix.
    let rgba = &pixmap.data_as_u8_slice()[..(width * height * 4) as usize];

    let mut bytes = Vec::new();
    match opts.format {
        ImageFormat::Png => PngEncoder::new(Cursor::new(&mut bytes))
            .write_image(rgba, width, height, ExtendedColorType::Rgba8)
            .map_err(|e| format!("PNG encoding failed: {e}"))?,
        ImageFormat::Jpeg => {
            let rgb: Vec<u8> = rgba
                .as_chunks::<4>()
                .0
                .iter()
                .flat_map(|&[r, g, b, _]| [r, g, b])
                .collect();
            JpegEncoder::new_with_quality(Cursor::new(&mut bytes), opts.quality.clamp(1, 100))
                .write_image(&rgb, width, height, ExtendedColorType::Rgb8)
                .map_err(|e| format!("JPEG encoding failed: {e}"))?
        }
    }
    Ok(RenderedImage {
        bytes,
        width,
        height,
        truncated,
    })
}
