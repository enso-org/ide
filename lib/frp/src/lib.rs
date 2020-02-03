//! This module implements an Functional Reactive Programming system. It is an advanced event
//! handling framework which allows describing events and actions by creating declarative event
//! flow diagrams.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

#![feature(specialization)]
#![feature(trait_alias)]
#![feature(weak_into_raw)]
#![feature(associated_type_defaults)]

pub mod data;
pub mod debug;
pub mod node;

pub use data::*;
pub use debug::*;
pub use node::*;


use basegl_prelude    as prelude;
use basegl_system_web as web;

use crate::prelude::*;

use percent_encoding;
use std::borrow::Cow;







// ===================
// === NodeWrapper ===
// ===================

// === NodeWrapper ===

/// `NodeWrapper` is an outer layer for every FRP node. For example, the `Source<Out>` node is just
/// an alias to `NodeWrapper<SourceShape<Out>>`, where `SourceShape` is it's internal representation.
/// This struct bundles each node with information about target edges. Although the edges are used
/// only to send events, they are bundled to every node type in order to keep the implementation
/// simple.
pub type NodeWrapper<Shape> = NodeWrapperTemplate<Shape,Output<Shape>>;

impl<Shape:KnownOutput> NodeWrapper<Shape> {
    /// Constructor.
    pub fn construct<Label>(label:Label, shape:Shape) -> Self
    where Label : Into<CowString> {
        let data = NodeWrapperTemplateData::construct(label,shape);
        let rc   = Rc::new(RefCell::new(data));
        let this = Self {rc};
        this.set_display_id(this.id());
        this
    }
}

impl<Shape,Out> NodeWrapperTemplate<Shape,Out> {
    /// Sends an event to all the children.
    pub fn emit_event(&self, event:&Out) {
        self.rc.borrow().targets.iter().for_each(|target| {
            target.on_event(event)
        })
    }
}

impl<Shape,T:Value>
HasEventTargets for NodeWrapperTemplate<Shape,EventData<T>> {
    fn add_event_target(&self, target:AnyEventConsumer<EventData<T>>) {
        self.rc.borrow_mut().targets.push(target);
    }
}


// === NodeWrapperTemplate ===

/// Internal representation for `NodeWrapper`.
#[derive(Debug,Derivative)]
#[derivative(Default(bound="Shape:Default"))]
#[derivative(Clone(bound=""))]
pub struct NodeWrapperTemplate<Shape,Out> {
    rc: Rc<RefCell<NodeWrapperTemplateData<Shape,Out>>>
}

impl<Shape,Out> CloneRef for NodeWrapperTemplate<Shape,Out> {}

impl<Shape,Out>
HasId for NodeWrapperTemplate<Shape,Out> {
    fn id(&self) -> usize {
        Rc::downgrade(&self.rc).as_raw() as *const() as usize
    }
}

impl<Shape,Out>
HasDisplayId for NodeWrapperTemplate<Shape,Out> {
    fn display_id     (&self) -> usize  { self.rc.borrow().display_id }
    fn set_display_id (&self, id:usize) { self.rc.borrow_mut().display_id = id; }
}

impl<Shape,Out:Data>
KnownOutput for NodeWrapperTemplate<Shape,Out> {
    type Output = Out;
}

impl<Shape,Out>
KnownEventInput for NodeWrapperTemplate<Shape,Out>
where Shape:KnownEventInput, EventInput<Shape>:Data {
    type EventInput = EventInput<Shape>;
}

impl<Shape,T:Value>
EventEmitter for NodeWrapperTemplate<Shape,EventData<T>> {
    fn emit(&self, event:&Self::Output) {
        self.emit_event(event);
    }
}

impl<Shape:HasInputs,Out>
HasInputs for NodeWrapperTemplate<Shape,Out> {
    fn inputs(&self) -> Vec<AnyNode> {
        self.rc.borrow().shape.inputs()
    }
}

impl<Shape,Out>
HasLabel for NodeWrapperTemplate<Shape,Out> {
    fn label(&self) -> CowString {
        self.rc.borrow().label.clone()
    }
}

