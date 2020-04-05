//! This module implements an Functional Reactive Programming system. It is an advanced event
//! handling framework which allows describing events and actions by creating declarative event
//! flow diagrams.
//!
//! Please read this document as the initial introduction to FRP concepts:
//! https://github.com/hansroland/reflex-dom-inbits/blob/master/tutorial.md

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

#![feature(unboxed_closures)]


#![allow(missing_docs)]


pub mod debug;
pub mod io;
pub mod macros;


pub use enso_prelude as prelude;
use prelude::*;





type Label = &'static str;


// =============
// === Value ===
// =============

pub trait Value = 'static + Clone + Debug + Default;



// =================
// === HasOutput ===
// =================

pub trait HasOutput {
    type Output : Value;
}

pub type Output<T> = <T as HasOutput>::Output;



// ====================
// === EventEmitter ===
// ====================


pub trait AnyFlow = 'static + LastValueProvider + EventEmitter + CloneRef;

impl<T:EventEmitter> EventEmitterPoly for T {}
pub trait EventEmitterPoly : EventEmitter {
    fn ping(&self) where Self : HasOutput<Output=()> {
        self.emit_event(&())
    }

    fn emit<T:ToRef<Output<Self>>>(&self, value:T) {
        self.emit_event(value.to_ref())
    }
}


pub trait EventEmitter : HasOutput {
    fn emit_event      (&self , value:&Self::Output);
    fn register_target (&self , tgt:FlowInput<Output<Self>>);
}

pub trait EventConsumer<T> {
    fn on_event(&self, value:&T);
}

pub trait LastValueProvider : HasOutput {
    fn value(&self) -> Self::Output;
}

pub trait EventConsumerDebug<T> : EventConsumer<T> + Debug {}
impl<X,T> EventConsumerDebug<T> for X where X : EventConsumer<T> + Debug {}



// ============
// === Node ===
// ============

// === Types ===

pub trait NodeDefinition = 'static + ?Sized + HasOutput;

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Node<T:NodeDefinition> {
    data : Rc<NodeData<T>>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakNode<T:NodeDefinition> {
    data : Weak<NodeData<T>>,
}

#[derive(Debug)]
pub struct NodeData<Def:NodeDefinition> {
    label      : Label,
    targets    : RefCell<Vec<FlowInput<Output<Def>>>>,
    value      : Rc<RefCell<Output<Def>>>,
    definition : Def,
}


// === Output ===

impl<Def:NodeDefinition> HasOutput for Node     <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for WeakNode <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for NodeData <Def> { type Output = Output<Def>; }


// === Node Impls ===

impl<Def:NodeDefinition> Node<Def> {
    pub fn construct(label:Label, definition:Def) -> Self {
        let targets = default();
        let value   = default();
        let data    = Rc::new(NodeData {label,targets,value,definition});
        Self {data}
    }

    pub fn construct_and_connect<S>(label:Label, flow:&S, definition:Def) -> Self
    where S:AnyFlow, NodeData<Def>:EventConsumer<Output<S>> {
        let this = Self::construct(label,definition);
        let weak = this.downgrade();
        flow.register_target(weak.into());
        this
    }

    pub fn construct2(label:Label, definition:Def) -> Flow<Output<Def>> {
        Self::construct(label,definition).into()
    }

    pub fn construct_and_connect2<S>(label:Label, flow:&S, definition:Def) -> Flow<Output<Def>>
    where S:AnyFlow, NodeData<Def>:EventConsumer<Output<S>> {
        Self::construct_and_connect(label,flow,definition).into()
    }

    pub fn downgrade(&self) -> WeakNode<Def> {
        let data = Rc::downgrade(&self.data);
        WeakNode {data}
    }
}

impl<Def:NodeDefinition> EventEmitter for Node<Def>  {
    fn emit_event(&self, value:&Output<Def>) {
        self.data.emit_event(value)
    }

    fn register_target(&self,tgt:FlowInput<Output<Self>>) {
        self.data.register_target(tgt)
    }
}

impl<Def:NodeDefinition> LastValueProvider for Node<Def> {
    fn value(&self) -> Self::Output {
        self.data.value()
    }
}


// === WeakNode Impls ===

impl<T:NodeDefinition> WeakNode<T> {
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.data.upgrade().map(|data| Node{data})
    }
}

impl<Def:NodeDefinition> EventEmitter for WeakNode<Def> {
    fn emit_event(&self, value:&Output<Def>) {
        self.data.upgrade().for_each(|data| data.emit_event(value))
    }

    fn register_target(&self,tgt:FlowInput<Output<Self>>) {
        self.data.upgrade().for_each(|data| data.register_target(tgt))
    }
}

impl<Def:NodeDefinition> LastValueProvider for WeakNode<Def> {
    fn value(&self) -> Self::Output {
        self.data.upgrade().map(|data| data.value()).unwrap_or_default()
    }
}

impl<Def:NodeDefinition+Debug> Debug for WeakNode<Def> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data.upgrade() {
            None    => write!(f,"WeakNode(Dropped)"),
            Some(t) => write!(f,"WeakNode({:?})",t),
        }

    }
}


// === NodeData Impls ===

