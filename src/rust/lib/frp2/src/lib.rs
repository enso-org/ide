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

use enso_prelude::*;









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


pub trait AnyStream = 'static + LastValueProvider + EventEmitter + CloneRef;

pub trait StreamNode : LastValueProvider + EventEmitter {}
impl<T> StreamNode for T where T : LastValueProvider + EventEmitter {}

pub trait EventEmitter : HasOutput {
    fn emit(&self, value:&Self::Output);
    fn register_target(&self,tgt:StreamInput<Output<Self>>);
}

pub trait EventConsumer<T> {
    fn on_event(&self, value:&T);
}

pub trait LastValueProvider : HasOutput {
    fn last_value(&self) -> Self::Output;
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
    targets    : RefCell<Vec<StreamInput<Output<Def>>>>,
    last_value : RefCell<Output<Def>>,
    definition : Def,
}


// === Output ===

impl<Def:NodeDefinition> HasOutput for Node     <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for WeakNode <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for NodeData <Def> { type Output = Output<Def>; }


// === Node Impls ===

impl<Def:NodeDefinition> Node<Def> {
    pub fn construct(definition:Def) -> Self {
        let targets    = default();
        let last_value = default();
        let data       = Rc::new(NodeData {targets,last_value,definition});
        Self {data}
    }

    pub fn construct_and_connect<Source:AnyStream>(source:&Source, definition:Def) -> Self
    where NodeData<Def> : EventConsumer<Output<Source>> {
        let this = Self::construct(definition);
        let weak = this.downgrade();
        source.register_target(weak.into());
        this
    }

    pub fn downgrade(&self) -> WeakNode<Def> {
        let data = Rc::downgrade(&self.data);
        WeakNode {data}
    }
}

impl<Def:NodeDefinition> EventEmitter for Node<Def>  {
    fn emit(&self, value:&Output<Def>) {
        self.data.emit(value)
    }

    fn register_target(&self,tgt:StreamInput<Output<Self>>) {
        self.data.register_target(tgt)
    }
}

impl<Def:NodeDefinition> LastValueProvider for Node<Def> {
    fn last_value(&self) -> Self::Output {
        self.data.last_value()
    }
}


// === WeakNode Impls ===

impl<T:NodeDefinition> WeakNode<T> {
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.data.upgrade().map(|data| Node{data})
    }
}

impl<Def:NodeDefinition> EventEmitter for WeakNode<Def> {
    fn emit(&self, value:&Output<Def>) {
        self.data.upgrade().for_each(|data| data.emit(value))
    }

    fn register_target(&self,tgt:StreamInput<Output<Self>>) {
        self.data.upgrade().for_each(|data| data.register_target(tgt))
    }
}

impl<Def:NodeDefinition> LastValueProvider for WeakNode<Def> {
    fn last_value(&self) -> Self::Output {
        self.data.upgrade().map(|data| data.last_value()).unwrap_or_default()
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

impl<Def:NodeDefinition> EventEmitter for NodeData<Def> {
    default fn emit(&self, value:&Self::Output) {
        *self.last_value.borrow_mut() = value.clone();
        let mut dirty = false;
        self.targets.borrow().iter().for_each(|weak| match weak.data.upgrade() {
            Some(tgt) => tgt.on_event(value),
            None      => dirty = true
        });
        if dirty {
            self.targets.borrow_mut().retain(|weak| weak.data.upgrade().is_none());
        }
    }

    default fn register_target(&self,tgt:StreamInput<Output<Self>>) {
        self.targets.borrow_mut().push(tgt);
    }
}

impl<Def:NodeDefinition> LastValueProvider for NodeData<Def> {
    fn last_value(&self) -> Self::Output {
        self.last_value.borrow().clone()
    }
}



// ==============
// === Stream ===
// ==============

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Stream<Out> {
    data : Weak<dyn StreamNode<Output=Out>>,
}

impl<Def:NodeDefinition> From<WeakNode<Def>> for Stream<Def::Output> {
    fn from(node:WeakNode<Def>) -> Self {
        Stream {data:node.data}
    }
}

impl<Out:Value> HasOutput for Stream<Out> {
    type Output = Out;
}

impl<Out:Value> EventEmitter for Stream<Out> {
    fn emit(&self, value:&Self::Output) {
        self.data.upgrade().for_each(|t| t.emit(value))
    }

    fn register_target(&self,tgt:StreamInput<Output<Self>>) {
        self.data.upgrade().for_each(|t| t.register_target(tgt))
    }
}

impl<Out:Value> LastValueProvider for Stream<Out> {
    fn last_value(&self) -> Self::Output {
        self.data.upgrade().map(|t| t.last_value()).unwrap_or_default()
    }
}

impl<Out> Debug for Stream<Out> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Stream")
    }
}



