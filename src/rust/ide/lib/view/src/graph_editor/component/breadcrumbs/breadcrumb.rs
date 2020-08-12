//! This module provides a clickable view for a single breadcrumb.

use crate::prelude::*;

use super::GLYPH_WIDTH;
use super::HORIZONTAL_MARGIN;
use super::VERTICAL_MARGIN;
use super::TEXT_SIZE;
use super::RelativePosition;

use crate::graph_editor::MethodPointer;

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
use ensogl::gui::component::Animation;
use logger::enabled::Logger;
use logger::AnyLogger;
use nalgebra::Vector2;
use std::f32::consts::PI;



// =================
// === Constants ===
// =================

/// Breadcrumb top margin.
pub const TOP_MARGIN:f32 = 0.0;
/// Breadcrumb left margin.
pub const LEFT_MARGIN:f32 = 0.0;
/// Breadcrumb right margin.
pub const RIGHT_MARGIN     : f32 = 0.0;
const ICON_LEFT_MARGIN     : f32 = 0.0;
const ICON_RIGHT_MARGIN    : f32 = HORIZONTAL_MARGIN;
const ICON_RADIUS          : f32 = 8.0;
const ICON_SIZE            : f32 = ICON_RADIUS * 2.0;
const ICON_RING_WIDTH      : f32 = (ICON_RADIUS/4.0) as i32 as f32;
const ICON_ARROW_SIZE      : f32 = (ICON_RADIUS/1.5) as i32 as f32;
const SEPARATOR_SIZE       : f32 = 8.0;
const SEPARATOR_LINE_WIDTH : f32 = 3.0;
const SHAPE_PADDING        : f32 = 1.0;
const SEPARATOR_MARGIN     : f32 = HORIZONTAL_MARGIN;


// === Colors ===

const FULL_COLOR        : color::Rgba = color::Rgba::new(1.0,1.0,1.0,0.7);
const TRANSPARENT_COLOR : color::Rgba = color::Rgba::new(1.0, 1.0, 1.0, 0.4);
/// Breadcrumb color when selected.
pub const SELECTED_COLOR : color::Rgba = color::Rgba::new(0.0,1.0,0.0,1.0);
/// Breadcrumb color when it's deselected on the left of the selected breadcrumb.
pub const LEFT_DESELECTED_COLOR : color::Rgba = color::Rgba::new(1.0,0.0,0.0,1.0);
/// Breadcrumb color when it's deselected on the right of the selected breadcrumb.
pub const RIGHT_DESELECTED_COLOR : color::Rgba = color::Rgba::new(0.0,0.0,1.0,1.0);
/// Breadcrumb color when hovered.
pub const HOVER_COLOR : color::Rgba = FULL_COLOR;



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
        (red:f32,green:f32,blue:f32,alpha:f32) {
                let outer_circle  = Circle((ICON_RADIUS).px());
                let inner_circle  = Circle((ICON_RADIUS - ICON_RING_WIDTH).px());
                let ring          = outer_circle - inner_circle;
                let size          = ICON_ARROW_SIZE;
                let arrow         = Triangle(size.px(),size.px()).rotate((PI/2.0).radians());
                let arrow         = arrow.translate_x(1.0.px());
                let shape         = ring + arrow;
                let color         = format!("vec4({},{},{},{})",red,green,blue,alpha);
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
    color   : Animation<Vector4<f32>>,
    fade_in : Animation<f32>
}

impl Animations {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        let color    = Animation::new(&network);
        let fade_in  = Animation::new(&network);
        Self{color,fade_in}
    }
}



// =================
// === FrpInputs ===
// =================

/// Breadcrumb frp network inputs.
#[derive(Debug,Clone,CloneRef)]
pub struct FrpInputs {
    /// Select the breadcrumb, triggering the selection animation.
    pub select   : frp::Source,
    /// Select the breadcrumb, triggering the deselection animation, using the (self,new) breadcrumb
    /// indices to determine if the breadcrumb is on the left or on the right of the newly selected
    /// breadcrumb.
    pub deselect : frp::Source<(usize,usize)>,
    /// Triggers the fade in animation, which only makes sense during the breadcrumb creation.
    pub fade_in  : frp::Source
}

impl FrpInputs {
    /// Constructor.
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

/// Breadcrumb frp network outputs.
#[derive(Debug,Clone,CloneRef)]
pub struct FrpOutputs {
    /// Signalizes that the breadcrumb was selected.
    pub selected : frp::Source,
    /// Signalizes that the breadcrumb's size changed.
    pub size     : frp::Source<Vector2<f32>>
}

impl FrpOutputs {
    /// Constructor.
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

/// A breadcrumb frp structure with its endpoints and network representation.
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
    /// Constructor.
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
    pub method_pointer : MethodPointer,
    pub expression_id  : ast::Id
}



// ========================
// === BreadcrumbModel ===
// ========================

/// Breadcrumbs model.
#[derive(Debug,Clone,CloneRef)]
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
    /// Breadcrumb information such as name and expression id.
    pub info          : Rc<BreadcrumbInfo>,
    relative_position : Rc<Cell<RelativePosition>>
}

