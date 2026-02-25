use super::*;
use crate::dml::color::ColorFormat;
use crate::dml::fill::FillFormat;
use crate::dml::line::LineFormat;
use crate::enums::dml::{MsoLineDashStyle, MsoThemeColorIndex, SystemColorVal};
use crate::enums::text::{
    MsoAutoSize, MsoTextUnderlineType, MsoVerticalAnchor, PpParagraphAlignment,
};
use crate::text::font::RgbColor;
use crate::units::Emu;
use crate::xml_util::WriteXml;

#[test]
fn test_parse_solid_fill_rgb() {
    let xml = br#"<a:spPr><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill></a:spPr>"#;
    let fill = parse_fill_from_xml(xml).unwrap();
    assert!(fill.is_some());
    match fill.unwrap() {
        FillFormat::Solid(sf) => match sf.color {
            ColorFormat::Rgb(rgb) => {
                assert_eq!(rgb.r, 255);
                assert_eq!(rgb.g, 0);
                assert_eq!(rgb.b, 0);
            }
            _ => panic!("Expected RGB color"),
        },
        _ => panic!("Expected Solid fill"),
    }
}

#[test]
fn test_parse_no_fill() {
    let xml = br#"<a:spPr><a:noFill/></a:spPr>"#;
    let fill = parse_fill_from_xml(xml).unwrap();
    assert_eq!(fill, Some(FillFormat::NoFill));
}

#[test]
fn test_parse_gradient_fill() {
    let xml = br#"<a:spPr><a:gradFill><a:gsLst><a:gs pos="0"><a:srgbClr val="FF0000"/></a:gs><a:gs pos="100000"><a:srgbClr val="0000FF"/></a:gs></a:gsLst><a:lin ang="16200000" scaled="0"/></a:gradFill></a:spPr>"#;
    let fill = parse_fill_from_xml(xml).unwrap();
    assert!(fill.is_some());
    match fill.unwrap() {
        FillFormat::Gradient(gf) => {
            assert_eq!(gf.stops.len(), 2);
            assert_eq!(gf.stops[0].position, 0.0);
            assert_eq!(gf.stops[1].position, 1.0);
            assert!(gf.angle.is_some());
        }
        _ => panic!("Expected Gradient fill"),
    }
}

#[test]
fn test_parse_solid_fill_scheme_color() {
    let xml = br#"<a:spPr><a:solidFill><a:schemeClr val="accent1"/></a:solidFill></a:spPr>"#;
    let fill = parse_fill_from_xml(xml).unwrap();
    assert!(fill.is_some());
    match fill.unwrap() {
        FillFormat::Solid(sf) => match sf.color {
            ColorFormat::Theme(tc) => {
                assert_eq!(tc.theme_color, MsoThemeColorIndex::Accent1);
                assert!(tc.brightness.is_none());
            }
            _ => panic!("Expected Theme color"),
        },
        _ => panic!("Expected Solid fill"),
    }
}

#[test]
fn test_parse_line_solid() {
    let xml = br#"<a:ln w="12700"><a:solidFill><a:srgbClr val="000000"/></a:solidFill></a:ln>"#;
    let line = parse_line_from_xml(xml).unwrap();
    assert!(line.is_some());
    let l = line.unwrap();
    assert_eq!(l.width, Some(Emu(12700)));
    assert!(l.color.is_some());
    assert!(l.fill.is_some());
}

#[test]
fn test_parse_line_with_dash() {
    let xml = br#"<a:ln w="25400"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:prstDash val="dash"/></a:ln>"#;
    let line = parse_line_from_xml(xml).unwrap();
    assert!(line.is_some());
    let l = line.unwrap();
    assert_eq!(l.width, Some(Emu(25400)));
    assert_eq!(l.dash_style, Some(MsoLineDashStyle::Dash));
}

