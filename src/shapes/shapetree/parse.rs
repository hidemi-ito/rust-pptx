//! XML parsing logic for extracting shapes from a slide's `<p:spTree>` element.

use quick_xml::events::{BytesStart, Event};
use quick_xml::Reader;

use crate::enums::shapes::{PlaceholderOrientation, PlaceholderSize, PpPlaceholderType};
use crate::error::{PptxError, PptxResult};
use crate::shapes::placeholder::PlaceholderFormat;
use crate::shapes::Shape;
use crate::units::{PlaceholderIndex, ShapeId};
use crate::xml_util::{attr_value, attr_value_ns, local_name_owned};

use super::parse_accum::{ShapeAccum, ShapeKind};
use super::xml_capture::{CaptureTarget, XmlCapture};

// --- Internal types for state machine parser ---

#[derive(Debug, PartialEq)]
enum ParseState {
    Seeking,
    InSpTree,
    InShape,
}

struct ElementCtx {
    local: String,
}

/// Process a Start or Empty element event within a shape context.
fn process_start_element(
    local: &str,
    e: &BytesStart<'_>,
    accum: &mut ShapeAccum,
) -> PptxResult<()> {
    match local {
        "cNvPr" => {
            accum.shape_id = ShapeId(parse_u32_attr(e, b"id")?);
            accum.name = attr_value(e, b"name")?
                .map(std::borrow::Cow::into_owned)
                .unwrap_or_default();
            if let Some(desc) = attr_value(e, b"descr")? {
                accum.description = Some(desc.into_owned());
            }
        }
        "cNvSpPr" => {
            accum.is_textbox = attr_value(e, b"txBox")?.as_deref() == Some("1");
        }
        "ph" => {
            accum.placeholder = Some(PlaceholderFormat {
                ph_type: attr_value(e, b"type")?.and_then(|c| PpPlaceholderType::from_xml_str(&c)),
                idx: PlaceholderIndex(parse_u32_attr(e, b"idx")?),
                orient: attr_value(e, b"orient")?
                    .and_then(|c| PlaceholderOrientation::from_xml_str(&c)),
                sz: attr_value(e, b"sz")?.and_then(|c| PlaceholderSize::from_xml_str(&c)),
            });
        }
        "off" => {
            accum.left = parse_i64_attr(e, b"x")?;
            accum.top = parse_i64_attr(e, b"y")?;
        }
        "ext" => {
            accum.width = parse_i64_attr(e, b"cx")?;
            accum.height = parse_i64_attr(e, b"cy")?;
        }
        "xfrm" => {
            if let Some(rot_str) = attr_value(e, b"rot")? {
                if let Ok(rot_val) = rot_str.parse::<i64>() {
                    // i64→f64: OOXML rotation values fit in 53-bit mantissa
                    #[allow(clippy::cast_precision_loss)]
                    {
                        accum.rotation = rot_val as f64 / 60000.0;
                    }
                }
            }
            accum.flip_h = attr_value(e, b"flipH")?.as_deref() == Some("1");
            accum.flip_v = attr_value(e, b"flipV")?.as_deref() == Some("1");
        }
        "prstGeom" => {
            accum.prst_geom = attr_value(e, b"prst")?.map(std::borrow::Cow::into_owned);
        }
        "txBody" => {
            accum.has_tx_body = true;
        }
        "blip" => {
            // r:embed attribute (namespaced)
            accum.image_r_id = attr_value_ns(e, b"embed")?.map(std::borrow::Cow::into_owned);
        }
        "graphicData" => {
            accum.graphic_data_uri = attr_value(e, b"uri")?.map(std::borrow::Cow::into_owned);
        }
        "relIds" => {
            // SmartArt diagram: <dgm:relIds r:dm="rIdN" .../>
            accum.smartart_r_id = attr_value_ns(e, b"dm")?.map(std::borrow::Cow::into_owned);
        }
        _ => {}
    }
    Ok(())
}

// --- Attribute parsing helpers ---

/// Parse a `u32` attribute, returning `0` when the attribute is absent or
/// non-numeric.  This is intentional: OOXML attributes such as `id`, `idx`,
/// and other numeric shape properties default to 0 when omitted.
fn parse_u32_attr(e: &BytesStart<'_>, key: &[u8]) -> PptxResult<u32> {
    Ok(attr_value(e, key)?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0))
}

/// Parse an `i64` attribute, returning `0` when the attribute is absent or
/// non-numeric.  Position and extent attributes (`x`, `y`, `cx`, `cy`) in
/// OOXML default to 0 EMU when not present.
fn parse_i64_attr(e: &BytesStart<'_>, key: &[u8]) -> PptxResult<i64> {
    Ok(attr_value(e, key)?
        .and_then(|s| s.parse().ok())
        .unwrap_or(0))
}

