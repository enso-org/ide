//! This module defines the `Container` struct and related functionality.

// FIXME There is a serious performance problem in this implementation. It assumes that the
// FIXME visualization is a child of the container. However, this is very inefficient. Consider a
// FIXME visualization containing 1M of points. When moving a node (and thus moving a container),
// FIXME this would iterate over 1M of display objects and update their positions. Instead of that,
// FIXME each visualization should be positioned by some wise uniform management, maybe by a
// FIXME separate camera (view?) per visualization? This is also connected to a question how to
// FIXME create efficient dashboard view.

mod action_bar;
mod visualization_chooser;

use crate::prelude::*;

use crate::component::text_list::TextList;
use crate::graph_editor::component::node::quick_action_bar::QuickActionBar;
use crate::data::EnsoCode;
use crate::visualization;

use action_bar::QuickActionBar;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::DomSymbol;
use ensogl::display::scene;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl::application::Application;
use ensogl::gui::component;
use ensogl::system::web;
use ensogl::system::web::StyleSetter;
use ensogl_theme as theme;




// =================
// === Constants ===
// =================

const DEFAULT_SIZE            : (f32,f32) = (200.0,200.0);
const CORNER_RADIUS           : f32       = super::super::node::CORNER_RADIUS;
const SHADOW_SIZE             : f32       = super::super::node::SHADOW_SIZE;
const QUICK_ACTION_BAR_HEIGHT : f32       = 2.0 * CORNER_RADIUS;


// =============
// === Shape ===
// =============

    /// Container background shape definition.
    ///
    /// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
    /// container.
    /// TODO : We do not use backgrounds because otherwise they would overlap JS
///        visualizations. Instead we added a HTML background to the `View`.
///        This should be further investigated while fixing rust visualization displaying. (#526)
pub mod background {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style,selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let width         = &width  - SHADOW_SIZE.px() * 2.0;
            let height        = &height - SHADOW_SIZE.px() * 2.0;
            let radius        = 1.px() * &radius;
            let color_path    = theme::vars::graph_editor::visualization::background::color;
            let color_bg      = style.get_color(color_path);
            let corner_radius = &radius * &roundness;
            let background    = Rect((&width,&height)).corners_radius(&corner_radius);
            let background    = background.fill(color::Rgba::from(color_bg));

            // === Shadow ===

            let border_size_f = 16.0;
            let corner_radius = corner_radius*1.75;
            let width         = &width  + SHADOW_SIZE.px() * 2.0;
            let height        = &height + SHADOW_SIZE.px() * 2.0;
            let shadow        = Rect((&width,&height)).corners_radius(&corner_radius).shrink(1.px());
            let shadow_color  = color::LinearGradient::new()
                .add(0.0,color::Rgba::new(0.0,0.0,0.0,0.0).into_linear())
                .add(1.0,color::Rgba::new(0.0,0.0,0.0,0.20).into_linear());
            let shadow_color  = color::SdfSampler::new(shadow_color)
                .max_distance(border_size_f)
                .slope(color::Slope::Exponent(2.0));
            let shadow        = shadow.fill(shadow_color);

            let out = shadow + background;
            out.into()
        }
    }
}

/// Container background shape definition.
///
/// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
/// container.
/// TODO : We do not use backgrounds because otherwise they would overlap JS
///        visualizations. Instead we added a HTML background to the `View`.
///        This should be further investigated while fixing rust visualization displaying. (#526)
pub mod fullscreen_background {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style,selected:f32,radius:f32,roundness:f32) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let radius        = 1.px() * &radius;
            let color_path    = theme::vars::graph_editor::visualization::background::color;
            let color_bg      = style.get_color(color_path);
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
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let radius               = 1.px() * &radius;
            let corner_radius        = &radius * &roundness;
            let color_overlay        = color::Rgba::new(1.0,0.0,0.0,0.000_000_1);
            let overlay              = Rect((&width,&height)).corners_radius(&corner_radius);
            let overlay              = overlay.fill(color_overlay);
            let out                  = overlay;
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
    pub set_visibility                 : frp::Source<bool>,
    pub toggle_visibility              : frp::Source,
    pub set_visualization              : frp::Source<Option<visualization::Instance>>,
    pub set_data                       : frp::Source<visualization::Data>,
    pub set_visualization_alternatives : frp::Source<Vec<visualization::Path>>,
    pub select                         : frp::Source,
    pub deselect                       : frp::Source,
    pub set_size                       : frp::Source<Vector2>,
    pub enable_fullscreen              : frp::Source,
    pub disable_fullscreen             : frp::Source,
    pub  preprocessor                   : frp::Stream<EnsoCode>,

    scene_shape                        : frp::Sampler<scene::Shape>,
    size                               : frp::Sampler<Vector2>,
    preprocessor_select                : frp::Source<EnsoCode>,
}

