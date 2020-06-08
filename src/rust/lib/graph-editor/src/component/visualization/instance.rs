//! This module defines the `Renderer` trait and related functionality.

use crate::prelude::*;
use crate::visualization::*;

use crate::frp;

use ensogl::display;
use crate::data::EnsoCode;



// ===========
// === FRP ===
// ===========

/// Inputs of the visualization FRP system. Please note that inputs and outputs are kept in separate
/// structures because the visualization author may want to keep the inputs in a model and allow it
/// to be clone-ref'd into FRP closures. If FRP inputs owned the network, it would cause memory
/// leak.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct FrpInputs {
    pub set_size  : frp::Source<V2>,
    pub send_data : frp::Source<Data>,
}

/// Visualization FRP network.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Frp {
    #[shrinkwrap(main_field)]
    pub inputs                : FrpInputs,
    pub network               : frp::Network,

    pub on_change             : frp::Stream<EnsoCode>,
    pub on_preprocess_change  : frp::Stream<EnsoCode>,
    pub on_data_receive_error : frp::Stream<Option<DataError>>,

    pub data_receive_error    : frp::Source<Option<DataError>>,
    pub change                : frp::Source<EnsoCode>,
    pub preprocess_change     : frp::Source<EnsoCode>,
}

impl FrpInputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            set_size           <- source();
            send_data          <- source();
        };
        Self {set_size,send_data}
    }
}

impl Frp {
    /// Constructor.
    pub fn new() -> Self {
        frp::new_network! { network
            def change             = source();
            def preprocess_change  = source();
            def data_receive_error = source();
        };
        let on_change             = change.clone_ref().into();
        let on_preprocess_change  = preprocess_change.clone_ref().into();
        let on_data_receive_error = data_receive_error.clone_ref().into();
        let inputs                = FrpInputs::new(&network);
        Self {network,on_change,on_preprocess_change,on_data_receive_error,change,preprocess_change
             ,inputs,data_receive_error}
    }
}

impl Default for Frp {
    fn default() -> Self {
        Self::new()
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
