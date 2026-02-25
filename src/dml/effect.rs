//! `DrawingML` effect types (shadow, glow, etc.).

use crate::dml::color::ColorFormat;
use crate::units::Emu;
use crate::xml_util::WriteXml;

/// The type of shadow effect.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShadowType {
    Outer,
    Inner,
    Perspective,
}

/// Shadow effect formatting.
///
/// Corresponds to `<a:effectLst>` containing `<a:outerShdw>`, `<a:innerShdw>`,
/// or perspective shadow elements.
#[derive(Debug, Clone, PartialEq)]
pub struct ShadowFormat {
    /// The type of shadow (outer, inner, or perspective).
    pub shadow_type: ShadowType,
    /// Shadow color.
    pub color: Option<ColorFormat>,
    /// Blur radius in EMU.
    pub blur_radius: Option<Emu>,
    /// Distance from the shape in EMU.
    pub distance: Option<Emu>,
    /// Direction angle in degrees (0-360).
    pub direction: Option<f64>,
    /// Opacity from 0.0 (fully transparent) to 1.0 (fully opaque).
    pub opacity: Option<f64>,
}

impl ShadowFormat {
    /// Create an outer shadow with the given parameters.
    #[must_use]
    pub const fn outer(color: ColorFormat, blur: Emu, distance: Emu, angle: f64) -> Self {
        Self {
            shadow_type: ShadowType::Outer,
            color: Some(color),
            blur_radius: Some(blur),
            distance: Some(distance),
            direction: Some(angle),
            opacity: None,
        }
    }

    /// Create an inner shadow with the given parameters.
    #[must_use]
    pub const fn inner(color: ColorFormat, blur: Emu, distance: Emu, angle: f64) -> Self {
        Self {
            shadow_type: ShadowType::Inner,
            color: Some(color),
            blur_radius: Some(blur),
            distance: Some(distance),
            direction: Some(angle),
            opacity: None,
        }
    }
}

impl WriteXml for ShadowFormat {
    fn write_xml<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str("<a:effectLst>")?;

        let tag = match self.shadow_type {
            ShadowType::Outer | ShadowType::Perspective => "a:outerShdw",
            ShadowType::Inner => "a:innerShdw",
        };

        w.write_char('<')?;
        w.write_str(tag)?;

        if let Some(blur) = self.blur_radius {
            write!(w, r#" blurRad="{blur}""#)?;
        }
        if let Some(dist) = self.distance {
            write!(w, r#" dist="{dist}""#)?;
        }
        if let Some(dir) = self.direction {
            // EMU values fit in i64 range
            #[allow(clippy::cast_possible_truncation)]
            let dir_val = (dir * 60_000.0) as i64;
            write!(w, r#" dir="{dir_val}""#)?;
        }

        if self.shadow_type == ShadowType::Perspective {
            w.write_str(r#" sx="100000" sy="23000" kx="1200000" algn="bl" rotWithShape="0""#)?;
        }

        w.write_char('>')?;

        // Color with optional opacity
        if let Some(ref color) = self.color {
            match color {
                ColorFormat::Rgb(rgb) => {
                    if let Some(opacity) = self.opacity {
                        // EMU values fit in i64 range
                        #[allow(clippy::cast_possible_truncation)]
                        let alpha = (opacity * 100_000.0) as i64;
                        write!(
                            w,
                            r#"<a:srgbClr val="{}"><a:alpha val="{}"/></a:srgbClr>"#,
                            rgb.to_hex(),
                            alpha
                        )?;
                    } else {
                        color.write_xml(w)?;
                    }
                }
                _ => {
                    color.write_xml(w)?;
                }
            }
        }

        w.write_str("</")?;
        w.write_str(tag)?;
        w.write_char('>')?;

        w.write_str("</a:effectLst>")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outer_shadow_xml() {
        let shadow =
            ShadowFormat::outer(ColorFormat::rgb(0, 0, 0), Emu(50_800), Emu(38_100), 270.0);
        let xml = shadow.to_xml_string();
        assert!(xml.starts_with("<a:effectLst>"));
        assert!(xml.contains("<a:outerShdw"));
        assert!(xml.contains(r#"blurRad="50800""#));
        assert!(xml.contains(r#"dist="38100""#));
        assert!(xml.contains("dir="));
        assert!(xml.contains("000000"));
        assert!(xml.ends_with("</a:effectLst>"));
    }

    #[test]
    fn test_inner_shadow_xml() {
        let shadow = ShadowFormat::inner(
            ColorFormat::rgb(128, 128, 128),
            Emu(25_400),
            Emu(12_700),
            90.0,
        );
        let xml = shadow.to_xml_string();
        assert!(xml.contains("<a:innerShdw"));
        assert!(xml.contains("808080"));
    }

    #[test]
    fn test_shadow_with_opacity() {
        let mut shadow =
            ShadowFormat::outer(ColorFormat::rgb(0, 0, 0), Emu(50_800), Emu(38_100), 270.0);
        shadow.opacity = Some(0.5);
        let xml = shadow.to_xml_string();
        assert!(xml.contains(r#"<a:alpha val="50000"/>"#));
    }
}
