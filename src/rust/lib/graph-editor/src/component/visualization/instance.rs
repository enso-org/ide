//! This module defines the `Renderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;
use crate::data::EnsoCode;



// =====================
// === Visualization ===
// =====================

/// Internal data of Visualization.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Instance {
    renderer : Rc<dyn InstanceX>,
}

impl Instance {
    /// Constructor.
    pub fn new<T>(renderer:T) -> Self
        where T : 'static + InstanceX {
        let renderer = Rc::new(renderer);
        Self {renderer}
    }
}

impl Deref for Instance {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        self.renderer.frp()
    }
}

impl display::Object for Instance {
    fn display_object(&self) -> &display::object::Instance {
        &self.renderer.display_object()
    }
}



// ================
// === InstanceX ===
// ================

pub trait InstanceX: display::Object + Debug {
    fn frp(&self) -> &Frp;
}



// ===========
// === FRP ===
// ===========

/// FRP API of a `Renderer`.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Frp {
    pub on_change            : frp::Stream<EnsoCode>,
    pub on_preprocess_change : frp::Stream<EnsoCode>,
    pub set_size             : frp::Source<V2>,
    pub send_data            : frp::Source<Data>,

    change            : frp::Source<EnsoCode>,
    preprocess_change : frp::Source<EnsoCode>,
}

impl Frp {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def change            = source();
            def preprocess_change = source();
            def set_size          = source();
            def send_data         = source();
        };
        let on_change            = change.clone_ref().into();
        let on_preprocess_change = preprocess_change.clone_ref().into();
        Self {on_change,on_preprocess_change,change,preprocess_change,set_size,send_data}
    }
}




