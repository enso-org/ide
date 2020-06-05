//! This module defines the `Container` struct and related functionality.

use crate::prelude::*;

use crate::component::visualization::traits::HasFullscreenDecoration;
use crate::component::visualization::traits::Resizable;
use crate::component::visualization::traits::SymbolWithLayout;
use crate::component::visualization::traits::TargetLayer;
use crate::frp;
use crate::visualization::*;

use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::DomSymbol;
use ensogl::display::Symbol;
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

const DEFAULT_SIZE  : V2  = V2(200.0,200.0);
const CORNER_RADIUS : f32 = super::super::node::CORNER_RADIUS;



// =============
// === Shape ===
// =============

/// Container frame shape definition.
///
/// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
/// container.
pub mod frame {
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

            let out = background;
            out.into()
        }
    }
}


/// Container frame shape definition.
///
/// Provides a backdrop and outline for visualisations. Can indicate the selection status of the
/// container.
pub mod frame2 {
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
            let background    = background.fill(color::Rgba::new(1.0,0.0,0.0,1.0));

            let out = background;
            out.into()
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
    pub set_visualization  : frp::Source<Option<Visualization>>,
    pub set_data           : frp::Source<Data>,
    pub select             : frp::Source,
    pub deselect           : frp::Source,
    pub set_size           : frp::Source<V2>,
    pub enable_fullscreen  : frp::Source,
    pub disable_fullscreen : frp::Source,
    pub clicked            : frp::Stream,
    on_click               : frp::Source,
    scene_shape            : frp::Sampler<scene::Shape>,
    size                   : frp::Sampler<V2>,
}

impl Frp {
    fn new(network:&frp::Network, scene:&Scene) -> Self {
        frp::extend! { network
            set_visibility     <- source();
            toggle_visibility  <- source();
            set_visualization  <- source();
            set_data           <- source();
            select             <- source();
            deselect           <- source();
            on_click           <- source();
            set_size           <- source();
            enable_fullscreen  <- source();
            disable_fullscreen <- source();
            size               <- set_size.sampler();
            let clicked         = on_click.clone_ref().into();
        };
        let scene_shape = scene.shape().clone_ref();
        Self {set_visibility,set_visualization,toggle_visibility,set_data,select,deselect,
              clicked,set_size,on_click,enable_fullscreen,disable_fullscreen,scene_shape,size}
    }
}



// ==============
// === Shapes ===
// ==============

#[derive(Debug)]
#[allow(missing_docs)]
pub struct Shapes {
    logger           : Logger,
    display_object   : display::object::Instance,
    frame            : component::ShapeView<frame::Shape>,
    frame_fullscreen : component::ShapeView<frame2::Shape>,
    overlay          : component::ShapeView<overlay::Shape>,
}

impl Shapes {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger           = logger.sub("shapes");
        let display_object   = display::object::Instance::new(&logger);
        let frame            = component::ShapeView::<frame::Shape>::new(&logger,scene);
        let frame_fullscreen = component::ShapeView::<frame2::Shape>::new(&logger,scene);
        let overlay          = component::ShapeView::<overlay::Shape>::new(&logger,scene);
        display_object.add_child(&overlay);
        display_object.add_child(&frame);
        scene.add_child(&frame_fullscreen);
//        display_object.add_child(&frame_fullscreen);

        let shape_system = scene.shapes.shape_system(PhantomData::<frame::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz.add(&shape_system.shape_system.symbol);

        let shape_system = scene.shapes.shape_system(PhantomData::<frame2::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.viz_fullscreen.add(&shape_system.shape_system.symbol);

        Self {logger,display_object,frame,frame_fullscreen,overlay}
    }
}

impl display::Object for Shapes {
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
    visualization  : RefCell<Option<Visualization>>,
    scene          : Scene,
    shapes         : Shapes,
    is_fullscreen  : Rc<Cell<bool>>,
}

impl ContainerModel {
    pub fn new(logger:&Logger, scene:&Scene, network:&frp::Network) -> Self {
        let logger         = logger.sub("visualization_container");
        let display_object = display::object::Instance::new(&logger);
        let visualization  = default();
        let frp            = Frp::new(&network,scene);
        let shapes         = Shapes::new(&logger,scene);
        let scene          = scene.clone_ref();
        let is_fullscreen  = default();
        Self {logger,frp,visualization,display_object,shapes,scene,is_fullscreen} . init()
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
        self.shapes.has_parent()
    }

    /// Set whether the visualization should be visible or not.
    fn set_visibility(&self, visibility:bool) {
        if visibility { self.add_child    (&self.shapes) }
        else          { self.remove_child (&self.shapes) }
    }


