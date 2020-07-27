//! This module provides a clickable view for a single breadcrumb.

use crate::prelude::*;

use super::GLYPH_WIDTH;
use super::HORIZONTAL_MARGIN;
use super::VERTICAL_MARGIN;
use super::TEXT_SIZE;

use enso_frp as frp;
use ensogl::animation::linear_interpolation;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::shape::text::glyph::system::Line;
use ensogl::display::shape::text::glyph::system::GlyphSystem;
use ensogl::display::Sprite;
use ensogl::gui::component;
use ensogl::gui::component::Animation;
use enso_protocol::language_server::MethodPointer;
use logger::enabled::Logger;
use logger::AnyLogger;
use nalgebra::Vector2;
use std::f32::consts::PI;



// =================
// === Constants ===
// =================

const ICON_MARGIN          : f32 = HORIZONTAL_MARGIN;
const ICON_RADIUS          : f32 = 8.0;
const ICON_SIZE            : f32 = ICON_RADIUS * 2.0;
const ICON_RING_WIDTH      : f32 = 2.0;
const ICON_ARROW_SIZE      : f32 = 5.0;
const SEPARATOR_SIZE       : f32 = 8.0;
const SEPARATOR_LINE_WIDTH : f32 = 3.0;
const SEPARATOR_MARGIN     : f32 = HORIZONTAL_MARGIN;
const FULL_COLOR           : color::Rgba = color::Rgba::new(1.0, 1.0, 1.0, 0.7);
const TRANSPARENT_COLOR    : color::Rgba = color::Rgba::new(1.0, 1.0, 1.0, 0.4);



// ==================
// === Background ===
// ==================

mod background {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let bg_color = color::Rgba::new(0.0,0.0,0.0,0.000_001);
            Plane().fill(bg_color).into()
        }
    }
}



// ============
// === Icon ===
// ============

mod icon {
    use super::*;

    ensogl::define_shape_system! {
        (opacity:f32) {
                let outer_circle  = Circle((ICON_RADIUS).px());
                let inner_circle  = Circle((ICON_RADIUS - ICON_RING_WIDTH).px());
                let ring          = outer_circle - inner_circle;
                let size          = ICON_ARROW_SIZE;
                let arrow         = Triangle(size.px(),size.px()).rotate((PI/2.0).radians());
                let arrow         = arrow.translate_x(0.5.px());
                let shape         = ring + arrow;
                let full_color    = format!("vec4({},{},{},{})"
                    ,FULL_COLOR.red,FULL_COLOR.green,FULL_COLOR.blue,FULL_COLOR.alpha);
                let transparent_color = format!("vec4({},{},{},{})"
                    ,TRANSPARENT_COLOR.red,TRANSPARENT_COLOR.green,TRANSPARENT_COLOR.blue
                    ,TRANSPARENT_COLOR.alpha);
                let color = format!("mix({},{},{})",transparent_color,full_color,opacity);
                let color : Var<color::Rgba> = color.into();
                shape.fill(color).into()
        }
    }
}



// =================
// === Separator ===
// =================

mod separator {
    use super::*;

    ensogl::define_shape_system! {
        () {
            let size           = SEPARATOR_SIZE;
            let angle          = PI/2.0;
            let front_triangle = Triangle(size.px(),size.px()).rotate(angle.radians());
            let back_triangle  = Triangle(size.px(),size.px()).rotate(angle.radians());
            let back_triangle  = back_triangle.translate_x(-SEPARATOR_LINE_WIDTH.px());
            let shape          = front_triangle - back_triangle;
            let color          = TRANSPARENT_COLOR;
            shape.fill(color).into()
        }
    }
}



// ==================
// === Animations ===
// ==================

/// ProjectName's animations handlers.
#[derive(Debug,Clone,CloneRef)]
pub struct Animations {
    opacity  : Animation<f32>,
    fade_in  : Animation<f32>
}

impl Animations {
    /// Create new animations handlers.
    pub fn new(network:&frp::Network) -> Self {
        let opacity  = Animation::new(&network);
        let fade_in  = Animation::new(&network);
        Self{opacity,fade_in}
    }
}



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpInputs {
    pub select   : frp::Source,
    pub deselect : frp::Source,
    pub fade_in  : frp::Source
}

impl FrpInputs {
    /// Create new FrpInputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            select   <- source();
            deselect <- source();
            fade_in  <- source();
        }
        Self{select,deselect,fade_in}
    }
}



// ==================
// === FrpOutputs ===
// ==================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpOutputs {
    pub selected : frp::Source,
    pub size     : frp::Source<Vector2<f32>>
}

impl FrpOutputs {
    /// Create new FrpOutputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend!{ network
            selected <- source();
            size     <- source();
        }
        Self{selected,size}
    }
}



// ===========
// === Frp ===
// ===========

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Frp {
    pub inputs  : FrpInputs,
    pub outputs : FrpOutputs,
    pub network : frp::Network,
}

impl Deref for Frp {
    type Target = FrpInputs;
    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl Default for Frp {
    fn default() -> Self {
        Self::new()
    }
}

impl Frp {
    /// Create new Frp.
    pub fn new() -> Self {
        let network = frp::Network::new();
        let inputs  = FrpInputs::new(&network);
        let outputs = FrpOutputs::new(&network);
        Self{network,inputs,outputs}
    }
}



// ======================
// === BreadcrumbInfo ===
// ======================

/// Breadcrumb information such as name and expression id.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct BreadcrumbInfo {
    pub method_pointer : Rc<MethodPointer>,
    pub expression_id  : uuid::Uuid
}