impl<Shape:HasInputs,Out>
GraphvizBuilder for NodeWrapperTemplate<Shape,Out> {
    fn graphviz_build(&self, builder:&mut Graphviz) {
        let type_name  = base_type_name::<Shape>();
        let label      = &self.rc.borrow().label;
        let id         = self.id();
        let display_id = self.display_id();
        if !builder.contains(id) {
            builder.add_node(id,display_id,type_name,label);
            for input in &self.rc.borrow().shape.inputs() {
                let input_id         = input.id();
                let input_display_id = input.display_id();
                let is_redirect      = input_id != input_display_id;
                let input_type       = input.output_type();
                let input_type_name  = input.output_type_value_name();
                input.graphviz_build(builder);
                builder.add_link(input_display_id,display_id,input_type,&input_type_name);
            }
        }
    }
}


// === NodeWrapperTemplateData ===

/// Internal representation for `NodeWrapperTemplate`.
#[derive(Debug,Derivative)]
#[derivative(Default(bound="Shape:Default"))]
pub struct NodeWrapperTemplateData<Shape,Out> {
    label      : CowString,
    display_id : usize,
    shape      : Shape,
    targets    : Vec<AnyEventConsumer<Out>>,
}

impl<Shape,Out> NodeWrapperTemplateData<Shape,Out> {
    /// Constructor.
    pub fn construct<Label>(label:Label, shape:Shape) -> Self
        where Label : Into<CowString> {
        let label      = label.into();
        let targets    = default();
        let display_id = 0;
        Self {label,display_id,shape,targets}
    }
}


// === Utils ===

fn base_type_name<T>() -> String {
    let qual_name = type_name::<T>();
    let base_name = qual_name.split("<").collect::<Vec<_>>()[0];
    let name      = base_name.rsplit("::").collect::<Vec<_>>()[0];
    let name      = name.split("Shape").collect::<Vec<_>>()[0];
    name.into()
}







// =========================
// === Inference Helpers ===
// =========================

/// Data product type-level inference guidance.
pub trait Infer<T> {
    /// Inference results.
    type Result;
}

/// Accessor for inferred type.
pub type Inferred<T,X> = <X as Infer<T>>::Result;


// === Rules ===

macro_rules! inference_rules {
    ($( $pat:tt => $result:ident )*) => {$(
        inference_rule! { $pat => $result }
    )*}

}

macro_rules! inference_rule {
    ( $t1:ident => $result:ident ) => {
        impl<X,T1> Infer <$t1<T1>> for X { type Result = $result<X>; }
    };

    ( ($t1:ident) => $result:ident ) => {
        impl<X,T1> Infer <$t1<T1>> for X { type Result = $result<X>; }
    };

    ( ($t1:ident, $t2:ident) => $result:ident ) => {
        impl<X,T1,T2> Infer <($t1<T1>,$t2<T2>)> for X { type Result = $result<X>; }
    };

    ( ($t1:ident, $t2:ident, $t3:ident) => $result:ident ) => {
        impl<X,T1,T2,T3> Infer <($t1<T1>,$t2<T2>,$t3<T3>)> for X { type Result = $result<X>; }
    };
}

inference_rules! {
    EventData    => EventData
    BehaviorData => BehaviorData

    (EventData    , EventData   ) => EventData
    (BehaviorData , EventData   ) => EventData
    (EventData    , BehaviorData) => EventData
    (BehaviorData , BehaviorData) => EventData
}



// =========================
// === ContainsEventData ===
// =========================

pub trait ContainsEventData {
    type Result : Data;
}

pub type SelectEventData<T> = <T as ContainsEventData>::Result;

impl<T1> ContainsEventData for EventData<T1>
where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2> ContainsEventData for (EventData<T1>,EventData<T2>)
where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2> ContainsEventData for (EventData<T1>,BehaviorData<T2>)
where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2> ContainsEventData for (BehaviorData<T1>,EventData<T2>)
where EventData<T2> : Data {
    type Result = EventData<T2>;
}

impl<T1,T2,T3> ContainsEventData for (EventData<T1>,BehaviorData<T2>,BehaviorData<T3>)
where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2,T3> ContainsEventData for (BehaviorData<T1>,EventData<T2>,BehaviorData<T3>)
where EventData<T2> : Data {
    type Result = EventData<T2>;
}

impl<T1,T2,T3> ContainsEventData for (BehaviorData<T1>,BehaviorData<T2>,EventData<T3>)
where EventData<T3> : Data {
    type Result = EventData<T3>;
}

