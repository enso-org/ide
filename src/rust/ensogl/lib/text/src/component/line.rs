use crate::prelude::*;
use crate::traits::*;

use crate::typeface::glyph;
use crate::typeface::pen;
use crate::typeface::glyph::Glyph;
use crate::buffer;

use ensogl::data::color;
use ensogl::display;
use crate::buffer::view::LineOffset;


// ============
// === Line ===
// ============

#[derive(Debug)]
pub struct Line {
    display_object : display::object::Instance,
    glyphs         : Vec<Glyph>,
}

impl Line {
    fn new(logger:impl AnyLogger) -> Self {
        let logger         = Logger::sub(logger,"line");
        let display_object = display::object::Instance::new(logger);
        let glyphs         = default();
        Self {display_object,glyphs}
    }

    fn resize_with(&mut self, size:usize, cons:impl Fn()->Glyph) {
        let display_object = self.display_object().clone_ref();
        self.glyphs.resize_with(size,move || {
            let glyph = cons();
            display_object.add_child(&glyph);
            glyph
        });
    }
}

impl display::Object for Line {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =============
// === Lines ===
// =============

#[derive(Clone,CloneRef,Debug,Default)]
struct Lines {
    rc : Rc<RefCell<Vec<Line>>>
}

impl Lines {
    pub fn len(&self) -> usize {
        self.rc.borrow().len()
    }

    pub fn resize_with(&self, size:usize, cons:impl Fn(usize)->Line) {
        let vec    = &mut self.rc.borrow_mut();
        let mut ix = vec.len();
        vec.resize_with(size,|| {
            let line = cons(ix);
            ix += 1;
            line
        })
    }
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
    logger         : Logger,
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    buffer_view    : buffer::View,
    lines          : Lines,

}

impl Area {
    /// Constructor.
    pub fn new
    (logger:impl AnyLogger, buffer_view:&buffer::View, glyph_system:&glyph::System) -> Self {
        let logger         = Logger::sub(logger,"text_area");
        let display_object = display::object::Instance::new(&logger);
        let glyph_system   = glyph_system.clone_ref();
        let buffer_view    = buffer_view.clone_ref();
        let lines          = default();
        Self {logger,display_object,glyph_system,buffer_view,lines} . init()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn init(self) -> Self {
        self.redraw();
        self
    }

    fn redraw(&self) {
        let line_count = self.buffer_view.line_count();
        self.lines.resize_with(line_count,|ix| self.new_line(ix));
        for (view_line_number,content) in self.buffer_view.lines().enumerate() {
            self.redraw_line(view_line_number,content)
        }
    }

    fn redraw_line(&self, view_line_number:usize, content:Cow<str>) {
        let font_size = 10.0; // FIXME
//        let color     = color::Rgba::new(1.0,0.0,0.0,1.0);
//        let style     = self.buffer_view.style.current(); // FIXME: refactor to redraw
        let line      = &mut self.lines.rc.borrow_mut()[view_line_number];

        let line_range = self.buffer_view.range_of_view_line_raw(buffer::Line(view_line_number));
        let mut line_style = self.buffer_view.focus_style(line_range.start .. line_range.end).iter();

//        let style_cursor = style.cursor(self.buffer_view.first_line_offset().raw);
        // FIXME clone:
        let pen       = pen::Iterator::new(10.0,content.chars(),self.glyph_system.font.clone_ref());
        line.resize_with(content.len(),||self.glyph_system.new_glyph());
        for (glyph,info) in line.glyphs.iter().zip(pen) {
            let style = line_style.next().unwrap_or_default();
            line_style.drop((info.char.len_utf8() - 1).bytes());
//            println!("?? {:?}", style_cursor.get_leaf());
//            style_cursor.next();
            let glyph_info   = self.glyph_system.font.get_glyph_info(info.char);
            let size         = glyph_info.scale.scale(font_size);
            let glyph_offset = glyph_info.offset.scale(font_size);
            let glyph_x      = info.offset + glyph_offset.x;
            let glyph_y      = glyph_offset.y;
            glyph.set_position_xy(Vector2(glyph_x,glyph_y));
            glyph.set_char(info.char);
            glyph.set_color(style.color);
            glyph.size.set(size);
        }
    }

    fn new_line(&self, index:usize) -> Line {
        let line     = Line::new(&self.logger);
        let y_offset = - (index as f32) * 12.0; // FIXME line height?
        line.set_position_y(y_offset);
        self.add_child(&line);
        line
    }
}

impl display::Object for Area {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
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
// === Display Object ===