impl<Def:NodeDefinition> NodeData<Def> {
    fn default_emit(&self, value:&Output<Self>) {
        *self.value.borrow_mut() = value.clone();
        let mut dirty = false;
        self.targets.borrow().iter().for_each(|weak| match weak.data.upgrade() {
            Some(tgt) => tgt.on_event(value),
            None      => dirty = true
        });
        if dirty {
            self.targets.borrow_mut().retain(|weak| weak.data.upgrade().is_none());
        }
    }
}

impl<Def:NodeDefinition> EventEmitter for NodeData<Def> {
    default fn emit_event(&self, value:&Output<Self>) {
        self.default_emit(value);
    }

    default fn register_target(&self,tgt:FlowInput<Output<Self>>) {
        self.targets.borrow_mut().push(tgt);
    }
}

impl<Def:NodeDefinition> LastValueProvider for NodeData<Def> {
    fn value(&self) -> Self::Output {
        self.value.borrow().clone()
    }
}



// ============
// === Flow ===
// ============

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Flow<Out=()> {
    data  : Weak<dyn EventEmitter<Output=Out>>,
    value : Rc<RefCell<Out>>,

}

impl<Def:NodeDefinition> From<Node<Def>> for Flow<Def::Output> {
    fn from(node:Node<Def>) -> Self {
        let value = node.data.value.clone_ref();
        let data  = Rc::downgrade(&node.data);
        Flow {data,value}
    }
}

impl<Out:Value> HasOutput for Flow<Out> {
    type Output = Out;
}

impl<Out:Value> EventEmitter for Flow<Out> {
    fn emit_event(&self, value:&Self::Output) {
        self.data.upgrade().for_each(|t| t.emit_event(value))
    }

    fn register_target(&self,tgt:FlowInput<Output<Self>>) {
        self.data.upgrade().for_each(|t| t.register_target(tgt))
    }
}

impl<Out:Value> LastValueProvider for Flow<Out> {
    fn value(&self) -> Self::Output {
        self.value.borrow().clone()
    }
}

impl<Out> Debug for Flow<Out> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Flow")
    }
}



// ===================
// === FlowInput ===
// ===================

#[derive(Clone)]
pub struct FlowInput<Input> {
    data : Weak<dyn EventConsumer<Input>>
}

impl<Def:NodeDefinition,Input> From<WeakNode<Def>> for FlowInput<Input>
where NodeData<Def> : EventConsumer<Input> {
    fn from(node:WeakNode<Def>) -> Self {
        Self {data:node.data}
    }
}

impl<Def:NodeDefinition,Input> From<&WeakNode<Def>> for FlowInput<Input>
    where NodeData<Def> : EventConsumer<Input> {
    fn from(node:&WeakNode<Def>) -> Self {
        Self {data:node.data.clone_ref()}
    }
}

impl<Input> Debug for FlowInput<Input> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"FlowInput")
    }
}



// =============
// === Never ===
// =============

macro_rules! docs_for_never { ($($tt:tt)*) => { #[doc="
Begin point in the FRP network. It never fires any events. Can be used as an input placeholder which
guarantees that no events would be ever emitted here.
"]$($tt)* }}

docs_for_never! { #[derive(Clone,Copy,Debug)]
pub struct NeverData <Out=()> { phantom:PhantomData<Out> }}
pub type   Never     <Out=()> = Node     <NeverData<Out>>;
pub type   WeakNever <Out=()> = WeakNode <NeverData<Out>>;

impl<Out:Value> HasOutput for NeverData<Out> {
    type Output = Out;
}

impl<Out:Value> Never<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom    = default();
        let definition = NeverData {phantom};
        Self::construct(label,definition)
    }
}

impl<Out> EventEmitter for NodeData<NeverData<Out>>
where NeverData<Out> : NodeDefinition {
    fn emit_event(&self, _value:&Output<Self>) {}
    fn register_target(&self, _tgt:FlowInput<Output<Self>>) {}
}



// ==============
// === Source ===
// ==============

macro_rules! docs_for_source { ($($tt:tt)*) => { #[doc="
Begin point in the FRP network. It does not accept inputs, but it is able to emit events. Often it
is used to indicate that something happened, like a button was pressed. In such cases its type
parameter is set to an empty tuple.
"]$($tt)* }}

docs_for_source! { #[derive(Clone,Copy,Debug)]
pub struct SourceData <Out=()> { phantom:PhantomData<Out> }}
pub type   Source     <Out=()> = Node     <SourceData<Out>>;
pub type   WeakSource <Out=()> = WeakNode <SourceData<Out>>;

impl<Out:Value> HasOutput for SourceData<Out> {
    type Output = Out;
}

impl<Out:Value> Source<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom    = default();
        let definition = SourceData {phantom};
        Self::construct(label,definition)
    }
}



// =============
// === Trace ===
// =============