impl<T1,T2,T3> ContainsEventData for (EventData<T1>,EventData<T2>,BehaviorData<T3>)
where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2,T3> ContainsEventData for (EventData<T1>,BehaviorData<T2>,EventData<T3>)
    where EventData<T1> : Data {
    type Result = EventData<T1>;
}

impl<T1,T2,T3> ContainsEventData for (BehaviorData<T1>,EventData<T2>,EventData<T3>)
where EventData<T2> : Data {
    type Result = EventData<T2>;
}

impl<T1,T2,T3> ContainsEventData for (EventData<T1>,EventData<T2>,EventData<T3>)
where EventData<T1> : Data {
    type Result = EventData<T1>;
}


// =================================================================================================
// === FRP Nodes ===================================================================================
// =================================================================================================

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
type Source<Out> = NodeWrapper<SourceShape<Out>>;

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



macro_rules! define_X_node {
    (
        $(#$meta:tt)*
        pub struct $name:ident $shape_name:ident [$($poly_input:ident)*]
            { $( $field:ident : $field_type:ty ),* }
    ) => {
        $(#$meta)*
        pub type $name<$($poly_input,)* Out> = NodeWrapper<$shape_name<$($poly_input,)* Out>>;

        $(#$meta)*
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub struct $shape_name<$($poly_input:Data,)* Out:Data> {
            $( $poly_input : Node<$poly_input> ),* ,
            $( $field      : $field_type ),*
        }

        impl<$($poly_input:Data,)* Out:Data>
        KnownOutput for $shape_name<$($poly_input,)* Out> {
            type Output = Out;
        }

        impl<$($poly_input:Data,)* Out:Data>
        KnownEventInput for $shape_name<$($poly_input,)* Out>
        where ($($poly_input),*) : ContainsEventData,
              SelectEventData<($($poly_input),*)> : Data {
            type EventInput = SelectEventData<($($poly_input),*)>;
        }
    }
}


macro_rules! define_node {
    (
        $(#$meta:tt)*
        $name:ident $shape_name:ident [$($poly_input:ident)*] -> [$($out:tt)*]
            { $( $field:ident : $field_type:ty ),* }
    ) => {
        $(#$meta)*
        pub type $name<$($poly_input,)*> = NodeWrapper<$shape_name<$($poly_input,)*>>;

        $(#$meta)*
        #[derive(Debug)]
        #[allow(non_camel_case_types)]
        pub struct $shape_name<$($poly_input:Data,)*> {
            $( $poly_input : Node<$poly_input> ),* ,
            $( $field      : $field_type ),*
        }

        impl<$($poly_input:Data,)*>
        KnownOutput for $shape_name<$($poly_input,)*> {
            type Output = $($out)*;
        }

        impl<$($poly_input:Data,)*>
        KnownEventInput for $shape_name<$($poly_input,)*>
        where ($($poly_input),*) : ContainsEventData,
              SelectEventData<($($poly_input),*)> : Data {
            type EventInput = SelectEventData<($($poly_input),*)>;
        }

//        impl<$($poly_input:Data,)*> $name<$($poly_input,)*>
//        where Self:KnownOutput , $(Node<$poly_input> : AddTarget<Self>,)* {
//            fn new_named<Label,$($poly_input:Data,)*>
//            (label:Label, $($poly_input:$poly_input,)*) -> Self {
//                let $poly_input = $poly_input.into();
//                let input_ref   = $poly_input.clone();
//            }
//        }

    }
}



// =============
// === Merge ===
// =============

define_node! {
    Merge MergeShape [source1 source2] -> [source1] {}
}

//pub type Merge<T1,T2> = NodeWrapper<MergeShape<T1,T2>>;
//
//#[derive(Debug)]
//#[allow(non_camel_case_types)]
//pub struct MergeShape<T1:Data,T2:Data> {
//    source1 : Node<T1>,
//    source2 : Node<T2>,
//}

//impl<T:Data> KnownOutput     for MergeShape<T,T> { type Output     = T; }
//impl<T:Data> KnownEventInput for MergeShape<T,T> { type EventInput = T; }


// === Constructor ===

impl<T1:Data,T2:Data> Merge<T1,T2>
where Node<T1> : AddTarget<Self>, Node<T2> : AddTarget<Self>, MergeShape<T1,T2>: KnownOutput {
    fn new_named<Label,Source1,Source2>(label:Label, source1:Source1, source2:Source2) -> Self
        where Label   : Into<CowString>,
              Source1 : Into<Node<T1>>,
              Source2 : Into<Node<T2>> {
        let source1     = source1.into();
        let source2     = source2.into();
        let source1_ref = source1.clone();
        let source2_ref = source2.clone();
        let this        = Self::construct(label,MergeShape{source1,source2});
        source1_ref.add_target(&this);
        source2_ref.add_target(&this);
        this
    }
}

impl<T1:Data,T2:Data> EventConsumer for Merge<T1,T2>
where MergeShape<T1,T2> : KnownEventInput<EventInput=Output<Self>>
{
    fn on_event(&self, event:&Self::EventInput) {
        self.emit_event(event);
    }
}

//pub trait EventConsumer: KnownEventInput + Debug {
//    /// Function called on every new received event.
//    fn on_event(&self, input:&Self::EventInput);
//}

impl<T:Data> HasInputs for MergeShape<T,T> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source1).into(), (&self.source2).into()]
    }
}



// ==============
// === Toggle ===
// ==============

pub type Toggle<T> = NodeWrapper<ToggleShape<T>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct ToggleShape<T:Data> {
    source : Node<T>,
    status : Cell<bool>,
}

impl<T:Data> KnownOutput     for ToggleShape<T> { type Output     = EventData<bool>; }
impl<T:Data> KnownEventInput for ToggleShape<T> { type EventInput = T; }


// === Constructor ===

impl<T:Value> Toggle<EventData<T>> {
    fn new_named<Label,Source> (label:Label, source:Source) -> Self
        where Label  : Into<CowString>,
              Source : Into<Event<T>> {
        let status     = default();
        let source     = source.into();
        let source_ref = source.clone();
        let this       = Self::construct(label,ToggleShape{source,status});
        source_ref.add_target(&this);
        this
    }
}

impl<T:Value> EventConsumer for Toggle<EventData<T>> {
    fn on_event(&self, _:&Self::EventInput) {
        let val = !self.rc.borrow().shape.status.get();
        self.rc.borrow().shape.status.set(val);
        self.emit_event(&EventData(val));
    }
}

impl<T:Data> HasInputs for ToggleShape<T> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source).into()]
    }
}



