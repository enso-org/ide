//! This module defines the visualization widgets and related functionality.
//!
//! The overall architecture of visualizations consists of three parts:
//! (1) the `DataRenderer` is a trait that sits at the core of the visualisation system. A
//! `DataRenderer` provides the `display::Object` that shows the actual visualization. It is fed
//! with data and provides updates about its state as well as data output.
//!
//! (2) the `Visualization` wraps the `DataRenderer` and implements the generic tasks that are the
//! same for all visualisations. That is, interfacing with the other UI elements, providing data
//! updates to the `DataRenderer`, and propagating information about the state changes in the
//! `DataRenderer`.
//!
//! (3) the `Container` sits on top of the Visualisation and provides UI elements that facilitate
//! generic interactions, for example, selecting a specific visualisation or setting input data for
//! a `Visualisation`. The `Container` also provides the FRP API that allows internal interaction
//! with the `Visualisation`.
//!
//! In addition this module also contains a `Data` struct that provides a dynamically typed way to
//! handle data for visualisations. This allows the `Visualisation` struct to be without type
//! parameters and simplifies the FRP communication and complexity of the node system.
//!
//! TODO split this into multiple files.
pub mod sample;
pub mod js;

use crate::prelude::*;

use crate::frp;

use ensogl::display;
use serde_json;
use web::StyleSetter;
use ensogl::display::object::traits::*;
use fmt;
use std::any;


// ============================================
// === Wrapper for Visualisation Input Data ===
// ============================================
/// Type indicator
/// TODO[mm] use enso types?
type DataType = any::TypeId;

/// Wrapper for data that can be consumed by a visualisation.
/// TODO[mm] consider static versus dynamic typing for visualizations and data!
#[derive(Clone,CloneRefDebug)]
#[allow(missing_docs)]
pub enum Data {
    JSON   { content : Rc<serde_json::Value> },
    Binary { content : Rc<dyn Any>           },
}

impl Data {
    /// Returns the data as as JSON. If the data cannot be returned as JSON, it will return a
    /// `DataError` instead.
    pub fn as_json(&self) -> Result<Rc<serde_json::Value>, DataError> {
        match &self {
            Data::JSON { content } => Ok(Rc::clone(content)),
            _ => { Err(DataError::InvalidDataType{})  },
        }
    }

    /// Returns the wrapped data in Rust format. If the data cannot be returned as rust datatype, a
    /// `DataError` is returned instead.
    fn as_binary<T:'static>(&self) -> Result<Rc<T>, DataError> {
        match &self {
            Data::JSON { .. } => { Err(DataError::InvalidDataType) },
            Data::Binary { content } => { Rc::clone(content).downcast()
                .or(Err(DataError::InvalidDataType))},
        }
    }
}


/// Indicates a problem with the provided data. That is, the data has the wrong format, or maybe
/// violates some other assumption of the visualization.
#[derive(Copy,Clone,Debug)]
pub enum DataError {
    /// Indicates that that the provided data type does not match the expected data type.
    /// TODO add expected/received data types as internal members.
    InvalidDataType
}



// ====================
// === DataRenderer ===
// ====================

/// At the core of the visualization system sits a `DataRenderer`. The DataRenderer is in charge of
/// producing a `display::Object` that will be shown in the scene. It will create FRP events to
/// indicate updates to its output data (e.g., through user interaction).
///
/// A DataRenderer can indicate what kind of data it can use to create a visualisation through the
/// `valid_input_types` method. This serves as a hint, it will also reject invalid input in the
/// `set_data` method with a `DataError`. The owner of the `DataRenderer` is in charge of producing
/// UI feedback to indicate a problem with the data.
pub trait DataRenderer: display::Object {
    /// Will be called to initialise the renderer and provides the parent FRP to set up event
    /// handling. This is needed so the `DataRenderer` can send events to the outside world,
    /// without the needs for callbacks.
    fn init(&self, frp:&VisualisationFrp);
    /// Indicate which `DataType`s can be rendered by this visualization.
    fn valid_input_types(&self) -> Vec<DataType>;
    /// Set the data that should be rendered. If the data is valid, it will return the data as
    /// processed by this `DataRenderer`, if the data is of an invalid data type, ir violates other
    /// assumptions of this `DataRenderer`, a `DataError` is returned.
    /// TODO[mm] reconsider returning the data here. Maybe just have an FRP event.
    fn set_data(&self, data:Data) -> Result<Data, DataError>;
    /// Set the size of viewport of the visualization. The visualisation must not render outside of
    /// this viewport. TODO[mm] define and ensure consistent origin of viewport.
    fn set_size(&self, size:Vector2<f32>);
}

/// TODO[mm] update this with actual required data for `PreprocessId`
type PreprocessId = String;
/// TODO consider getting rid of all callbacks in favour of FRP events.
type StatusCallback = Rc<dyn Fn()>;
type DataCallback = Rc<dyn Fn(&Data)>;
type PreprocessorCallback = Rc<dyn Fn(Rc<dyn Fn(PreprocessId)>)>;

/// Events that are emited by the visualisation.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct VisualisationFrp {
    pub network           : frp::Network,
    /// Will be emitted if the visualisation state changes (e.g., through UI interaction).
    pub on_change         : frp::Source<Option<Data>>,
}

impl Default for VisualisationFrp {
    fn default() -> Self {
        frp::new_network! { visualization_events
            def on_change    = source::<Option<Data>> ();
        };
        let network = visualization_events;
        Self {network,on_change}
    }
}