macro_rules! docs_for_trace { ($($tt:tt)*) => { #[doc="
Print the incoming events to console and pass them to output.
"]$($tt)* }}

/// Print the incoming events to console and pass them to output.
#[derive(Clone,Debug)]
pub struct TraceData <Out> { phantom:PhantomData<Out>, message:String }
pub type   Trace     <Out> = Node     <TraceData<Out>>;
pub type   WeakTrace <Out> = WeakNode <TraceData<Out>>;

impl<Out:Value> HasOutput for TraceData<Out> {
    type Output = Out;
}

impl<Out:Value> Trace<Out> {
    /// Constructor.
    pub fn new<M,S>(label:Label, message:M, flow:&S) -> Self
    where M:Into<String>, S:AnyFlow<Output=Out> {
        let phantom = default();
        let message = message.into();
        let def     = TraceData {phantom,message};
        Self::construct_and_connect(label,flow,def)
    }
}

impl<Out:Value> EventConsumer<Out> for NodeData<TraceData<Out>> {
    fn on_event(&self, event:&Out) {
        println!("[FRP] {}: {:?}", self.definition.message, event);
        self.emit(event);
    }
}



// ==============
// === Toggle ===
// ==============

macro_rules! docs_for_toggle { ($($tt:tt)*) => { #[doc="
Emits `true`, `false`, `true`, `false`, ... on every incoming event.
"]$($tt)* }}

docs_for_toggle! { #[derive(Clone,Debug)]
pub struct ToggleData { value:Cell<bool> }}
pub type   Toggle     = Node     <ToggleData>;
pub type   WeakToggle = WeakNode <ToggleData>;

impl HasOutput for ToggleData {
    type Output = bool;
}

impl Toggle {
    /// Constructor.
    pub fn new<S:AnyFlow>(label:Label, flow:&S) -> Self {
        Self::new_with(label,flow,default())
    }

    /// Constructor with explicit start value.
    pub fn new_with<S:AnyFlow>(label:Label, flow:&S, init:bool) -> Self {
        let value = Cell::new(init);
        let def   = ToggleData {value};
        Self::construct_and_connect(label,flow,def)
    }
}

impl<T> EventConsumer<T> for NodeData<ToggleData> {
    fn on_event(&self, _:&T) {
        let value = !self.definition.value.get();
        self.definition.value.set(value);
        self.emit(value);
    }
}



// =============
// === Count ===
// =============

macro_rules! docs_for_count { ($($tt:tt)*) => { #[doc="
Count the incoming events.
"]$($tt)* }}

docs_for_count! { #[derive(Clone,Debug)]
pub struct CountData { value:Cell<usize> }}
pub type   Count     = Node     <CountData>;
pub type   WeakCount = WeakNode <CountData>;

impl HasOutput for CountData {
    type Output = usize;
}

impl Count {
    /// Constructor.
    pub fn new<S>(label:Label, flow:&S) -> Self
    where S:AnyFlow {
        let value = default();
        let def   = CountData {value};
        Self::construct_and_connect(label,flow,def)
    }
}

impl<T> EventConsumer<T> for NodeData<CountData> {
    fn on_event(&self, _:&T) {
        let value = self.definition.value.get() + 1;
        self.definition.value.set(value);
        self.emit(value);
    }
}


// ================
// === Constant ===
// ================

macro_rules! docs_for_constant { ($($tt:tt)*) => { #[doc="
Replaces the incoming event with the predefined value.
"]$($tt)* }}

docs_for_constant! { #[derive(Clone,Debug)]
pub struct ConstantData <Out=()> { value:Out }}
pub type   Constant     <Out=()> = Node     <ConstantData<Out>>;
pub type   WeakConstant <Out=()> = WeakNode <ConstantData<Out>>;

impl<Out:Value> HasOutput for ConstantData<Out> {
    type Output = Out;
}

impl<Out:Value> Constant<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, flow:&S, value:Out) -> Self
    where S:AnyFlow {
        let def = ConstantData {value};
        Self::construct_and_connect(label,flow,def)
    }
}

impl<Out:Value,T> EventConsumer<T> for NodeData<ConstantData<Out>> {
    fn on_event(&self, _:&T) {
        self.emit(&self.definition.value);
    }
}



// ================
// === Previous ===
// ================

macro_rules! docs_for_previous { ($($tt:tt)*) => { #[doc="
Remembers the value of the input flow and outputs the previously received one.
"]$($tt)* }}

docs_for_previous! { #[derive(Clone,Debug)]
pub struct PreviousData <Out=()> { previous:RefCell<Out> }}
pub type   Previous     <Out=()> = Node     <PreviousData<Out>>;
pub type   WeakPrevious <Out=()> = WeakNode <PreviousData<Out>>;

impl<Out:Value> HasOutput for PreviousData<Out> {
    type Output = Out;
}

impl<Out:Value> Previous<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, flow:&S) -> Self
        where S:AnyFlow<Output=Out> {
        let previous = default();
        let def      = PreviousData {previous};
        Self::construct_and_connect(label,flow,def)
    }
}

impl<Out:Value> EventConsumer<Out> for NodeData<PreviousData<Out>> {
    fn on_event(&self, event:&Out) {
        let previous = mem::replace(&mut *self.definition.previous.borrow_mut(),event.clone());
        self.emit(previous);
    }
}



// ==============
// === Sample ===
// ==============

macro_rules! docs_for_sample { ($($tt:tt)*) => { #[doc="
Samples the first flow (behavior) on every incoming event of the second flow. The incoming event
is dropped and a new event with the behavior's value is emitted.
"]$($tt)* }}

docs_for_sample! { #[derive(Clone,Debug)]
pub struct SampleData <Behavior> { behavior:Behavior }}
pub type   Sample     <Behavior> = Node     <SampleData<Behavior>>;
pub type   WeakSample <Behavior> = WeakNode <SampleData<Behavior>>;

impl<Behavior:HasOutput> HasOutput for SampleData<Behavior> {
    type Output = Output<Behavior>;
}

impl<Behavior:AnyFlow> Sample<Behavior> {
    /// Constructor.
    pub fn new<Event:AnyFlow>(label:Label, event:&Event, behavior:&Behavior) -> Self {
        let behavior   = behavior.clone_ref();
        let definition = SampleData {behavior};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T,Behavior:AnyFlow> EventConsumer<T> for NodeData<SampleData<Behavior>> {
    fn on_event(&self, _:&T) {
        self.emit(self.definition.behavior.value());
    }
}



// ============
// === Gate ===
// ============

macro_rules! docs_for_gate { ($($tt:tt)*) => { #[doc="
Passes the incoming event of the fisr flow only if the value of the second flow is `true`.
"]$($tt)* }}

docs_for_gate! { #[derive(Clone,Debug)]
pub struct GateData <T,B> { behavior:B, phantom:PhantomData<T> }}
pub type   Gate     <T,B> = Node     <GateData<T,B>>;
pub type   WeakGate <T,B> = WeakNode <GateData<T,B>>;

impl<T:Value,B> HasOutput for GateData<T,B> {
    type Output = T;
}

impl<T,B> Gate<T,B>
where T:Value, B:AnyFlow<Output=bool> {
    /// Constructor.
    pub fn new<E>(label:Label, event:&E, behavior:&B) -> Self
    where E:AnyFlow<Output=T> {
        let behavior   = behavior.clone_ref();
        let phantom    = default();
        let definition = GateData {behavior,phantom};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T,B> EventConsumer<T> for NodeData<GateData<T,B>>
where T:Value, B:AnyFlow<Output=bool> {
    fn on_event(&self, event:&T) {
        if self.definition.behavior.value() {
            self.emit(event)
        }
    }
}



// =============
// === Merge ===
// =============

macro_rules! docs_for_merge { ($($tt:tt)*) => { #[doc="
Merges multiple input flows into a single output flow. All input flows have to share the same
output data type. Please note that `Merge` can be used to create recursive FRP networks by creating
an empty merge and using the `add` method to attach new flows to it. When a recursive network is
created, `Merge` breaks the cycle. After passing the first event, no more events will be passed
till the end of the current FRP network resolution.
"]$($tt)* }}

docs_for_merge! { #[derive(Clone,Debug)]
pub struct MergeData <Out> { phantom:PhantomData<Out>, during_call:Cell<bool> }}
pub type   Merge     <Out> = Node     <MergeData<Out>>;
pub type   WeakMerge <Out> = WeakNode <MergeData<Out>>;

impl<Out:Value> HasOutput for MergeData<Out> {
    type Output = Out;
}

impl<Out:Value> Merge<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom     = default();
        let during_call = default();
        let def     = MergeData {phantom,during_call};
        Self::construct(label,def)
    }

    /// Takes ownership of self and returns it with a new flow attached.
    pub fn with<S>(self, flow:&S) -> Self
        where S:AnyFlow<Output=Out> {
        flow.register_target(self.downgrade().into());
        self
    }

    /// Constructor for 1 input flow.
    pub fn new1<S1>(label:Label, s1:&S1) -> Self
        where S1:AnyFlow<Output=Out> {
        Self::new(label).with(s1)
    }

    /// Constructor for 2 input flows.
    pub fn new2<S1,S2>(label:Label, s1:&S1, s2:&S2) -> Self
        where S1:AnyFlow<Output=Out>,
              S2:AnyFlow<Output=Out> {
        Self::new(label).with(s1).with(s2)
    }

    /// Constructor for 3 input flows.
    pub fn new3<S1,S2,S3>(label:Label, s1:&S1, s2:&S2, s3:&S3) -> Self
        where S1:AnyFlow<Output=Out>,
              S2:AnyFlow<Output=Out>,
              S3:AnyFlow<Output=Out> {
        Self::new(label).with(s1).with(s2).with(s3)
    }

    /// Constructor for 4 input flows.
    pub fn new4<S1,S2,S3,S4>(label:Label, s1:&S1, s2:&S2, s3:&S3, s4:&S4) -> Self
        where S1:AnyFlow<Output=Out>,
              S2:AnyFlow<Output=Out>,
              S3:AnyFlow<Output=Out>,
              S4:AnyFlow<Output=Out> {
        Self::new(label).with(s1).with(s2).with(s3).with(s4)
    }
}

impl<Out:Value> WeakMerge<Out> {
    /// Takes ownership of self and returns it with a new flow attached.
    pub fn with<S>(self, flow:&S) -> Self
    where S:AnyFlow<Output=Out> {
        flow.register_target(self.clone_ref().into());
        self
    }
}

impl<S1,Out> Add<&S1> for &Merge<Out>
    where S1:AnyFlow<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, flow:&S1) -> Self::Output {
        flow.register_target(self.downgrade().into());
        self
    }
}

impl<S1,Out> Add<&S1> for &WeakMerge<Out>
    where S1:AnyFlow<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, flow:&S1) -> Self::Output {
        flow.register_target(self.into());
        self
    }
}

impl<Out:Value> EventConsumer<Out> for NodeData<MergeData<Out>> {
    fn on_event(&self, event:&Out) {
        self.emit(event);
    }
}

impl<Out> EventEmitter for NodeData<MergeData<Out>>
where MergeData<Out> : NodeDefinition {
    fn emit_event(&self, value:&Output<Self>) {
        if !self.definition.during_call.get() {
            self.definition.during_call.set(true);
            self.default_emit(value);
            self.definition.during_call.set(false);
        }
    }
}



// ============
// === Zip2 ===
// ============

macro_rules! docs_for_zip2 { ($($tt:tt)*) => { #[doc="
Merges two input flows into a flow containing values from both of them. On event from any of the
flows, all flows are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip2! { #[derive(Clone,Copy,Debug)]
pub struct Zip2Data <S1,S2> { flow1:S1, flow2:S2 }}
pub type   Zip2     <S1,S2> = Node     <Zip2Data<S1,S2>>;
pub type   WeakZip2 <S1,S2> = WeakNode <Zip2Data<S1,S2>>;

impl<S1,S2> HasOutput for Zip2Data<S1,S2>
    where S1:AnyFlow, S2:AnyFlow {
    type Output = (Output<S1>,Output<S2>);
}

impl<S1,S2> Zip2<S1,S2>
    where S1:AnyFlow, S2:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let def   = Zip2Data {flow1,flow2};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.into());
        this
    }
}

impl<S1,S2,Out> EventConsumer<Out> for NodeData<Zip2Data<S1,S2>>
    where S1:AnyFlow, S2:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        self.emit((value1,value2));
    }
}



// ============
// === Zip3 ===
// ============

macro_rules! docs_for_zip3 { ($($tt:tt)*) => { #[doc="
Merges three input flows into a flow containing values from all of them. On event from any of
the flows, all flows are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip3! { #[derive(Clone,Copy,Debug)]
pub struct Zip3Data <S1,S2,S3> { flow1:S1, flow2:S2, flow3:S3 }}
pub type   Zip3     <S1,S2,S3> = Node     <Zip3Data<S1,S2,S3>>;
pub type   WeakZip3 <S1,S2,S3> = WeakNode <Zip3Data<S1,S2,S3>>;

impl<S1,S2,S3> HasOutput for Zip3Data<S1,S2,S3>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow {
    type Output = (Output<S1>,Output<S2>,Output<S3>);
}

impl<S1,S2,S3> Zip3<S1,S2,S3>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let def   = Zip3Data {flow1,flow2,flow3};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,Out> EventConsumer<Out> for NodeData<Zip3Data<S1,S2,S3>>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        self.emit((value1,value2,value3));
    }
}



// ============
// === Zip4 ===
// ============

macro_rules! docs_for_zip4 { ($($tt:tt)*) => { #[doc="
Merges four input flows into a flow containing values from all of them. On event from any of the
flows, all flows are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip4! { #[derive(Clone,Copy,Debug)]
pub struct Zip4Data <S1,S2,S3,S4> { flow1:S1, flow2:S2, flow3:S3, flow4:S4 }}
pub type   Zip4     <S1,S2,S3,S4> = Node     <Zip4Data<S1,S2,S3,S4>>;
pub type   WeakZip4 <S1,S2,S3,S4> = WeakNode <Zip4Data<S1,S2,S3,S4>>;

impl<S1,S2,S3,S4> HasOutput for Zip4Data<S1,S2,S3,S4>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow {
    type Output = (Output<S1>,Output<S2>,Output<S3>,Output<S4>);
}

impl<S1,S2,S3,S4> Zip4<S1,S2,S3,S4>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3, s4:&S4) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let flow4 = s4.clone_ref();
        let def   = Zip4Data {flow1,flow2,flow3,flow4};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.clone_ref().into());
        s4.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,Out> EventConsumer<Out> for NodeData<Zip4Data<S1,S2,S3,S4>>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        self.emit((value1,value2,value3,value4));
    }
}



// ===========
// === Map ===
// ===========

macro_rules! docs_for_map { ($($tt:tt)*) => { #[doc="
On every event from the first input flow, sample all other input flows and run the provided
function on all gathered values. If you want to run the function on event from any input flow,
use the `apply` function family instead.
"]$($tt)* }}

docs_for_map! {
#[derive(Clone)]
pub struct MapData <S,F> { flow:S, function:F }}
pub type   Map     <S,F> = Node     <MapData<S,F>>;
pub type   WeakMap <S,F> = WeakNode <MapData<S,F>>;

impl<S,F,Out> HasOutput for MapData<S,F>
where S:AnyFlow, Out:Value, F:'static+Fn(&Output<S>)->Out {
    type Output = Out;
}

impl<S,F,Out> Map<S,F>
where S:AnyFlow, Out:Value, F:'static+Fn(&Output<S>)->Out {
    /// Constructor.
    pub fn new(label:Label, s:&S, function:F) -> Self {
        let flow     = s.clone_ref();
        let definition = MapData {flow,function};
        Self::construct_and_connect(label,s,definition)
    }
}

impl<S,F,Out> EventConsumer<Output<S>> for NodeData<MapData<S,F>>
where S:AnyFlow, Out:Value, F:'static+Fn(&Output<S>)->Out {
    fn on_event(&self, value:&Output<S>) {
        let out = (self.definition.function)(value);
        self.emit(out);
    }
}

impl<S,F> Debug for MapData<S,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapData")
    }
}



// ============
// === Map2 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map2Data <S1,S2,F> { flow1:S1, flow2:S2, function:F }}
pub type   Map2     <S1,S2,F> = Node     <Map2Data<S1,S2,F>>;
pub type   WeakMap2 <S1,S2,F> = WeakNode <Map2Data<S1,S2,F>>;

impl<S1,S2,F,Out> HasOutput for Map2Data<S1,S2,F>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    type Output = Out;
}

impl<S1,S2,F,Out> Map2<S1,S2,F>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let def   = Map2Data {flow1,flow2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,F,Out> EventConsumer<Output<S1>> for NodeData<Map2Data<S1,S2,F>>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.flow2.value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<S1,S2,F> Debug for Map2Data<S1,S2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map2Data")
    }
}