// ===================
// === StreamInput ===
// ===================

#[derive(Clone)]
pub struct StreamInput<Input> {
    data : Weak<dyn EventConsumer<Input>>
}

impl<Def:NodeDefinition,Input> From<WeakNode<Def>> for StreamInput<Input>
where NodeData<Def> : EventConsumer<Input> {
    fn from(node:WeakNode<Def>) -> Self {
        Self {data:node.data}
    }
}

impl<Def:NodeDefinition,Input> From<&WeakNode<Def>> for StreamInput<Input>
    where NodeData<Def> : EventConsumer<Input> {
    fn from(node:&WeakNode<Def>) -> Self {
        Self {data:node.data.clone_ref()}
    }
}

impl<Input> Debug for StreamInput<Input> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"StreamInput")
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
    pub fn new() -> Self {
        let phantom    = default();
        let definition = NeverData {phantom};
        Self::construct(definition)
    }
}

impl<Out> EventEmitter for NodeData<NeverData<Out>>
where NeverData<Out> : NodeDefinition {
    default fn emit(&self, _value:&Self::Output) {}
    default fn register_target(&self, _tgt:StreamInput<Output<Self>>) {}
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
    pub fn new() -> Self {
        let phantom    = default();
        let definition = SourceData {phantom};
        Self::construct(definition)
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
    pub fn new<M,S>(message:M, stream:&S) -> Self
    where M:Into<String>, S:AnyStream<Output=Out> {
        let phantom = default();
        let message = message.into();
        let def     = TraceData {phantom,message};
        Self::construct_and_connect(stream,def)
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
    pub fn new<S:AnyStream>(stream:&S) -> Self {
        Self::new_with(stream,default())
    }

    /// Constructor with explicit start value.
    pub fn new_with<S:AnyStream>(stream:&S, init:bool) -> Self {
        let value = Cell::new(init);
        let def   = ToggleData {value};
        Self::construct_and_connect(stream,def)
    }
}

impl<T> EventConsumer<T> for NodeData<ToggleData> {
    fn on_event(&self, _:&T) {
        let value = !self.definition.value.get();
        self.definition.value.set(value);
        self.emit(&value);
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
    pub fn new<S>(stream:&S) -> Self
    where S:AnyStream {
        let value = default();
        let def   = CountData {value};
        Self::construct_and_connect(stream,def)
    }
}

impl<T> EventConsumer<T> for NodeData<CountData> {
    fn on_event(&self, _:&T) {
        let value = self.definition.value.get() + 1;
        self.definition.value.set(value);
        self.emit(&value);
    }
}


// ================
// === Previous ===
// ================

macro_rules! docs_for_previous { ($($tt:tt)*) => { #[doc="
Remembers the value of the input stream and outputs the previously received one.
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
    pub fn new<S>(stream:&S) -> Self
    where S : AnyStream<Output=Out> {
        let previous = default();
        let def      = PreviousData {previous};
        Self::construct_and_connect(stream,def)
    }
}

impl<Out:Value> EventConsumer<Out> for NodeData<PreviousData<Out>> {
    fn on_event(&self, event:&Out) {
        let previous = mem::replace(&mut *self.definition.previous.borrow_mut(),event.clone());
        self.emit(&previous);
    }
}



// ==============
// === Sample ===
// ==============

macro_rules! docs_for_sample { ($($tt:tt)*) => { #[doc="
Samples the first stream (behavior) on every incoming event of the second stream. The incoming event
is dropped and a new event with the behavior's value is emitted.
"]$($tt)* }}

docs_for_sample! { #[derive(Clone,Debug)]
pub struct SampleData <Behavior> { behavior:Behavior }}
pub type   Sample     <Behavior> = Node     <SampleData<Behavior>>;
pub type   WeakSample <Behavior> = WeakNode <SampleData<Behavior>>;

impl<Behavior:HasOutput> HasOutput for SampleData<Behavior> {
    type Output = Output<Behavior>;
}

impl<Behavior:AnyStream> Sample<Behavior> {
    /// Constructor.
    pub fn new<Event:AnyStream>(event:&Event, behavior:&Behavior) -> Self {
        let behavior   = behavior.clone_ref();
        let definition = SampleData {behavior};
        Self::construct_and_connect(event,definition)
    }
}

impl<T,Behavior:AnyStream> EventConsumer<T> for NodeData<SampleData<Behavior>> {
    fn on_event(&self, _:&T) {
        self.emit(&self.definition.behavior.last_value());
    }
}



// ============
// === Gate ===
// ============

macro_rules! docs_for_gate { ($($tt:tt)*) => { #[doc="
Passes the incoming event of the fisr stream only if the value of the second stream is `true`.
"]$($tt)* }}

docs_for_gate! { #[derive(Clone,Debug)]
pub struct GateData <T,B> { behavior:B, phantom:PhantomData<T> }}
pub type   Gate     <T,B> = Node     <GateData<T,B>>;
pub type   WeakGate <T,B> = WeakNode <GateData<T,B>>;

impl<T:Value,B> HasOutput for GateData<T,B> {
    type Output = T;
}

impl<T,B> Gate<T,B>
where T:Value, B:AnyStream<Output=bool> {
    /// Constructor.
    pub fn new<Event>(event:&Event,behavior:&B) -> Self
    where Event : AnyStream<Output=T> {
        let behavior   = behavior.clone_ref();
        let phantom    = default();
        let definition = GateData {behavior,phantom};
        Self::construct_and_connect(event,definition)
    }
}

impl<T,B> EventConsumer<T> for NodeData<GateData<T,B>>
where T:Value, B:AnyStream<Output=bool> {
    fn on_event(&self, event:&T) {
        if self.definition.behavior.last_value() {
            self.emit(event)
        }
    }
}



// =============
// === Merge ===
// =============

macro_rules! docs_for_merge { ($($tt:tt)*) => { #[doc="
Merges multiple input streams into a single output stream. All input streams have to share the same
output data type. Please note that `Merge` can be used to create recursive FRP networks by creating
an empty merge and using the `add` method to attach new streams to it.
"]$($tt)* }}

docs_for_merge! { #[derive(Clone,Copy,Debug)]
pub struct MergeData <Out> { phantom : PhantomData<Out> }}
pub type   Merge     <Out> = Node     <MergeData<Out>>;
pub type   WeakMerge <Out> = WeakNode <MergeData<Out>>;

impl<Out:Value> HasOutput for MergeData<Out> {
    type Output = Out;
}

impl<Out:Value> Merge<Out> {
    /// Constructor.
    pub fn new() -> Self {
        let phantom = default();
        let def     = MergeData {phantom};
        Self::construct(def)
    }

    pub fn with<S>(self, stream:&S) -> Self
        where S:AnyStream<Output=Out> {
        stream.register_target(self.downgrade().into());
        self
    }

    /// Constructor for 2 input streams.
    pub fn new2<S1,S2>(s1:&S1, s2:&S2) -> Self
        where S1:AnyStream<Output=Out>,
              S2:AnyStream<Output=Out> {
        Self::new().with(s1).with(s2)
    }

    /// Constructor for 3 input streams.
    pub fn new3<S1,S2,S3>(s1:&S1, s2:&S2, s3:&S3) -> Self
        where S1:AnyStream<Output=Out>,
              S2:AnyStream<Output=Out>,
              S3:AnyStream<Output=Out> {
        Self::new().with(s1).with(s2).with(s3)
    }

    /// Constructor for 4 input streams.
    pub fn new4<S1,S2,S3,S4>(s1:&S1, s2:&S2, s3:&S3, s4:&S4) -> Self
        where S1:AnyStream<Output=Out>,
              S2:AnyStream<Output=Out>,
              S3:AnyStream<Output=Out>,
              S4:AnyStream<Output=Out> {
        Self::new().with(s1).with(s2).with(s3).with(s4)
    }
}

impl<S1,Out> Add<&S1> for &Merge<Out>
    where S1:AnyStream<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, stream:&S1) -> Self::Output {
        stream.register_target(self.downgrade().into());
        self
    }
}

impl<S1,Out> Add<&S1> for &WeakMerge<Out>
    where S1:AnyStream<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, stream:&S1) -> Self::Output {
        stream.register_target(self.into());
        self
    }
}

impl<Out:Value> EventConsumer<Out> for NodeData<MergeData<Out>> {
    fn on_event(&self, event:&Out) {
        self.emit(event);
    }
}



// ============
// === Zip2 ===
// ============

macro_rules! docs_for_zip2 { ($($tt:tt)*) => { #[doc="
Merges two input streams into a stream containing values from both of them. On event from any of the
streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip2! { #[derive(Clone,Copy,Debug)]
pub struct Zip2Data <S1,S2> { stream1:S1, stream2:S2 }}
pub type   Zip2     <S1,S2> = Node     <Zip2Data<S1,S2>>;
pub type   WeakZip2 <S1,S2> = WeakNode <Zip2Data<S1,S2>>;

impl<S1,S2> HasOutput for Zip2Data<S1,S2>
    where S1:AnyStream, S2:AnyStream {
    type Output = (Output<S1>,Output<S2>);
}

impl<S1,S2> Zip2<S1,S2>
    where S1:AnyStream, S2:AnyStream {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let def     = Zip2Data {stream1,stream2};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.into());
        this
    }
}

impl<S1,S2,Out> EventConsumer<Out> for NodeData<Zip2Data<S1,S2>>
    where S1:AnyStream, S2:AnyStream {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        self.emit(&(value1,value2));
    }
}



// ============
// === Zip3 ===
// ============

macro_rules! docs_for_zip3 { ($($tt:tt)*) => { #[doc="
Merges three input streams into a stream containing values from all of them. On event from any of
the streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip3! { #[derive(Clone,Copy,Debug)]
pub struct Zip3Data <S1,S2,S3> { stream1:S1, stream2:S2, stream3:S3 }}
pub type   Zip3     <S1,S2,S3> = Node     <Zip3Data<S1,S2,S3>>;
pub type   WeakZip3 <S1,S2,S3> = WeakNode <Zip3Data<S1,S2,S3>>;

impl<S1,S2,S3> HasOutput for Zip3Data<S1,S2,S3>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream {
    type Output = (Output<S1>,Output<S2>,Output<S3>);
}

impl<S1,S2,S3> Zip3<S1,S2,S3>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let def     = Zip3Data {stream1,stream2,stream3};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,Out> EventConsumer<Out> for NodeData<Zip3Data<S1,S2,S3>>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        self.emit(&(value1,value2,value3));
    }
}



// ============
// === Zip4 ===
// ============

macro_rules! docs_for_zip4 { ($($tt:tt)*) => { #[doc="
Merges four input streams into a stream containing values from all of them. On event from any of the
streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip4! { #[derive(Clone,Copy,Debug)]
pub struct Zip4Data <S1,S2,S3,S4> { stream1:S1, stream2:S2, stream3:S3, stream4:S4 }}
pub type   Zip4     <S1,S2,S3,S4> = Node     <Zip4Data<S1,S2,S3,S4>>;
pub type   WeakZip4 <S1,S2,S3,S4> = WeakNode <Zip4Data<S1,S2,S3,S4>>;

impl<S1,S2,S3,S4> HasOutput for Zip4Data<S1,S2,S3,S4>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream {
    type Output = (Output<S1>,Output<S2>,Output<S3>,Output<S4>);
}

impl<S1,S2,S3,S4> Zip4<S1,S2,S3,S4>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3, s4:&S4) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let stream4 = s4.clone_ref();
        let def     = Zip4Data {stream1,stream2,stream3,stream4};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.clone_ref().into());
        s4.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,Out> EventConsumer<Out> for NodeData<Zip4Data<S1,S2,S3,S4>>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        let value4 = self.definition.stream4.last_value();
        self.emit(&(value1,value2,value3,value4));
    }
}



// ===========
// === Map ===
// ===========

macro_rules! docs_for_map { ($($tt:tt)*) => { #[doc="
On every event from the first input stream, sample all other input streams and run the provided
function on all gathered values. If you want to run the function on event from any input stream,
use the `apply` function family instead.
"]$($tt)* }}

docs_for_map! {
#[derive(Clone)]
pub struct MapData <S,F> { stream:S, function:F }}
pub type   Map     <S,F> = Node     <MapData<S,F>>;
pub type   WeakMap <S,F> = WeakNode <MapData<S,F>>;

impl<S,F,Out> HasOutput for MapData<S,F>
where S:AnyStream, Out:Value, F:'static+Fn(&Output<S>)->Out {
    type Output = Out;
}

impl<S,F,Out> Map<S,F>
where S:AnyStream, Out:Value, F:'static+Fn(&Output<S>)->Out {
    /// Constructor.
    pub fn new(s:&S, function:F) -> Self {
        let stream     = s.clone_ref();
        let definition = MapData {stream,function};
        Self::construct_and_connect(s,definition)
    }
}

impl<S,F,Out> EventConsumer<Output<S>> for NodeData<MapData<S,F>>
where S:AnyStream, Out:Value, F:'static+Fn(&Output<S>)->Out {
    fn on_event(&self, value:&Output<S>) {
        let out = (self.definition.function)(value);
        self.emit(&out);
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
pub struct Map2Data <S1,S2,F> { stream1:S1, stream2:S2, function:F }}
pub type   Map2     <S1,S2,F> = Node     <Map2Data<S1,S2,F>>;
pub type   WeakMap2 <S1,S2,F> = WeakNode <Map2Data<S1,S2,F>>;

impl<S1,S2,F,Out> HasOutput for Map2Data<S1,S2,F>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    type Output = Out;
}

impl<S1,S2,F,Out> Map2<S1,S2,F>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let def     = Map2Data {stream1,stream2,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,F,Out> EventConsumer<Output<S1>> for NodeData<Map2Data<S1,S2,F>>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.stream2.last_value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(&out);
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
pub struct Map3Data <S1,S2,S3,F> { stream1:S1, stream2:S2, stream3:S3, function:F }}
pub type   Map3     <S1,S2,S3,F> = Node     <Map3Data<S1,S2,S3,F>>;
pub type   WeakMap3 <S1,S2,S3,F> = WeakNode <Map3Data<S1,S2,S3,F>>;

impl<S1,S2,S3,F,Out> HasOutput for Map3Data<S1,S2,S3,F>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,F,Out> Map3<S1,S2,S3,F>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let def     = Map3Data {stream1,stream2,stream3,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,F,Out> EventConsumer<Output<S1>> for NodeData<Map3Data<S1,S2,S3,F>>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(&out);
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
pub struct Map4Data <S1,S2,S3,S4,F> { stream1:S1, stream2:S2, stream3:S3, stream4:S4, function:F }}
pub type   Map4     <S1,S2,S3,S4,F> = Node     <Map4Data<S1,S2,S3,S4,F>>;
pub type   WeakMap4 <S1,S2,S3,S4,F> = WeakNode <Map4Data<S1,S2,S3,S4,F>>;

impl<S1,S2,S3,S4,F,Out> HasOutput for Map4Data<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,S4,F,Out> Map4<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3, s4:&S4, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let stream4 = s4.clone_ref();
        let def     = Map4Data {stream1,stream2,stream3,stream4,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,F,Out> EventConsumer<Output<S1>> for NodeData<Map4Data<S1,S2,S3,S4,F>>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    fn on_event(&self, value1:&Output<S1>) {
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        let value4 = self.definition.stream4.last_value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(&out);
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
On every input event sample all input streams and run the provided function on all gathered values.
If you want to run the function only on event on the first input, use the `map` function family
instead.
"]$($tt)* }}

docs_for_apply! { #[derive(Clone)]
pub struct Apply2Data <S1,S2,F> { stream1:S1, stream2:S2, function:F }}
pub type   Apply2     <S1,S2,F> = Node     <Apply2Data<S1,S2,F>>;
pub type   WeakApply2 <S1,S2,F> = WeakNode <Apply2Data<S1,S2,F>>;

impl<S1,S2,F,Out> HasOutput for Apply2Data<S1,S2,F>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    type Output = Out;
}

impl<S1,S2,F,Out> Apply2<S1,S2,F>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let def     = Apply2Data {stream1,stream2,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.into());
        this
    }
}

impl<S1,S2,F,Out,T> EventConsumer<T> for NodeData<Apply2Data<S1,S2,F>>
where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(&out);
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
pub struct Apply3Data <S1,S2,S3,F> { stream1:S1, stream2:S2, stream3:S3, function:F }}
pub type   Apply3     <S1,S2,S3,F> = Node     <Apply3Data<S1,S2,S3,F>>;
pub type   WeakApply3 <S1,S2,S3,F> = WeakNode <Apply3Data<S1,S2,S3,F>>;

