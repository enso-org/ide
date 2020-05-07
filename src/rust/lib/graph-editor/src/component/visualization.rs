//! This module defines the visualization widgets and related functionality.
//!
//! At the core of this functionality is the `Visualisation` that takes in data and renders an
//! output visualisation which is displayed in a `Container`. The `Container` provides generic UI
//! elements that facilitate generic interactions, for example, visualisation selection. The
//! `Container` also provides the FRP API that allows internal interaction with the
//! `Visualisation`. Data for a visualisation has to be provided wrapped in the `Data` struct.

use crate::prelude::*;

use crate::frp;

use ensogl::display::DomSymbol;
use ensogl::display;
use ensogl::system::web;
use serde_json;
use web::StyleSetter;



// ============================================
// === Wrapper for Visualisation Input Data ===
// ============================================

/// Wrapper for data that can be consumed by a visualisation.
// TODO replace with better typed data wrapper.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub enum Data {
    JSON { content : Rc<serde_json::Value> },
    Binary,
}

impl Data {
    /// Render the data as JSON.
    pub fn as_json(&self) -> String {
        match &self {
            Data::JSON { content } => content.to_string(),
            _ => "{}".to_string(),
        }
    }
}



// =============================================
// === Internal Visualisation Representation ===
// =============================================

/// Inner representation of a visualisation.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    content : DomSymbol
}

impl display::Object  for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.content.display_object()
    }
}

impl Visualization {
    /// Update the visualisation with the given data.
    // TODO remove dummy functionality and use an actual visualisation
    pub fn set_data(&self, data:Data){
                self.content.dom().set_inner_html(
                    &format!(r#"
<svg>
  <circle style="fill: #69b3a2" stroke="black" cx=50 cy=50 r={}></circle>
</svg>
"#, data.as_json()));
     }

    /// Set whether the visualisation should be visible or not.
    fn set_visibility(&self, is_visible:bool) {
        let visible = if is_visible {"visible" } else { "hidden" };
        self.content.dom().set_style_or_panic("visibility", visible);
    }
}

impl From<DomSymbol> for Visualization {
    fn from(symbol:DomSymbol) -> Self {
        Visualization { content : symbol }
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
            def set_visibility    = source::<bool>                  ();
            def toggle_visibility = source::<()>                    ();
            def set_visualization = source::<Option<Visualization>> ();
            def set_data          = source::<Option<Data>>          ();
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

/// Weak version of `Container`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakContainer {
    data : Weak<ContainerData>,
    frp  : Weak<ContainerFrp>,
}

/// Internal data of a `Container`.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct ContainerData {
    logger         : Logger,
    display_object : display::object::Instance,
    size           : Cell<Vector2<f32>>,
    visualization  : RefCell<Option<Visualization>>,
}

impl ContainerData {
    /// Set whether the visualisation should be visible or not.
    pub fn set_visibility(&self, is_visible:bool) {
        if let Some(vis) = self.visualization.borrow().as_ref() {
            // FIXME remove the `set_visibility` call when the display_object calls are fixed.
            vis.set_visibility(is_visible);
            if is_visible {
                vis.display_object().set_parent(&self.display_object);
            } else {
                vis.display_object().unset_parent();
            }
        }
    }

    /// Indicates whether the visualisation is visible.
    pub fn is_visible(&self) -> bool {
        self.display_object.has_parent()
    }

    /// Toggle visibility.
    pub fn toggle_visibility(&self) {
        self.set_visibility(!self.is_visible())
    }

    /// Update the data in the inner visualisation.
    pub fn set_data(&self, data:Data) {
        self.visualization.borrow().for_each_ref(|vis| vis.set_data(data));
    }

    /// Update the content properties with the values from the `ContainerData`.
    ///
    /// Needs to called when a visualisation has been set.
    fn update_visualisation_properties(&self) {
        let size     = self.size.get();
        let position = self.display_object.position();
        self.visualization.borrow().for_each_ref(|vis| {
            vis.content.set_size(size);
            vis.content.set_position(position);
        });
    }

    /// Set the visualization shown in this container..
    pub fn set_visualisation(&self, visualization:Visualization) {
        self.display_object.add_child(&visualization);
        self.visualization.replace(Some(visualization));
        self.update_visualisation_properties();
    }
}


impl Container {
    /// Constructor.
    pub fn new() -> Self {
        let logger         = Logger::new("visualization_container");
        let visualization  = default();
        let size           = Cell::new(Vector2::new(100.0, 100.0));
        let display_object = display::object::Instance::new(&logger);

        let data = ContainerData {logger,visualization,size,display_object};
        let data = Rc::new(data);

        let frp = default();

        Self {data, frp} . init_frp()
    }

    fn init_frp(self) -> Self {
        let frp     = &self.frp;
        let network = &self.frp.network;

        frp::extend! { network

            let container_data = &self.data;

            def _f_hide = frp.set_visibility.map(f!((container_data)(is_visible) {
                container_data.set_visibility(*is_visible);
            }));

            def _f_toggle = frp.toggle_visibility.map(f!((container_data)(_) {
                container_data.toggle_visibility()
            }));

            def _f_hide = frp.set_visualization.map(f!((container_data)(visualisation) {
                if let Some(visualisation) = visualisation.as_ref() {
                    container_data.set_visualisation(visualisation.clone_ref());
                }
            }));

            def _f_hide = frp.set_data.map(f!((container_data)(data) {
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

impl StrongRef for Container {
    type WeakRef = WeakContainer;
    fn downgrade(&self) -> WeakContainer {
        WeakContainer {data:Rc::downgrade(&self.data),frp:Rc::downgrade(&self.frp)}
    }
}

impl WeakRef for WeakContainer {
    type StrongRef = Container;
    fn upgrade(&self) -> Option<Container> {
        match (self.data.upgrade(),self.frp.upgrade()){
            (Some(data), Some(frp)) => Some(Container {data,frp}),
            _ => None
        }
    }
}

impl display::Object for Container {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.display_object
    }
}

/// Dummy content for testing.
// FIXME[mm] remove this when actual content is available.
pub(crate) fn default_content() -> DomSymbol {
    let div = web::create_div();
    div.set_style_or_panic("width","100px");
    div.set_style_or_panic("height","100px");

    let content = web::create_element("div");
    content.set_inner_html(
        r#"<svg>
  <circle style="fill: #69b3a2" stroke="black" cx=50 cy=50 r=20></circle>
</svg>
"#);
    content.set_attribute("width","100%").unwrap();
    content.set_attribute("height","100%").unwrap();

    div.append_child(&content).unwrap();

    let r          = 102_u8;
    let g          = 153_u8;
    let b          = 194_u8;
    let color      = iformat!("rgb({r},{g},{b})");
    div.set_style_or_panic("background-color",color);

    let symbol = DomSymbol::new(&div);
    symbol.dom().set_attribute("id","vis").unwrap();
    symbol.dom().style().set_property("overflow","hidden").unwrap();
    symbol

}