// ============
// === Map3 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map3Data <S1,S2,S3,F> { flow1:S1, flow2:S2, flow3:S3, function:F }}
pub type   Map3     <S1,S2,S3,F> = Node     <Map3Data<S1,S2,S3,F>>;
pub type   WeakMap3 <S1,S2,S3,F> = WeakNode <Map3Data<S1,S2,S3,F>>;

impl<S1,S2,S3,F,Out> HasOutput for Map3Data<S1,S2,S3,F>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,F,Out> Map3<S1,S2,S3,F>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let def   = Map3Data {flow1,flow2,flow3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,F,Out> EventConsumer<Output<S1>> for NodeData<Map3Data<S1,S2,S3,F>>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<S1,S2,S3,F> Debug for Map3Data<S1,S2,S3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map3Data")
    }
}



// ============
// === Map4 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map4Data <S1,S2,S3,S4,F> { flow1:S1, flow2:S2, flow3:S3, flow4:S4, function:F }}
pub type   Map4     <S1,S2,S3,S4,F> = Node     <Map4Data<S1,S2,S3,S4,F>>;
pub type   WeakMap4 <S1,S2,S3,S4,F> = WeakNode <Map4Data<S1,S2,S3,S4,F>>;

impl<S1,S2,S3,S4,F,Out> HasOutput for Map4Data<S1,S2,S3,S4,F>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,S4,F,Out> Map4<S1,S2,S3,S4,F>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3, s4:&S4, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let flow4 = s4.clone_ref();
        let def   = Map4Data {flow1,flow2,flow3,flow4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,F,Out> EventConsumer<Output<S1>> for NodeData<Map4Data<S1,S2,S3,S4,F>>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<S1,S2,S3,S4,F> Debug for Map4Data<S1,S2,S3,S4,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map4Data")
    }
}