impl<S1,S2,S3,F,Out> HasOutput for Apply3Data<S1,S2,S3,F>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,F,Out> Apply3<S1,S2,S3,F>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let def     = Apply3Data {stream1,stream2,stream3,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,F,Out,T> EventConsumer<T> for NodeData<Apply3Data<S1,S2,S3,F>>
where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
      F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(&out);
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
pub struct Apply4Data <S1,S2,S3,S4,F> {stream1:S1, stream2:S2, stream3:S3, stream4:S4, function:F}}
pub type   Apply4     <S1,S2,S3,S4,F> = Node     <Apply4Data<S1,S2,S3,S4,F>>;
pub type   WeakApply4 <S1,S2,S3,S4,F> = WeakNode <Apply4Data<S1,S2,S3,S4,F>>;

impl<S1,S2,S3,S4,F,Out> HasOutput for Apply4Data<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    type Output = Out;
}

impl<S1,S2,S3,S4,F,Out> Apply4<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    /// Constructor.
    pub fn new(s1:&S1, s2:&S2, s3:&S3, s4:&S4, function:F) -> Self {
        let stream1 = s1.clone_ref();
        let stream2 = s2.clone_ref();
        let stream3 = s3.clone_ref();
        let stream4 = s4.clone_ref();
        let def     = Apply4Data {stream1,stream2,stream3,stream4,function};
        let this    = Self::construct(def);
        let weak    = this.downgrade();
        s1.register_target(weak.clone_ref().into());
        s2.register_target(weak.clone_ref().into());
        s3.register_target(weak.clone_ref().into());
        s4.register_target(weak.into());
        this
    }
}

