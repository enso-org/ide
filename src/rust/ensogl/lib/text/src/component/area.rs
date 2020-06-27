use crate::prelude::*;
use crate::traits::*;

use crate::typeface::glyph;
use crate::typeface::pen;
use crate::typeface::glyph::Glyph;
use crate::buffer;
use crate::buffer::data::unit::*;

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
use ensogl::gui::cursor as mouse_cursor;
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
            $(pub $in_field : frp::Source<$in_field_type>),*
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
            setter           : FrpOutputsSetter,
            $(pub $out_field : frp::Stream<$out_field_type>),*
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
            let out = Rect((1000.px(),1000.px())).corners_radius(8.px()).fill(color::Rgba::new(1.0,1.0,1.0,0.05));
            out.into()
        }
    }
}



// ==============
// === Cursor ===
// ==============

/// Canvas node shape definition.
pub mod cursor {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, selection:f32) {
            let out = Rect((2.px(),12.px())).corners_radius(1.px()).fill(color::Rgba::new(1.0,1.0,1.0,0.8));
            out.into()
        }
    }
}



//// ====================
//// === PrintedGlyph ===
//// ====================
//
//#[derive(Debug)]
//pub struct PrintedGlyph {
//    glyph  : Glyph,
//    offset : f32,
//}
//
//impl Deref for PrintedGlyph {
//    type Target = Glyph;
//    fn deref(&self) -> &Self::Target {
//        &self.glyph
//    }
//}
//
//impl PrintedGlyph {
//    pub fn new(glyph:Glyph, offset:f32) -> Self {
//        Self {glyph,offset}
//    }
//}
//
//impl From<Glyph> for PrintedGlyph {
//    fn from(glyph:Glyph) -> Self {
//        Self::new(glyph,default())
//    }
//}
//
//impl display::Object for PrintedGlyph {
//    fn display_object(&self) -> &display::object::Instance {
//        self.glyph.display_object()
//    }
//}


#[derive(Clone,Copy,Debug)]
pub struct Div {
    x_offset    : f32,
    byte_offset : Bytes,
}

impl Div {
    pub fn new(x_offset:f32, byte_offset:Bytes) -> Self {
        Self {x_offset, byte_offset}
    }
}


// ============
// === Line ===
// ============

#[derive(Debug)]
pub struct Line {
    display_object : display::object::Instance,
    glyphs         : Vec<Glyph>,
    divs           : Vec<Div>,
    centers        : Vec<f32>,
    byte_size      : Bytes,
}

impl Line {
    fn new(logger:impl AnyLogger) -> Self {
        let logger         = Logger::sub(logger,"line");
        let display_object = display::object::Instance::new(logger);
        let glyphs         = default();
        let divs           = default();
        let centers        = default();
        let byte_size      = default();
        Self {display_object,glyphs,divs,centers,byte_size}
    }

    /// Set the division points (offsets between letters). Also updates center points.
    fn set_divs(&mut self, divs:Vec<Div>) {
        let div_iter         = divs.iter();
        let div_iter_skipped = divs.iter().skip(1);
        self.centers         = div_iter.zip(div_iter_skipped).map(|(t,s)|(t.x_offset+s.x_offset)/2.0).collect();
        self.divs = divs;
    }

    fn div_index_close_to(&self, offset:f32) -> usize {
        self.centers.binary_search_by(|t|t.partial_cmp(&offset).unwrap()).unwrap_both()
    }

    fn div_index_by_byte_offset(&self, offset:Bytes) -> usize {
        self.divs.binary_search_by(|t|t.byte_offset.partial_cmp(&offset).unwrap()).unwrap_both()
    }

    fn div_by_byte_offset(&self, offset:Bytes) -> Div {
        self.divs[self.div_index_by_byte_offset(offset)]
    }

    fn resize_with(&mut self, size:usize, cons:impl Fn()->Glyph) {
        let display_object = self.display_object().clone_ref();
        self.glyphs.resize_with(size,move || {
            let glyph = cons();
            display_object.add_child(&glyph);
            glyph
        });
    }