/// Inner representation of a visualisation.
#[derive(Clone)]
#[allow(missing_docs)]
pub struct Visualization {
    frp          : VisualisationFrp,
    // TODO[mm] consider whether to use a `Box` and be exclusive owner of the DataRenderer.
    renderer     : Rc<dyn DataRenderer>,
    preprocessor : Option<PreprocessId>,
    on_show      : Option<StatusCallback>,
    on_hide      : Option<StatusCallback>,
    on_change    : Option<DataCallback>,
    on_preprocess_change : Option<PreprocessorCallback>,
}

impl Debug for Visualization {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO[mm] extend to provide actually useful information.
        f.write_str("<Visualisation>")
    }
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

impl Visualization {

    /// Create a new `Visualization` with the given `DataRenderer`.
    pub fn new(renderer:Rc<dyn DataRenderer>) -> Self {
        let preprocessor         = None;
        let on_hide              = None;
        let on_show              = None;
        let on_change            = None;
        let on_preprocess_change = None;
        let frp                  = VisualisationFrp::default();
        Visualization { frp,renderer,preprocessor,on_change,on_preprocess_change,on_hide,on_show}
            .init()
    }

    fn init(self) -> Self {
        self.renderer.init(&self.frp);
        self
    }

    /// Update the visualisation with the given data. Returns an error if the data did not match
    /// the visualization.
    pub fn set_data(&self, data:Data) -> Result<(),DataError> {
        let output_data = self.renderer.set_data(data)?;
        if let Some(callback) = self.on_change.as_ref() {
            callback(&output_data)
        }
        Ok(())
     }

    /// Set the viewport size of the visualisation.
    pub fn set_size(&self, size: Vector2<f32>) {
        self.renderer.set_size(size)
    }
}



// =========================
// === Visualization FRP ===
// =========================

/// Event system of the `Container`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ContainerFrp {
    pub network           : frp::Network,
    pub set_visibility    : frp::Source<bool>,
    pub toggle_visibility : frp::Source,
    pub set_visualization : frp::Source<Option<Visualization>>,
    pub set_data          : frp::Source<Option<Data>>,
}

impl Default for ContainerFrp {
    fn default() -> Self {
        frp::new_network! { visualization_events
            def set_visibility    = source();
            def toggle_visibility = source();
            def set_visualization = source();
            def set_data          = source();
        };
        let network = visualization_events;
        Self {network,set_visibility,set_visualization,toggle_visibility,set_data }
    }
}



// ================================
// === Visualizations Container ===
// ================================

/// Container that wraps a `Visualization` for rendering and interaction in the GUI.
///
/// The API to interact with the visualisation is exposed through the `ContainerFrp`.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Container {
    // The internals are split into two structs: `ContainerData` and `ContainerFrp`. The
    // `ContainerData` contains the actual data and logic for the `Container`. The `ContainerFrp`
    // contains the FRP api and network. This split is required to avoid creating cycles in the FRP
    // network: the FRP network holds `Rc`s to the `ContainerData` and thus must not live in the
    // same struct.

    #[shrinkwrap(main_field)]
        data : Rc<ContainerData>,
    pub frp  : Rc<ContainerFrp>,
}

/// Internal data of a `Container`.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct ContainerData {
    logger        : Logger,
    display_object: display::object::Instance,
    size          : Cell<Vector2<f32>>,
    visualization : RefCell<Option<Visualization>>,
    data          : RefCell<Option<Data>>,
}

impl ContainerData {
    /// Set whether the visualisation should be visible or not.
    pub fn set_visibility(&self, visibility:bool) {
        if let Some(vis) = self.visualization.borrow().as_ref() {
            if visibility { self.add_child(&vis) } else { vis.unset_parent() }
        }
    }

    /// Indicates whether the visualisation is visible.
    pub fn is_visible(&self) -> bool {
        self.visualization.borrow().as_ref().map(|t| t.has_parent()).unwrap_or(false)
    }

    /// Toggle visibility.
    pub fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    /// Update the data in the inner visualisation.
    pub fn set_data(&self, data:Data) {
        self.data.set(data.clone_ref());
        if let Some(vis) = self.visualization.borrow().as_ref() {
            // TODO add indicator that data does not match
            vis.set_data(data).unwrap();
        }
    }

    /// Set the visualization shown in this container.
    pub fn set_visualisation(&self, visualization:Visualization) {
        let size = self.size.get();
        visualization.content.set_size(size);
        self.display_object.add_child(&visualization);
        self.visualization.replace(Some(visualization));
        self.set_visibility(false);
    }
}

impl display::Object for ContainerData {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


impl Container {
    /// Constructor.
    pub fn new() -> Self {
        let logger         = Logger::new("visualization");
        let visualization  = default();
        let size           = Cell::new(Vector2::new(100.0, 100.0));
        let display_object = display::object::Instance::new(&logger);
        let data           = default();

        let data           = ContainerData {logger,visualization,data,size,display_object};
        let data           = Rc::new(data);
        let frp            = default();
        Self {data, frp} . init_frp()
    }

    fn init_frp(self) -> Self {
        let frp     = &self.frp;
        let network = &self.frp.network;

        frp::extend! { network

            let container_data = &self.data;

            def _set_visibility = frp.set_visibility.map(f!((container_data)(is_visible) {
                container_data.set_visibility(*is_visible);
            }));

            def _toggle_visibility = frp.toggle_visibility.map(f!((container_data)(_) {
                container_data.toggle_visibility()
            }));

            def _set_visualization = frp.set_visualization.map(f!((container_data)(visualisation) {
                if let Some(visualisation) = visualisation.as_ref() {
                    container_data.set_visualisation(visualisation.clone_ref());
                }
            }));

            def _set_data = frp.set_data.map(f!((container_data)(data) {
                if let Some(data) = data.as_ref() {
                     container_data.set_data(data.clone_ref());
                }
            }));
        }
        self
    }
}

impl Default for Container {
    fn default() -> Self {
        Container::new()
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.display_object
    }
}
