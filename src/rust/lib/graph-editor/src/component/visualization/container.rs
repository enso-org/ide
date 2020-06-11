//! This module defines the `Container` struct and related functionality.

// FIXME There is a serious performance problem in this implementation. It assumes that the
// FIXME visualization is a child of the container. However, this is very inefficient. Consider a
// FIXME visualization containing 1M of points. When moving a node (and thus moving a container),
// FIXME this would iterate over 1M of display objects and update their positions. Instead of that,
// FIXME each visualization should be positioned by some wise uniform management, maybe by a
// FIXME separate camera (view?) per visualization? This is also connected to a question how to
// FIXME create efficient dashboard view.

use crate::prelude::*;

use crate::data::EnsoCode;
use crate::frp;
use crate::visualization;

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
/// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
/// container.
pub mod background {
    use super::*;

    // TODO use style
    ensogl::define_shape_system! {
        (selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let radius        = 1.px() * &radius;
            let color_bg      = color::Lcha::new(0.2,0.013,0.18,1.0);
            let corner_radius = &radius * &roundness;
            let background    = Rect((&width,&height)).corners_radius(&corner_radius);
            let background    = background.fill(color::Rgba::from(color_bg));
            background.into()
        }
    }
}


/// Container background shape definition.
///
/// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
/// container.
pub mod fullscreen_background {
    use super::*;

    // TODO use style
    ensogl::define_shape_system! {
        (selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let radius        = 1.px() * &radius;
            let color_bg      = color::Lcha::new(0.2,0.013,0.18,1.0);
            let corner_radius = &radius * &roundness;
            let background    = Rect((&width,&height)).corners_radius(&corner_radius);
            let background    = background.fill(color::Rgba::from(color_bg));
            background.into()
        }
    }
}

/// Container overlay shape definition. Used to capture events over the visualisation within the
/// container.
pub mod overlay {
    use super::*;

    ensogl::define_shape_system! {
        (selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
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
    pub set_visualization  : frp::Source<Option<visualization::Instance>>,
    pub set_data           : frp::Source<visualization::Data>,
    pub select             : frp::Source,
    pub deselect           : frp::Source,
    pub set_size           : frp::Source<Vector2>,
    pub enable_fullscreen  : frp::Source,
    pub disable_fullscreen : frp::Source,
    pub clicked            : frp::Stream,
    pub preprocessor       : frp::Stream<EnsoCode>,
    on_click               : frp::Source,
    scene_shape            : frp::Sampler<scene::Shape>,
    size                   : frp::Sampler<Vector2>,
    preprocessor_select    : frp::Source<EnsoCode>,
}

impl Frp {
    fn new(network:&frp::Network, scene:&Scene) -> Self {
        frp::extend! { network
            set_visibility      <- source();
            toggle_visibility   <- source();
            set_visualization   <- source();
            set_data            <- source();
            select              <- source();
            deselect            <- source();
            on_click            <- source();
            set_size            <- source();
            enable_fullscreen   <- source();
            disable_fullscreen  <- source();
            preprocessor_select <- source();
            size                <- set_size.sampler();
            let clicked          = on_click.clone_ref().into();
            let preprocessor     = preprocessor_select.clone_ref().into();
        };
        let scene_shape = scene.shape().clone_ref();
        Self {set_visibility,set_visualization,toggle_visibility,set_data,select,deselect,
              clicked,set_size,on_click,enable_fullscreen,disable_fullscreen,scene_shape,size,
              preprocessor,preprocessor_select}
    }
}



// ============
// === View ===
// ============

/// View of the visualization container.
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
// === FullscreenView ===
// ======================

/// View of the visualization container meant to be used in fullscreen mode. Its components are
/// rendered on top-level layers of the stage.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct FullscreenView {
    logger           : Logger,
    display_object   : display::object::Instance,
    background : component::ShapeView<fullscreen_background::Shape>,
}

impl FullscreenView {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = Logger::sub(logger,"fullscreen_view");
        let display_object = display::object::Instance::new(&logger);
        let background     = component::ShapeView::<fullscreen_background::Shape>::new(&logger,scene);
        display_object.add_child(&background);

        let shape_system = scene.shapes.shape_system(PhantomData::<fullscreen_background::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz_fullscreen.add(&shape_system.shape_system.symbol);

        Self {logger,display_object,background}
    }
}

impl display::Object for FullscreenView {
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
    logger          : Logger,
    display_object  : display::object::Instance,
    frp             : Frp,
    visualization   : RefCell<Option<visualization::Instance>>,
    scene           : Scene,
    view            : View,
    fullscreen_view : FullscreenView,
    is_fullscreen   : Rc<Cell<bool>>,
}

impl ContainerModel {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene, network:&frp::Network) -> Self {
        let logger          = Logger::sub(logger,"visualization_container");
        let display_object  = display::object::Instance::new(&logger);
        let visualization   = default();
        let frp             = Frp::new(&network,scene);
        let view            = View::new(&logger,scene);
        let fullscreen_view = FullscreenView::new(&logger,scene);
        let scene           = scene.clone_ref();
        let is_fullscreen   = default();
        Self {logger,frp,visualization,display_object,view,fullscreen_view,scene,is_fullscreen}
            . init()
    }