// ========================
// === BreadcrumbModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct BreadcrumbModel {
    logger         : Logger,
    display_object : display::object::Instance,
    view           : component::ShapeView<background::Shape>,
    separator      : component::ShapeView<separator::Shape>,
    icon           : component::ShapeView<icon::Shape>,
    glyph_system   : GlyphSystem,
    label          : Line,
    animations     : Animations,
    is_selected    : Rc<Cell<bool>>,
    pub info       : Rc<BreadcrumbInfo>
}

impl BreadcrumbModel {
    /// Constructor.
    pub fn new<'t,S:Into<&'t Scene>>
    (scene:S, frp:&Frp,method_pointer:&Rc<MethodPointer>, expression_id:&uuid::Uuid) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("Breadcrumbs");
        let display_object = display::object::Instance::new(&logger);
        let view_logger    = Logger::sub(&logger,"view_logger");
        let view           = component::ShapeView::<background::Shape>::new(&view_logger, scene);
        let icon           = component::ShapeView::<icon::Shape>::new(&view_logger, scene);
        let separator      = component::ShapeView::<separator::Shape>::new(&view_logger, scene);
        let font           = scene.fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let glyph_system   = GlyphSystem::new(scene,font);
        let label          = glyph_system.new_line();
        let expression_id  = *expression_id;
        let method_pointer = method_pointer.clone();
        let info           = Rc::new(BreadcrumbInfo{method_pointer,expression_id});
        let animations     = Animations::new(&frp.network);
        let is_selected    = default();
        Self{logger,view,icon,separator,display_object,glyph_system,label,info,animations
            ,is_selected}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.view);
        self.view.add_child(&self.separator);
        self.separator.add_child(&self.icon);
        self.icon.add_child(&self.label);

        let color  = if self.is_selected() { FULL_COLOR } else { TRANSPARENT_COLOR };

        self.label.set_font_size(TEXT_SIZE);
        self.label.set_font_color(color);
        self.label.set_text(&self.info.method_pointer.name);
        self.label.set_position(Vector3(ICON_SIZE/2.0 + ICON_MARGIN,-ICON_SIZE/4.0,0.0));

        let width  = self.width();
        let height = self.height();
        let offset = self.compute_separator_margin() + SEPARATOR_SIZE/2.0;

        self.view.shape.sprite.size.set(Vector2::new(width,height));
        self.fade_in(0.0);
        self.separator.shape.sprite.size.set(Vector2::new(SEPARATOR_SIZE+1.0,SEPARATOR_SIZE+1.0));
        self.separator.set_position(Vector3(offset-width/2.0,0.0,0.0));
        self.icon.shape.sprite.size.set(Vector2::new(ICON_SIZE+1.0,ICON_SIZE+1.0));
        self.icon.shape.opacity.set(self.is_selected() as i32 as f32);
        self.icon.set_position(Vector3(offset+ICON_SIZE/2.0,0.0,0.0));

        self
    }

    fn compute_separator_margin(&self) -> f32 {
        self.label.font_size() * SEPARATOR_MARGIN / 6.0
    }

    fn label_width(&self) -> f32 {
        self.info.method_pointer.name.len() as f32 * GLYPH_WIDTH
    }

    /// Get the width of the view.
    pub fn width(&self) -> f32 {
        self.compute_separator_margin()*2.0+SEPARATOR_SIZE+ICON_SIZE+ICON_MARGIN+self.label_width()
    }

    /// Get the height of the view.
    pub fn height(&self) -> f32 {
        self.label.font_size() + VERTICAL_MARGIN * 2.0
    }

    fn fade_in(&self, value:f32) {
        let width  = self.width();
        let height = self.height();
        self.view.set_position(Vector3(width * value,-height,0.0)/2.0);
    }

    fn set_opacity(&self, value:f32) {
        let color = linear_interpolation(TRANSPARENT_COLOR,FULL_COLOR,value);
        self.label.set_font_color(color);
        self.icon.shape.opacity.set(value);
    }

    fn select(&self) {
        self.is_selected.set(true);
        self.animations.opacity.set_target_value(1.0);
    }

    fn deselect(&self) {
        self.is_selected.set(false);
        self.animations.opacity.set_target_value(0.0);
    }

    fn is_selected(&self) -> bool {
        self.is_selected.get()
    }
}

impl display::Object for BreadcrumbModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ==================
// === Breadcrumb ===
// ==================

#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Breadcrumb {
    #[shrinkwrap(main_field)]
    model   : Rc<BreadcrumbModel>,
    pub frp : Frp
}

impl Breadcrumb {
    /// Constructor.
    pub fn new<'t,S:Into<&'t Scene>>
    (scene:S, method_pointer:&Rc<MethodPointer>, expression_id:&uuid::Uuid) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(BreadcrumbModel::new(scene,&frp,method_pointer,expression_id));
        let network = &frp.network;

        frp::extend! { network
            eval_ frp.fade_in(model.animations.fade_in.set_target_value(1.0));
            eval_ frp.select(model.select());
            eval_ frp.deselect(model.deselect());
            eval_ model.view.events.mouse_over(model.animations.opacity.set_target_value(1.0));
            eval_ model.view.events.mouse_out(
                model.animations.opacity.set_target_value(model.is_selected() as i32 as f32);
            );
            eval_ model.view.events.mouse_down(frp.outputs.selected.emit(()));
        }


        // === Animations ===

        frp::extend! {network
            eval model.animations.fade_in.value((value) model.fade_in(*value));
            eval model.animations.opacity.value((value) model.set_opacity(*value));
        }

        Self{frp,model}
    }
}

impl display::Object for Breadcrumb {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
