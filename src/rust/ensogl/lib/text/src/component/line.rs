use crate::prelude::*;

use crate::typeface::glyph;
use crate::typeface::glyph::Glyph;
use crate::buffer;

use ensogl::data::color;
use ensogl::display;



// ============
// === Line ===
// ============

#[derive(Clone,CloneRef,Debug)]
pub struct Line {
    glyphs : Rc<RefCell<Vec<Glyph>>>,
}



// ============
// === Area ===
// ============

/// A structure keeping line of glyphs with proper alignment.
///
/// Not all the glyphs in `glyphs` vector may be actually in use. This structure is meant to keep
/// changing text, and for best performance it re-uses the created Glyphs (what means the specific
/// buffer space). Therefore you can set a cap for line length by using the `set_fixed_capacity`
/// method.
#[derive(Clone,CloneRef,Debug)]
pub struct Area {
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    buffer         : buffer::View,
    lines          : Rc<RefCell<Vec<Line>>>,

}

impl Area {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, buffer:&buffer::View, glyph_system:&glyph::System) -> Self {
        let display_object = display::object::Instance::new(Logger::sub(logger,"line"));
        let glyph_system   = glyph_system.clone_ref();
        let buffer         = buffer.clone_ref();
        let lines          = default();
        Self {display_object,glyph_system,buffer,lines}
    }
}

//
//// === Internal API ===
//
//impl Line {
//    fn resize(&self) {
//        let content_len        = self.content.borrow().len();
//        let target_glyph_count = self.fixed_capacity.get().unwrap_or(content_len);
//        let glyph_count        = self.glyphs.borrow().len();
//        if target_glyph_count > glyph_count {
//            let new_count  = target_glyph_count - glyph_count;
//            let new_glyphs = (0..new_count).map(|_| {
//                let glyph = self.glyph_system.new_glyph();
//                self.add_child(&glyph);
//                glyph
//            });
//            self.glyphs.borrow_mut().extend(new_glyphs)
//        }
//        if glyph_count > target_glyph_count {
//            self.glyphs.borrow_mut().truncate(target_glyph_count)
//        }
//    }
//
//    /// Updates properties of all glyphs, including characters they display, size, and colors.
//    fn redraw(&self) {
//        self.resize();
//
//        let content     = self.content.borrow();
//        let font        = self.glyph_system.font.clone_ref();
//        let font_size   = self.font_size.get();
//        let chars       = content.chars();
//        let pen         = PenIterator::new(font_size,chars,font);
//        let content_len = content.len();
//        let color       = self.font_color.get().into();
//
//        for (glyph,(chr,x_offset)) in self.glyphs.borrow().iter().zip(pen) {
//            let glyph_info   = self.glyph_system.font.get_glyph_info(chr);
//            let size         = glyph_info.scale.scale(font_size);
//            let glyph_offset = glyph_info.offset.scale(font_size);
//            let glyph_x      = x_offset + glyph_offset.x;
//            let glyph_y      = glyph_offset.y;
//            glyph.set_position(Vector3::new(glyph_x,glyph_y,0.0));
//            glyph.set_glyph(chr);
//            glyph.color().set(color);
//            glyph.size.set(size);
//        }
//
//        for glyph in self.glyphs.borrow().iter().skip(content_len) {
//            glyph.size.set(Vector2::new(0.0,0.0));
//        }
//    }
//}
//
//
//// === Display Object ===
//
//impl display::Object for Line {
//    fn display_object(&self) -> &display::object::Instance {
//        &self.display_object
//    }
//}