#[test]
fn test_parse_text_frame_basic() {
    let xml = br#"<p:txBody><a:bodyPr wrap="square" lIns="91440" tIns="45720" rIns="91440" bIns="45720" anchor="ctr"/><a:lstStyle/><a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="en-US" sz="1800" b="1"/><a:t>Hello World</a:t></a:r></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap();
    assert!(tf.is_some());
    let tf = tf.unwrap();
    assert!(tf.word_wrap);
    assert_eq!(tf.vertical_anchor, Some(MsoVerticalAnchor::Middle));
    assert_eq!(tf.margin_left, Some(crate::units::Emu(91440)));
    assert_eq!(tf.paragraphs().len(), 1);
    let para = &tf.paragraphs()[0];
    assert_eq!(para.alignment, Some(PpParagraphAlignment::Center));
    assert_eq!(para.runs().len(), 1);
    assert_eq!(para.runs()[0].text(), "Hello World");
    assert_eq!(para.runs()[0].font().bold, Some(true));
    assert_eq!(para.runs()[0].font().size, Some(18.0));
}

#[test]
fn test_parse_text_frame_multi_paragraph() {
    let xml = br#"<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US"/><a:t>Line 1</a:t></a:r></a:p><a:p><a:r><a:rPr lang="en-US"/><a:t>Line 2</a:t></a:r></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    assert_eq!(tf.paragraphs().len(), 2);
    assert_eq!(tf.text(), "Line 1\nLine 2");
}

#[test]
fn test_parse_text_frame_with_font_color() {
    let xml = br#"<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:r><a:rPr lang="en-US" sz="2400" b="1" i="1" u="sng"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:latin typeface="Arial"/></a:rPr><a:t>Styled</a:t></a:r></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    let run = &tf.paragraphs()[0].runs()[0];
    let font = run.font();
    assert_eq!(font.bold, Some(true));
    assert_eq!(font.italic, Some(true));
    assert_eq!(font.size, Some(24.0));
    assert_eq!(font.underline, Some(MsoTextUnderlineType::SingleLine));
    assert_eq!(font.color, Some(RgbColor::new(255, 0, 0)));
    assert_eq!(font.name.as_deref(), Some("Arial"));
}

#[test]
fn test_parse_text_frame_no_wrap() {
    let xml = br#"<p:txBody><a:bodyPr wrap="none"/><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    assert!(!tf.word_wrap);
}

#[test]
fn test_parse_text_frame_autofit() {
    let xml = br#"<p:txBody><a:bodyPr><a:normAutofit fontScale="80000"/></a:bodyPr><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    assert_eq!(tf.auto_size, MsoAutoSize::TextToFitShape);
    assert_eq!(tf.font_scale, Some(80.0));
}

#[test]
fn test_parse_sp_pr() {
    let xml = br#"<p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="100" cy="100"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom><a:solidFill><a:srgbClr val="00FF00"/></a:solidFill><a:ln w="12700"><a:solidFill><a:srgbClr val="000000"/></a:solidFill></a:ln></p:spPr>"#;
    let (fill, line) = parse_sp_pr(xml).unwrap();
    assert!(fill.is_some());
    assert!(line.is_some());
    match fill.unwrap() {
        FillFormat::Solid(sf) => match sf.color {
            ColorFormat::Rgb(rgb) => {
                assert_eq!(rgb.r, 0);
                assert_eq!(rgb.g, 255);
                assert_eq!(rgb.b, 0);
            }
            _ => panic!("Expected RGB color"),
        },
        _ => panic!("Expected Solid fill"),
    }
    let l = line.unwrap();
    assert_eq!(l.width, Some(Emu(12700)));
}

#[test]
fn test_parse_text_frame_empty_body() {
    let xml =
        br#"<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr lang="en-US"/></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    assert_eq!(tf.paragraphs().len(), 1);
    assert_eq!(tf.text(), "");
}

#[test]
fn test_parse_line_no_fill() {
    let xml = br#"<a:ln w="9525"><a:noFill/></a:ln>"#;
    let line = parse_line_from_xml(xml).unwrap();
    assert!(line.is_some());
    let l = line.unwrap();
    assert_eq!(l.width, Some(Emu(9525)));
    assert_eq!(l.fill, Some(FillFormat::NoFill));
}