// ==============
// === Apply2 ===
// ==============

macro_rules! docs_for_apply { ($($tt:tt)*) => { #[doc="
On every input event sample all input flows and run the provided function on all gathered values.
If you want to run the function only on event on the first input, use the `map` function family
instead.
"]$($tt)* }}

docs_for_apply! { #[derive(Clone)]
pub struct Apply2Data <S1,S2,F> { flow1:S1, flow2:S2, function:F }}
pub type   Apply2     <S1,S2,F> = Node     <Apply2Data<S1,S2,F>>;
pub type   WeakApply2 <S1,S2,F> = WeakNode <Apply2Data<S1,S2,F>>;

impl<S1,S2,F,Out> HasOutput for Apply2Data<S1,S2,F>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    type Output = Out;
}

impl<S1,S2,F,Out> Apply2<S1,S2,F>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let def   = Apply2Data {flow1,flow2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.into());
        this
    }
}

impl<S1,S2,F,Out,T> EventConsumer<T> for NodeData<Apply2Data<S1,S2,F>>
where S1:AnyFlow, S2:AnyFlow, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<S1,S2,F> Debug for Apply2Data<S1,S2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply2Data")
    }
}



// ==============
// === Apply3 ===
// ==============

docs_for_apply! { #[derive(Clone)]
pub struct Apply3Data <S1,S2,S3,F> { flow1:S1, flow2:S2, flow3:S3, function:F }}
pub type   Apply3     <S1,S2,S3,F> = Node     <Apply3Data<S1,S2,S3,F>>;
pub type   WeakApply3 <S1,S2,S3,F> = WeakNode <Apply3Data<S1,S2,S3,F>>;

