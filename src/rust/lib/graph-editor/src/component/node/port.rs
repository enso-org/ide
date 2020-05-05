//! Definition of the Port component.

use crate::prelude::*;

//use crate::component::node::port::Registry;

use enso_frp;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::Attribute;
use ensogl::display::Buffer;
use ensogl::display::Sprite;
use ensogl::display::scene::Scene;
use ensogl::display::scene::ShapeRegistry;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::animation;
use ensogl::gui::component;
use ensogl::display::shape::text::glyph::font::FontRegistry;
use ensogl::display::shape::text::glyph::system::GlyphSystem;



// ============
// === Port ===
// ============

/// Canvas node shape definition.
pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style, selection:f32, creation:f32) {
            let width  : Var<Distance<Pixels>> = "input_size.x".into();
            let height : Var<Distance<Pixels>> = "input_size.y".into();
            let radius = 6.px();
            let shape = Rect((&width,&height)).corners_radius(radius);
            let shape = shape.fill(color::Rgba::from(color::Lcha::new(0.6,0.5,0.76,1.0)));
            shape.into()
        }
    }
}


// ==============
// === Events ===
// ==============

/// Port events.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Events {
    pub network    : frp::Network,
    pub select     : frp::Source,
    pub deselect   : frp::Source,
}



// ============
// === Port ===
// ============

/// Port definition.
#[derive(AsRef,Clone,CloneRef,Debug,Deref)]
pub struct Port {
    data : Rc<PortData>,
}

impl AsRef<Port> for Port {
    fn as_ref(&self) -> &Self {
        self
    }
}

/// Weak version of `Port`.
#[derive(Clone,CloneRef,Debug)]
pub struct WeakPort {
    data : Weak<PortData>
}

impl WeakElement for WeakPort {
    type Strong = Port;

    fn new(view: &Self::Strong) -> Self {
        view.downgrade()
    }

    fn view(&self) -> Option<Self::Strong> {
        self.upgrade()
    }
}

///// Shape view for Port.
//#[derive(Debug,Clone,CloneRef,Copy)]
//pub struct PortView {}
//impl component::ShapeViewDefinition for PortView {
//    type Shape = shape::Shape;
//}

/// Internal data of `Port`
#[derive(Debug)]
#[allow(missing_docs)]
pub struct PortData {
    pub object : display::object::Instance,
    pub logger : Logger,
    pub events : Events,
    pub view   : component::ShapeView<shape::Shape>,
}

impl Port {
    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        frp::new_network! { node_network
            def label    = source::<String> ();
            def select   = source::<()>     ();
            def deselect = source::<()>     ();
        }
        let network = node_network;
        let logger  = Logger::new("node");
        let view    = component::ShapeView::<shape::Shape>::new(&logger,scene);
        let events  = Events {network,select,deselect};
        let object  = display::object::Instance::new(&logger);
        object.add_child(&view.display_object);

        let width = 38.5;
        let height = 20.0;

        view.shape.sprite.size().set(Vector2::new(width,height));
        view.mod_position(|t| t.x += width/2.0 + 85.0);
        view.mod_position(|t| t.y += height/2.0 + 4.0);
        let data    = Rc::new(PortData {object,logger,events,view});

        Self {data}
    }
}

impl StrongRef for Port {
    type WeakRef = WeakPort;
    fn downgrade(&self) -> WeakPort {
        WeakPort {data:Rc::downgrade(&self.data)}
    }
}

impl WeakRef for WeakPort {
    type StrongRef = Port;
    fn upgrade(&self) -> Option<Port> {
        self.data.upgrade().map(|data| Port{data})
    }
}

impl display::Object for Port {
    fn display_object(&self) -> &display::object::Instance {
        &self.object
    }
}

impl display::WeakObject for WeakPort {
    fn try_display_object(&self) -> Option<display::object::Instance> {
        self.upgrade().map(|ref t| t.display_object().clone_ref())
    }
}
