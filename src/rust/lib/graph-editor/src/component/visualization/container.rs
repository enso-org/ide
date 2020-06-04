//! This module defines the `Container` struct and related functionality.

use crate::prelude::*;

use crate::component::visualization::traits::HasSymbols;
use crate::component::visualization::traits::HasFullscreenDecoration;
use crate::component::visualization::traits::HasDomSymbols;
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

            let frame_outer = Rect((&width,&height)).corners_radius(&corner_radius);

            let padding            = &radius * 2.0 * &selected;
            let padding_aliased    = padding - 1.px();
            let width_frame_inner  = &width  - &padding_aliased;
            let height_frame_inner = &height - &padding_aliased;
            let width_frame_inner  = Var::<Distance<Pixels>>::from(width_frame_inner);
            let height_frame_inner = Var::<Distance<Pixels>>::from(height_frame_inner);
            let inner_radius       = &corner_radius * (Var::<f32>::from(1.0) - &selected);
            let frame_inner        = Rect((&width_frame_inner,&height_frame_inner));
            let frame_rounded      = frame_inner.corners_radius(&inner_radius);

            let frame       = frame_outer.difference(frame_rounded);
            let color_frame = color::Lcha::new(0.72,0.5,0.22,1.0);
            let frame       = frame.fill(color::Rgba::from(color_frame));

             let out = background + frame;

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
    pub set_visibility    : frp::Source<bool>,
    pub toggle_visibility : frp::Source,
    pub set_visualization : frp::Source<Option<Visualization>>,
    pub set_data          : frp::Source<Data>,
    pub select            : frp::Source,
    pub deselect          : frp::Source,
    pub set_size          : frp::Source<V2>,
    pub clicked           : frp::Stream,
    on_click              : frp::Source,
}

impl Frp {
    fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def set_visibility    = source();
            def toggle_visibility = source();
            def set_visualization = source();
            def set_data          = source();
            def select            = source();
            def deselect          = source();
            def on_click          = source();
            def set_size          = source();
            let clicked           = on_click.clone_ref().into();
        };
        Self {set_visibility,set_visualization,toggle_visibility,set_data,select,deselect,
              clicked,set_size,on_click}
    }
}



// ==============
// === Shapes ===
// ==============

#[derive(Debug)]
#[allow(missing_docs)]
pub struct Shapes {
    logger         : Logger,
    display_object : display::object::Instance,
    frame          : component::ShapeView<frame::Shape>,
    overlay        : component::ShapeView<overlay::Shape>,
}

impl Shapes {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = logger.sub("shapes");
        let display_object = display::object::Instance::new(&logger);
        let frame          = component::ShapeView::<frame::Shape>::new(&logger,scene);
        let overlay        = component::ShapeView::<overlay::Shape>::new(&logger,scene);
        display_object.add_child(&overlay);
        display_object.add_child(&frame);
        Self {logger,display_object,frame,overlay}
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
    size           : Cell<V2>,
    visualization  : RefCell<Option<Visualization>>,
    scene          : Scene,
    shapes         : Shapes,
}

impl ContainerModel {
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let logger         = logger.sub("visualization_container");
        let display_object = display::object::Instance::new(&logger);
        let visualization  = default();
        let size           = Cell::new(DEFAULT_SIZE);
        let shapes         = Shapes::new(&logger,scene);
        let scene          = scene.clone_ref();
        Self {logger,visualization,size,display_object,shapes,scene} . init()
    }

    fn init(self) -> Self {
        // FIXME: These 2 lines fix a bug with display objects visible on stage.
        self.set_visibility(true);
        self.set_visibility(false);

        self.update_shape_sizes();
        self.set_corner_roundness(1.0);
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
}


// === Private API ===

impl ContainerModel {
    fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    fn set_visualization(&self, visualization:Option<Visualization>) {
        if let Some(visualization) = visualization {
            let size = self.size.get();
            visualization.set_size(size);
            self.shapes.add_child(&visualization);
            self.visualization.replace(Some(visualization));
        }
    }

    fn set_visualization_data(&self, data:&Data) {
        self.visualization.borrow().for_each_ref(|vis| vis.frp.set_data.emit(data))
    }

    fn update_shape_sizes(&self) {
        let size = self.size.get().into();
        self.shapes.frame   . shape.radius.set(CORNER_RADIUS);
        self.shapes.overlay . shape.radius.set(CORNER_RADIUS);
        self.shapes.frame   . shape.sprite.size().set(size);
        self.shapes.overlay . shape.sprite.size().set(size);
    }

    fn set_size(&self, size:Vector3<f32>) {
        if let Some(vis) = self.visualization.borrow().as_ref() {
            let radius = CORNER_RADIUS * 2.0;
            let radius = Vector2::new(radius, radius);
            let vis_size = size.xy() - radius;
            vis.set_size(vis_size.into());
        }

        self.size.set(size.xy().into());
        self.update_shape_sizes();
    }

    fn set_corner_roundness(&self, value:f32) {
        self.shapes.overlay.shape.roundness.set(value);
        self.shapes.frame.shape.roundness.set(value);
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
        let frp     = Frp::new(&network);
        let model   = Rc::new(ContainerModel::new(logger,scene));
        model.set_visualization(Some(Registry::default_visualisation(scene)));
        Self {model,frp,network} . init()
    }

    fn init(self) -> Self {
        let inputs    = &self.frp;
        let network   = &self.network;
        let model     = &self.model;
        let selection = Animation::new(network);
        frp::extend! { network
            eval  selection.value          ((value) model.shapes.frame.shape.selected.set(*value));
            eval  inputs.set_visibility    ((v) model.set_visibility(*v));
            eval_ inputs.toggle_visibility (model.toggle_visibility());
            eval  inputs.set_visualization ((v) model.set_visualization(v.clone()));
            eval  inputs.set_data          ((t) model.set_visualization_data(t));
            eval_ inputs.select            (selection.set_target_value(1.0));
            eval_ inputs.deselect          (selection.set_target_value(0.0));
            eval_ model.shapes.overlay.events.mouse_down (inputs.on_click.emit(()));
        }
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