impl<S1,S2,S3,F,Out> HasOutput for Apply3Data<S1,S2,S3,F>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,F,Out> Apply3<S1,S2,S3,F>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let def   = Apply3Data {flow1,flow2,flow3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,F,Out,T> EventConsumer<T> for NodeData<Apply3Data<S1,S2,S3,F>>
where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<S1,S2,S3,F> Debug for Apply3Data<S1,S2,S3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply3Data")
    }
}



// ==============
// === Apply4 ===
// ==============

docs_for_apply! { #[derive(Clone)]
pub struct Apply4Data <S1,S2,S3,S4,F> {flow1:S1, flow2:S2, flow3:S3, flow4:S4, function:F}}
pub type   Apply4     <S1,S2,S3,S4,F> = Node     <Apply4Data<S1,S2,S3,S4,F>>;
pub type   WeakApply4 <S1,S2,S3,S4,F> = WeakNode <Apply4Data<S1,S2,S3,S4,F>>;

impl<S1,S2,S3,S4,F,Out> HasOutput for Apply4Data<S1,S2,S3,S4,F>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,S4,F,Out> Apply4<S1,S2,S3,S4,F>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    /// Constructor.
    pub fn new(label:Label, s1:&S1, s2:&S2, s3:&S3, s4:&S4, function:F) -> Self {
        let flow1 = s1.clone_ref();
        let flow2 = s2.clone_ref();
        let flow3 = s3.clone_ref();
        let flow4 = s4.clone_ref();
        let def   = Apply4Data {flow1,flow2,flow3,flow4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.clone_ref().into());
        s4.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,F,Out,T> EventConsumer<T> for NodeData<Apply4Data<S1,S2,S3,S4,F>>
    where S1:AnyFlow, S2:AnyFlow, S3:AnyFlow, S4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<S1,S2,S3,S4,F> Debug for Apply4Data<S1,S2,S3,S4,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply4Data")
    }
}



