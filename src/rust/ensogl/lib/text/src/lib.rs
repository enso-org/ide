//! Ensogl text rendering implementation.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

pub mod glyph;
pub mod buffer;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl::prelude::*;
}

pub use ensogl::display;
pub use ensogl::data;

use buffer::*;

use selection::Selection;

//use prelude::*;
//
//
use crate::buffer::location::*;


use text::rope::spans::Spans;
use text::rope::spans::SpansBuilder;
use text::rope::breaks::{BreakBuilder, Breaks, BreaksInfo, BreaksMetric};
use text::rope::{Interval, LinesMetric, Rope, RopeDelta, RopeInfo};
//use rope::LinesMetric;
//use rope::rope::BaseMetric;
//use rope::tree::*;
//
//
//
//
//
//
//pub struct Line {
//    text  : Rope,
//    index : usize,
//}

use std::cmp::max;
use std::cmp::min;




pub fn main() {
    let buffer = Buffer::from("Test text!");
    buffer.color.set(1..3,color::Rgba::new(1.0,0.0,0.0,1.0));
    let mut view = buffer.new_view();

    view.add_selection(Selection::new(ByteOffset(0),ByteOffset(0)));


//    let foo = buffer.color.iter().collect_vec();
    let foo = buffer.color.subseq(2..5);
//    let foo = foo.iter().collect_vec();
    println!("{:#?}",foo);

    println!("{:#?}",view.selections());

    view.move_carets(Movement::Right);

    println!("{:#?}",view.selections());
}



//#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
//pub struct BufferId(pub usize);
//
//pub struct BufferMap {
//    map : BTreeMap<BufferId,Buffer>
//}
//






















use crate::prelude::*;
use ensogl::data::color;
use crate::display::shape::text::glyph::font;
use crate::display::shape::text::glyph::pen::PenIterator;
use glyph::Glyph;


// ============
// === Line ===
// ============

/// A structure keeping line of glyphs with proper alignment.
///
/// Not all the glyphs in `glyphs` vector may be actually in use. This structure is meant to keep
/// changing text, and for best performance it re-uses the created Glyphs (what means the specific
/// buffer space). Therefore you can set a cap for line length by using the `set_fixed_capacity`
/// method.
#[derive(Clone,CloneRef,Debug)]
pub struct Line {
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    content        : Rc<RefCell<String>>,
    glyphs         : Rc<RefCell<Vec<Glyph>>>,
    font_color     : Rc<Cell<color::Rgba>>,
    font_size      : Rc<Cell<f32>>,
    fixed_capacity : Rc<Cell<Option<usize>>>,
}

impl Line {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, glyph_system:&glyph::System) -> Self {
        let logger         = Logger::sub(logger,"line");
        let display_object = display::object::Instance::new(logger);
        let glyph_system   = glyph_system.clone_ref();
        let font_size      = Rc::new(Cell::new(11.0));
        let font_color     = Rc::new(Cell::new(color::Rgba::new(0.0,0.0,0.0,1.0)));
        let content        = default();
        let glyphs         = default();
        let fixed_capacity = default();
        Line {display_object,glyph_system,glyphs,font_size,font_color,content,fixed_capacity}
    }

    /// Replace currently visible text.
    pub fn set_text<S:Into<String>>(&self, content:S) {
        *self.content.borrow_mut() = content.into();
        self.redraw();
    }
}


// === Setters ===

#[allow(missing_docs)]
impl Line {
    pub fn set_font_color<C:Into<color::Rgba>>(&self, color:C) {
        let color = color.into();
        self.font_color.set(color);
        for glyph in &*self.glyphs.borrow() {
            glyph.set_color(color);
        }
    }

    pub fn set_font_size(&self, size:f32) {
        self.font_size.set(size);
        self.redraw();
    }

    pub fn change_fixed_capacity(&self, count:Option<usize>) {
        self.fixed_capacity.set(count);
        self.resize();
    }

    pub fn set_fixed_capacity(&self, count:usize) {
        self.change_fixed_capacity(Some(count));
    }

    pub fn unset_fixed_capacity(&self) {
        self.change_fixed_capacity(None);
    }
}


// === Getters ===

#[allow(missing_docs)]
impl Line {
    pub fn font_size(&self) -> f32 {
        self.font_size.get()
    }

    pub fn length(&self) -> usize {
        self.content.borrow().len()
    }

//    pub fn font(&self) -> font::Handle {
//        self.glyph_system.font.clone_ref()
//    }
}


// === Internal API ===

impl Line {
    /// Resizes the line to contain enough glyphs to display the full `content`. In case the
    /// `fixed_capacity` was set, it will add or remove the glyphs to match it.
    fn resize(&self) {
        let content_len        = self.content.borrow().len();
        let target_glyph_count = self.fixed_capacity.get().unwrap_or(content_len);
        let glyph_count        = self.glyphs.borrow().len();
        if target_glyph_count > glyph_count {
            let new_count  = target_glyph_count - glyph_count;
            let new_glyphs = (0..new_count).map(|_| {
                let glyph = self.glyph_system.new_glyph();
                self.add_child(&glyph);
                glyph
            });
            self.glyphs.borrow_mut().extend(new_glyphs)
        }
        if glyph_count > target_glyph_count {
            self.glyphs.borrow_mut().truncate(target_glyph_count)
        }
    }

    /// Updates properties of all glyphs, including characters they display, size, and colors.
    fn redraw(&self) {
        self.resize();

        let content     = self.content.borrow();
        let font        = self.glyph_system.font.clone_ref();
        let font_size   = self.font_size.get();
        let chars       = content.chars();
        let pen         = PenIterator::new(font_size,chars,font);
        let content_len = content.len();
        let color       = self.font_color.get();

        for (glyph,(chr,x_offset)) in self.glyphs.borrow().iter().zip(pen) {
            let glyph_info   = self.glyph_system.font.get_glyph_info(chr);
            let size         = glyph_info.scale.scale(font_size);
            let glyph_offset = glyph_info.offset.scale(font_size);
            let glyph_x      = x_offset + glyph_offset.x;
            let glyph_y      = glyph_offset.y;
            glyph.set_position(Vector3::new(glyph_x,glyph_y,0.0));
            glyph.set_glyph(chr);
            glyph.set_color(color);
            glyph.size.set(size);
        }

        for glyph in self.glyphs.borrow().iter().skip(content_len) {
            glyph.size.set(Vector2::new(0.0,0.0));
        }
    }
}


// === Display Object ===

impl display::Object for Line {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}




///// Test.
//pub fn main() {
////    let mut text = Rope::from("hello\nworld\n!!!\nyo");
////    let mut cursor = Cursor::new(&text, 0);
////
////    while cursor.pos() < text.len() - 2 {
////        cursor.next::<BaseMetric>();
////
////        println!("{:?}",cursor.pos());
////    }
////    a.edit(5..6, "!");
////    for i in 0..1000000 {
////        let l = a.len();
////        a.edit(l..l, &(i.to_string() + "\n"));
////    }
////    let l = a.len();
////    for s in a.clone().iter_chunks(1000..3000) {
////        println!("chunk {:?}", s);
////    }
////    a.edit(1000..l, "");
////    //a = a.subrange(0, 1000);
////    println!("{:?}", String::from(a));
//}