// ============
// === Hold ===
// ============

pub type Hold<T> = NodeWrapper<HoldShape<T>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct HoldShape<T:Data> {
    source   : Node<T>,
    last_val : RefCell<Content<T>>,
}

impl<T:Value> KnownOutput for HoldShape<EventData<T>> {
    type Output = BehaviorData<T>;
}
impl<T:Data> KnownEventInput for HoldShape<T> { type EventInput = T; }


// === Constructor ===

impl<T:Value> Hold<EventData<T>>
    where Node<EventData<T>> : AddTarget<Self> {
    fn new_named<Label,Source>(label:Label, source:Source) -> Self
        where Label  : Into<CowString>,
              Source : Into<Node<EventData<T>>> {
        let last_val   = default();
        let source     = source.into();
        let source_ref = source.clone();
        let this       = Self::construct(label,HoldShape{source,last_val});
        source_ref.add_target(&this);
        this
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

impl<T:Data> HasInputs for HoldShape<T> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source).into()]
    }
}



// =================
// === Recursive ===
// =================

pub type Recursive<T> = NodeWrapper<RecursiveShape2<T>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct RecursiveShape2<T:Data> {
    source : RefCell<Option<Node<T>>>,
}

impl<T:Data> KnownOutput for RecursiveShape2<T> {
    type Output = T;
}

impl<T:Data> KnownEventInput for RecursiveShape2<T> {
    type EventInput = T;
}


// === Constructor ===

impl<T:Data> Recursive<T> {
    pub fn new_named<Label>(label:Label) -> Self
        where Label : Into<CowString> {
        let source = default();
        Self::construct(label,RecursiveShape2{source})
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

impl<T:Data> HasInputs for RecursiveShape2<T> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![self.source.borrow().as_ref().unwrap().clone_ref().into()]
    }
}




// ==============
// === Sample ===
// ==============

pub type Sample<In1,In2> = NodeWrapper<SampleShape<In1,In2>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct SampleShape<In1:Data,In2:Data> {
    source1 : Node<In1>,
    source2 : Node<In2>,
}

impl<In1,In2> KnownOutput for SampleShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In2>;
}

impl<In1,In2> KnownOutput for SampleShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In1>;
}

