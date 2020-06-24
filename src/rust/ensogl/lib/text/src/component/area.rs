use crate::prelude::*;
use crate::traits::*;

use crate::typeface::glyph;
use crate::typeface::pen;
use crate::typeface::glyph::Glyph;
use crate::buffer;

use ensogl::display::Buffer;
use ensogl::display::Attribute;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::data::color;
use ensogl::display;
use crate::buffer::view::LineOffset;
use ensogl::gui::component;
use crate::typeface;
use ensogl::gui::cursor;
use enso_frp as frp;



// ==================
// === Frp Macros ===
// ==================

// FIXME: these are generic FRP utilities. To be refactored out after the API settles down.
// FIXME: the same are defined in text/view
macro_rules! define_frp {
    (
        Inputs  { $($in_field  : ident : $in_field_type  : ty),* $(,)? }
        Outputs { $($out_field : ident : $out_field_type : ty),* $(,)? }
    ) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct Frp {
            pub network : frp::Network,
            pub input   : FrpInputs,
            pub output  : FrpOutputs,
        }

        impl Frp {
            pub fn new(network:frp::Network, input:FrpInputs, output:FrpOutputs) -> Self {
                Self {network,input,output}
            }
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpInputs {
            $($in_field : frp::Source<$in_field_type>),*
        }

        impl FrpInputs {
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { network
                    $($in_field <- source();)*
                }
                Self { $($in_field),* }
            }
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpOutputsSetter {
            $($out_field : frp::Any<$out_field_type>),*
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpOutputs {
            setter       : FrpOutputsSetter,
            $($out_field : frp::Stream<$out_field_type>),*
        }

        impl FrpOutputsSetter {
            pub fn new(network:&frp::Network) -> Self {
                frp::extend! { network
                    $($out_field <- any(...);)*
                }
                Self {$($out_field),*}
            }
        }

        impl FrpOutputs {
            pub fn new(network:&frp::Network) -> Self {
                let setter = FrpOutputsSetter::new(network);
                $(let $out_field = setter.$out_field.clone_ref().into();)*
                Self {setter,$($out_field),*}
            }
        }
    };
}



// ==================
// === Background ===
// ==================

/// Canvas node shape definition.
pub mod background {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, selection:f32) {
            let out = Rect((1000.px(),1000.px())).corners_radius(8.px()).fill(color::Rgba::new(1.0,1.0,1.0,0.1));
            out.into()
        }
    }
}



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


// ===========
// === FRP ===
// ===========

define_frp! {
    Inputs {
    }

    Outputs {
        cursor_style : cursor::Style,
    }
}


// ============
// === Area ===
// ============


#[derive(Debug)]
pub struct Area {
    data    : AreaData,
    frp     : Frp,
}

impl Deref for Area {
    type Target = AreaData;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl Area {
    pub fn new(logger:impl AnyLogger, scene:&Scene) -> Self {
        let network = frp::Network::new();
        let data    = AreaData::new(logger,scene,&network);
        let output = FrpOutputs::new(&network);
        let frp     = Frp::new(network,data.frp.clone_ref(),output);
        Self {data,frp} . init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        frp::extend! { network

            eval_ self.background.events.mouse_down ([] println!("press"));
            eval_ self.background.events.mouse_over ([] println!("over"));

            cursor_over <- self.background.events.mouse_over.constant(cursor::Style::new_text_cursor());
            cursor_out  <- self.background.events.mouse_over.constant(cursor::Style::default());
            cursor      <- any(cursor_over,cursor_out);
            self.frp.output.setter.cursor_style <+ cursor;

        }

        self
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct AreaData {
    logger         : Logger,
    frp            : FrpInputs,
    buffer         : buffer::View,
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    lines          : Lines,
    background     : component::ShapeView<background::Shape>,
}

impl Deref for AreaData {
    type Target = buffer::View;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl AreaData {
    /// Constructor.
    pub fn new
    (logger:impl AnyLogger, scene:&Scene, network:&frp::Network) -> Self {
        let logger         = Logger::sub(logger,"text_area");
        let bg_logger      = Logger::sub(&logger,"background");
        let background     = component::ShapeView::<background::Shape>::new(&bg_logger,scene);
        let fonts          = scene.extension::<typeface::font::Registry>();
        let font           = fonts.default();
        let glyph_system   = typeface::glyph::System::new(scene,font);
        let display_object = display::object::Instance::new(&logger);
        let glyph_system   = glyph_system.clone_ref();
        let buffer         = default();
        let lines          = default();
        let frp            = FrpInputs::new(network);
        display_object.add_child(&background);
        background.shape.sprite.size.set(Vector2(150.0,100.0));
        background.mod_position(|p| p.x += 50.0);
        Self {logger,frp,display_object,glyph_system,buffer,lines,background} . init()
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    fn init(self) -> Self {
        self.redraw();
        self
    }

    // FIXME: make private
    pub fn redraw(&self) {
        println!(">>>> {:?}", self.buffer.view_buffer.selection.borrow());

        let line_count = self.buffer.line_count();
        self.lines.resize_with(line_count,|ix| self.new_line(ix));
        for (view_line_number,content) in self.buffer.lines().into_iter().enumerate() {
            self.redraw_line(view_line_number,content)
        }
    }

    fn redraw_line(&self, view_line_number:usize, content:String) { // content:Cow<str>
        let line           = &mut self.lines.rc.borrow_mut()[view_line_number];
        let line_range     = self.buffer.range_of_view_line_raw(buffer::Line(view_line_number));
        let mut line_style = self.buffer.focus_style(line_range.start .. line_range.end).iter();

        let mut pen = pen::Pen::new(&self.glyph_system.font);
        line.resize_with(content.len(),||self.glyph_system.new_glyph());
        for (glyph,chr) in line.glyphs.iter().zip(content.chars()) {
            let style    = line_style.next().unwrap_or_default();
            let chr_size = style.size.raw;
            let info     = pen.advance(chr,chr_size);
            line_style.drop((info.char.len_utf8() - 1).bytes());
            let glyph_info   = self.glyph_system.font.get_glyph_info(info.char);
            let size         = glyph_info.scale.scale(chr_size);
            let glyph_offset = glyph_info.offset.scale(chr_size);
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

impl display::Object for AreaData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

impl display::Object for Area {
    fn display_object(&self) -> &display::object::Instance {
        self.data.display_object()
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

