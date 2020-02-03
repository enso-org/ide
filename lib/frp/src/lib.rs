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
            $( $poly_input : Node<$poly_input> ),* ,
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
                fn new_named<Label,$([<T $poly_input>],)*>
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



/// Similar to `deinfe_node` but specialized for lambda definitions.
///
/// Generates a ot of boilerplate for a new node definition. It generates the node struct, the
/// constructor, input / output relations, etc. In order to learn more, see the expanded version
/// of this macro used below.
macro_rules! define_lambda_node {
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

        paste::item! {
            pub trait [<$name New>]<$($poly_input,)* Func> {
                fn new_named<Label:Into<CowString>>
                (label:Label, $($poly_input:$poly_input,)* f:Func) -> Self;
            }

            impl<$($poly_input,)* OutVal, $([<T $poly_input>],)* Function>
            [<$name New>]<$([<T $poly_input>],)* Function>
            for $name<$($poly_input,)* Inferred<($($poly_input),*),OutVal>>
                where $($poly_input       : Data,)*
                      $([<T $poly_input>] : Into<Node<$poly_input>>,)*
                      $(Node<$poly_input> : AddTarget<Self>,)*
                      OutVal              : Infer<($($poly_input),*)>,
                      Function            : 'static + Fn($(&Content<$poly_input>),*) -> OutVal,
                      Inferred<($($poly_input),*),OutVal> : Data<Content=OutVal> {
                fn new_named<Label>
                (label:Label, $($poly_input:[<T $poly_input>],)* func:Function) -> Self
                where Label : Into<CowString> {
                    $(let $poly_input = $poly_input.into();)*
                    let func        = func.into();
                    let shape       = $shape_name{$($poly_input,)* func};
                    let this        = Self::construct(label,shape);
                    {
                        let shape = &this.rc.borrow().shape;
                        $(shape.$poly_input.add_target(&this);)*
                    }
                    this
                }
            }
        }

        impl<$($poly_input:Data,)* Out:Data> HasInputs for $shape_name<$($poly_input,)* Out> {
            fn inputs(&self) -> Vec<AnyNode> {
                vec![$((&self.$poly_input).into()),*]
            }
        }
    }
}


// ==============
// === Lambda ===
// ==============

define_lambda_node! {
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

impl<In:Value,Out:Data> EventConsumer for Lambda<EventData<In>,Out> {
    fn on_event(&self, input:&Self::EventInput) {
        let output = (self.rc.borrow().shape.func.raw)(unwrap(input));
        self.emit_event(&output);
    }
}



// ===============
// === Lambda2 ===
// ===============

define_lambda_node! {
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
    where In1:Data, In2:Data, Out:Data,
          Func : 'static + Fn(&Content<In1>,&Content<In2>) -> Content<Out> {
    fn from(func:Func) -> Self {
        let raw = Rc::new(move |a:&Content<In1>,b:&Content<In2>| { wrap(func(a,b)) });
        Self {raw}
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



// =============
// === Utils ===
// =============

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



// =================================================================================================
// === Examples ====================================================================================
// =================================================================================================

macro_rules! frp {
    ( $( $var:ident = $node:ident $(<$ty:ty>)*   ($($args:tt)*); )* ) => {$(
        let $var = $node $(::<$ty>)* :: new_named(stringify!{$var}, $($args)* );
    )*}
}



// ===============
// === Dynamic ===
// ===============

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
        (&Merge::new_named(label,&self.event,&that.event)).into()
    }

    pub fn toggle<Label>(&self, label:Label) -> Dynamic<bool>
        where Label:Into<CowString> {
        (&Toggle::new_named(label,&self.event)).into()
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
