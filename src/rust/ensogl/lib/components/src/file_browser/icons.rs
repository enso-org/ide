
use ensogl_core::prelude::*;

use ensogl_core::display::object::{ObjectOps, Instance};
use ensogl_core::display::shape::*;
use ensogl_core::data::color;
use ensogl_core::display;


const SHRINK_FACTOR : f32 = 0.0;
// const SHRINK_FACTOR : f32 = 0.4;


// ===================
// === DynamicIcon ===
// ===================

pub trait DynamicIcon: display::Object+Debug {
    fn set_stroke_width(&self,width:f32);
    fn set_color(&self,color:color::Rgba);
}



// ==============
// === Folder ===
// ==============

pub mod folder {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let stroke_width : Var<Pixels> = stroke_width.into();

            let base = Rect((15.0.px(),11.0.px()))
                .corners_radius(1.5.px())
                .translate((0.0.px(),-0.5.px()));
            let tab = Rect((5.5.px(),4.0.px()))
                .corners_radius(1.5.px())
                .translate((-4.75.px(),4.0.px()));

            let outline = base + tab;
            let cut_out = outline.shrink(&stroke_width);

            let middle_line = Rect((15.0.px(),&stroke_width)).translate((0.0.px(),2.5.px() - &stroke_width / 2.0));

