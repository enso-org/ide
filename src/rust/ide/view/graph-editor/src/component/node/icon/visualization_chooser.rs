//! Definition of the icon displayed to show the menu to choose the
//! visualisation for a node.


use crate::prelude::*;

use ensogl::display::shape::*;
use ensogl::data::color;



// =======================
// === Node Icon Shape ===
// =======================

ensogl::define_shape_system! {
    () {
        let width            = Var::<Pixels>::from("input_size.x");
        let height           = Var::<Pixels>::from("input_size.y");
        let triangle         = Triangle(width.clone(),height.clone());
        let triangle_down    = triangle.rotate(Var::<f32>::from(std::f32::consts::PI));

        let hover_area       = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));

        let fill_color       = color::Rgba::from(color::Lcha::new(0.8,0.013,0.18,1.0));
        let triangle_colored = triangle_down.fill(fill_color);

        (triangle_colored + hover_area) .into()
    }
}
