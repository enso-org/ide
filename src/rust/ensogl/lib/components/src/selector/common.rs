//! Common functionality for both the Number and Range selector.
use crate::prelude::*;

use enso_frp as frp;
use enso_frp::Network;
use ensogl_core::frp::io::Mouse;
use ensogl_core::gui::component::ShapeViewEvents;

pub mod base_frp;
pub mod model;
pub mod shape;

pub use base_frp::*;
pub use model::*;



// ==============
// === Bounds ===
// ==============

/// Bounds of a selection. This indicates the lowest and highest value that can be selected in a
/// selection component.
pub type Bounds = (f32, f32);

/// Frp utility method to normalise the given value to the given Bounds.
pub fn normalise_value((value,range):&(f32,Bounds)) -> f32 {
    (value - range.0) / (range.1 - range.0)
}

/// Frp utility method to compute the absolute value from a normalised value.
/// Inverse of `normalise_value`.
pub fn absolute_value((range,normalised_value):&(Bounds,f32)) -> f32 {
    ((normalised_value * (range.1 - range.0)) + range.0)
}

/// Returns the normalised value that correspond to the click posiiton on the shape.
/// For use in FRP `map` method, thus taking references.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn position_to_normalised_value(pos:&Vector2,width:&f32) -> f32 {
    ((pos.x / (width/2.0)) + 1.0) / 2.0
}

/// Check whether the given value is within the given bounds. End points are exclusive.
fn value_in_bounds(value:f32, bounds:Bounds) -> bool {
    value > bounds.0 && value < bounds.1
}

/// Check whether the given bounds are completely contained in the second bounds.
pub fn range_in_bounds(range:Bounds, bounds:Bounds) -> bool {
    value_in_bounds(range.0,bounds) && value_in_bounds(range.1,bounds)
}

/// Clamp `value` to the `overflow_bounds`, or to [0, 1] if no bounds are given.
/// For use in FRP `map` method, thus taking references.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn clamp_with_overflow(value:&f32, overflow_bounds:&Option<Bounds>) -> f32 {
    if let Some(overflow_bounds) = overflow_bounds{
        value.clamp(overflow_bounds.0,overflow_bounds.1)
    } else {
        value.clamp(0.0, 1.0)
    }
}

/// Indicates whether the `value` would be clamped when given to `clamp_with_overflow`.
/// For use in FRP `map` method, thus taking references.
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn should_clamp_with_overflow(range:&Bounds, overflow_bounds:&Option<Bounds>) -> bool {
    if let Some(overflow_bounds) = overflow_bounds {
        range_in_bounds(*range,*overflow_bounds)
    } else {
        range_in_bounds(*range,(0.0,1.0))
    }
}



// =============
// === Range ===
// =============

pub type Range = (f32, f32);

/// Return the range with sorted components.
pub fn sorted_range(range:Range) -> Range {
    if range.0 > range.1 {
        (range.1, range.0)
    } else {
        range
    }
}



// =======================
// === Shape Utilities ===
// =======================


/// Return whether a dragging action has been started from the given shape.
/// A dragging action is started by a mouse down on a shape, followed by a movement of the mouse.
/// It is ended by a mouse up.
pub fn shape_is_dragged(network:&Network, shape:&ShapeViewEvents, mouse:&Mouse) -> frp::Stream<bool>  {
    frp::extend! { network
        mouse_up              <- mouse.up.constant(());
        mouse_down            <- mouse.down.constant(());
        over_shape            <- bool(&shape.mouse_out,&shape.mouse_over);
        mouse_down_over_shape <- mouse_down.gate(&over_shape);
        is_dragging_shape     <- bool(&mouse_up,&mouse_down_over_shape);
    }
    is_dragging_shape
}

/// Returns the position of a mouse down on a shape. The position is given relative to the origin
/// of the shape position.
pub fn relative_shape_click_position(model:&Model, network:&Network, shape:&ShapeViewEvents, mouse:&Mouse) -> frp::Stream<Vector2>  {
    let model = model.clone_ref();
    frp::extend! { network
        mouse_down               <- mouse.down.constant(());
        over_shape               <- bool(&shape.mouse_out,&shape.mouse_over);
        mouse_down_over_shape    <- mouse_down.gate(&over_shape);
        background_click_positon <- mouse.position.sample(&mouse_down_over_shape);
        background_click_positon <- background_click_positon.map(move |pos| pos - model.position().xy());
    }
    background_click_positon
}
