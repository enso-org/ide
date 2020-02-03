//! This module defines a set of primitive FRP building blocks.

use crate::prelude::*;

use crate::data::*;
use crate::node::*;
use crate::nodes::inference::*;



// ==============
// === Macros ===
// ==============

/// Generates a ot of boilerplate for a new node definition. It generates the node struct, the
/// constructor, input / output relations, etc. In order to learn more, see the expanded version
/// of this macro used below.
macro_rules! define_node {
    (
        $(#$meta:tt)*
        $name:ident $shape_name:ident [$($poly_input:ident)*] $(-> [$($out:tt)*])?
            { $( $field:ident : $field_type:ty ),* }
    ) => {
        $(#$meta)*
        pub type $name<$($poly_input,)*> = NodeWrapper<$shape_name<$($poly_input,)*>>;

        $(#$meta)*
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub struct $shape_name<$($poly_input:Data,)*> {
            $( $poly_input : Node<$poly_input>,)*
            $( $field      : $field_type ),*
        }

        define_node_output! { $shape_name [$($poly_input)*] $(-> [$($out)*])? }

        impl<$($poly_input:Data,)*>
        KnownEventInput for $shape_name<$($poly_input,)*>
        where ($($poly_input),*) : ContainsEventData,
              SelectEventData<($($poly_input),*)> : Data {
            type EventInput = SelectEventData<($($poly_input),*)>;
        }


        paste::item! {
            impl<$($poly_input:Data,)*> $name<$($poly_input,)*>
            where $shape_name<$($poly_input),*> : KnownOutput,
                  $(Node<$poly_input>           : AddTarget<Self>,)*
                  $(Content<$poly_input>        : Value,)*
            {
                pub fn new_named<Label,$([<T $poly_input>],)*>
                (label:Label, $($poly_input:[<T $poly_input>],)*) -> Self
                where Label               : Into<CowString>,
                      $([<T $poly_input>] : Into<Node<$poly_input>>),*
                {
                    $(let $poly_input = $poly_input.into();)*
                    $(let $field = default();)*
                    let shape  = $shape_name { $($poly_input,)* $($field,)* };
                    let this  = Self::construct(label,shape);
                    {
                        let shape = &this.rc.borrow().shape;
                        $(shape.$poly_input.add_target(&this);)*
                    }
                    this
                }
            }
        }

        impl<$($poly_input:Data),*> HasInputs for $shape_name<$($poly_input),*> {
            fn inputs(&self) -> Vec<AnyNode> {
                vec![$((&self.$poly_input).into()),*]
            }
        }
    }
}

/// Internal utility for the `define_node` macro.
macro_rules! define_node_output {
    ( $shape_name:ident [$($poly_input:ident)*] -> [$($out:tt)*] ) => {
        impl<$($poly_input:Data,)*>
        KnownOutput for $shape_name<$($poly_input,)*>
        where $($out)* : Data {
            type Output = $($out)*;
        }
    };

    ( $($t:tt)* ) => {};
}



// ==============
// === Source ===
// ==============

// === Storage ===

/// Internal source storage accessor.
pub type SourceStorage<T> = <T as KnownSourceStorage>::SourceStorage;

/// Internal source storage type.
pub trait KnownSourceStorage {
    /// The result type.
    type SourceStorage : Default;
}

impl<T>         KnownSourceStorage for EventData   <T> {type SourceStorage = ();}
impl<T:Default> KnownSourceStorage for BehaviorData<T> {type SourceStorage = BehaviorData<T>;}


// === Definition ===

/// Source is a begin point in the FRP network. It is able to emit events or initialize behaviors.
pub type Source<Out> = NodeWrapper<SourceShape<Out>>;

/// Internal definition of the source FRP node.
#[derive(Derivative)]
#[derivative(Default (bound="SourceStorage<Out>:Default"))]
#[derivative(Debug   (bound="SourceStorage<Out>:Debug"))]
pub struct SourceShape<Out:KnownSourceStorage> {
    storage: SourceStorage<Out>
}

impl<Out> KnownOutput for SourceShape<Out>
    where Out : KnownSourceStorage + Data {
    type Output = Out;
}

impl<Out> Source<Out>
    where Out : KnownSourceStorage + Data {
    /// Constructor.
    pub fn new_named<Label:Into<CowString>>(label:Label) -> Self {
        Self::construct(label,default())
    }
}

impl<Out> HasCurrentValue for Source<BehaviorData<Out>>
    where Out : Value {
    fn current_value(&self) -> Out {
        self.rc.borrow().shape.storage.value()
    }
}

impl<Out:KnownSourceStorage> HasInputs for SourceShape<Out> {
    fn inputs(&self) -> Vec<AnyNode> {
        default()
    }
}



// =============
// === Merge ===
// =============

define_node! {
    Merge MergeShape [source1 source2] -> [source1] {}
}

impl<T1:Data,T2:Data> EventConsumer for Merge<T1,T2>
    where MergeShape<T1,T2> : KnownEventInput<EventInput=Output<Self>> {
    fn on_event(&self, event:&Self::EventInput) {
        self.emit_event(event);
    }
}



// ==============
// === Toggle ===
// ==============

define_node! {
    Toggle ToggleShape [source] -> [EventData<bool>] {
        status : Cell<bool>
    }
}

impl<T:Value> EventConsumer for Toggle<EventData<T>> {
    fn on_event(&self, _:&Self::EventInput) {
        let val = !self.rc.borrow().shape.status.get();
        self.rc.borrow().shape.status.set(val);
        self.emit_event(&EventData(val));
    }
}



// ============
// === Hold ===
// ============

define_node! {
    Hold HoldShape [source] -> [BehaviorData<Content<source>>] {
        last_val : RefCell<Content<source>>
    }
}

impl<T:Value> EventConsumer for Hold<EventData<T>> {
    fn on_event(&self, event:&Self::EventInput) {
        *self.rc.borrow().shape.last_val.borrow_mut() = event.value().clone();
    }
}

impl<T> HasCurrentValue for Hold<EventData<T>>
    where T : Value {
    fn current_value(&self) -> T {
        self.rc.borrow().shape.last_val.borrow().clone()
    }
}



// ==============
// === Sample ===
// ==============

define_node! {
    Sample SampleShape [source1 source2] {}
}

impl<In1,In2> KnownOutput for SampleShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In2>;
}

impl<In1,In2> KnownOutput for SampleShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In1>;
}

impl<In1,In2> EventConsumer for Sample<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    fn on_event(&self, _:&Self::EventInput) {
        let value = self.rc.borrow().shape.source1.current_value();
        self.emit_event(&EventData(value));
    }
}

