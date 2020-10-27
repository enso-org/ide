//! This module contains functionality for events from multiple shapes that should act like one.

use crate::prelude::*;

use crate::display::shape::primitive::system::Shape;
use crate::gui::component::ShapeView;

use enso_frp as frp;



// ===========
// === Frp ===
// ===========

crate::define_endpoints! {
    Input {}
    Output {
        mouse_over (),
        mouse_out  (),
    }
}



// ===================
// === MouseEvents ===
// ===================

/// `Events` defines a common FRP api that handles mouse over/out events for  multiple
/// sub-shapes. It avoids boilerplate of setting up FRP bindings for every single shape,
/// instead the `Shape` frp endpoints can be used.
#[derive(Clone,CloneRef,Default,Debug)]
#[allow(missing_docs)]
pub struct MouseEvents {
    pub frp : Frp,
}

impl MouseEvents {
    /// Constructor.
    fn new() -> Self {
        default()
    }

    /// Connect the given `ShapeViewEvents` to the `Events` output.
    pub fn add_sub_shape<T:Shape>(&self, view:&ShapeView<T>) {
        let compound_frp = &self.frp;
        let sub_frp      = &view.events;

        // TODO[mm] avoid extra in/out events when switching shapes
        frp::extend! { network
            compound_frp.source.mouse_over <+ sub_frp.mouse_over;
            compound_frp.source.mouse_out  <+ sub_frp.mouse_out;
        }
    }
}