impl BreadcrumbModel {
    /// Constructor.
    pub fn new<'t,S:Into<&'t Scene>>
    (scene:S, frp:&Frp,method_pointer:&MethodPointer, expression_id:&ast::Id) -> Self {
        let scene             = scene.into();
        let logger            = Logger::new("Breadcrumbs");
        let display_object    = display::object::Instance::new(&logger);
        let view_logger       = Logger::sub(&logger,"view_logger");
        let view              = component::ShapeView::<background::Shape>::new(&view_logger, scene);
        let icon              = component::ShapeView::<icon::Shape>::new(&view_logger, scene);
        let separator         = component::ShapeView::<separator::Shape>::new(&view_logger, scene);
        let font              = scene.fonts.get_or_load_embedded_font("DejaVuSansMono").unwrap();
        let glyph_system      = GlyphSystem::new(scene,font);
        let label             = glyph_system.new_line();
        let expression_id     = *expression_id;
        let method_pointer    = method_pointer.clone();
        let info              = Rc::new(BreadcrumbInfo{method_pointer,expression_id});
        let animations        = Animations::new(&frp.network);
        let is_selected       = default();
        let relative_position = default();
        Self{logger,view,icon,separator,display_object,glyph_system,label,info,animations
            ,is_selected,relative_position}.init()
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
        self.label.set_position(Vector3(ICON_RADIUS + ICON_RIGHT_MARGIN, -TEXT_SIZE/3.0, 0.0));

        let width  = self.width();
        let height = self.height();
        let offset = self.compute_separator_margin() + SEPARATOR_SIZE/2.0;

        self.view.shape.sprite.size.set(Vector2::new(width,height));
        self.fade_in(0.0);
        let separator_size = SEPARATOR_SIZE+SHAPE_PADDING;
        let icon_size      = ICON_SIZE+SHAPE_PADDING;
        self.separator.shape.sprite.size.set(Vector2::new(separator_size,separator_size));
        self.separator.set_position_x((offset-width/2.0).round());
        self.icon.shape.sprite.size.set(Vector2::new(icon_size,icon_size));
        self.icon.set_position_x((offset+ICON_SIZE/2.0+LEFT_MARGIN+ICON_LEFT_MARGIN).round());

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
        let width = self.compute_separator_margin()*2.0+SEPARATOR_SIZE+ICON_SIZE+ICON_RIGHT_MARGIN;
        let width = width+self.label_width()+LEFT_MARGIN+RIGHT_MARGIN+ICON_LEFT_MARGIN;
        width.ceil()
    }

    /// Get the height of the view.
    pub fn height(&self) -> f32 {
        self.label.font_size() + VERTICAL_MARGIN * 2.0
    }

    fn fade_in(&self, value:f32) {
        let width      = self.width();
        let height     = self.height();
        let x_position = width*value/2.0;
        let y_position = -height/2.0-TOP_MARGIN;
        self.view.set_position(Vector3(x_position.round(),y_position.round(),0.0));
    }

    fn set_color(&self, value:Vector4<f32>) {
        let color = color::Rgba::from(value);
        self.label.set_font_color(color);
        self.icon.shape.red.set(color.red);
        self.icon.shape.green.set(color.green);
        self.icon.shape.blue.set(color.blue);
        self.icon.shape.alpha.set(color.alpha);
    }

    fn select(&self) {
        self.is_selected.set(true);
        self.animations.color.set_target_value(SELECTED_COLOR.into());
    }

    fn deselect(&self, old:usize, new:usize) {
        self.relative_position.set((new>old).as_option().map(|_| RelativePosition::LEFT).unwrap_or(RelativePosition::RIGHT));
        self.is_selected.set(false);
        self.animations.color.set_target_value(self.deselected_color().into());
    }

    fn deselected_color(&self) -> color::Rgba {
        match self.relative_position.get() {
            RelativePosition::RIGHT => RIGHT_DESELECTED_COLOR,
            RelativePosition::LEFT  => LEFT_DESELECTED_COLOR
        }
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

/// The breadcrumb's view which displays its name and exposes mouse press interactions.
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
    (scene:S, method_pointer:&MethodPointer, expression_id:&ast::Id) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(BreadcrumbModel::new(scene,&frp,method_pointer,expression_id));
        let network = &frp.network;

        frp::extend! { network
            eval_ frp.fade_in(model.animations.fade_in.set_target_value(1.0));
            eval_ frp.select(model.select());
            eval frp.deselect(((old,new)) model.deselect(*old,*new));
            //FIXME[dg]: selected should be a gate
            eval model.view.events.mouse_over([model] (_) {
                if !model.is_selected() {
                    model.animations.color.set_target_value(HOVER_COLOR.into())
                }
            });
            eval model.view.events.mouse_out([model] (_) {
                if !model.is_selected() {
                    model.animations.color.set_target_value(model.deselected_color().into())
                }
            });
            eval_ model.view.events.mouse_down(frp.outputs.selected.emit(()));
        }


        // === Animations ===

        frp::extend! {network
            eval model.animations.fade_in.value((value) model.fade_in(*value));
            eval model.animations.color.value((value) model.set_color(*value));
        }

        Self{frp,model}
    }
}

impl display::Object for Breadcrumb {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
