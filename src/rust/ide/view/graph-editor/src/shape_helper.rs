/// This module contains generic helper functionality for dealing with shapes.
/// TODO consider turning this into a crate or move the functionality to a more appropriate place.

use ensogl::data::color;



// =================
// === Constants ===
// =================

pub const HOVER_COLOR : color::Rgba = color::Rgba::new(1.0,0.0,0.0,0.000_001);




// ======================
// === Compound Shape ===
// ======================

/// This module contains functionality for handling multiple shapes that should act like one.
pub mod compound_shape {
    use crate::prelude::*;
    use enso_frp as frp;


    use ensogl::gui::component::ShapeView;

    // FIXME `_dummy` input is needed as the input section can not be omitted.
    ensogl_text::define_endpoints! {
        Input {
            _dummy ()
        }
        Output {
            mouse_over (),
            mouse_out  (),
        }
    }

    /// `Events` defines a common FRP api that handles mouse over/out events for  multiple
    /// sub-shapes. It avoids boilerplate of setting up FRP bindings for every single shape,
    ///instead the `Shape` frp endpoints can be used.
    #[derive(Clone,CloneRef,Debug)]
    pub struct Events {
        pub frp : Frp,
    }

    impl Events {

        fn new() -> Self {
            let frp = Frp::new_network();
            Self{frp}
        }

        /// Connect the given `ShapeViewEvents` to the `Events` output.
        pub fn add_sub_shape<T>(&self, view:&ShapeView<T>) {
            let _network       = &self.frp.network;
            let compound_frp  = &self.frp;
            let sub_frp       = &view.events;

            // TODO avoid extra in/out events when switching shapes
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
}
