//! Font resolution and text measurement.
//!
//! Canvas `measureText` and the DOM layout pass must agree on what a string
//! measures with a given family and size, so both go through this module. The
//! family is resolved against the platform font manager, which is what makes a
//! page's metrics follow the installed fonts instead of a guessed table.
use skia_safe::{Font, FontMgr, FontStyle, Typeface};

/// Generic families mean the platform's own face, the way CSS defines them.
const GENERIC: &[(&str, &str)] = &[
    ("proportional", "Helvetica"),
    ("sans-serif", "Helvetica"),
    ("serif", "Times"),
    ("ui-serif", "Times"),
    ("monospace", "Menlo"),
    ("ui-monospace", "Menlo"),
    ("cursive", "Apple Chancery"),
    ("fantasy", "Papyrus"),
    ("emoji", "Apple Color Emoji"),
    ("math", "STIX Two Math"),
];

/// The first family in a CSS font-family list, without quotes or fallbacks.
pub(crate) fn first_family(list: &str) -> String {
    let first = list.split(',').next().unwrap_or(list).trim();
    let first = first.trim_matches(|c| c == '"' || c == '\'').trim();
    if first.is_empty() {
        "Helvetica".to_string()
    } else {
        first.to_string()
    }
}

fn platform_family(name: &str) -> &str {
    // `-apple-system` and the `system-ui` generics are the platform UI face.
    // Skia's macOS manager reaches CoreText but the system font's optical size
    // variants (SF Text below 20px, SF Display above) do not reproduce Safari's
    // advances, and Helvetica is the closer of the two; `TODO.md` records the
    // remaining difference and why it is not a lookup table here.
    if name.eq_ignore_ascii_case("-apple-system") || name.eq_ignore_ascii_case("blinkmacsystemfont")
    {
        return "Helvetica";
    }
    if name.eq_ignore_ascii_case("system-ui") || name.eq_ignore_ascii_case("ui-sans-serif") {
        return "Helvetica";
    }
    GENERIC
        .iter()
        .find(|(generic, _)| name.eq_ignore_ascii_case(generic))
        .map(|(_, family)| *family)
        .unwrap_or(name)
}

fn style(bold: bool, italic: bool) -> FontStyle {
    match (bold, italic) {
        (true, true) => FontStyle::bold_italic(),
        (true, false) => FontStyle::bold(),
        (false, true) => FontStyle::italic(),
        (false, false) => FontStyle::normal(),
    }
}

/// A typeface for `family`, or `None` when the platform has no such family.
pub(crate) fn typeface(family: &str, bold: bool, italic: bool) -> Option<Typeface> {
    let manager = FontMgr::new();
    let requested = style(bold, italic);
    manager
        .match_family_style(platform_family(family), requested)
        .or_else(|| manager.match_family_style("Helvetica", requested))
        .or_else(|| manager.legacy_make_typeface(None, requested))
}

/// A shaped font at `size` pixels.
pub(crate) fn font(family: &str, size: f32, bold: bool, italic: bool) -> Option<Font> {
    let typeface = typeface(family, bold, italic)?;
    let mut font = Font::new(typeface, size.max(0.1));
    // Fractional advances are what a browser reports, so keep them.
    font.set_subpixel(true);
    font.set_linear_metrics(true);
    Some(font)
}

/// What a browser reports for one measured string.
pub(crate) struct Metrics {
    pub width: f32,
    /// Distance above the baseline, positive.
    pub ascent: f32,
    /// Distance below the baseline, positive.
    pub descent: f32,
    pub leading: f32,
}

impl Metrics {
    /// Empty text still contributes the font's line box.
    pub fn for_font(font: &Font) -> Self {
        let (_, font_metrics) = font.metrics();
        Self {
            width: 0.0,
            ascent: -font_metrics.ascent,
            descent: font_metrics.descent,
            leading: font_metrics.leading,
        }
    }
}

/// Measure `text` in `family` at `size`. A missing family falls back the way
/// the platform font manager does, never to a synthesized advance table.
pub(crate) fn measure(
    text: &str,
    family: &str,
    size: f32,
    bold: bool,
    italic: bool,
) -> Option<Metrics> {
    let font = font(family, size, bold, italic)?;
    let mut metrics = Metrics::for_font(&font);
    if !text.is_empty() {
        let (width, _) = font.measure_str(text, None);
        metrics.width = width;
    }
    Some(metrics)
}
