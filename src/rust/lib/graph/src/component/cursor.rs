
use crate::prelude::*;

use ensogl::control::callback::CallbackMut1;
use ensogl::data::color::Srgba;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::display::{Sprite, Attribute};
use ensogl::math::Vector2;
use ensogl::math::Vector3;
use logger::Logger;
use std::any::TypeId;
use enso_prelude::std_reexports::fmt::{Formatter, Error};
use ensogl::animation::physics::inertia::DynInertiaSimulator;
use enso_frp;
use enso_frp as frp;
use enso_frp::frp;
use enso_frp::core::node::class::EventEmitterPoly;
use ensogl::display::{AnyBuffer,Buffer};
use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystemDefinition;
use ensogl::display::world::World;
use ensogl::gui::component::Component;
use ensogl::display::scene;
use ensogl::display::scene::{Scene,MouseTarget,ShapeRegistry};
use ensogl::display::layout::alignment;
use ensogl::system::web;
use ensogl::control::callback::CallbackHandle;



// ==============
// === Cursor ===
// ==============

pub mod shape {
    use super::*;

    ensogl::shape! {
        (position:Vector2<f32>, selection_size:Vector2<f32>) {
            let radius = 10.px();
            let side   = &radius * 2.0;
            let width  = Var::<Distance<Pixels>>::from("input_selection_size.x");
            let height = Var::<Distance<Pixels>>::from("input_selection_size.y");
            let cursor = Rect((&side + width.abs(),&side + height.abs()))
                .corners_radius(radius)
                .translate((-&width/2.0, -&height/2.0))
                .translate(("input_position.x","input_position.y"))
                .fill(Srgba::new(0.0,0.0,0.0,0.3));
            cursor.into()
        }
    }
}


#[derive(Clone,CloneRef,Debug)]
pub struct Cursor {
    pub logger         : Logger,
    pub display_object : display::object::Node,
    pub shape          : Rc<RefCell<Option<shape::ShapeDefinition>>>,
    pub scene_view     : Rc<RefCell<Option<scene::View>>>,
    pub resize_handle  : Rc<RefCell<Option<CallbackHandle>>>,
}

impl Component for Cursor {
    fn on_view_cons(&self, scene:&Scene, shape_registry:&ShapeRegistry) {
        let shape       = shape_registry.new_instance::<shape::ShapeDefinition>();
        let scene_shape = scene.shape();
        self.display_object.add_child(&shape);
        shape.sprite.size().set(Vector2::new(scene_shape.width(),scene_shape.height()));
        let handle = scene.on_resize(enclose!((shape) move |scene_shape:&web::dom::ShapeData| {
            shape.sprite.size().set(Vector2::new(scene_shape.width(),scene_shape.height()));
        }));
        *self.resize_handle.borrow_mut() = Some(handle);
        *self.shape.borrow_mut() = Some(shape);

        let shape_system = shape_registry.shape_system(PhantomData::<shape::ShapeDefinition>);

        shape_system.shape_system.set_alignment(alignment::HorizontalAlignment::Left, alignment::VerticalAlignment::Bottom);

        let scene_view = scene.views.new();
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene_view.add(&shape_system.shape_system.symbol);
        *self.scene_view.borrow_mut() = Some(scene_view);
    }
}

impl Cursor {
    pub fn new() -> Self {
        let logger         = Logger::new("cursor");
        let display_object = display::object::Node::new(&logger);
        let shape          = default();
        let scene_view     = default();
        let resize_handle  = default();
        Cursor {logger,display_object,shape,scene_view,resize_handle} . component_init()
    }
}

impl MouseTarget for Cursor {}

impl<'t> From<&'t Cursor> for &'t display::object::Node {
    fn from(t:&'t Cursor) -> Self {
        &t.display_object
    }
}