impl Frp {
    fn new(network:&frp::Network, scene:&Scene) -> Self {
        frp::extend! { network
            set_visibility                 <- source();
            toggle_visibility              <- source();
            set_visualization              <- source();
            set_data                       <- source();
            set_visualization_alternatives <- source();
            select                         <- source();
            deselect                       <- source();

            set_size                       <- source();
            enable_fullscreen              <- source();
            disable_fullscreen             <- source();
            preprocessor_select            <- source();
            size                           <- set_size.sampler();

            let preprocessor     = preprocessor_select.clone_ref().into();
        };
        let scene_shape = scene.shape().clone_ref();
        Self {set_visibility,set_visualization,toggle_visibility,set_data,select,deselect,
              set_size,enable_fullscreen,disable_fullscreen,scene_shape,size,preprocessor,
              preprocessor_select,set_visualization_alternatives}
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
    // background     : component::ShapeView<background::Shape>,
    overlay        : component::ShapeView<overlay::Shape>,
    background_dom : DomSymbol
}

impl View {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = Logger::sub(logger,"view");
        let display_object = display::object::Instance::new(&logger);
        // let background     = component::ShapeView::<background::Shape>::new(&logger,scene);
        let overlay        = component::ShapeView::<overlay::Shape>::new(&logger,scene);
        display_object.add_child(&overlay);
        // display_object.add_child(&background);

        // let shape_system = scene.shapes.shape_system(PhantomData::<background::Shape>);
        // scene.views.main.remove(&shape_system.shape_system.symbol);
        // scene.views.viz.add(&shape_system.shape_system.symbol);

        let shape_system = scene.shapes.shape_system(PhantomData::<overlay::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz.add(&shape_system.shape_system.symbol);

        let div            = web::create_div();
        let background_dom = DomSymbol::new(&div);
        // TODO : We added a HTML background to the `View`, because "shape" background was overlapping
        //        the JS visualization. This should be further investigated while fixing rust
        //        visualization displaying. (#796)
        background_dom.dom().set_style_or_warn("width"         ,"0"                        ,&logger);
        background_dom.dom().set_style_or_warn("height"        ,"0"                        ,&logger);
        background_dom.dom().set_style_or_warn("z-index"       ,"1"                        ,&logger);
        background_dom.dom().set_style_or_warn("overflow-y"    ,"auto"                     ,&logger);
        background_dom.dom().set_style_or_warn("overflow-x"    ,"auto"                     ,&logger);
        background_dom.dom().set_style_or_warn("background"    ,"#FDF9F6FA"                ,&logger);
        background_dom.dom().set_style_or_warn("border-radius" ,"14px"                     ,&logger);
        background_dom.dom().set_style_or_warn("box-shadow"    ,"0 0 16px rgba(0,0,0,0.14)",&logger);
        display_object.add_child(&background_dom);

        scene.dom.layers.back.manage(&background_dom);

        Self {logger,display_object,overlay,background_dom}
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
    logger         : Logger,
    display_object : display::object::Instance,
    // background     : component::ShapeView<fullscreen_background::Shape>,
    background_dom : DomSymbol
}

impl FullscreenView {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = Logger::sub(logger,"fullscreen_view");
        let display_object = display::object::Instance::new(&logger);
        // let background     = component::ShapeView::<fullscreen_background::Shape>::new(&logger,scene);
        // display_object.add_child(&background);

