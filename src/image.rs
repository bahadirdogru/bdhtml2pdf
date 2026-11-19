//! Raster output (PNG, JPEG), like wkhtmltoimage.
//!
//! The page is laid out once at the viewport width with `screen` media and
//! painted by Blitz (`blitz-paint`) on the CPU (`vello_cpu`). Local resources
//! load through fulgur's loader, so the PDF and the image see the same files
//! and the same sandbox (only files inside the input directory).

use std::io::Cursor;
use std::path::Path;

use anyrender::{AntiAliasing, PaintScene as _, render_to_buffer};
use anyrender_vello_cpu::VelloCpuImageRenderer;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use peniko::kurbo::{Affine, Rect};
use peniko::{Color, Fill};

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

/// Viewport height for `vh` units and for pages shorter than one screen
/// when capturing the whole page.
pub const DEFAULT_VIEWPORT_HEIGHT: u32 = 768;

/// The largest image side in device pixels. Larger pages are cut off at the
/// bottom (and reported); a 1024-pixel-wide image of this height takes
/// 128 MiB as RGBA.
pub const MAX_IMAGE_SIDE: u32 = 32_768;

pub struct RenderedImage {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    /// The page was taller than [`MAX_IMAGE_SIDE`] and was cut off.
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
    let viewport_height = opts.height.unwrap_or(DEFAULT_VIEWPORT_HEIGHT);
    let (mut doc, _, _) = fulgur::blitz_adapter::parse_html_with_local_resources(
        html,
        opts.width as f32,
        viewport_height,
        &[],
        true,
        Some(base_dir),
    );
    doc.resolve(0.0);

    let css_height = match opts.height {
        Some(h) => h as f32,
        None => {
            let layout = doc.root_element().final_layout();
            layout.size.height.max(viewport_height as f32)
        }
    };
    let scale = opts.zoom as f64;
    let device = |css: f32| ((css as f64 * scale).ceil() as u64).max(1);
    let (want_w, want_h) = (device(opts.width as f32), device(css_height));
    let width = want_w.min(MAX_IMAGE_SIDE as u64) as u32;
    let height = want_h.min(MAX_IMAGE_SIDE as u64) as u32;
    let truncated = want_h > height as u64;
    if want_w > width as u64 {
        return Err(format!(
            "image is {want_w} pixels wide; the limit is {MAX_IMAGE_SIDE} (lower --width or --zoom)"
        ));
    }

    let rgba = render_to_buffer::<VelloCpuImageRenderer, _>(
        |scene| {
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                Color::WHITE,
                None,
                &Rect::new(0.0, 0.0, width as f64, height as f64),
                AntiAliasing::Enabled,
            );
            blitz_paint::paint_scene(scene, &mut doc, scale, width, height, 0, 0);
        },
        width,
        height,
    );

    let mut bytes = Vec::new();
    match opts.format {
        ImageFormat::Png => PngEncoder::new(Cursor::new(&mut bytes))
            .write_image(&rgba, width, height, ExtendedColorType::Rgba8)
            .map_err(|e| format!("PNG encoding failed: {e}"))?,
        ImageFormat::Jpeg => {
            // The background is opaque white, so dropping alpha is lossless.
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