    fn init(self) -> Self {
        self.update_shape_sizes();
        self.init_corner_roundness();
        // FIXME: These 2 lines fix a bug with display objects visible on stage.
        self.set_visibility(true);
        self.set_visibility(false);
        self
    }

    /// Indicates whether the visualization is visible.
    pub fn is_visible(&self) -> bool {
        self.view.has_parent()
    }
}


// === Private API ===

impl ContainerModel {
    fn set_visibility(&self, visibility:bool) {
        if visibility {
            self.add_child(&self.view);
            self.scene.add_child(&self.fullscreen_view);
        }
        else {
            self.remove_child(&self.view);
            self.scene.remove_child(&self.fullscreen_view);
        }
    }

    fn enable_fullscreen(&self) {
        self.is_fullscreen.set(true);
        if let Some(viz) = &*self.visualization.borrow() {
            self.fullscreen_view.add_child(viz)
        }
    }

    fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    fn set_visualization(&self, visualization:Option<visualization::Instance>) {
        if let Some(visualization) = visualization {
            let size = self.frp.size.value();
            visualization.set_size.emit(size);
            self.view.add_child(&visualization);
            self.visualization.replace(Some(visualization));
        }
    }

    fn set_visualization_data(&self, data:&visualization::Data) {
        self.visualization.borrow().for_each_ref(|vis| vis.send_data.emit(data))
    }

    fn update_shape_sizes(&self) {
        let size = self.frp.size.value();
        self.set_size(size);
    }

    fn set_size(&self, size:impl Into<Vector2>) {
        let size = size.into();
        if self.is_fullscreen.get() {
            self.fullscreen_view.background . shape.radius.set(CORNER_RADIUS);
            self.fullscreen_view.background . shape.sprite.size.set(size);
            self.view.background   . shape.sprite.size.set(zero());
            self.view.overlay . shape.sprite.size.set(zero());
        } else {
            self.view.background.shape.radius.set(CORNER_RADIUS);
            self.view.overlay.shape.radius.set(CORNER_RADIUS);
            self.view.background.shape.sprite.size.set(size);
            self.view.overlay.shape.sprite.size.set(size);
            self.fullscreen_view.background . shape.sprite.size.set(zero());
        }

        if let Some(viz) = &*self.visualization.borrow() {
            viz.set_size.emit(size);
        }
    }

    fn init_corner_roundness(&self) {
        self.set_corner_roundness(1.0)
    }

    fn set_corner_roundness(&self, value:f32) {
        self.view.overlay.shape.roundness.set(value);
        self.view.background.shape.roundness.set(value);
        self.fullscreen_view.background.shape.roundness.set(value);
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

// TODO: Finish the fullscreen management when implementing layout management.

/// Container that wraps a `visualization::Instance` for rendering and interaction in the GUI.
///
/// The API to interact with the visualization is exposed through the `Frp`.
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

    fn init(self,scene:&Scene) -> Self {
        let inputs     = &self.frp;
        let network    = &self.network;
        let model      = &self.model;
        let fullscreen = Animation::new(network);
        let size       = Animation::<Vector2>::new(network);
        let fullscreen_position = Animation::<Vector3>::new(network);

        frp::extend! { network
            eval  inputs.set_visibility    ((v) model.set_visibility(*v));
            eval_ inputs.toggle_visibility (model.toggle_visibility());
            eval  inputs.set_visualization ((v) model.set_visualization(v.clone()));
            eval  inputs.set_data          ((t) model.set_visualization_data(t));
            eval_ inputs.enable_fullscreen (model.set_visibility(true));
            eval_ inputs.enable_fullscreen (model.enable_fullscreen());
            eval_ inputs.enable_fullscreen (fullscreen.set_target_value(1.0));
            eval  inputs.set_size          ((s) size.set_target_value(*s));

            _eval <- fullscreen.value.all_with3(&size.value,&inputs.scene_shape,
                f!([model] (weight,viz_size,scene_size) {
                    let weight_inv           = 1.0 - weight;
                    let scene_size : Vector2 = scene_size.into();
                    let current_size         = viz_size * weight_inv + scene_size * *weight;
                    model.set_corner_roundness(weight_inv);
                    model.set_size(current_size);

                    let m1  = model.scene.views.viz_fullscreen.camera.inversed_view_matrix();
                    let m2  = model.scene.views.viz.camera.view_matrix();
                    let pos = model.global_position();
                    let pos = Vector4::new(pos.x,pos.y,pos.z,1.0);
                    let pos = m2 * (m1 * pos);
                    let pp = Vector3(pos.x,pos.y,pos.z);
                    let current_pos = pp * weight_inv;
                    model.fullscreen_view.set_position(current_pos);

            }));

            eval fullscreen_position.value ((p) model.fullscreen_view.set_position(*p));
            eval model.frp.preprocessor    ((code) inputs.preprocessor_select.emit(code));
        }

        inputs.set_size.emit(Vector2(DEFAULT_SIZE.0,DEFAULT_SIZE.1));
        size.skip();
        model.set_visualization(Some(visualization::Registry::default_visualisation(scene)));
        self
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
