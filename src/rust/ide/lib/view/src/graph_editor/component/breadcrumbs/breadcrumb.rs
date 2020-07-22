//! This module provides a clickable view for a single breadcrumb.

use crate::prelude::*;

use super::GLYPH_WIDTH;
use super::HORIZONTAL_MARGIN;
use super::VERTICAL_MARGIN;
use super::TEXT_SIZE;

use enso_frp as frp;
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
use logger::enabled::Logger;
use logger::AnyLogger;
use nalgebra::Vector2;



// ==================
// === Background ===
// ==================

mod background {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, selection:f32) {
            let bg_color = color::Rgba::new(0.0,0.0,0.0,0.000_001);
            Plane().fill(bg_color).into()
        }
    }
}



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef,Copy)]
#[allow(missing_docs)]
pub struct FrpInputs {}

impl FrpInputs {
    /// Create new FrpInputs.
    pub fn new(_network:&frp::Network) -> Self {
        Self{}
    }
}



// ==================
// === FrpOutputs ===
// ==================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpOutputs {
    pub selected : frp::Source
}

impl FrpOutputs {
    /// Create new FrpOutputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend!{ network
            def selected = source();
        }
        Self{selected}
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
    pub name          : String,
    pub expression_id : uuid::Uuid
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
    glyph_system   : GlyphSystem,
    label          : Line,
    pub info       : Rc<BreadcrumbInfo>
}

impl BreadcrumbModel {
    /// Create a new BreadcrumbModel.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S,name:impl Str, expression_id:uuid::Uuid) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("Breadcrumbs");
        let display_object = display::object::Instance::new(&logger);
        let view_logger    = Logger::sub(&logger,"view_logger");
        let view           = component::ShapeView::<background::Shape>::new(&view_logger, scene);
        let name           = name.into();
        let font           = scene.fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let glyph_system   = GlyphSystem::new(scene,font);
        let label          = glyph_system.new_line();
        let info           = Rc::new(BreadcrumbInfo{name,expression_id});
        Self{logger,view,display_object,glyph_system,label,info}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.view);

        let width       = self.width();
        let height      = TEXT_SIZE + VERTICAL_MARGIN * 2.0;
        let color       = color::Rgba::new(1.0, 1.0, 1.0, 0.7);
        self.label.set_font_size(TEXT_SIZE);
        self.label.set_font_color(color);
        //FIXME[dg]: Remove text separators.
        self.label.set_text(format!("> {}", self.info.name));
        self.label.set_position(Vector3::new(HORIZONTAL_MARGIN,-TEXT_SIZE-VERTICAL_MARGIN,0.0));
        self.view.shape.sprite.size.set(Vector2::new(width,height));
        self.view.set_position(Vector3::new(width,-height,0.0)/2.0);
        self.add_child(&self.label);

        self
    }

    /// Get the width of the breadcrumb view.
    pub fn width(&self) -> f32 {
        //FIXME[dg]: Remove text separators.
        let number_of_separator_glyphs = 2;
        let glyphs = (self.info.name.len() + number_of_separator_glyphs) as f32;
        glyphs * GLYPH_WIDTH + HORIZONTAL_MARGIN
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

/// The project name's view used for visualizing the project name and renaming it.
#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Breadcrumb {
    #[shrinkwrap(main_field)]
    model   : Rc<BreadcrumbModel>,
    pub frp : Frp
}

impl Breadcrumb {
    /// Create a new ProjectName view.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S, name:impl Str, expression_id:uuid::Uuid) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(BreadcrumbModel::new(scene,name,expression_id));
        let network = &frp.network;

        frp::extend! {network
            eval_ model.view.events.mouse_down(frp.outputs.selected.emit(()));
        }

        Self{frp,model}
    }
}

impl display::Object for Breadcrumb {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
