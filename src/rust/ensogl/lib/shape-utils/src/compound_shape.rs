//! This module contains functionality for handling multiple shapes that should act like one.

use crate::prelude::*;

use enso_frp as frp;
use ensogl_core::display::shape::primitive::system::Shape;
use ensogl_core::gui::component::ShapeView;


ensogl_core::define_endpoints! {
    Input {}
    Output {
        mouse_over (),
        mouse_out  (),
    }
}

/// `Events` defines a common FRP api that handles mouse over/out events for  multiple
/// sub-shapes. It avoids boilerplate of setting up FRP bindings for every single shape,
/// instead the `Shape` frp endpoints can be used.
#[derive(Clone,CloneRef,Debug)]
pub struct Events {
    /// Frp API.
    pub frp : Frp,
}

impl Events {

    fn new() -> Self {
        let frp = Frp::new_network();
        Self{frp}
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

impl Default for Events {
    fn default() -> Self {
        Events::new()
    }
}