impl<In1,In2> EventConsumer for Sample<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    fn on_event(&self, _:&Self::EventInput) {
        let value = self.rc.borrow().shape.source2.current_value();
        self.emit_event(&EventData(value));
    }
}



// ============
// === Gate ===
// ============

define_node! {
    Gate GateShape [source1 source2] {}
}

impl<In1,In2> KnownOutput for GateShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In1>;
}

impl<In1,In2> KnownOutput for GateShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In2>;
}

impl<In:Value> EventConsumer for Gate<BehaviorData<bool>,EventData<In>> {
    fn on_event(&self, event:&Self::EventInput) {
        let check = self.rc.borrow().shape.source1.current_value();
        if check { self.emit_event(event); }
    }
}



// =================
// === Recursive ===
// =================

pub type Recursive<T> = NodeWrapper<RecursiveShape<T>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct RecursiveShape<T:Data> {
    source : RefCell<Option<Node<T>>>,
}

impl<T:Data> KnownOutput for RecursiveShape<T> {
    type Output = T;
}

impl<T:Data> KnownEventInput for RecursiveShape<T> {
    type EventInput = T;
}


// === Constructor ===

impl<T:Data> Recursive<T> {
    pub fn new_named<Label>(label:Label) -> Self
        where Label : Into<CowString> {
        let source = default();
        Self::construct(label,RecursiveShape{source})
    }

    pub fn initialize<S>(&self, t:S)
        where S       : Into<Node<T>>,
              Node<T> : AddTarget<Self> {
        let node = t.into();
        node.add_target(self);
        self.set_display_id(node.display_id());
        *self.rc.borrow().shape.source.borrow_mut() = Some(node);
    }
}

impl<T:Data> EventConsumer for Recursive<T> {
    fn on_event(&self, event:&T) {
        self.emit_event(event);
    }
}

impl<T:Value> HasCurrentValue for Recursive<BehaviorData<T>> {
    fn current_value(&self) -> T {
        self.rc.borrow().shape.source.borrow().as_ref().unwrap().current_value()
    }
}

impl<T:Data> HasInputs for RecursiveShape<T> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![self.source.borrow().as_ref().unwrap().clone_ref().into()]
    }
}