impl<In1,In2> KnownEventInput for SampleShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type EventInput = EventData<In1>;
}

impl<In1,In2> KnownEventInput for SampleShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type EventInput = EventData<In2>;
}


// === Constructor ===

impl<In1:Data, In2:Data> Sample<In1,In2>
    where Node<In1>            : AddTarget<Self>,
          Node<In2>            : AddTarget<Self>,
          SampleShape<In1,In2> : KnownOutput {
    fn new_named<Label,Source1,Source2> (label:Label, source1:Source1, source2:Source2) -> Self
        where Label   : Into<CowString>,
              Source1 : Into<Node<In1>>,
              Source2 : Into<Node<In2>> {
        let source1     = source1.into();
        let source2     = source2.into();
        let source1_ref = source1.clone();
        let source2_ref = source2.clone();
        let this        = Self::construct(label,SampleShape{source1,source2});
        source1_ref.add_target(&this);
        source2_ref.add_target(&this);
        this
    }
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

impl<In1:Data, In2:Data> HasInputs for SampleShape<In1,In2> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source1).into(), (&self.source2).into()]
    }
}



// ============
// === Gate ===
// ============

pub type Gate<In1,In2> = NodeWrapper<GateShape<In1,In2>>;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub struct GateShape<In1:Data,In2:Data> {
    source1 : Node<In1>,
    source2 : Node<In2>,
}

impl<In1,In2> KnownOutput for GateShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In1>;
}

impl<In1,In2> KnownOutput for GateShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type Output = EventData<In2>;
}

impl<In1,In2> KnownEventInput for GateShape<EventData<In1>,BehaviorData<In2>>
    where In1:Value, In2:Value {
    type EventInput = EventData<In1>;
}

impl<In1,In2> KnownEventInput for GateShape<BehaviorData<In1>,EventData<In2>>
    where In1:Value, In2:Value {
    type EventInput = EventData<In2>;
}


// === Constructor ===

impl<In2:Value> Gate<BehaviorData<bool>,EventData<In2>> {
    fn new_named<Label,Source1,Source2> (label:Label, source1:Source1, source2:Source2) -> Self
        where Label   : Into<CowString>,
              Source1 : Into<Behavior<bool>>,
              Source2 : Into<Event<In2>> {
        let source1     = source1.into();
        let source2     = source2.into();
        let source1_ref = source1.clone();
        let source2_ref = source2.clone();
        let this        = Self::construct(label,GateShape{source1,source2});
        source1_ref.add_target(&this);
        source2_ref.add_target(&this);
        this
    }
}

impl<In:Value> EventConsumer for Gate<BehaviorData<bool>,EventData<In>> {
    fn on_event(&self, event:&Self::EventInput) {
        let check = self.rc.borrow().shape.source1.current_value();
        if check {
            self.emit_event(event);
        }
    }
}

impl<In1:Data, In2:Data> HasInputs for GateShape<In1,In2> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source1).into(), (&self.source2).into()]
    }
}



// ==============
// === Lambda ===
// ==============

define_X_node! {
    /// Transforms input data with the provided function. Lambda accepts a single input and outputs
    /// message of the same type as the input message.
    pub struct Lambda LambdaShape [source] {
        func : Lambda1Func<source,Out>
    }
}


// === LambdaFunc ===

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Lambda1Func<In1:Data,Out:Data> {
    #[derivative(Debug="ignore")]
    raw : Rc<dyn Fn(&Content<In1>) -> Out>
}

impl<In1,Out,Func> From<Func> for Lambda1Func<In1,Out>
    where In1  : Data,
          Out  : Data,
          Func : 'static + Fn(&Content<In1>) -> Content<Out> {
    fn from(func:Func) -> Self {
        let raw = Rc::new(move |a:&Content<In1>| { wrap(func(a)) });
        Self {raw}
    }
}


// === Constructor ===

/// Constructor abstraction. Used only to satisfy Rust type system.
pub trait LambdaNew<Source,Func> {
    /// Constructor.
    fn new_named<Label:Into<CowString>>(label:Label, source:Source,f:Func) -> Self;
}