    fn byte_size(&self) -> Bytes {
        self.byte_size
//        self.divs.last().map(|t|t.byte_offset).unwrap_or_default()
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

    pub fn line_index_of_byte_offset(&self, tgt_offset:Bytes) -> usize {
        println!("---------");
        println!("tgt_offset: {:?}",tgt_offset);
        let lines = self.rc.borrow();
        let max_index  = lines.len() - 1;
        let mut index  = 0;
        let mut offset = 0.bytes();
        loop {
            println!("[{:?}] {:?}",index,lines[index].byte_size());
            offset += lines[index].byte_size();
            if offset > tgt_offset || index == max_index { break }
            index += 1;
        }
        index
    }
}


// ===========
// === FRP ===
// ===========

define_frp! {
    Inputs {
    }

    Outputs {
        mouse_cursor_style : mouse_cursor::Style,
    }
}


// ============
// === Area ===
// ============

pub const LINE_HEIGHT : f32 = 14.0; // FIXME

#[derive(Debug)]
pub struct Area {
    data    : AreaData,
    pub frp : Frp,
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
        let output  = FrpOutputs::new(&network);
        let frp     = Frp::new(network,data.frp.clone_ref(),output);
        Self {data,frp} . init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let mouse   = &self.scene.mouse.frp;
        let model   = &self.data;

        frp::extend! { network
            cursor_over  <- self.background.events.mouse_over.constant(mouse_cursor::Style::new_text_cursor());
            cursor_out   <- self.background.events.mouse_out.constant(mouse_cursor::Style::default());
            mouse_cursor <- any(cursor_over,cursor_out);
            self.frp.output.setter.mouse_cursor_style <+ mouse_cursor;

            mouse_down_pos <- mouse.position.sample(&self.background.events.mouse_down);
            _eval <- mouse_down_pos.map2(&model.scene.frp.shape, f!([model](screen_pos,shape) {

                let origin_world_space = Vector4(0.0,0.0,0.0,1.0);
                let origin_clip_space  = model.scene.camera().view_projection_matrix() * origin_world_space;
                let inv_object_matrix  = model.transform_matrix().try_inverse().unwrap();

                let clip_space_z = origin_clip_space.z;
                let clip_space_x = origin_clip_space.w * 2.0 * screen_pos.x / shape.width;
                let clip_space_y = origin_clip_space.w * 2.0 * screen_pos.y / shape.height;
                let clip_space   = Vector4(clip_space_x,clip_space_y,clip_space_z,origin_clip_space.w);
                let world_space  = model.scene.camera().inversed_view_projection_matrix() * clip_space;
                let object_space = inv_object_matrix * world_space;

                let line_index = (-object_space.y / LINE_HEIGHT) as usize;
                let line_index = std::cmp::min(line_index,model.lines.len() - 1);
                let div_index  = model.lines.rc.borrow()[line_index].div_index_close_to(object_space.x);
                let div        = model.lines.rc.borrow()[line_index].divs[div_index];

                model.buffer.frp.input.set_cursor.emit(Location(buffer::Line(line_index),Column(div.byte_offset.raw)));


//                println!("------");
//                println!("{:?}",line_index as usize);
//                let m1       = model.scene.views.cursor.camera.inversed_view_matrix();
//                let m1       = model.transform_matrix().try_inverse().unwrap();
//                let m2       = model.scene.camera().inversed_view_projection_matrix();
//                let
//                let position = Vector4::new(p.x,p.y,-model.scene.camera().position().z,1.0);
//                let position = m1 * (m2 * position);
//
//                println!(">! ({},{}) ({},{})",p.x,p.y,position.x,position.y); // fixme w sliderach po kliknieciu tez trzbea znac pozycje lokal

            }));

            eval model.buffer.frp.output.selection ([model](selections) {
                let mut cursors = vec![];
                for selection in selections {
                    let line_index = model.lines.line_index_of_byte_offset(selection.start);
                    let line_offset = model.buffer.offset_of_view_line(buffer::Line(line_index));
                    let offset_in_line = selection.start - line_offset;
                    let div = model.lines.rc.borrow()[line_index].div_by_byte_offset(offset_in_line);
                    let logger = Logger::sub(&model.logger,"cursor");
                    let cursor = component::ShapeView::<cursor::Shape>::new(&logger,&model.scene);
                    cursor.shape.sprite.size.set(Vector2(4.0,20.0));
                    cursor.set_position_y(-6.0 - LINE_HEIGHT * line_index as f32);
                    cursor.set_position_x(div.x_offset);
                    model.add_child(&cursor);
                    cursors.push(cursor);
                }
                *model.cursors.borrow_mut() = cursors;
            });

        }

