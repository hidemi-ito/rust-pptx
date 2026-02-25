use crate::xml_util::{xml_escape, WriteXml};

use super::GroupShape;

impl WriteXml for GroupShape {
    fn write_xml<W: std::fmt::Write>(&self, w: &mut W) -> std::fmt::Result {
        w.write_str("<p:grpSp>")?;

        // --- nvGrpSpPr ---
        w.write_str("<p:nvGrpSpPr>")?;
        write!(
            w,
            r#"<p:cNvPr id="{}" name="{}"/>"#,
            self.shape_id,
            xml_escape(&self.name)
        )?;
        w.write_str("<p:cNvGrpSpPr/>")?;
        w.write_str("<p:nvPr/>")?;
        w.write_str("</p:nvGrpSpPr>")?;

        // --- grpSpPr ---
        w.write_str("<p:grpSpPr>")?;

        w.write_str("<a:xfrm")?;
        if self.rotation != 0.0 {
            // f64→i64: rotation degrees * 60000 fits in i64
            #[allow(clippy::cast_possible_truncation)]
            let rot = (self.rotation * 60000.0) as i64;
            write!(w, r#" rot="{rot}""#)?;
        }
        w.write_char('>')?;
        write!(w, r#"<a:off x="{}" y="{}"/>"#, self.left.0, self.top.0)?;
        write!(
            w,
            r#"<a:ext cx="{}" cy="{}"/>"#,
            self.width.0, self.height.0
        )?;
        // Child coordinate space maps to the same as the group bounds by default
        write!(w, r#"<a:chOff x="{}" y="{}"/>"#, self.left.0, self.top.0)?;
        write!(
            w,
            r#"<a:chExt cx="{}" cy="{}"/>"#,
            self.width.0, self.height.0
        )?;
        w.write_str("</a:xfrm>")?;
        w.write_str("</p:grpSpPr>")?;

        // --- child shapes ---
        for shape in &self.shapes {
            shape.write_xml(w)?;
        }

        w.write_str("</p:grpSp>")?;
        Ok(())
    }
}