#[test]
fn test_parse_fill_background() {
    let xml = br#"<a:spPr><a:grpFill/></a:spPr>"#;
    let fill = parse_fill_from_xml(xml).unwrap();
    assert_eq!(fill, Some(FillFormat::Background));
}

#[test]
fn test_parse_color_system() {
    let xml = br#"<a:sysClr val="windowText" lastClr="000000"/>"#;
    let color = parse_color_from_xml(xml).unwrap();
    assert!(color.is_some());
    match color.unwrap() {
        ColorFormat::System(sys) => {
            assert_eq!(sys.val, SystemColorVal::WindowText);
            assert_eq!(sys.last_color.as_deref(), Some("000000"));
        }
        _ => panic!("Expected System color"),
    }
}

#[test]
fn test_parse_text_frame_paragraph_level() {
    let xml = br#"<p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:pPr lvl="2" algn="r"/><a:r><a:rPr lang="en-US"/><a:t>Indented</a:t></a:r></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    let para = &tf.paragraphs()[0];
    assert_eq!(para.level, 2);
    assert_eq!(para.alignment, Some(PpParagraphAlignment::Right));
}

#[test]
fn test_round_trip_text_frame() {
    // Parse a text frame, then generate XML and parse again
    let xml = br#"<p:txBody><a:bodyPr wrap="square" lIns="91440" tIns="45720" rIns="91440" bIns="45720" anchor="ctr"/><a:lstStyle/><a:p><a:pPr algn="ctr"/><a:r><a:rPr lang="en-US" sz="1800" b="1"><a:solidFill><a:srgbClr val="FF0000"/></a:solidFill><a:latin typeface="Arial"/></a:rPr><a:t>Hello</a:t></a:r></a:p></p:txBody>"#;
    let tf = parse_text_frame_from_xml(xml).unwrap().unwrap();
    assert_eq!(tf.text(), "Hello");
    assert_eq!(tf.paragraphs()[0].runs()[0].font().size, Some(18.0));
    assert_eq!(tf.paragraphs()[0].runs()[0].font().bold, Some(true));
    assert_eq!(
        tf.paragraphs()[0].runs()[0].font().color,
        Some(RgbColor::new(255, 0, 0))
    );
    assert_eq!(
        tf.paragraphs()[0].runs()[0].font().name.as_deref(),
        Some("Arial")
    );

    // Generate XML from parsed struct
    let generated = tf.to_xml_string();
    // Re-parse the generated XML
    let tf2 = parse_text_frame_from_xml(generated.as_bytes())
        .unwrap()
        .unwrap();
    assert_eq!(tf2.text(), "Hello");
    assert_eq!(tf2.paragraphs()[0].runs()[0].font().bold, Some(true));
    assert_eq!(tf2.paragraphs()[0].runs()[0].font().size, Some(18.0));
}

#[test]
fn test_round_trip_fill_and_line() {
    // Create fill + line, generate XML, parse back
    let fill = FillFormat::solid(ColorFormat::rgb(128, 0, 255));
    let line = LineFormat::solid(ColorFormat::rgb(0, 0, 0), Emu(12700));

    // Generate XML
    let mut sp_pr = String::from("<p:spPr>");
    sp_pr.push_str(&fill.to_xml_string());
    if let Some(ln_xml) = line.to_xml_string() {
        sp_pr.push_str(&ln_xml);
    }
    sp_pr.push_str("</p:spPr>");

    // Parse back
    let (parsed_fill, parsed_line) = parse_sp_pr(sp_pr.as_bytes()).unwrap();
    assert!(parsed_fill.is_some());
    assert!(parsed_line.is_some());

    match parsed_fill.unwrap() {
        FillFormat::Solid(sf) => match sf.color {
            ColorFormat::Rgb(rgb) => {
                assert_eq!(rgb.r, 128);
                assert_eq!(rgb.g, 0);
                assert_eq!(rgb.b, 255);
            }
            _ => panic!("Expected RGB"),
        },
        _ => panic!("Expected Solid"),
    }

    let l = parsed_line.unwrap();
    assert_eq!(l.width, Some(Emu(12700)));
}