        self
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct AreaData {
    scene          : Scene,
    logger         : Logger,
    frp            : FrpInputs,
    buffer         : buffer::View,
    display_object : display::object::Instance,
    glyph_system   : glyph::System,
    lines          : Lines,
    cursors        : Rc<RefCell<Vec<component::ShapeView<cursor::Shape>>>>,
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
        let scene          = scene.clone_ref();
        let logger         = Logger::sub(logger,"text_area");
        let bg_logger      = Logger::sub(&logger,"background");
        let cursors        = default();
        let background     = component::ShapeView::<background::Shape>::new(&bg_logger,&scene);
        let fonts          = scene.extension::<typeface::font::Registry>();
        let font           = fonts.load("DejaVuSansMono");
        let glyph_system   = typeface::glyph::System::new(&scene,font);
        let display_object = display::object::Instance::new(&logger);
        let glyph_system   = glyph_system.clone_ref();
        let buffer         = default();
        let lines          = default();
        let frp            = FrpInputs::new(network);
        display_object.add_child(&background);
        background.shape.sprite.size.set(Vector2(150.0,100.0));
        background.mod_position(|p| p.x += 50.0);
        Self {scene,logger,frp,display_object,glyph_system,buffer,lines,cursors,background} . init()
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
        let lines      = self.buffer.lines();
        let line_count = lines.len();
        self.lines.resize_with(line_count,|ix| self.new_line(ix));
        for (view_line_number,content) in lines.into_iter().enumerate() {
            self.redraw_line(view_line_number,content)
        }
    }

    fn redraw_line(&self, view_line_number:usize, content:String) { // fixme content:Cow<str>
        let line           = &mut self.lines.rc.borrow_mut()[view_line_number];
        let line_range     = self.buffer.range_of_view_line_raw(buffer::Line(view_line_number));
        let mut line_style = self.buffer.focus_style(line_range.start .. line_range.end).iter();
        line.byte_size     = self.buffer.line_byte_size(buffer::Line(view_line_number));

        let mut pen         = pen::Pen::new(&self.glyph_system.font);
        let mut divs        = vec![];
        let mut byte_offset = 0.bytes();
        line.resize_with(content.len(),||self.glyph_system.new_glyph());
        for (glyph,chr) in line.glyphs.iter_mut().zip(content.chars()) {
            let style     = line_style.next().unwrap_or_default();
            let chr_size  = style.size.raw;
            let info      = pen.advance(chr,chr_size);
            let chr_bytes = info.char.len_utf8().bytes();
            line_style.drop(chr_bytes - 1.bytes());
            let glyph_info   = self.glyph_system.font.get_glyph_info(info.char);
            let size         = glyph_info.scale.scale(chr_size);
            let glyph_offset = glyph_info.offset.scale(chr_size);
            let glyph_x      = info.offset + glyph_offset.x;
            let glyph_y      = glyph_offset.y;
            glyph.set_position_xy(Vector2(glyph_x,glyph_y));
            glyph.set_char(info.char);
            glyph.set_color(style.color);
            glyph.size.set(size);


            divs.push(Div::new(info.offset,byte_offset));
            byte_offset += chr_bytes;

        }

        divs.push(Div::new(pen.advance_final(),byte_offset));

//        for div in divs.iter() {
//            let logger = Logger::sub(&self.logger,"cursor");
//            let cursor = component::ShapeView::<cursor::Shape>::new(&logger,&self.scene);
//            cursor.shape.sprite.size.set(Vector2(4.0,20.0));
//            cursor.set_position_y(-6.0);
//            cursor.set_position_x(*div);
//            self.add_child(&cursor);
//            self.cursors.borrow_mut().push(cursor);
//        }

        line.set_divs(divs);

    }

    fn new_line(&self, index:usize) -> Line {
        let line     = Line::new(&self.logger);
        let y_offset = - ((index + 1) as f32) * LINE_HEIGHT + 4.0; // FIXME line height?
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

