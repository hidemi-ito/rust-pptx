//! XML-to-struct parsers for read-modify-write support.
//!
//! These functions parse OOXML shape sub-elements (text frame, fill, line)
//! back into their Rust struct representation, enabling round-tripping:
//! open .pptx -> parse shapes into structs -> modify -> save.

mod color;
mod fill;
mod line;
mod text_frame;
mod text_helpers;

pub use fill::parse_fill_from_xml;
pub use line::parse_line_from_xml;
pub use text_frame::parse_text_frame_from_xml;

use quick_xml::events::Event;
use quick_xml::Reader;

use color::parse_color_from_xml;

use crate::dml::fill::FillFormat;
use crate::dml::line::LineFormat;
use crate::error::{PptxError, PptxResult};
use crate::xml_util::{attr_value, local_name_str, read_inner_xml};

// ============================================================
// spPr parsing (extract fill + line from full spPr XML)
// ============================================================

/// Parse both fill and line from a `<p:spPr>` or `<a:spPr>` XML fragment.
///
/// Returns `(fill, line)`.
pub fn parse_sp_pr(sp_pr_bytes: &[u8]) -> PptxResult<(Option<FillFormat>, Option<LineFormat>)> {
    let mut reader = Reader::from_reader(sp_pr_bytes);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut fill: Option<FillFormat> = None;
    let mut line: Option<LineFormat> = None;
    let mut depth: u32 = 0;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let qn = e.name();
                let local = local_name_str(qn.as_ref());
                match local {
                    "solidFill" if depth <= 2 => {
                        let inner = read_inner_xml(&mut reader, "solidFill")
                            .map_err(|e| PptxError::InvalidXml(format!("solidFill: {e}")))?;
                        if let Some(color) = parse_color_from_xml(&inner)? {
                            fill = Some(FillFormat::Solid(crate::dml::fill::SolidFill { color }));
                        }
                    }
                    "gradFill" if depth <= 2 => {
                        let inner = read_inner_xml(&mut reader, "gradFill")
                            .map_err(|e| PptxError::InvalidXml(format!("gradFill: {e}")))?;
                        fill = fill::parse_gradient_from_inner(&inner)?;
                    }
                    "ln" => {
                        let w_attr = attr_value(e, b"w")?;
                        let inner = read_inner_xml(&mut reader, "ln")
                            .map_err(|e| PptxError::InvalidXml(format!("ln: {e}")))?;
                        // Reconstruct the <a:ln> element for parse_line_from_xml
                        let mut ln_xml = String::from("<a:ln");
                        if let Some(w) = w_attr {
                            ln_xml.push_str(&format!(r#" w="{w}""#));
                        }
                        ln_xml.push('>');
                        ln_xml.push_str(&String::from_utf8_lossy(&inner));
                        ln_xml.push_str("</a:ln>");
                        line = line::parse_line_from_xml(ln_xml.as_bytes())?;
                    }
                    _ => {}
                }
            }
            Ok(Event::Empty(ref e)) => {
                let qn = e.name();
                let local = local_name_str(qn.as_ref());
                match local {
                    "noFill" if depth <= 1 => {
                        fill = Some(FillFormat::NoFill);
                    }
                    "grpFill" if depth <= 1 => {
                        fill = Some(FillFormat::Background);
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PptxError::InvalidXml(format!("spPr XML error: {e}"))),
            _ => {}
        }
    }

    Ok((fill, line))
}

#[cfg(test)]
mod tests;
