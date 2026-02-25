use super::*;
use crate::enums::shapes::PresetGeometry;
use crate::units::Inches;

#[test]
fn test_parse_empty_slide() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
</p:spTree></p:cSld></p:sld>"#;

    let tree = ShapeTree::from_slide_xml(xml).unwrap();
    assert_eq!(tree.len(), 0);
}

#[test]
fn test_parse_slide_with_shapes() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:sp>
  <p:nvSpPr><p:cNvPr id="2" name="Title 1"/><p:cNvSpPr><a:spLocks noGrp="1"/></p:cNvSpPr><p:nvPr><p:ph type="title"/></p:nvPr></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="457200" y="274638"/><a:ext cx="8229600" cy="1143000"/></a:xfrm></p:spPr>
  <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr/></a:p></p:txBody>
</p:sp>
<p:sp>
  <p:nvSpPr><p:cNvPr id="3" name="TextBox 2"/><p:cNvSpPr txBox="1"/><p:nvPr/></p:nvSpPr>
  <p:spPr><a:xfrm><a:off x="100000" y="200000"/><a:ext cx="300000" cy="400000"/></a:xfrm><a:prstGeom prst="rect"><a:avLst/></a:prstGeom></p:spPr>
  <p:txBody><a:bodyPr/><a:lstStyle/><a:p><a:endParaRPr/></a:p></p:txBody>
</p:sp>
</p:spTree></p:cSld></p:sld>"#;

    let tree = ShapeTree::from_slide_xml(xml).unwrap();
    assert_eq!(tree.len(), 2);

    let s0 = &tree.shapes[0];
    assert_eq!(s0.name(), "Title 1");
    assert_eq!(s0.shape_id(), ShapeId(2));
    assert!(s0.is_placeholder());
    assert_eq!(s0.left(), Emu(457200));
    assert_eq!(s0.width(), Emu(8229600));

    let s1 = &tree.shapes[1];
    assert_eq!(s1.name(), "TextBox 2");
    assert_eq!(s1.shape_id(), ShapeId(3));
    assert!(!s1.is_placeholder());
    assert!(s1.has_text_frame());
    if let Shape::AutoShape(a) = s1 {
        assert!(a.is_textbox);
        assert_eq!(a.prst_geom, Some(PresetGeometry::Rect));
    }
}

#[test]
fn test_parse_picture() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:pic>
  <p:nvPicPr><p:cNvPr id="4" name="Picture 3" descr="Test image"/><p:cNvPicPr/><p:nvPr/></p:nvPicPr>
  <p:blipFill><a:blip r:embed="rId2"/><a:stretch><a:fillRect/></a:stretch></p:blipFill>
  <p:spPr><a:xfrm><a:off x="0" y="0"/><a:ext cx="914400" cy="914400"/></a:xfrm></p:spPr>
</p:pic>
</p:spTree></p:cSld></p:sld>"#;

    let tree = ShapeTree::from_slide_xml(xml).unwrap();
    assert_eq!(tree.len(), 1);
    let s = &tree.shapes[0];
    assert_eq!(s.name(), "Picture 3");
    if let Shape::Picture(p) = s {
        assert_eq!(p.image_r_id.as_deref(), Some("rId2"));
        assert_eq!(p.description.as_deref(), Some("Test image"));
        assert_eq!(p.width, Emu(914400));
    } else {
        panic!("Expected Picture shape");
    }
}

#[test]
fn test_parse_connector() {
    let xml = br#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<p:sld xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships" xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main">
<p:cSld><p:spTree>
<p:nvGrpSpPr><p:cNvPr id="1" name=""/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr>
<p:grpSpPr/>
<p:cxnSp>
  <p:nvCxnSpPr><p:cNvPr id="5" name="Connector 4"/><p:cNvCxnSpPr/><p:nvPr/></p:nvCxnSpPr>
  <p:spPr><a:xfrm flipH="1"><a:off x="100" y="200"/><a:ext cx="500" cy="600"/></a:xfrm><a:prstGeom prst="line"><a:avLst/></a:prstGeom></p:spPr>
</p:cxnSp>
</p:spTree></p:cSld></p:sld>"#;

    let tree = ShapeTree::from_slide_xml(xml).unwrap();
    assert_eq!(tree.len(), 1);
    if let Shape::Connector(c) = &tree.shapes[0] {
        assert_eq!(c.name, "Connector 4");
        assert!(c.flip_h);
        assert!(!c.flip_v);
        assert_eq!(c.prst_geom, Some(PresetGeometry::Line));
    } else {
        panic!("Expected Connector shape");
    }
}