        let shape_system = scene.shapes.shape_system(PhantomData::<fullscreen_background::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz_fullscreen.add(&shape_system.shape_system.symbol);

        let div            = web::create_div();
        let background_dom = DomSymbol::new(&div);
        // TODO : We added a HTML background to the `View`, because "shape" background was overlapping
        //        the JS visualization. This should be further investigated while fixing rust
        //        visualization displaying. (#796)
        background_dom.dom().set_style_or_warn("width"         ,"0"                           ,&logger);
        background_dom.dom().set_style_or_warn("height"        ,"0"                           ,&logger);
        background_dom.dom().set_style_or_warn("z-index"       ,"1"                           ,&logger);
        background_dom.dom().set_style_or_warn("overflow-y"    ,"auto"                        ,&logger);
        background_dom.dom().set_style_or_warn("overflow-x"    ,"auto"                        ,&logger);
        background_dom.dom().set_style_or_warn("background"    ,"#FDF9F6FA"                   ,&logger);
        background_dom.dom().set_style_or_warn("border-radius" ,"0"                           ,&logger);
        display_object.add_child(&background_dom);

        scene.dom.layers.back.manage(&background_dom);

        Self {logger,display_object,background_dom}
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
    registry        : visualization::Registry,

    action_bar           : QuickActionBar,
}



impl ContainerModel {
    /// Constructor.
    pub fn new
    (logger:&Logger, app:&Application, network:&frp::Network, registry:visualization::Registry)
    -> Self {
        let scene           = app.display.scene();
        let logger          = Logger::sub(logger,"visualization_container");
        let display_object  = display::object::Instance::new(&logger);
        let visualization   = default();
        let frp             = Frp::new(&network,scene);
        let view            = View::new(&logger,scene);
        let fullscreen_view = FullscreenView::new(&logger,scene);
        let scene           = scene.clone_ref();
        let is_fullscreen   = default();

        let action_bar = QuickActionBar::new(&app);
        view.add_child(&action_bar);

        Self {logger,frp,visualization,display_object,view,fullscreen_view,scene,is_fullscreen,
              action_bar,registry}
            . init()
    }

    fn init(self) -> Self {
        self.update_shape_sizes();
        self.init_corner_roundness();
        // FIXME: These 4 lines fix a bug with display objects visible on stage.
        self.set_visibility(true);
        self.set_visibility(false);
        self
    }

    /// Indicates whether the visualization container is visible and active.
    /// Note: can't be called `is_visible` due to a naming conflict with `display::object::class`.
    pub fn is_active(&self) -> bool {
        self.view.has_parent()
    }
}


// === Private API ===

impl ContainerModel {
    fn set_visibility(&self, visibility:bool) {
        if visibility {
            self.add_child(&self.view);
            self.show_visualisation();
            self.scene.add_child(&self.fullscreen_view);
        }
        else {
            // FIXME: If we hide the children also, they stay visible and no longer move
            // with the parent
            self.show_visualisation();

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
        self.set_visibility(!self.is_active())
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
            // self.fullscreen_view.background.shape.radius.set(CORNER_RADIUS);
            // self.fullscreen_view.background.shape.sprite.size.set(size);
            // self.view.background.shape.sprite.size.set(zero());
            self.view.overlay.shape.sprite.size.set(zero());
            self.view.background_dom.dom().set_style_or_warn("width" ,"0",&self.logger);
            self.view.background_dom.dom().set_style_or_warn("height","0",&self.logger);
            self.fullscreen_view.background_dom.dom().set_style_or_warn("width", format!("{}px", size[0]), &self.logger);
            self.fullscreen_view.background_dom.dom().set_style_or_warn("height", format!("{}px", size[1]), &self.logger);
            self.action_bar.frp.set_size.emit(Vector2::zero());
        } else {
            // self.view.background.shape.radius.set(CORNER_RADIUS);
            self.view.overlay.shape.radius.set(CORNER_RADIUS);
            // self.view.background.shape.sprite.size.set(size);
            self.view.overlay.shape.sprite.size.set(size);
            self.view.background_dom.dom().set_style_or_warn("width" ,format!("{}px",size[0]),&self.logger);
            self.view.background_dom.dom().set_style_or_warn("height",format!("{}px",size[1]),&self.logger);
            self.fullscreen_view.background_dom.dom().set_style_or_warn("width", "0", &self.logger);
            self.fullscreen_view.background_dom.dom().set_style_or_warn("height", "0", &self.logger);
            // self.fullscreen_view.background.shape.sprite.size.set(zero());

            let quick_action_size = Vector2::new(size.x, QUICK_ACTION_BAR_HEIGHT);
            self.action_bar.frp.set_size.emit(quick_action_size);
        }

        self.action_bar.set_position_y((size.y - QUICK_ACTION_BAR_HEIGHT) / 2.0);

        if let Some(viz) = &*self.visualization.borrow() {
            viz.set_size.emit(size);
        }
    }

