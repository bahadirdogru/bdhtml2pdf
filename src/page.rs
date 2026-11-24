//! Page geometry, independent of the PDF backend. All values in points.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PageOptions {
    pub width: f32,
    pub height: f32,
    /// Whether the size overrides the page's CSS `@page` size; when false
    /// the size applies only where `@page` sets none.
    pub size_override: bool,
    pub margin_top: f32,
    pub margin_right: f32,
    pub margin_bottom: f32,
    pub margin_left: f32,
    /// Whether the margins override the page's CSS `@page` margins; when
    /// false they apply only where `@page` sets none.
    pub margins_override: bool,
    /// Content scale: the page is laid out `1 / scale` times larger in CSS
    /// px and printed at `scale` (1 = CSS size).
    pub scale: f32,
    /// Shrink content wider than the page to fit it, down to this scale;
    /// `None` never shrinks.
    pub shrink_to_fit: Option<f32>,
}

/// The scale wkhtmltopdf prints at with smart shrinking on (its default):
/// WebKit lays a printed page out 1.25 times wider than the paper.
pub const SMART_SHRINK_SCALE: f32 = 0.8;

/// The lowest scale content wider than the page is shrunk to, as in WebKit
/// and Blink printing (a maximum shrink factor of 2).
pub const MIN_SHRINK_SCALE: f32 = 0.5;

const fn mm(v: f32) -> f32 {
    v * 72.0 / 25.4
}

/// Named page sizes (portrait), in points.
pub fn named_size(name: &str) -> Option<(f32, f32)> {
    Some(match name.trim().to_ascii_uppercase().as_str() {
        "A3" => (mm(297.0), mm(420.0)),
        "A4" => (mm(210.0), mm(297.0)),
        "A5" => (mm(148.0), mm(210.0)),
        "B4" => (mm(250.0), mm(353.0)),
        "B5" => (mm(176.0), mm(250.0)),
        "LETTER" => (612.0, 792.0),
        "LEGAL" => (612.0, 1008.0),
        "LEDGER" | "TABLOID" => (792.0, 1224.0),
        _ => return None,
    })
}

pub const NAMED_SIZES: &str = "A3, A4, A5, B4, B5, Letter, Legal, Ledger";

/// Parse a CSS-like length ("10mm", "1in", "12pt", "2cm", "20px"; bare number = mm) to points.
pub fn length_pt(s: &str) -> Result<f32, String> {
    let s = s.trim();
    let split = s.find(|c: char| c.is_ascii_alphabetic()).unwrap_or(s.len());
    let (num, unit) = s.split_at(split);
    let v: f32 = num
        .trim()
        .parse()
        .map_err(|_| format!("invalid length: {s:?}"))?;
    if !v.is_finite() || v < 0.0 {
        return Err(format!("invalid length: {s:?}"));
    }
    Ok(match unit.to_ascii_lowercase().as_str() {
        "" | "mm" => mm(v),
        "cm" => mm(v * 10.0),
        "in" => v * 72.0,
        "pt" => v,
        "px" => v * 0.75,
        u => return Err(format!("unknown unit {u:?} in {s:?}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lengths() {
        assert!((length_pt("10mm").unwrap() - 28.346).abs() < 0.01);
        assert!((length_pt("1cm").unwrap() - 28.346).abs() < 0.01);
        assert_eq!(length_pt("1in").unwrap(), 72.0);
        assert_eq!(length_pt("12pt").unwrap(), 12.0);
        assert_eq!(length_pt("20px").unwrap(), 15.0);
        assert!(length_pt("5furlong").is_err());
        assert!(length_pt("-1mm").is_err());
    }

    #[test]
    fn sizes() {
        let (w, h) = named_size("a4").unwrap();
        assert!((w - 595.27).abs() < 0.1 && (h - 841.89).abs() < 0.1);
        assert!(named_size("A9").is_none());
    }
}
