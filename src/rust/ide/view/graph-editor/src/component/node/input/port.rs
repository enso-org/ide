use crate::prelude::*;

use ensogl::data::color;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::gui::component;

use crate::node::input::area;



// ==================
// === Port Shape ===
// ==================

/// Port shape definition.
pub mod shape {
    use super::*;
    ensogl::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let shape  = Rect((&width,&height));
            if !area::DEBUG {
                let color = Var::<color::Rgba>::from("srgba(1.0,1.0,1.0,0.00001)");
                shape.fill(color).into()
            } else {
                let shape = shape.corners_radius(6.px());
                let color = Var::<color::Rgba>::from("srgba(1.0,0.0,0.0,0.1)");
                shape.fill(color).into()
            }
        }
    }
}
pub use shape::Shape;

/// Function used to hack depth sorting. To be removed when it will be implemented in core engine.
pub fn depth_sort_hack(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<shape::Shape>::new(&logger,scene);
}



// =============
// === Model ===
// =============

ensogl::define_endpoints! {
    Input {
        set_optional (bool),
        set_disabled (bool),
        set_hover    (bool),
    }

    Output {
        color (color::Lcha)
    }
}

#[derive(Clone,Debug,Default)]
pub struct Model {
    pub frp         : Frp,
    pub shape       : Option<component::ShapeView<Shape>>,
    pub name        : Option<String>,
    pub index       : usize,
    pub local_index : usize,
    pub length      : usize,
    pub color       : color::Animation2,
}

impl Deref for Model {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Model {
    pub fn new() -> Self {
        default()
    }
}