    fn enable_fullscreen(&self) {


    }
}


// === Private API ===

impl ContainerModel {
    fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    fn set_visualization(&self, visualization:Option<Visualization>) {
        if let Some(visualization) = visualization {
            let size = self.frp.size.value();
            visualization.set_size.emit(size);
            self.shapes.add_child(&visualization);
            self.visualization.replace(Some(visualization));
        }
    }

    fn set_visualization_data(&self, data:&Data) {
        self.visualization.borrow().for_each_ref(|vis| vis.send_data.emit(data))
    }

    fn update_shape_sizes(&self) {
        let size = self.frp.size.value().into();
        self.shapes.frame            . shape.radius.set(CORNER_RADIUS);
        self.shapes.frame_fullscreen . shape.radius.set(CORNER_RADIUS);
        self.shapes.overlay          . shape.radius.set(CORNER_RADIUS);
        self.shapes.frame            . shape.sprite.size().set(size);
        self.shapes.frame_fullscreen . shape.sprite.size().set(size);
        self.shapes.overlay          . shape.sprite.size().set(size);
    }

    fn set_size(&self, size:impl Into<V2>) {
        let size = size.into();
        self.shapes.frame            . shape.radius.set(CORNER_RADIUS);
        self.shapes.frame_fullscreen . shape.radius.set(CORNER_RADIUS);
        self.shapes.overlay          . shape.radius.set(CORNER_RADIUS);
        self.shapes.frame            . shape.sprite.size().set(size.into());
        self.shapes.frame_fullscreen . shape.sprite.size().set(size.into());
        self.shapes.overlay          . shape.sprite.size().set(size.into());

        if let Some(viz) = &*self.visualization.borrow() {
            viz.set_size.emit(size);
        }

//        if let Some(vis) = self.visualization.borrow().as_ref() {
//            let radius   = CORNER_RADIUS * 2.0;
//            let radius   = V2(radius,radius);
//            let vis_size = size - radius;
//            vis.set_size.emit(vis_size);
//        }
//
//        self.size.set(size);
//        self.update_shape_sizes();

    }

    fn init_corner_roundness(&self) {
        self.set_corner_roundness(1.0)
    }