            let shape      = outline - cut_out + middle_line;
            let shape      = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct Folder(folder::View);

impl Folder {
    pub fn new() -> Self {
        let shape_view = folder::View::new(Logger::new("file_browser::icon::Folder"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = Folder(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for Folder {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for Folder {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}



// ============
// === Home ===
// ============

pub mod home {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let base = Rect((12.0.px(),8.5.px()))
                .corners_radiuses(0.0.px(), 0.0.px(), 2.0.px(), 2.0.px())
                .translate((0.0.px(),-2.75.px()));
            let cut_out = Rect((10.px(),8.5.px()))
                .corners_radiuses(0.0.px(), 0.0.px(), 1.0.px(), 1.0.px())
                .translate((0.0.px(),-1.75.px()));

            let door_inner = Rect((1.0.px(), 3.5.px()))
                .translate((0.0.px(),-4.25.px()));
            let door_outer = door_inner.grow(1.0.px());
            let door = door_outer - door_inner;

            let roof_left = Rect((9.975.px(),1.0.px()))
                .rotate(-40.0f32.to_radians().radians())
                .translate((-3.5.px(),3.0.px()));
            let roof_right = Rect((9.975.px(),1.0.px()))
                .rotate(40.0f32.to_radians().radians())
                .translate((3.5.px(),3.0.px()));
            let roof = roof_left + roof_right;

            let chimney = Rect((1.0.px(),3.5.px()))
                .translate((5.0.px(), 3.25.px()));

            let shape = base - cut_out + door + roof + chimney;
            let shape = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct Home(home::View);

impl Home {
    pub fn new() -> Self {
        let shape_view = home::View::new(Logger::new("file_browser::icon::Home"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = Home(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for Home {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for Home {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}



// ============
// === Root ===
// ============

pub mod root {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let outer = Circle(6.5.px());
            let cut_out = Circle(5.5.px());
            let outer = outer - cut_out;

            let inner = Circle(2.0.px());

            let shape = inner + outer;
            let shape = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct Root(root::View);

impl Root {
    pub fn new() -> Self {
        let shape_view = root::View::new(Logger::new("file_browser::icon::Root"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = Root(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for Root {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for Root {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}



// ============
// === File ===
// ============

pub mod file {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let block00 = Rect((4.0.px(),3.0.px())).translate((-5.0.px(),-4.0.px()));
            let block10 = Rect((4.0.px(),3.0.px())).translate(( 0.0.px(),-4.0.px()));
            let block20 = Rect((4.0.px(),3.0.px())).translate(( 5.0.px(),-4.0.px()));

            let block01 = Rect((4.0.px(),3.0.px())).translate((-5.0.px(),0.0.px()));
            let block11 = Rect((4.0.px(),3.0.px())).translate(( 0.0.px(),0.0.px()));
            let block21 = Rect((4.0.px(),3.0.px())).translate(( 5.0.px(),0.0.px()));

            let block02 = Rect((4.0.px(),3.0.px())).translate((-5.0.px(), 4.0.px()));
            let block12 = Rect((4.0.px(),3.0.px())).translate(( 0.0.px(), 4.0.px()));
            let block22 = Rect((4.0.px(),3.0.px())).translate(( 5.0.px(), 4.0.px()));

            let grid = block00 + block10 + block20 + block01 + block11 + block21 + block02 + block12 + block22;

            let frame = Rect((14.0.px(),11.0.px())).corners_radius(1.5.px());

            let shape      = grid * frame;
            let shape      = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct File(file::View);

impl File {
    pub fn new() -> Self {
        let shape_view = file::View::new(Logger::new("file_browser::icon::File"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = File(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for File {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for File {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}



// =============
// === Arrow ===
// =============

pub mod arrow {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let stroke_width : Var<Pixels> = stroke_width.into();
            let delta_x: f32 = 2.75;
            let delta_y: f32 = 3.0;
            let angle   = delta_y.atan2(delta_x);
            let stroke_length: Var<Pixels> = &stroke_width + (delta_x.pow(2.0) + delta_y.pow(2.0)).sqrt().px();
            let upper = Rect((&stroke_length,&stroke_width))
                .corners_radius(&stroke_width/2.0)
                .rotate(angle.radians())
                .translate_y((delta_y/2.0).px());
            let lower = Rect((&stroke_length,&stroke_width))
                .corners_radius(&stroke_width/2.0)
                .rotate(-angle.radians())
                .translate_y(-(delta_y/2.0).px());

            let shape      = upper + lower;
            let shape      = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct Arrow(arrow::View);

impl Arrow {
    pub fn new() -> Self {
        let shape_view = arrow::View::new(Logger::new("file_browser::icon::Arrow"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = Arrow(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for Arrow {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for Arrow {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}



// ===============
// === Project ===
// ===============

pub mod project {
    use super::*;

    ensogl_core::define_shape_system! {
        (style:Style,color_rgba:Vector4,stroke_width:f32) {
            let left = Rect((1.0.px(),10.0.px())).translate_x(-5.0.px());
            let right = Rect((1.0.px(),10.0.px())).translate_x(5.0.px());

            let top_ellipse = Ellipse(5.5.px(),1.5.px());
            let top_upper = &top_ellipse - top_ellipse.translate_y(-1.0.px());
            let top_lower = &top_ellipse - top_ellipse.translate_y(1.0.px());
            let top = top_upper + top_lower;
            let top = top.translate_y(5.0.px());

            let bottom_outer_ellipse = Ellipse(5.5.px(),2.0.px());
            let bottom_inner_ellipse = Ellipse(4.5.px(),1.5.px());
            let bottom = &bottom_outer_ellipse.translate_y(-0.5.px()) - bottom_inner_ellipse;
            let bottom = bottom * HalfPlane();
            let bottom = bottom.translate_y(-4.5.px());

            let upper_middle_ellipse = Ellipse(5.0.px(),1.6666.px());
            let upper_middle = &upper_middle_ellipse - upper_middle_ellipse.translate_y(0.5.px());
            let upper_middle = upper_middle.translate_y(1.9166.px());

            let lower_middle_ellipse = Ellipse(5.0.px(),1.83333.px());
            let lower_middle = &lower_middle_ellipse - lower_middle_ellipse.translate_y(0.5.px());
            let lower_middle = lower_middle.translate_y(-1.4166.px());

            let shape = left + right + top + bottom + upper_middle + lower_middle;
            let shape = shape.fill(color_rgba);
            let shape = shape.shrink(SHRINK_FACTOR.px());
            shape.into()
        }
    }
}

#[derive(Debug)]
pub struct Project(project::View);

impl Project {
    pub fn new() -> Self {
        let shape_view = project::View::new(Logger::new("file_browser::icon::Project"));
        shape_view.size.set(Vector2(16.0,16.0));
        let icon = Project(shape_view);
        icon.set_stroke_width(1.0);
        icon.set_color(color::Rgba::red());
        icon
    }
}

impl display::Object for Project {
    fn display_object(&self) -> &Instance {
        self.0.display_object()
    }
}

impl DynamicIcon for Project {
    fn set_stroke_width(&self, width: f32) {
        self.0.stroke_width.set(width);
    }

    fn set_color(&self, color: color::Rgba) {
        self.0.color_rgba.set(color.into());
    }
}