impl<S1,S2,S3,S4,F,Out,T> EventConsumer<T> for NodeData<Apply4Data<S1,S2,S3,S4,F>>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.stream1.last_value();
        let value2 = self.definition.stream2.last_value();
        let value3 = self.definition.stream3.last_value();
        let value4 = self.definition.stream4.last_value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(&out);
    }
}

impl<S1,S2,S3,S4,F> Debug for Apply4Data<S1,S2,S3,S4,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply4Data")
    }
}




// =============
// === Graph ===
// =============

// === Definition ===

#[derive(Debug)]
pub struct GraphData {
    nodes : RefCell<Vec<Box<dyn Any>>>
}

#[derive(Clone,CloneRef,Debug)]
pub struct Graph {
    data : Rc<GraphData>
}

#[derive(Clone,CloneRef,Debug)]
pub struct WeakGraph {
    data : Weak<GraphData>
}


// === API ===

impl GraphData {
    /// Constructor.
    pub fn new() -> Self {
        let nodes = default();
        Self {nodes}
    }
}

impl Graph {
    /// Constructor.
    pub fn new() -> Self {
        let data = Rc::new(GraphData::new());
        Self {data}
    }

    pub fn downgrade(&self) -> WeakGraph {
        WeakGraph {data:Rc::downgrade(&self.data)}
    }