    fn init_corner_roundness(&self) {
        self.set_corner_roundness(1.0)
    }

    fn set_corner_roundness(&self, value:f32) {
        self.view.overlay.shape.roundness.set(value);
        // self.view.background.shape.roundness.set(value);
        // self.fullscreen_view.background.shape.roundness.set(value);
    }

    fn show_visualisation(&self) {
        if let Some(vis) = self.visualization.borrow().as_ref() {
            vis.display_object().set_parent(&self.view)
        }
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
    pub fn new(logger:&Logger,app:&Application,registry:visualization::Registry) -> Self {
        let scene   = app.display.scene();
        let network = frp::Network::new();
        let model   = Rc::new(ContainerModel::new(logger,app,&network,registry));
        let frp     = model.frp.clone_ref();
        Self {model,frp,network} . init(scene)
    }

    /// Sets pointer-events `value` of all `HtmlElement`s of the `visualization` class.
    pub fn set_all_visualizations_pointer_events(value:impl Str) {
        use wasm_bindgen::JsCast;
        let value    = value.into();
        let elements = ensogl::system::web::get_elements_by_class_name("visualization");
        let logger   = Logger::new("Visualizations");
        if let Ok(elements) = elements {
            for element in elements {
                if let Ok(html_element) = element.dyn_into::<web_sys::HtmlElement>() {
                    html_element.set_style_or_warn("pointer-events",&value,&logger)
                }
            }
        }
    }

    fn init(self,scene:&Scene) -> Self {
        let inputs              = &self.frp;
        let network             = &self.network;
        let model               = &self.model;
        let fullscreen          = Animation::new(network);
        let size                = Animation::<Vector2>::new(network);
        let fullscreen_position = Animation::<Vector3>::new(network);

        // let visualization_chooser  = &model.visualization_chooser.frp;
        let action_bar       = &model.action_bar.frp;
        let registry = &model.registry;

        frp::extend! { network
            eval  inputs.set_visibility                 ((v) model.set_visibility(*v));
            eval_ inputs.toggle_visibility              (model.toggle_visibility());
            eval  inputs.set_visualization              ((v) {
                model.set_visualization(v.clone());
                model.action_bar.frp.set_label.emit("VisName".to_string());
            });
            eval  inputs.set_data                       ((t) model.set_visualization_data(t));
            eval  inputs.set_visualization_alternatives ((alternatives) {
                action_bar.input.set_visualization_alternatives.emit(alternatives)
            });

            eval_ inputs.enable_fullscreen (model.set_visibility(true));
            eval_ inputs.enable_fullscreen (model.enable_fullscreen());
            eval_ inputs.enable_fullscreen (fullscreen.set_target_value(1.0));
            eval  inputs.set_size          ((s) size.set_target_value(*s));
            eval_ model.view.overlay.events.mouse_down([] {
                Container::set_all_visualizations_pointer_events("auto");
            });

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
        // Visualisation chooser frp bindings
        frp::extend! { network
            eval action_bar.visualisation_selection([model,registry,scene,action_bar](visualization_path) {
                if let Some(path) = visualization_path {
                    if let Some(definition) = registry.definition_from_path(path) {
                        if let Ok(visualization) = definition.new_instance(&scene) {
                            model.set_visualization(Some(visualization));
                            let alternatives = model.registry.valid_alternatives(&definition);
                            model.frp.set_visualization_alternatives.emit(alternatives);
                            model.action_bar.frp.set_label.emit(format!("{}", &definition.signature.path));
                        }
                    }
                    action_bar.hide_icons.emit(());
                }
            });
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