    fn set_corner_roundness(&self, value:f32) {
        self.shapes.overlay.shape.roundness.set(value);
        self.shapes.frame.shape.roundness.set(value);
        self.shapes.frame_fullscreen.shape.roundness.set(value);
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

/// Container that wraps a `Visualization` for rendering and interaction in the GUI.
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
        let selection  = Animation::new(network);
        let fullscreen = Animation::new(network);
        let size       = Animation::<V2>::new(network);
        let fullscreen_position   = Animation::<V3>::new(network);


        frp::extend! { network
            eval  selection.value          ((value) model.shapes.frame.shape.selected.set(*value));
            eval  inputs.set_visibility    ((v) model.set_visibility(*v));
            eval_ inputs.toggle_visibility (model.toggle_visibility());
            eval  inputs.set_visualization ((v) model.set_visualization(v.clone()));
            eval  inputs.set_data          ((t) model.set_visualization_data(t));
            eval_ inputs.select            (selection.set_target_value(1.0));
            eval_ inputs.deselect          (selection.set_target_value(0.0));
            eval_ model.shapes.overlay.events.mouse_down (inputs.on_click.emit(()));

            eval_ inputs.enable_fullscreen (model.set_visibility(true));
            eval_ inputs.enable_fullscreen (model.enable_fullscreen());
            eval_ inputs.enable_fullscreen (fullscreen.set_target_value(1.0));

            _eval <- inputs.enable_fullscreen.map2(&inputs.scene_shape,f!([model,fullscreen_position,size](_,scene_size){
                let m1  = model.scene.views.viz_fullscreen.camera.inversed_view_matrix();
                let m2  = model.scene.views.viz.camera.view_matrix();
                let pos = model.global_position();
                let pos = Vector4::new(pos.x,pos.y,pos.z,1.0);
                let pos = m2 * (m1 * pos);
                let pp = V3(pos.x,pos.y,pos.z);
                fullscreen_position.set_target_value(pp);
                fullscreen_position.skip();
//                let tgt_pos = V3(pos.x,pos.y,pos.z) * weight;
                let scene_size : V2 = scene_size.into();
                let tgt_pos = V3(scene_size.x/2.0,scene_size.y/2.0,0.0);
                fullscreen_position.set_target_value(tgt_pos);

                size.set_target_value(scene_size.into());
            }));

//            _eval <- fullscreen.value.all_with3
//                (&inputs.set_size,&inputs.scene_shape
//                ,f!([model,size,fullscreen_position](weight,viz_size,scene_size) {
//                    let weight_inv      = 1.0 - weight;
//                    let scene_size : V2 = scene_size.into();
//                    let target_size     = viz_size * weight_inv + scene_size * weight;
//                    model.set_corner_roundness(weight_inv);
//                    size.set_target_value(target_size);
//            }));

            eval inputs.set_size ((s) size.set_target_value(s.into()));
//
//            _foo <- fullscreen.value.all_with3(&size.value,&inputs.scene_shape,
//                f!([model] (weight,viz_size,scene_size) {
//                    let weight_inv      = 1.0 - weight;
//                    let scene_size : V2 = scene_size.into();
//                    let current_size    = weight_inv * viz_size + weight * scene_size;
//                    model.set_corner_roundness(weight_inv);
//                    model.set_size(current_size);
//
//
//                let m1  = model.scene.views.viz_fullscreen.camera.inversed_view_matrix();
//                let m2  = model.scene.views.viz.camera.view_matrix();
//                let pos = model.global_position();
//                let pos = Vector4::new(pos.x,pos.y,pos.z,1.0);
//                let pos = m2 * (m1 * pos);
//                let pp = V3(pos.x,pos.y,pos.z);
////                fullscreen_position.set_target_value(pp);
////                fullscreen_position.skip();
////                let tgt_pos = V3(pos.x,pos.y,pos.z) * weight;
//                let scene_size : V2 = scene_size.into();
//                let tgt_pos = V3(scene_size.x/2.0,scene_size.y/2.0,0.0);
////                fullscreen_position.set_target_value(tgt_pos);
//
//                let current_pos = pp * weight_inv + tgt_pos * weight;
//                model.shapes.frame_fullscreen.set_position(current_pos.into());
//
//            }));

            eval size.value     ((v) model.set_size(v));
            eval fullscreen_position.value ((p)  model.shapes.frame_fullscreen.set_position(p.into()));

        }

//        inputs.set_size.emit(DEFAULT_SIZE);

        inputs.set_size.emit(DEFAULT_SIZE);
        size.skip();

        model.set_visualization(Some(Registry::default_visualisation(scene)));


        self
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}

//impl HasFullscreenDecoration for Container {
//    fn enable_fullscreen_decoration(&self) {
//        self.model.set_corner_roundness(0.0);
//    }
//
//    fn disable_fullscreen_decoration(&self) {
//        self.model.set_corner_roundness(1.0);
//    }
//}

//impl Resizable for Container {
//    fn set_size(&self, size:Vector3<f32>) {
//        self.model.set_size(size);
//    }
//
//    fn size(&self) -> Vector3<f32>{
//        Vector3::new(self.model.size.get().x,self.model.size.get().y, 0.0)
//    }
//}

//impl HasSymbols for Container {
//    fn symbols(&self) -> Vec<Symbol> {
//        let mut symbols  = self.container_main_symbols();
//        if let Some(vis) = self.model.visualization.borrow().as_ref() {
//            symbols.extend(vis.symbols());
//        };
//        symbols
//    }
//
//    fn symbols_with_data(&self) -> Vec<SymbolWithLayout> {
//        let target_layer = TargetLayer::Main;
//        let symbols      = self.container_main_symbols().into_iter();
//        let symbols  = symbols.map(move |symbol| SymbolWithLayout {symbol,target_layer});
//        let vis_symbols  = self.model.visualization.borrow().as_ref().map(|vis| vis.symbols_with_data()).unwrap_or_default();
//        symbols.chain(vis_symbols).collect()
//    }
//}
//
//impl HasDomSymbols for Container {
//    fn dom_symbols(&self) -> Vec<DomSymbol> {
//        if let Some(vis) = self.model.visualization.borrow().as_ref() {
//            vis.dom_symbols()
//        } else{
//            vec![]
//        }
//    }
//}

//    /// Return the symbols of the container, not of the visualization.
//    fn container_main_symbols(&self) -> Vec<Symbol> {
//        let shape_system_frame   = self.scene.shapes.shape_system(PhantomData::<frame::Shape>);
//        let shape_system_overlay = self.scene.shapes.shape_system(PhantomData::<overlay::Shape>);
//        vec![
//            shape_system_frame.shape_system.symbol.clone_ref(),
//            shape_system_overlay.shape_system.symbol.clone_ref(),
//        ]
//    }