    pub fn register<T:NodeDefinition>(&self, node:Node<T>) -> WeakNode<T> {
        let weak = node.downgrade();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        weak
    }
}

impl WeakGraph {
    pub fn upgrade(&self) -> Option<Graph> {
        self.data.upgrade().map(|data| Graph {data})
    }
}

impl Graph {
    docs_for_never! {
    pub fn never<T:Value>(&self) -> WeakNever<T> {
        self.register(Never::new())
    }}

    docs_for_source! {
    pub fn source<T:Value>(&self) -> WeakSource<T> {
        self.register(Source::new())
    }}

    docs_for_source! {
    pub fn source_(&self) -> WeakSource<()> {
        self.register(Source::new())
    }}

    docs_for_trace! {
    pub fn trace<M,S,Out>(&self, message:M, stream:&S) -> WeakTrace<Out>
    where M:Into<String>, S:AnyStream<Output=Out>, Out:Value {
        self.register(Trace::new(message,stream))
    }}

    docs_for_toggle! {
    pub fn toggle<S:AnyStream>(&self, stream:&S) -> WeakToggle {
        self.register(Toggle::new(stream))
    }}

    docs_for_count! {
    pub fn count<S:AnyStream>(&self, stream:&S) -> WeakCount {
        self.register(Count::new(stream))
    }}