impl<In,OutVal,Func,Source> LambdaNew<Source,Func> for Lambda<In,Inferred<In,OutVal>>
    where In       : Data,
          OutVal   : Infer<In>,
          Func     : 'static + Fn(&Content<In>) -> OutVal,
          Source   : Into<Node<In>>,
          Node<In> : AddTarget<Self>,
          Inferred<In,OutVal> : Data<Content=OutVal> {
    fn new_named<Label>(label:Label, source:Source, func:Func) -> Self
        where Label : Into<CowString> {
        let source     = source.into();
        let source_ref = source.clone();
        let func       = func.into();
        let this       = Self::construct(label,LambdaShape{source,func});
        source_ref.add_target(&this);
        this
    }
}

impl<In:Value,Out:Data> EventConsumer for Lambda<EventData<In>,Out> {
    fn on_event(&self, input:&Self::EventInput) {
        let output = (self.rc.borrow().shape.func.raw)(unwrap(input));
        self.emit_event(&output);
    }
}


pub fn trace<T,Label,Source>(label:Label, source:Source) -> Lambda<T,T>
    where T          : Data,
          Label      : Str,
          Source     : Into<Node<T>>,
          Content<T> : Value + Infer<T,Result=T>,
          Node<T>    : AddTarget<Lambda<T,T>> {
    let label = label.into();
    Lambda::new_named("trace",source, move |t| {
        println!("TRACE [{}]: {:?}", label, t);
        t.clone()
    })
}

impl<In1:Data, Out:Data> HasInputs for LambdaShape<In1,Out> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source).into()]
    }
}



// ===============
// === Lambda2 ===
// ===============

define_X_node! {
    /// Transforms input data with the provided function. `Lambda2` accepts two inputs. If at least
    /// one of the inputs was event, the output message will be event as well. In case both inputs
    /// were behavior, a new behavior will be produced.
    pub struct Lambda2 Lambda2Shape [source1 source2] {
        func : Lambda2Func<source1,source2,Out>
    }
}


// === LambdaFunc ===

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Lambda2Func<In1:Data,In2:Data,Out:Data> {
    #[derivative(Debug="ignore")]
    raw : Rc<dyn Fn(&Content<In1>,&Content<In2>) -> Out>
}

impl<In1,In2,Out,Func> From<Func> for Lambda2Func<In1,In2,Out>
    where In1  : Data,
          In2  : Data,
          Out  : Data,
          Func : 'static + Fn(&Content<In1>,&Content<In2>) -> Content<Out> {
    fn from(func:Func) -> Self {
        let raw = Rc::new(move |a:&Content<In1>,b:&Content<In2>| { wrap(func(a,b)) });
        Self {raw}
    }
}


// === Construction ===

/// Constructor abstraction. Used only to satisfy Rust type system.
pub trait Lambda2New<Source1,Source2,Function> {
    /// Constructor.
    fn new_named<Label>(label:Label, source:Source1, source2:Source2, f:Function) -> Self
        where Label : Into<CowString>;
}

impl<In1,In2,OutVal,Source1,Source2,Function>
Lambda2New<Source1,Source2,Function> for Lambda2<In1,In2,Inferred<(In1,In2),OutVal>>
    where In1       : Data,
          In2       : Data,
          OutVal    : Infer<(In1,In2)>,
          Source1   : Into<Node<In1>>,
          Source2   : Into<Node<In2>>,
          Function  : 'static + Fn(&Content<In1>,&Content<In2>) -> OutVal,
          Node<In1> : AddTarget<Self>,
          Node<In2> : AddTarget<Self>,
          Inferred<(In1,In2),OutVal> : Data<Content=OutVal> {
    fn new_named<Label>(label:Label, source1:Source1, source2:Source2, func:Function) -> Self
        where Label : Into<CowString> {
        let source1     = source1.into();
        let source2     = source2.into();
        let source1_ref = source1.clone();
        let source2_ref = source2.clone();
        let func        = func.into();
        let this        = Self::construct(label,Lambda2Shape{source1,source2,func});
        source1_ref.add_target(&this);
        source2_ref.add_target(&this);
        this
    }
}

impl<In1,In2,Out> EventConsumer for Lambda2<EventData<In1>,BehaviorData<In2>,Out>
    where In1:Value, In2:Value, Out:Data {
    fn on_event(&self, event:&Self::EventInput) {
        let value2 = self.rc.borrow().shape.source2.current_value();
        let output = (self.rc.borrow().shape.func.raw)(&event.value(),&value2);
        self.emit_event(&output);
    }
}

