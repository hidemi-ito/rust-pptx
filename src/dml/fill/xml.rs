//! `WriteXml` implementation for `FillFormat`.

use super::FillFormat;
use crate::xml_util::WriteXml;

impl WriteXml for FillFormat {
    fn write_xml<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        match self {
            Self::NoFill => w.write_str("<a:noFill/>"),
            Self::Solid(sf) => {
                w.write_str("<a:solidFill>")?;
                sf.color.write_xml(w)?;
                w.write_str("</a:solidFill>")
            }
            Self::Gradient(gf) => {
                w.write_str("<a:gradFill>")?;
                w.write_str("<a:gsLst>")?;
                for stop in &gf.stops {
                    // EMU values fit in i64 range
                    #[allow(clippy::cast_possible_truncation)]
                    let pos = (stop.position * 100_000.0) as i64;
                    write!(w, r#"<a:gs pos="{pos}">"#)?;
                    stop.color.write_xml(w)?;
                    w.write_str("</a:gs>")?;
                }
                w.write_str("</a:gsLst>")?;
                if let Some(angle) = gf.angle {
                    let cw_angle = if angle == 0.0 { 0.0 } else { 360.0 - angle };
                    // EMU values fit in i64 range
                    #[allow(clippy::cast_possible_truncation)]
                    let ang = (cw_angle * 60_000.0) as i64;
                    write!(w, r#"<a:lin ang="{ang}" scaled="0"/>"#)?;
                }
                w.write_str("</a:gradFill>")
            }
            Self::Pattern(pf) => {
                w.write_str("<a:pattFill")?;
                if let Some(ref preset) = pf.preset {
                    write!(w, r#" prst="{}""#, preset.to_xml_str())?;
                }
                w.write_char('>')?;
                if let Some(ref fg) = pf.fore_color {
                    w.write_str("<a:fgClr>")?;
                    fg.write_xml(w)?;
                    w.write_str("</a:fgClr>")?;
                }
                if let Some(ref bg) = pf.back_color {
                    w.write_str("<a:bgClr>")?;
                    bg.write_xml(w)?;
                    w.write_str("</a:bgClr>")?;
                }
                w.write_str("</a:pattFill>")
            }
            Self::Picture(pf) => {
                w.write_str("<a:blipFill>")?;
                write!(w, r#"<a:blip r:embed="{}"/>"#, pf.image_r_id)?;
                if pf.tile {
                    w.write_str("<a:tile/>")?;
                } else if pf.stretch {
                    w.write_str("<a:stretch><a:fillRect/></a:stretch>")?;
                }
                w.write_str("</a:blipFill>")
            }
            Self::Background => w.write_str("<a:grpFill/>"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dml::color::ColorFormat;
    use crate::dml::fill::types::PatternFill;
    use crate::enums::dml::MsoFillType;
    use crate::enums::dml_pattern::MsoPatternType;

    #[test]
    fn test_no_fill_xml() {
        let f = FillFormat::no_fill();
        assert_eq!(f.to_xml_string(), "<a:noFill/>");
    }

    #[test]
    fn test_solid_fill_xml() {
        let f = FillFormat::solid(ColorFormat::rgb(255, 0, 0));
        let xml = f.to_xml_string();
        assert!(xml.starts_with("<a:solidFill>"));
        assert!(xml.contains(r#"<a:srgbClr val="FF0000"/>"#));
        assert!(xml.ends_with("</a:solidFill>"));
    }

    #[test]
    fn test_gradient_fill_xml() {
        let f = FillFormat::linear_gradient(
            ColorFormat::rgb(255, 0, 0),
            ColorFormat::rgb(0, 0, 255),
            90.0,
        );
        let xml = f.to_xml_string();
        assert!(xml.starts_with("<a:gradFill>"));
        assert!(xml.contains("<a:gsLst>"));
        assert!(xml.contains(r#"pos="0""#));
        assert!(xml.contains(r#"pos="100000""#));
        assert!(xml.contains("FF0000"));
        assert!(xml.contains("0000FF"));
        assert!(xml.contains("<a:lin"));
        assert!(xml.ends_with("</a:gradFill>"));
    }

    #[test]
    fn test_pattern_fill_xml() {
        let f = FillFormat::Pattern(PatternFill {
            preset: Some(MsoPatternType::Cross),
            fore_color: Some(ColorFormat::rgb(0, 0, 0)),
            back_color: Some(ColorFormat::rgb(255, 255, 255)),
        });
        let xml = f.to_xml_string();
        assert!(xml.contains(r#"prst="cross""#));
        assert!(xml.contains("<a:fgClr>"));
        assert!(xml.contains("<a:bgClr>"));
        assert!(xml.contains("000000"));
        assert!(xml.contains("FFFFFF"));
    }

    #[test]
    fn test_pattern_fill_with_enum_variants() {
        let f = FillFormat::Pattern(PatternFill {
            preset: Some(MsoPatternType::Percent50),
            fore_color: None,
            back_color: None,
        });
        let xml = f.to_xml_string();
        assert!(xml.contains(r#"prst="pct50""#));
    }

    #[test]
    fn test_pattern_fill_no_preset() {
        let f = FillFormat::Pattern(PatternFill {
            preset: None,
            fore_color: Some(ColorFormat::rgb(255, 0, 0)),
            back_color: None,
        });
        let xml = f.to_xml_string();
        assert!(xml.starts_with("<a:pattFill>"));
        assert!(!xml.contains("prst="));
    }

    #[test]
    fn test_solid_fill_convenience() {
        let f = FillFormat::solid(ColorFormat::rgb(128, 128, 128));
        match &f {
            FillFormat::Solid(sf) => {
                assert_eq!(sf.color, ColorFormat::rgb(128, 128, 128));
            }
            _ => panic!("expected Solid variant"), // EXCEPTION(test-only)
        }
    }

    #[test]
    fn test_picture_fill_stretch() {
        // EXCEPTION(unwrap): test-only code with known-valid input
        let f = FillFormat::picture("rId1").unwrap();
        let xml = f.to_xml_string();
        assert!(xml.contains(r#"r:embed="rId1""#));
        assert!(xml.contains("<a:stretch>"));
        assert!(!xml.contains("<a:tile/>"));
    }

    #[test]
    fn test_picture_fill_tiled() {
        // EXCEPTION(unwrap): test-only code with known-valid input
        let f = FillFormat::picture_tiled("rId2").unwrap();
        let xml = f.to_xml_string();
        assert!(xml.contains(r#"r:embed="rId2""#));
        assert!(xml.contains("<a:tile/>"));
        assert!(!xml.contains("<a:stretch>"));
    }

    #[test]
    fn test_background_fill_xml() {
        let f = FillFormat::background();
        assert_eq!(f.to_xml_string(), "<a:grpFill/>");
    }

    #[test]
    fn test_fill_type_no_fill() {
        let f = FillFormat::no_fill();
        assert_eq!(f.fill_type(), MsoFillType::Background);
    }

    #[test]
    fn test_fill_type_solid() {
        let f = FillFormat::solid(ColorFormat::rgb(255, 0, 0));
        assert_eq!(f.fill_type(), MsoFillType::Solid);
    }

    #[test]
    fn test_fill_type_gradient() {
        let f = FillFormat::linear_gradient(
            ColorFormat::rgb(255, 0, 0),
            ColorFormat::rgb(0, 0, 255),
            90.0,
        );
        assert_eq!(f.fill_type(), MsoFillType::Gradient);
    }

    #[test]
    fn test_fill_type_pattern() {
        let f = FillFormat::Pattern(PatternFill {
            preset: Some(MsoPatternType::Cross),
            fore_color: None,
            back_color: None,
        });
        assert_eq!(f.fill_type(), MsoFillType::Patterned);
    }

    #[test]
    fn test_fill_type_picture() {
        // EXCEPTION(unwrap): test-only code with known-valid input
        let f = FillFormat::picture("rId1").unwrap();
        assert_eq!(f.fill_type(), MsoFillType::Picture);
    }

    #[test]
    fn test_fill_type_background() {
        let f = FillFormat::background();
        assert_eq!(f.fill_type(), MsoFillType::Group);
    }
}