    docs_for_previous! {
    pub fn previous<S,Out> (&self, stream:&S) -> WeakPrevious<Out>
    where S:AnyStream<Output=Out>, Out:Value {
        self.register(Previous::new(stream))
    }}

    docs_for_sample! {
    pub fn sample<E:AnyStream,B:AnyStream>(&self, event:&E, behavior:&B) -> WeakSample<B> {
        self.register(Sample::new(event,behavior))
    }}

    docs_for_gate! {
    pub fn gate<T,E,B>(&self, event:&E, behavior:&B) -> WeakGate<T,B>
    where T:Value, E:AnyStream<Output=T>, B:AnyStream<Output=bool> {
        self.register(Gate::new(event,behavior))
    }}


    // === Merge ===

    docs_for_merge! {
    pub fn merge<Out:Value>(&self) -> WeakMerge<Out> {
        self.register(Merge::new())
    }}

    docs_for_merge! {
    pub fn merge2<S1,S2,Out:Value>(&self, s1:&S2, s2:&S2) -> WeakMerge<Out>
    where S1:AnyStream<Output=Out>, S2:AnyStream<Output=Out> {
        self.register(Merge::new2(s1,s2))
    }}

    docs_for_merge! {
    pub fn merge3<S1,S2,S3,Out:Value>(&self, s1:&S2, s2:&S2, s3:&S3) -> WeakMerge<Out>
    where S1:AnyStream<Output=Out>, S2:AnyStream<Output=Out>, S3:AnyStream<Output=Out> {
        self.register(Merge::new3(s1,s2,s3))
    }}