/// Parse shapes from slide XML (the full `<p:sld>` or similar element).
///
/// Extracts all shape elements from the `<p:spTree>` within `<p:cSld>`.
#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub(super) fn parse_shapes_from_slide_xml(xml: &[u8]) -> PptxResult<Vec<Shape>> {
    let mut reader = Reader::from_reader(xml);
    reader.config_mut().trim_text(true);

    let mut shapes = Vec::new();
    let mut buf = Vec::new();
    let mut state = ParseState::Seeking;

    // State-machine parser: track where we are in the XML tree
    // using a stack of element contexts.
    let mut element_stack: Vec<ElementCtx> = Vec::new();
    let mut sp_tree_depth: Option<usize> = None;

    // Accumulators for shape being built
    let mut current_shape: Option<ShapeAccum> = None;

    // XML capture state for sub-elements (spPr, txBody, ln)
    let mut capture: Option<XmlCapture> = None;

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let qname = e.name();
                let local = local_name_owned(qname.as_ref());
                let stack_depth = element_stack.len();

                // If we're capturing, record this event AND still process
                if let Some(ref mut cap) = capture {
                    cap.depth += 1;
                    cap.push_start(e);
                    // Also forward to shape accumulator for basic property extraction
                    if let Some(ref mut accum) = current_shape {
                        process_start_element(&local, e, accum)?;
                    }
                } else {
                    match &state {
                        ParseState::Seeking => {
                            if local == "spTree" {
                                sp_tree_depth = Some(stack_depth);
                                state = ParseState::InSpTree;
                            }
                        }
                        ParseState::InSpTree => {
                            // Direct child of spTree
                            if Some(stack_depth) == sp_tree_depth.map(|d| d + 1) {
                                let kind = match local.as_str() {
                                    "sp" => Some(ShapeKind::Sp),
                                    "pic" => Some(ShapeKind::Pic),
                                    "graphicFrame" => Some(ShapeKind::GraphicFrame),
                                    "cxnSp" => Some(ShapeKind::CxnSp),
                                    "grpSp" => Some(ShapeKind::GrpSp),
                                    _ => None,
                                };
                                if let Some(k) = kind {
                                    current_shape = Some(ShapeAccum::new(k));
                                    state = ParseState::InShape;
                                }
                            }
                        }
                        ParseState::InShape => {
                            if let Some(ref mut accum) = current_shape {
                                process_start_element(&local, e, accum)?;
                            }
                            // Start capturing spPr or txBody
                            match local.as_str() {
                                "spPr" => {
                                    let mut cap = XmlCapture::new(CaptureTarget::SpPr);
                                    cap.push_start_with_tag("p:spPr", e);
                                    cap.depth = 1;
                                    capture = Some(cap);
                                }
                                "txBody" => {
                                    let mut cap = XmlCapture::new(CaptureTarget::TxBody);
                                    cap.push_start_with_tag("p:txBody", e);
                                    cap.depth = 1;
                                    capture = Some(cap);
                                }
                                _ => {}
                            }
                        }
                    }
                }

                element_stack.push(ElementCtx { local });
            }
            Ok(Event::Empty(ref e)) => {
                let qname = e.name();
                let local = local_name_owned(qname.as_ref());

                if let Some(ref mut cap) = capture {
                    cap.push_empty(e);
                    // Also forward to shape accumulator for basic property extraction
                    if let Some(ref mut accum) = current_shape {
                        process_start_element(&local, e, accum)?;
                    }
                } else if state == ParseState::InShape {
                    if let Some(ref mut accum) = current_shape {
                        process_start_element(&local, e, accum)?;
                    }
                }
            }
            Ok(Event::Text(ref t)) => {
                if let Some(ref mut cap) = capture {
                    cap.push_text(t.as_ref());
                }
            }
            Ok(Event::End(ref e)) => {
                if let Some(ref mut cap) = capture {
                    cap.depth -= 1;
                    if cap.depth == 0 {
                        // Close the captured element with the full QName
                        cap.push_end_raw(e.name().as_ref());
                        // Store captured XML in the accumulator.
                        // EXCEPTION(infallible): `capture` is `Some` here because we are
                        // inside the `if let Some(ref mut cap) = capture` branch and `take()`
                        // on the same binding always returns `Some`.
                        let Some(finished) = capture.take() else {
                            unreachable!("capture is always Some when depth reaches zero");
                        };
                        if let Some(ref mut accum) = current_shape {
                            match finished.target {
                                CaptureTarget::SpPr => {
                                    accum.sp_pr_xml = Some(finished.xml);
                                }
                                CaptureTarget::TxBody => {
                                    accum.tx_body_xml_bytes = Some(finished.xml);
                                }
                            }
                        }
                    } else {
                        cap.push_end_raw(e.name().as_ref());
                    }
                } else {
                    let popped = element_stack.pop();

                    if state == ParseState::InShape {
                        // Check if we're closing the shape element
                        if let Some(sp_depth) = sp_tree_depth {
                            if element_stack.len() == sp_depth + 1 {
                                // We've closed the shape element
                                if let Some(accum) = current_shape.take() {
                                    shapes.push(accum.into_shape());
                                }
                                state = ParseState::InSpTree;
                            }
                        }
                    } else if state == ParseState::InSpTree {
                        if let Some(ref popped) = popped {
                            if popped.local == "spTree" {
                                state = ParseState::Seeking;
                                sp_tree_depth = None;
                            }
                        }
                    }
                    // Note: element_stack pop already done above when not capturing
                    continue;
                }
                // Pop the element stack when capturing too
                element_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(PptxError::Xml(e)),
            _ => {}
        }
    }

    Ok(shapes)
}