impl<In1,In2,Out> EventConsumer for Lambda2<BehaviorData<In1>,EventData<In2>,Out>
    where In1:Value, In2:Value, Out:Data {
    fn on_event(&self, event:&Self::EventInput) {
        let value1 = self.rc.borrow().shape.source1.current_value();
        let output = (self.rc.borrow().shape.func.raw)(&value1,&event.value());
        self.emit_event(&output);
    }
}

impl<In1:Data, In2:Data, Out:Data> HasInputs for Lambda2Shape<In1,In2,Out> {
    fn inputs(&self) -> Vec<AnyNode> {
        vec![(&self.source1).into(),(&self.source2).into()]
    }
}



// =================================================================================================
// === Examples ====================================================================================
// =================================================================================================

macro_rules! frp {
    ( $( $var:ident = $node:ident $(<$ty:ty>)*   ($($args:tt)*); )* ) => {$(
        let $var = $node $(::<$ty>)* :: new_named(stringify!{$var}, $($args)* );
    )*}
}


pub struct Dynamic<Out:Value> {
    pub event    : Event    <Out>,
    pub behavior : Behavior <Out>,
}

impl<Out:Value> Dynamic<Out> {
    pub fn new<E,B>(event:E, behavior:B) -> Self
        where E:Into<Event<Out>>, B:Into<Behavior<Out>> {
        let event    = event.into();
        let behavior = behavior.into();
        Self {event,behavior}
    }

    pub fn merge<Label>(&self, label:Label, that:&Dynamic<Out>) -> Self
        where Label:Into<CowString> {
        (&Merge::new_named(label,self,that)).into()
    }

    pub fn toggle<Label>(&self, label:Label) -> Dynamic<bool>
        where Label:Into<CowString> {
        (&Toggle::new_named(label,self)).into()
    }

    pub fn gate<Label>(&self, label:Label, that:&Dynamic<bool>) -> Self
        where Label:Into<CowString> {
        (&Gate::new_named(label,that,self)).into()
    }

    pub fn sample<Label,T>(&self, label:Label, that:&Dynamic<T>) -> Self
        where Label : Into<CowString>,
              T     : Value {
        (&Sample::new_named(label,&self.behavior,that)).into()
    }

    pub fn map<Label,F,R>(&self, label:Label, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              R     : Value,
              F     : 'static + Fn(&Out) -> R {
        (&Lambda::new_named(label,&self.event,f)).into()
    }

    pub fn map2<Label,T,F,R>(&self, label:Label, that:&Dynamic<T>, f:F) -> Dynamic<R>
        where Label : Into<CowString>,
              T     : Value,
              R     : Value,
              F     : 'static + Fn(&Out,&T) -> R {
        (&Lambda2::new_named(label,&self.event,that,f)).into()
    }

    pub fn constant<Label,T>(&self, label:Label, value:T) -> Dynamic<T>
        where Label:Into<CowString>, T:Value {
        self.map(label,move |_| value.clone())
    }
}

impl<Out:Value> Dynamic<Out> {
    pub fn source<Label>(label:Label) -> Self
        where Label : Into<CowString> {
        let event = Source::<EventData<Out>>::new_named(label);
        (&event).into()
    }
}


impl<Out:Value, T:Into<Event<Out>>> From<T> for Dynamic<Out> {
    fn from(t:T) -> Self {
        let event    = t.into();
        let behavior = Hold :: new_named(event.label(),&event);
        behavior.set_display_id(event.display_id());
        let event    = (&event).into();
        let behavior = (&behavior).into();
        Dynamic {event,behavior}
    }
}

impl<Out:Value> From<&Dynamic<Out>> for Event<Out> {
    fn from(t:&Dynamic<Out>) -> Self {
        t.event.clone_ref()
    }
}

impl<Out:Value> From<&Dynamic<Out>> for Behavior<Out> {
    fn from(t:&Dynamic<Out>) -> Self {
        t.behavior.clone_ref()
    }
}


//use std::concat;