    docs_for_merge! {
    pub fn merge4<S1,S2,S3,S4,Out:Value>(&self, s1:&S2, s2:&S2, s3:&S3, s4:&S4) -> WeakMerge<Out>
    where S1:AnyStream<Output=Out>,
          S2:AnyStream<Output=Out>,
          S3:AnyStream<Output=Out>,
          S4:AnyStream<Output=Out> {
        self.register(Merge::new4(s1,s2,s3,s4))
    }}


    // === Zip ===

    docs_for_zip2! {
    pub fn zip<S1,S2>(&self, stream1:&S1, stream2:&S2) -> WeakZip2<S1,S2>
    where S1:AnyStream, S2:AnyStream {
        self.register(Zip2::new(stream1,stream2))
    }}

    docs_for_zip2! {
    pub fn zip2<S1,S2>(&self, stream1:&S1, stream2:&S2) -> WeakZip2<S1,S2>
    where S1:AnyStream, S2:AnyStream {
        self.register(Zip2::new(stream1,stream2))
    }}

    docs_for_zip3! {
    pub fn zip3<S1,S2,S3>(&self, stream1:&S1, stream2:&S2, stream3:&S3) -> WeakZip3<S1,S2,S3>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream {
        self.register(Zip3::new(stream1,stream2,stream3))
    }}

