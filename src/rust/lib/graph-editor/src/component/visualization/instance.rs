//! This module defines the `Renderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;
use crate::data::EnsoCode;



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



// ================
// === Instance ===
// ================

/// Abstraction for any visualization instance.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Instance {
    display_object : display::object::Instance,
    frp            : Frp
}

impl Instance {
    /// Constructor.
    pub fn new(display_object:impl display::Object, frp:impl Into<Frp>) -> Self {
        let display_object = display_object.display_object().clone_ref();
        let frp            = frp.into();
        Self {display_object,frp}
    }
}

impl Deref for Instance {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl display::Object for Instance {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
