
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
use ensogl::display::shape;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystemDefinition;
use ensogl::display::world::World;
use ensogl::display::scene::{Scene,MouseTarget};



// ==============
// === Cursor ===
// ==============

ensogl::component! { Cursor
    Definition {}

    Shape (position:Vector2<f32>, selection_size:Vector2<f32>) {
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

impl Cursor {
    pub fn new() -> Self {
        let definition = Definition {};
        Self::create("node",definition).init()
    }

    fn init(self) -> Self {
        self
    }

}

impl MouseTarget for Definition {}