    docs_for_zip4! {
    pub fn zip4<S1,S2,S3,S4>
    (&self, stream1:&S1, stream2:&S2, stream3:&S3, stream4:&S4) -> WeakZip4<S1,S2,S3,S4>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream {
        self.register(Zip4::new(stream1,stream2,stream3,stream4))
    }}


    // === Map ===

    docs_for_map! {
    pub fn map<S,F,Out>(&self, source:&S, f:F) -> WeakMap<S,F>
    where S:AnyStream, Out:Value, F:'static+Fn(&Output<S>)->Out {
        self.register(Map::new(source,f))
    }}

    docs_for_map! {
    pub fn map2<S1,S2,F,Out>(&self, s1:&S1, s2:&S2, f:F) -> WeakMap2<S1,S2,F>
    where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
        self.register(Map2::new(s1,s2,f))
    }}

    docs_for_map! {
    pub fn map3<S1,S2,S3,F,Out>(&self, s1:&S1, s2:&S2, s3:&S3, f:F) -> WeakMap3<S1,S2,S3,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
        self.register(Map3::new(s1,s2,s3,f))
    }}

    docs_for_map! {
    pub fn map4<S1,S2,S3,S4,F,Out>
    (&self, s1:&S1, s2:&S2, s3:&S3, s4:&S4, f:F) -> WeakMap4<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
        self.register(Map4::new(s1,s2,s3,s4,f))
    }}


    // === Apply ===

    docs_for_apply! {
    pub fn apply2<S1,S2,F,Out>(&self, s1:&S1, s2:&S2, f:F) -> WeakApply2<S1,S2,F>
    where S1:AnyStream, S2:AnyStream, Out:Value, F:'static+Fn(&Output<S1>,&Output<S2>)->Out {
        self.register(Apply2::new(s1,s2,f))
    }}

    docs_for_apply! {
    pub fn apply3<S1,S2,S3,F,Out>(&self, s1:&S1, s2:&S2, s3:&S3, f:F) -> WeakApply3<S1,S2,S3,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>)->Out {
        self.register(Apply3::new(s1,s2,s3,f))
    }}

    docs_for_apply! {
    pub fn apply4<S1,S2,S3,S4,F,Out>
    (&self, s1:&S1, s2:&S2, s3:&S3, s4:&S4, f:F) -> WeakApply4<S1,S2,S3,S4,F>
    where S1:AnyStream, S2:AnyStream, S3:AnyStream, S4:AnyStream, Out:Value,
          F:'static+Fn(&Output<S1>,&Output<S2>,&Output<S3>,&Output<S4>)->Out {
        self.register(Apply4::new(s1,s2,s3,s4,f))
    }}
}


///////////////////////////////////

#[allow(unused_variables)]
pub fn test() {
    println!("hello");
    let frp  = Graph::new();
    let source  = frp.source::<f32>();
    let source2 = frp.source::<()>();
    let tg     = frp.toggle(&source);
    let fff    = frp.map(&tg,|t| { println!("{:?}",t) });
    let bb     = frp.sample(&source2,&tg);
    let bb2 : Stream<bool> = bb.into();
    let fff2   = frp.map(&bb2,|t| { println!(">> {:?}",t) });

    println!("{:?}",tg);

    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
}
