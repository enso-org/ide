//! This module defines the visualization widgets and related functionality.
//!
//! At the core of this functionality is the `Visualisation` that takes in data and renders an
//! output visualisation which is displayed in a `Container`. The `Container` provides generic UI
//! elements that facilitate generic interactions, for example, visualisation selection. The
//! `Container` also provides the FRP API that allows internal interaction with the
//! `Visualisation`. Data for a visualisation has to be provided wrapped in the `Data` struct.
pub mod sample;

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
            _ => { Err(DataError::InvalidDataType)  },
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

#[derive(Clone,Debug)]
pub enum DataError {
    InvalidDataType
}

// =============================================
// === Internal Visualisation Representation ===
// =============================================

pub trait DataRenderer: display::Object {
    /// Indicate which DataTypes can be rendered by this visualization.
    fn valid_input_types(&self) -> Vec<DataType>;

    /// Set the data that should be rendered. Returns the data as processed by this visualization.
    /// TODO consider having different input and output types.
    fn set_data(&self, data:Data) -> Result<Data, DataError>;

    /// Set the size of the visualization.
    fn set_size(&self, size:Vector2<f32>);
}

/// TODO check what is required here.
type PreprocessId = String;
type StatusCallback = Rc<dyn Fn()>;
type DataCallback = Rc<dyn Fn(&Data)>;
type PreprocessorCallback = Rc<dyn Fn(Rc<dyn Fn(PreprocessId)>)>;


/// Inner representation of a visualisation.
#[derive(Clone)]
#[allow(missing_docs)]
pub struct Visualization {
    renderer     : Rc<dyn DataRenderer>,
    preprocessor : Option<PreprocessId>,
    on_show      : Option<StatusCallback>,
    on_hide      : Option<StatusCallback>,
    on_change    : Option<DataCallback>,
    on_preprocess_change : Option<PreprocessorCallback>,
}

impl Debug for Visualization {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // unimplemented!()
        Ok(())
    }
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}

impl Visualization {

    pub fn new(renderer:Rc<dyn DataRenderer>) -> Self {
        let preprocessor         = None;
        let on_hide              = None;
        let on_show              = None;
        let on_change            = None;
        let on_preprocess_change = None;
        Visualization { renderer,preprocessor,on_change,on_preprocess_change,on_hide,on_show}
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

    pub fn set_size(&self, size: Vector2<f32>) {
        self.renderer.set_size(size)
    }
}



// =========================
// === Visualization FRP ===
// =========================

/// Visualization events.
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
