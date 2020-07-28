//! This module defines the `Container` struct and related functionality.

use crate::prelude::*;

// use crate::graph_editor::documentation_view;


use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::gui::component;


// =================
// === Constants ===
// =================

const DEFAULT_SIZE  : (f32,f32) = (200.0,200.0);
const CORNER_RADIUS : f32       = super::super::node::CORNER_RADIUS;



// =============
// === Shape ===
// =============

/// Container background shape definition.
///
/// Provides a backdrop and outline for documentation view. Can indicate the selection status of the
/// container.
pub mod background {
    use super::*;

    ensogl::define_shape_system! {
        (selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let radius        = 1.px() * &radius;
            // let color_bg      = color::Lcha::new(0.2,0.013,0.18,1.0);
            let color_bg      = color::Lcha::new(1.0,0.0,0.0,1.0);
            let corner_radius = &radius * &roundness;
            let background    = Rect((&width,&height)).corners_radius(&corner_radius);
            let background    = background.fill(color::Rgba::from(color_bg));
            background.into()
        }
    }
}


/// Container overlay shape definition. Used to capture events over the documentation view within
/// the container.
pub mod overlay {
    use super::*;

    ensogl::define_shape_system! {
        (selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let radius        = 1.px() * &radius;
            let corner_radius = &radius * &roundness;
            let color_overlay = color::Rgba::new(1.0,0.0,0.0,0.000_000_1);
            let overlay       = Rect((&width,&height)).corners_radius(&corner_radius);
            let overlay       = overlay.fill(color_overlay);
            let out           = overlay;
            out.into()
        }
    }
}



// ===========
// === FRP ===
// ===========

/// Event system of the `Container`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    pub set_visibility     : frp::Source<bool>,
    pub toggle_visibility  : frp::Source,
    pub set_data           : frp::Source<String>,
    pub select             : frp::Source,
    pub deselect           : frp::Source,
    pub set_size           : frp::Source<Vector2>,
    pub clicked            : frp::Stream,
    on_click               : frp::Source,
    scene_shape            : frp::Sampler<scene::Shape>,
    size                   : frp::Sampler<Vector2>,
}

impl Frp {
    fn new(network:&frp::Network, scene:&Scene) -> Self {
        frp::extend! { network
            set_visibility      <- source();
            toggle_visibility   <- source();
            set_data            <- source();
            select              <- source();
            deselect            <- source();
            on_click            <- source();
            set_size            <- source();
            size                <- set_size.sampler();
            let clicked          = on_click.clone_ref().into();
        };
        let scene_shape = scene.shape().clone_ref();
        Self {set_visibility,toggle_visibility,set_data,select,deselect,clicked,set_size,on_click,
            scene_shape,size}
    }
}



// ============
// === View ===
// ============

/// View of the documentation container.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct View {
    logger         : Logger,
    display_object : display::object::Instance,
    background     : component::ShapeView<background::Shape>,
    overlay        : component::ShapeView<overlay::Shape>,
}

impl View {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = Logger::sub(logger,"view");
        let display_object = display::object::Instance::new(&logger);
        let background     = component::ShapeView::<background::Shape>::new(&logger,scene);
        let overlay        = component::ShapeView::<overlay::Shape>::new(&logger,scene);
        display_object.add_child(&overlay);
        display_object.add_child(&background);

        let shape_system = scene.shapes.shape_system(PhantomData::<background::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz.add(&shape_system.shape_system.symbol);

        Self {logger,display_object,background,overlay}
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ======================
// === ContainerModel ===
// ======================

/// Internal data of a `Container`.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ContainerModel {
    logger         : Logger,
    display_object : display::object::Instance,
    frp            : Frp,
    data           : RefCell<Option<String>>,
    scene          : Scene,
    view           : View,
}

impl ContainerModel {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene, network:&frp::Network) -> Self {
        let logger         = Logger::sub(logger,"documentation_container");
        let display_object = display::object::Instance::new(&logger);
        let data           = default();
        let frp            = Frp::new(&network,scene);
        let view           = View::new(&logger,scene);
        let scene          = scene.clone_ref();
        Self {logger,frp,data,display_object,view,scene}.init()
    }

    fn init(self) -> Self {
        self.update_shape_sizes();
        self.init_corner_roundness();
        // FIXME: These 2 lines fix a bug with display objects visible on stage.
        self.set_visibility(true);
        // self.set_visibility(false);
        self
    }

    /// Indicates whether the documentation is visible.
    pub fn is_visible(&self) -> bool {
        self.view.has_parent()
    }
}


// === Private API ===

impl ContainerModel {
    fn set_visibility(&self, visibility:bool) {
        if visibility {
            self.add_child(&self.view);
        }
        else {
            self.remove_child(&self.view);
        }
    }

    fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    fn set_doc_data(&self, data:&String) {
        self.data.borrow().for_each_ref(|_x| data)
    }

    fn update_shape_sizes(&self) {
        let size = self.frp.size.value();
        self.set_size(size);
    }

    fn set_size(&self, size:impl Into<Vector2>) {
        let size = size.into();
        self.view.background.shape.radius.set(CORNER_RADIUS);
        self.view.overlay.shape.radius.set(CORNER_RADIUS);
        self.view.background.shape.sprite.size.set(size);
        self.view.overlay.shape.sprite.size.set(size);
    }

    fn init_corner_roundness(&self) {
        self.set_corner_roundness(1.0)
    }

    fn set_corner_roundness(&self, value:f32) {
        self.view.overlay.shape.roundness.set(value);
        self.view.background.shape.roundness.set(value);
    }
}

impl display::Object for ContainerModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =================
// === Container ===
// =================

/// Container that wraps a `documentation_view::Instance` for rendering and interaction in the GUI.
///
/// The API to interact with the documentation_view is exposed through the `Frp`.
#[derive(Clone,CloneRef,Debug,Derivative,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Container {
    #[shrinkwrap(main_field)]
    pub model : Rc<ContainerModel>,
    pub frp   : Frp,
    network   : frp::Network,
}

impl Container {
    /// Constructor.
    pub fn new(logger:&Logger,scene:&Scene) -> Self {
        let network = frp::Network::new();
        let model   = Rc::new(ContainerModel::new(logger,scene,&network));
        let frp     = model.frp.clone_ref();
        Self {model,frp,network} . init(scene)
    }

    fn init(self,_scene:&Scene) -> Self {
        let inputs  = &self.frp;
        let network = &self.network;
        let model   = &self.model;
        let size    = Animation::<Vector2>::new(network);

        frp::extend! { network
            eval  inputs.set_visibility    ((v) model.set_visibility(*v));
            eval_ inputs.toggle_visibility (model.toggle_visibility());
            eval  inputs.set_data          ((t) model.set_doc_data(t));
            eval  inputs.set_size          ((s) size.set_target_value(*s));
        }

        inputs.set_size.emit(Vector2(DEFAULT_SIZE.0,DEFAULT_SIZE.1));
        size.skip();
        model.set_doc_data(&"<html></html>".to_string());
        self
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