//#[allow(missing_docs)]
//mod tests {
//    use super::*;
//
//    use crate::system::web;
//    use crate::control::io::mouse2;
//    use crate::control::io::mouse2::MouseManager;
//
//
//    // ================
//    // === Position ===
//    // ================
//
//    #[derive(Clone,Copy,Debug,Default)]
//    pub struct Position {
//        x:i32,
//        y:i32,
//    }
//
//    impl Position {
//        pub fn new(x:i32, y:i32) -> Self {
//            Self {x,y}
//        }
//    }
//
//    impl std::ops::Sub<&Position> for &Position {
//        type Output = Position;
//        fn sub(self, rhs: &Position) -> Self::Output {
//            let x = self.x - rhs.x;
//            let y = self.y - rhs.y;
//            Position {x,y}
//        }
//    }
//
//
//    macro_rules! frp_def {
//        ($var:ident = $fn:ident $(.$fn2:ident)* $(::<$ty:ty>)? ($($args:tt)*)) => {
//            let $var = Dynamic $(::<$ty>)? :: $fn $(.$fn2)*
//            ( concat! {stringify!{$var}}, $($args)* );
//        };
//
//        ($scope:ident . $var:ident = $fn:ident $(::<$ty:ty>)? ($($args:tt)*)) => {
//            let $var = Dynamic $(::<$ty>)? :: $fn
//            ( concat! {stringify!{$scope},".",stringify!{$var}}, $($args)* );
//        };
//
//        ($scope:ident . $var:ident = $fn1:ident . $fn2:ident $(.$fn3:ident)* $(::<$ty:ty>)? ($($args:tt)*)) => {
//            let $var = $fn1 . $fn2 $(.$fn3)* $(::<$ty>)?
//            ( concat! {stringify!{$scope},".",stringify!{$var}}, $($args)* );
//        };
//    }
//
//    // ============
//    // === Test ===
//    // ============
//
//    pub struct Mouse {
//        pub up       : Dynamic<()>,
//        pub down     : Dynamic<()>,
//        pub is_down  : Dynamic<bool>,
//        pub position : Dynamic<Position>,
//    }
//
//    impl Mouse {
//        pub fn new() -> Self {
//            frp_def! { mouse.up        = source() }
//            frp_def! { mouse.down      = source() }
//            frp_def! { mouse.position  = source() }
//            frp_def! { mouse.down_bool = down.constant(true) }
//            frp_def! { mouse.up_bool   = up.constant(false) }
//            frp_def! { mouse.is_down   = down_bool.merge(&up_bool) }
//            Self {up,down,is_down,position}
//        }
//    }
//
//    #[allow(unused_variables)]
//    pub fn test (callback: Box<dyn Fn(f32,f32)>) -> MouseManager {
//
//        let document        = web::document().unwrap();
//        let mouse_manager   = MouseManager::new(&document);
//
//
//
//        println!("\n\n\n--- FRP ---\n");
//
//
//        let mouse = Mouse::new();
//
//        let mouse_down_position    = mouse.position.sample("mouse_down_position",&mouse.down);
//        let mouse_position_if_down = mouse.position.gate("mouse_position_if_down",&mouse.is_down);
//
//        let final_position_ref_i  = Recursive::<EventData<Position>>::new_named("final_position_ref");
//        let final_position_ref    = Dynamic::from(&final_position_ref_i);
//
//        let pos_diff_on_down   = mouse_down_position.map2("pos_diff_on_down", &final_position_ref, |m,f| {m - f});
//        let final_position  = mouse_position_if_down.map2("final_position", &pos_diff_on_down, |m,f| {m - f});
//        let debug              = final_position.sample("debug", &mouse.position);
//
//
//
//        final_position_ref_i.initialize(&final_position);
//
//        final_position_ref.event.set_display_id(final_position.event.display_id());
//        final_position_ref.behavior.set_display_id(final_position.event.display_id());
//
//
//
//        trace("X" , &debug.event);
//
//
//        final_position.map("foo",move|p| {callback(p.x as f32,-p.y as f32)});
//
//        final_position.behavior.display_graphviz();
//
//        let target = mouse.position.event.clone_ref();
//        let handle = mouse_manager.on_move.add(move |event:&mouse2::event::OnMove| {
//            target.emit(&EventData(Position::new(event.client_x(),event.client_y())));
//        });
//        handle.forget();
//
//        let target = mouse.down.event.clone_ref();
//        let handle = mouse_manager.on_down.add(move |event:&mouse2::event::OnDown| {
//            target.emit(&EventData(()));
//        });
//        handle.forget();
//
//        let target = mouse.up.event.clone_ref();
//        let handle = mouse_manager.on_up.add(move |event:&mouse2::event::OnUp| {
//            target.emit(&EventData(()));
//        });
//        handle.forget();
//
//        mouse_manager
//
//    }
//}
//pub use tests::*;