#[test]
fn test_new_textbox_xml() {
    let xml = ShapeTree::new_textbox_xml(
        ShapeId(5),
        "TextBox 4",
        Emu(914400),
        Emu(914400),
        Emu(2743200),
        Emu(457200),
    );
    assert!(xml.contains(r#"id="5""#));
    assert!(xml.contains(r#"name="TextBox 4""#));
    assert!(xml.contains(r#"txBox="1""#));
    assert!(xml.contains(r#"x="914400""#));
}

#[test]
fn test_new_autoshape_xml() {
    let xml = ShapeTree::new_autoshape_xml(
        ShapeId(6),
        "Oval 5",
        Emu(100000),
        Emu(200000),
        Emu(300000),
        Emu(400000),
        "ellipse",
    );
    assert!(xml.contains(r#"prst="ellipse""#));
    assert!(xml.contains(r#"id="6""#));
}

#[test]
fn test_new_picture_xml() {
    let xml = ShapeTree::new_picture_xml(
        ShapeId(7),
        "Picture 6",
        "A photo",
        "rId2",
        Emu(0),
        Emu(0),
        Emu(914400),
        Emu(914400),
    );
    assert!(xml.contains(r#"r:embed="rId2""#));
    assert!(xml.contains(r#"descr="A photo""#));
}

#[test]
fn test_new_table_xml() {
    let xml = ShapeTree::new_table_xml(
        ShapeId(8),
        "Table 7",
        2,
        3,
        Emu(914400),
        Emu(914400),
        Emu(2743200),
        Emu(914400),
    );
    assert!(xml.contains("a:tbl"));
    assert!(xml.contains("a:tr"));
    assert!(xml.contains("a:tc"));
}

#[test]
fn test_units_in_shapes() {
    let left: Emu = Inches(1.0).into();
    let top: Emu = Inches(2.0).into();
    assert_eq!(left, Emu(914400));
    assert_eq!(top, Emu(1828800));
}

#[test]
fn test_shape_name_for_prst_common() {
    assert_eq!(shape_name_for_prst("rect"), "Rectangle");
    assert_eq!(shape_name_for_prst("ellipse"), "Oval");
    assert_eq!(shape_name_for_prst("roundRect"), "Rounded Rectangle");
    assert_eq!(shape_name_for_prst("diamond"), "Diamond");
    assert_eq!(shape_name_for_prst("cloud"), "Cloud");
    assert_eq!(shape_name_for_prst("heart"), "Heart");
}

#[test]
fn test_shape_name_for_prst_unknown() {
    assert_eq!(shape_name_for_prst("unknownShape"), "Freeform");
}

#[test]
fn test_turbo_add_flag() {
    let mut tree = ShapeTree::default();
    assert!(!tree.turbo_add_enabled());
    tree.set_turbo_add_enabled(true);
    assert!(tree.turbo_add_enabled());
    tree.set_turbo_add_enabled(false);
    assert!(!tree.turbo_add_enabled());
}

#[test]
fn test_new_connector_xml_with_flip() {
    let xml = ShapeTree::new_connector_xml_with_flip(
        ShapeId(5),
        "Connector 1",
        Emu(100),
        Emu(200),
        Emu(300),
        Emu(400),
        "line",
        true,
        false,
    );
    assert!(xml.contains(r#"flipH="1""#));
    assert!(!xml.contains(r#"flipV="1""#));
    assert!(xml.contains(r#"prst="line""#));
}

#[test]
fn test_new_connector_xml_with_flip_both() {
    let xml = ShapeTree::new_connector_xml_with_flip(
        ShapeId(5),
        "Connector 1",
        Emu(100),
        Emu(200),
        Emu(300),
        Emu(400),
        "line",
        true,
        true,
    );
    assert!(xml.contains(r#"flipH="1""#));
    assert!(xml.contains(r#"flipV="1""#));
}

#[test]
fn test_new_connector_xml_with_no_flip() {
    let xml = ShapeTree::new_connector_xml_with_flip(
        ShapeId(5),
        "Connector 1",
        Emu(100),
        Emu(200),
        Emu(300),
        Emu(400),
        "line",
        false,
        false,
    );
    assert!(!xml.contains("flipH"));
    assert!(!xml.contains("flipV"));
}

#[test]
fn test_new_group_shape_xml() {
    let xml = ShapeTree::new_group_shape_xml(
        ShapeId(10),
        "Group 1",
        Emu(0),
        Emu(0),
        Emu(914400),
        Emu(914400),
    );
    assert!(xml.contains(r#"id="10""#));
    assert!(xml.contains(r#"name="Group 1""#));
    assert!(xml.contains("<p:grpSp"));
    assert!(xml.contains("<a:chOff"));
    assert!(xml.contains("<a:chExt"));
}