// ===============
// === Network ===
// ===============

// === Definition ===

#[derive(Clone,CloneRef,Debug)]
pub struct Network {
    data : Rc<NetworkData>
}

#[derive(Clone,CloneRef,Debug)]
pub struct WeakNetwork {
    data : Weak<NetworkData>
}

#[derive(Debug)]
pub struct NetworkData {
    nodes : RefCell<Vec<Box<dyn Any>>>
}


// === API ===

impl NetworkData {
    /// Constructor.
    pub fn new() -> Self {
        let nodes = default();
        Self {nodes}
    }
}

impl Network {
    /// Constructor.
    pub fn new() -> Self {
        let data = Rc::new(NetworkData::new());
        Self {data}
    }

    pub fn downgrade(&self) -> WeakNetwork {
        WeakNetwork {data:Rc::downgrade(&self.data)}
    }

    pub fn register_raw<T:NodeDefinition>(&self, node:Node<T>) -> WeakNode<T> {
        let weak = node.downgrade();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        weak
    }

    pub fn register<Def:NodeDefinition>(&self, node:Node<Def>) -> Flow<Output<Def>> {
        let flow = node.clone_ref().into();
        let node   = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        flow
    }
}

impl WeakNetwork {
    pub fn upgrade(&self) -> Option<Network> {
        self.data.upgrade().map(|data| Network {data})
    }
}

impl Network {
    docs_for_never! {
    pub fn never<T:Value>(&self, label:Label) -> Flow<T> {
        self.register(Never::new(label,))
    }}

    docs_for_source! {
    pub fn source<T:Value>(&self, label:Label) -> Flow<T> {
        self.register(Source::new(label,))
    }}

    docs_for_source! {
    pub fn source_(&self, label:Label) -> Flow<()> {
        self.register(Source::new(label,))
    }}

    docs_for_trace! {
    pub fn trace<M:Into<String>,T:Value>
    (&self, label:Label, message:M, flow:&Flow<T>) -> Flow<T> {
        self.register(Trace::new(label,message,flow))
    }}

    docs_for_toggle! {
    pub fn toggle<T:Value>(&self, label:Label, flow:&Flow<T>) -> Flow<bool> {
        self.register(Toggle::new(label,flow))
    }}

    docs_for_count! {
    pub fn count<S:AnyFlow>(&self, label:Label, flow:&S) -> Flow<usize> {
        self.register(Count::new(label,flow))
    }}

    docs_for_constant! {
    pub fn constant<S:Value,T:Value> (&self, label:Label, flow:&Flow<S>, value:T) -> Flow<T> {
        self.register(Constant::new(label,flow,value))
    }}

    docs_for_previous! {
    pub fn previous<T:Value> (&self, label:Label, flow:&Flow<T>) -> Flow<T> {
        self.register(Previous::new(label,flow))
    }}

    docs_for_sample! {
    pub fn sample<S:Value,T:Value>
    (&self, label:Label, event:&Flow<S>, behavior:&Flow<T>) -> Flow<T> {
        self.register(Sample::new(label,event,behavior))
    }}

    docs_for_gate! {
    pub fn gate<T:Value>(&self, label:Label, event:&Flow<T>, check:&Flow<bool>) -> Flow<T> {
        self.register(Gate::new(label,event,check))
    }}


    // === Merge ===

    docs_for_merge! {
    pub fn merge_<Out:Value>(&self, label:Label) -> WeakMerge<Out> {
        self.register_raw(Merge::new(label,))
    }}

    docs_for_merge! {
    pub fn merge<T:Value>(&self, label:Label, s1:&Flow<T>, s2:&Flow<T>) -> Flow<T> {
        self.register(Merge::new2(label,s1,s2))
    }}

    docs_for_merge! {
    pub fn merge1<T:Value>(&self, label:Label, s1:&Flow<T>) -> Flow<T> {
        self.register(Merge::new1(label,s1))
    }}

    docs_for_merge! {
    pub fn merge2<T:Value>(&self, label:Label, s1:&Flow<T>, s2:&Flow<T>) -> Flow<T> {
        self.register(Merge::new2(label,s1,s2))
    }}

