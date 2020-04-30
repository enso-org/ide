//! This module defines the visualisation widgets.
use crate::prelude::*;

use crate::frp;

use ensogl::display::DomSymbol;
use ensogl::display::object::class::Object;
use ensogl::display::object::class::ObjectOps;
use ensogl::display;
use ensogl::system::web;
use web::StyleSetter;


// ============================
// === Visualization Events ===
// ============================

/// Content that can be used in a visualisation.
/// TODO extend to enum over different content types.
pub type Content = Option<Rc<DomSymbol>>;

/// Visualization events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network        : frp::Network,
    pub show           : frp::Source,
    pub hide           : frp::Source,
    pub update_content : frp::Source<Option<Rc<DomSymbol>>>,
}

impl Default for Events {
    fn default() -> Self {
        frp::new_network! { visualisation_events
            def show           = source::<()> ();
            def hide           = source::<()> ();
            def update_content = source::<Content> ();
        };
        let network = visualisation_events;
        Self {network,show,hide,update_content}
    }
}



// ======================
// === Visualizations ===
// ======================

/// Visualization definition.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Visualization {
    pub data : Rc<VisualizationData>
}

/// Weak version of `Visualization`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakVisualization {
    data : Weak<VisualizationData>
}

/// Internal data of a `Visualization`.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub struct VisualizationData {
    pub logger : Logger,
    pub events : Events,

    node       : display::object::Instance,
    size       : Cell<Vector2<f32>>,
    position   : Cell<Vector3<f32>>,
    visible    : Cell<bool>,

    content   : RefCell<Content>,
}

impl Visualization {
    /// Constructor.
    pub fn new() -> Self {

        let logger   = Logger::new("visualisation");
        let events   = Events::default();
        // TODO replace with actual content;
        let content  = RefCell::new(None);
        let size     = Cell::new(Vector2::new(100.0, 100.0));
        let position = Cell::new(Vector3::new(  0.0,-110.0, 0.0));
        let visible  = Cell::new(true);
        let node     = display::object::Instance::new(&logger);

        let data     = VisualizationData{logger,events,content,size,position,visible,node};
        let data     = Rc::new(data);
        Self {data} . init_frp()
    }

    /// Dummy content for testing.
    // FIXME remove this when actual content is available.
    pub fn default_content() -> DomSymbol {
        let div = web::create_div();
        div.set_style_or_panic("width","100px");
        div.set_style_or_panic("height","100px");
        div.set_style_or_panic("overflow","hidden");


        let content = web::create_element("div");
        content.set_inner_html(
r#"<svg>
  <circle style="fill: #69b3a2" stroke="black" cx=50 cy=50 r=20></circle>
</svg>"#);
        content.set_attribute("width","100%").unwrap();
        content.set_attribute("height","100%").unwrap();

        div.append_child(&content).unwrap();

        let r          = 102_u8;
        let g          = 153_u8;
        let b          = 194_u8;
        let color      = iformat!("rgb({r},{g},{b})");
        div.set_style_or_panic("background-color",color);

        DomSymbol::new(&div)
    }

    /// Update the content properties with the values from the `VisualizationData`.
    ///
    /// Needs to called when those values change or new content has been set.
    fn set_content_properties(&self) {
        let size       = self.data.size.get();
        let position   = self.data.position.get();

        if let Some(object) = self.data.content.borrow().as_ref() {
            object.set_size(size);
            object.set_position(position);
        };
    }

    /// Get the visualisation content.
    pub fn content(&self) -> Content {
        self.data.content.borrow().clone()
    }

    /// Set the visualisation content.
    pub fn set_content(&self, content: Content) {
        if let Some(content) = content.as_ref(){
            self.display_object().add_child(content.as_ref());
        }
        self.data.content.replace(content);
        self.set_content_properties();
    }

    fn init_frp(self) -> Self {
        let network = &self.data.events.network;

        frp::extend! { network
            let weak_vis = self.downgrade();
            def _f_show = self.data.events.show.map(move |_| {
               if let Some(vis) = weak_vis.upgrade() {
                    vis.set_visibility(true)
               }
            });

            let weak_vis = self.downgrade();
            def _f_hide= self.data.events.hide.map(move |_| {
                if let Some(vis) = weak_vis.upgrade() {
                    vis.set_visibility(false)
               }
            });

            let weak_vis = self.downgrade();
            def _f_hide= self.data.events.update_content.map(move |content| {
                if let Some(vis) = weak_vis.upgrade() {
                    vis.set_content(content.clone());
                }
            });
        }

        self
    }

    /// Toggle visibility on or off.
    pub fn set_visibility(&self, visible: bool) {
        self.data.visible.set(visible)  ;
        let content = self.data.content.borrow();
        /// TODO do something more sensible toi hide the content.
        if let Some(ref content) = content.deref() {
            let dom_element = content.dom();
            if visible {
                dom_element.set_style_or_panic("visibility", "hidden");
            } else {
                dom_element.set_style_or_panic("visibility", "visible");
            }
        }
    }
}

impl Default for Visualization {
    fn default() -> Self {
        Visualization::new()
    }
}

impl StrongRef for Visualization {
    type WeakRef = WeakVisualization;
    fn downgrade(&self) -> WeakVisualization {
        WeakVisualization {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakVisualization{
    type StrongRef = Visualization;
    fn upgrade(&self) -> Option<Visualization> {
        self.data.upgrade().map(|data| Visualization{data})
    }
}

impl Object for Visualization {
    fn display_object(&self) -> &display::object::Instance {
        &self.data.node
    }
}