    docs_for_merge! {
    pub fn merge3<T:Value>
    (&self, label:Label, s1:&Flow<T>, s2:&Flow<T>, s3:&Flow<T>) -> Flow<T> {
        self.register(Merge::new3(label,s1,s2,s3))
    }}

    docs_for_merge! {
    pub fn merge4<T:Value>
    (&self, label:Label, s1:&Flow<T>, s2:&Flow<T>, s3:&Flow<T>, s4:&Flow<T>) -> Flow<T> {
        self.register(Merge::new4(label,s1,s2,s3,s4))
    }}


    // === Zip ===

    docs_for_zip2! {
    pub fn zip<T1:Value,T2:Value>
    (&self, label:Label, s1:&Flow<T1>, s2:&Flow<T2>) -> Flow<(T1,T2)> {
        self.register(Zip2::new(label,s1,s2))
    }}

    docs_for_zip2! {
    pub fn zip2<T1:Value,T2:Value>
    (&self, label:Label, s1:&Flow<T1>, s2:&Flow<T2>) -> Flow<(T1,T2)> {
        self.register(Zip2::new(label,s1,s2))
    }}

    docs_for_zip3! {
    pub fn zip3<T1:Value,T2:Value,T3:Value>
    (&self, label:Label, s1:&Flow<T1>, s2:&Flow<T2>, s3:&Flow<T3>) -> Flow<(T1,T2,T3)> {
        self.register(Zip3::new(label,s1,s2,s3))
    }}

    docs_for_zip4! {
    pub fn zip4<T1:Value,T2:Value,T3:Value,T4:Value>
    (&self, label:Label, s1:&Flow<T1>, s2:&Flow<T2>, s3:&Flow<T3>, s4:&Flow<T4>
    ) -> Flow<(T1,T2,T3,T4)> {
        self.register(Zip4::new(label,s1,s2,s3,s4))
    }}


    // === Map ===

    docs_for_map! {
    pub fn map<S:Value, T:Value, F:'static+Fn(&S)->T>
    (&self, label:Label, source:&Flow<S>, f:F) -> Flow<T> {
        self.register(Map::new(label,source,f))
    }}

    docs_for_map! {
    pub fn map2<S1:Value, S2:Value, T:Value, F:'static+Fn(&S1,&S2)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, f:F) -> Flow<T> {
        self.register(Map2::new(label,s1,s2,f))
    }}

    docs_for_map! {
    pub fn map3<S1:Value, S2:Value, S3:Value, T:Value, F:'static+Fn(&S1,&S2,&S3)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, s3:&Flow<S3>, f:F) -> Flow<T> {
        self.register(Map3::new(label,s1,s2,s3,f))
    }}

    docs_for_map! {
    pub fn map4<S1:Value, S2:Value, S3:Value, S4:Value, T:Value, F:'static+Fn(&S1,&S2,&S3,&S4)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, s3:&Flow<S3>, s4:&Flow<S4>, f:F) -> Flow<T> {
        self.register(Map4::new(label,s1,s2,s3,s4,f))
    }}


    // === Apply ===

    docs_for_apply! {
    pub fn apply2<S1:Value, S2:Value, T:Value, F:'static+Fn(&S1,&S2)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, f:F) -> Flow<T> {
        self.register(Apply2::new(label,s1,s2,f))
    }}

    docs_for_apply! {
    pub fn apply3<S1:Value, S2:Value, S3:Value, T:Value, F:'static+Fn(&S1,&S2,&S3)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, s3:&Flow<S3>, f:F) -> Flow<T> {
        self.register(Apply3::new(label,s1,s2,s3,f))
    }}

    docs_for_apply! {
    pub fn apply4<S1:Value, S2:Value, S3:Value, S4:Value, T:Value, F:'static+Fn(&S1,&S2,&S3,&S4)->T>
    (&self, label:Label, s1:&Flow<S1>, s2:&Flow<S2>, s3:&Flow<S3>, s4:&Flow<S4>, f:F) -> Flow<T> {
        self.register(Apply4::new(label,s1,s2,s3,s4,f))
    }}
}


///////////////////////////////////



#[allow(unused_variables)]
pub fn test() {
    println!("hello");

    new_network! { network
        def source  = source::<f32>();
        def source2 = source::<()>();
        def tg      = toggle(&source);
        def fff     = map(&tg,|t| { println!("{:?}",t) });
        def bb      = sample(&source2,&tg);

        let bb2 : Flow<bool> = bb.into();

        def fff2   = map(&bb2,|t| { println!(">> {:?}",t) });
        def m      = merge_::<usize>();
        def c      = count(&m);
        def t      = trace("t",&c);
    }

    m.add(&c);

    println!("{:?}",tg);

    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);

    m.emit(&0);
    m.emit(&0);
    m.emit(&0);
}



#[cfg(test)]
mod tests {
    use crate as frp;
    use crate::*;

    #[test]
    fn counter() {
        frp::new_network! { network1
            def source = source();
        }
        frp::new_network! { network2
            def count = source.count();
        }
        assert_eq!(count.value(),0);
        source.ping();
        assert_eq!(count.value(),1);
        source.ping();
        assert_eq!(count.value(),2);
        mem::drop(network1);
        source.ping();
        assert_eq!(count.value(),2);
        mem::drop(network2);
        source.ping();
        assert_eq!(count.value(),2);
    }
}
