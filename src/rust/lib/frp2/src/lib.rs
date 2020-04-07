//! This module implements an Functional Reactive Programming system. It is an advanced event
//! handling framework which allows describing events and actions by creating declarative event
//! flow diagrams.
//!
//! Please read this document as the initial introduction to FRP concepts:
//! https://github.com/hansroland/reflex-dom-inbits/blob/master/tutorial.md
//!
//! The architecture of the FRP engine is pretty complex and it is hard to understand all the
//! dependencies by reading the code only. In order to make it easier, the following diagram was
//! prepared. It visualizes a simple FRP network of `source -> count -> map(|t| t.to_string())`:
//!
//! - Node colors indicate to which network the nodes belong (nodes of that color will be dropped as
//!   soon as the network gets dropped). Nodes with double color will be dropped as soon as an event
//!   will be emitted to nodes from a dropped network.
//! - Black solid edges without arrows are just fields inside structure.
//! - Black solid edges with arrows are `Rc` pointers.
//! - Black dashed edges with arrows are `Weak` pointers.
//! - Red solid edges with arrows are trait object interfaces.
//!
//! ```dot
//! digraph G {
//!     layout=neato
//!     node [shape=box style=rounded]
//!     Network1[pos="-3,-1!",color=blue]
//!     Network2[pos="-3,-5.5!",color=crimson]
//!     Network1 -> node_1
//!     Network2 -> node_2
//!     Network2 -> node_3
//!
//!     node_1[label="Source<()>", pos="-0.5,-1!", color=blue]
//!     SourceData[pos="-0.5,0!",color=blue]
//!     node_1 -> SourceData
//!     node_data_1[label="NodeData<()>",pos="1.5,-1!",color=blue]
//!     node_1 -> node_data_1
//!     flow_1[label="Flow<()>", pos="3.5,-1!"]
//!     flow_1 -> node_data_1 [style=dashed]
//!
//!     FlowInput_1[label="FlowInput<()>", pos="1.5,-2!",color=blue]
//!     FlowInput_1_[pos="1.5,-2!",label="",color=crimson, width=1.49, height=0.6]
//!     node_data_1 -> FlowInput_1 [arrowhead=none]
//!     WeakCount_[pos="1.5,-3!",label="",color=crimson, width=1.25, height=0.6]
//!     WeakCount[pos="1.5,-3!",color=blue]
//!     FlowInput_1 -> WeakCount [color=red]
//!     node_2[label="Count",pos="-0.5,-4!",color=crimson]
//!     CountData[pos="-0.5,-3!",color=crimson]
//!     node_2 -> CountData
//!     node_data_2[label="NodeData<usize>",pos="1.5,-4!",color=crimson]
//!     node_2 -> node_data_2
//!     WeakCount -> node_data_2 [style=dashed]
//!     WeakCount -> CountData
//!     flow_2[label="Flow<usize>",pos="3.5,-4!"]
//!     flow_2 -> node_data_2 [style=dashed]
//!
//!     FlowInput_2[label="FlowInput<usize>", pos="1.5,-5!",color=crimson]
//!     node_data_2 -> FlowInput_2
//!     WeakMap[pos="1.5,-6!",color=crimson]
//!     FlowInput_2 -> WeakMap [color=red]
//!     node_3[label="Map<Count,[f]>",pos="-0.5,-7!",color=crimson]
//!     MapData[pos="-0.5,-6!",color=crimson]
//!     node_3 -> MapData
//!     node_data_3[label="NodeData<String>",pos="1.5,-7!",color=crimson]
//!     node_3 -> node_data_3
//!     WeakMap -> node_data_3 [style=dashed] [weight=10]
//!     WeakMap -> MapData
//!     flow_3[label="Flow<String>", pos="3.5,-7!"]
//!     flow_3 -> node_data_3 [style=dashed]
//! }
//! ```

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
use ensogl_system_web as web;





#[derive(Debug,Clone,CloneRef,Copy,Eq,From,Hash,Into,PartialEq)]
pub struct Id {
    raw : usize
}

pub trait HasId {
    fn id(&self) -> Id;
}

pub trait HasTypeLabel {
    fn type_label(&self) -> String;
}


// =============
// === Debug ===
// =============

pub trait InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link>;
}


impl<T> InputBehaviors for T {
    default fn input_behaviors(&self) -> Vec<Link> {
        vec![]
    }
}










type Label = &'static str;

pub trait HasLabel {
    fn label(&self) -> Label;
}


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


pub trait AnyFlow = 'static + ValueProvider + EventEmitter + CloneRef + HasId;

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
    fn register_target (&self , target:FlowInput<Output<Self>>);
}

pub trait EventConsumer<T> {
    fn on_event(&self, value:&T);
}

pub trait WeakEventConsumer<T> {
    /// Consumes the event and return true if existed. Returns false if it was already dropped.
    fn on_event_if_exists(&self, value:&T) -> bool;
}


pub trait ValueProvider : HasOutput {
    fn value(&self) -> Self::Output;
}

pub trait EventConsumerDebug<T> : EventConsumer<T> + Debug {}
impl<X,T> EventConsumerDebug<T> for X where X : EventConsumer<T> + Debug {}



// ================
// === NodeData ===
// ================

#[derive(Debug)]
pub struct NodeData<Out=()> {
    label       : Label,
    targets     : RefCell<Vec<FlowInput<Out>>>,
    value       : RefCell<Out>,
    during_call : Cell<bool>,
}

impl<Out:Default> NodeData<Out> {
    pub fn new(label:Label) -> Self {
        let targets     = default();
        let value       = default();
        let during_call = default();
        Self {label,targets,value,during_call}
    }
}

impl<Out:Value> HasOutput for NodeData<Out> {
    type Output = Out;
}

impl<Out:Value> EventEmitter for NodeData<Out> {
    fn emit_event(&self, value:&Out) {
        if !self.during_call.get() {
            self.during_call.set(true);
            *self.value.borrow_mut() = value.clone();
            self.targets.borrow_mut().retain(|target| target.data.on_event_if_exists(value));
            self.during_call.set(false);
        }
    }

    fn register_target(&self,target:FlowInput<Out>) {
        self.targets.borrow_mut().push(target)
    }
}

impl<Out:Value> ValueProvider for NodeData<Out> {
    fn value(&self) -> Out {
        self.value.borrow().clone()
    }
}


// ============
// === Node ===
// ============

// === Types ===

pub trait NodeDefinition = 'static + ?Sized + HasOutput;

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Node<Def:NodeDefinition> {
    data       : Rc<NodeData<Output<Def>>>,
    definition : Rc<Def>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakNode<Def:NodeDefinition> {
    data       : Weak<NodeData<Output<Def>>>,
    definition : Rc<Def>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Flow<Out=()> {
    data : Weak<NodeData<Out>>,
}


// === Output ===

impl<Def:NodeDefinition> HasOutput for Node     <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for WeakNode <Def> { type Output = Output<Def>; }


// === Node Impls ===

impl<Def:NodeDefinition> Node<Def> {
    pub fn construct(label:Label, definition:Def) -> Self {
        let data       = Rc::new(NodeData::new(label));
        let definition = Rc::new(definition);
        Self {data,definition}
    }

    pub fn construct_and_connect<S>(label:Label, flow:&S, definition:Def) -> Self
    where S:AnyFlow, Self:EventConsumer<Output<S>> {
        let this = Self::construct(label,definition);
        let weak = this.downgrade();
        flow.register_target(weak.into());
        this
    }

    pub fn downgrade(&self) -> WeakNode<Def> {
        let data       = Rc::downgrade(&self.data);
        let definition = self.definition.clone_ref();
        WeakNode {data,definition}
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

impl<Def:NodeDefinition> ValueProvider for Node<Def> {
    fn value(&self) -> Self::Output {
        self.data.value.borrow().clone()
    }
}

impl<Def:NodeDefinition> HasId for Node<Def> {
    fn id(&self) -> Id {
        self.downgrade().id()
    }
}

impl<Def:NodeDefinition> InputBehaviors for Node<Def>
where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![] // FIXME
//        self.data.input_behaviors()
    }
}

impl<Def:NodeDefinition> HasLabel for Node<Def>
where Def:InputBehaviors {
    fn label(&self) -> Label {
        self.data.label
    }
}

// FIXME code quality below:
impl<Def:NodeDefinition> HasTypeLabel for Node<Def>
where Def:InputBehaviors {
    fn type_label(&self) -> String {
        let label = type_name::<Def>().to_string();
        let label = label.split(|c| c == '<').collect::<Vec<_>>()[0];
        let mut label = label.split(|c| c == ':').collect::<Vec<_>>();
        label.reverse();
        let mut label = label[0];
        let sfx = "Data";
        if label.ends_with(sfx) {
            label = &label[0..label.len()-sfx.len()];
        }
        label.into()
    }
}


// === WeakNode Impls ===

impl<T:NodeDefinition> WeakNode<T> {
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.data.upgrade().map(|data| {
            let definition = self.definition.clone_ref();
            Node{data,definition}
        })
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

impl<Def:NodeDefinition> ValueProvider for WeakNode<Def> {
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

impl<Def:NodeDefinition> HasId for WeakNode<Def> {
    fn id(&self) -> Id {
        let raw = self.data.as_raw() as *const() as usize;
        raw.into()
    }
}

impl<Def:NodeDefinition> InputBehaviors for WeakNode<Def>
    where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        self.data.upgrade().map(|t| t.input_behaviors()).unwrap_or_default()
    }
}

impl<Def:NodeDefinition,T> WeakEventConsumer<T> for WeakNode<Def>
where Node<Def> : EventConsumer<T> {
    fn on_event_if_exists(&self, value:&T) -> bool {
        self.upgrade().map(|node| {node.on_event(value);}).is_some()
    }
}



// ============
// === Flow ===
// ============

impl<Def:NodeDefinition> From<Node<Def>> for Flow<Def::Output> {
    fn from(node:Node<Def>) -> Self {
        let data = Rc::downgrade(&node.data);
        Flow {data}
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

impl<Out:Value> ValueProvider for Flow<Out> {
    fn value(&self) -> Self::Output {
        self.data.upgrade().map(|t| t.value()).unwrap_or_default()
    }
}

impl<Out> Debug for Flow<Out> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Flow")
    }
}

impl<Out> HasId for Flow<Out> {
    fn id(&self) -> Id {
        0.into() // FIXME
//        let raw = Rc::downgrade(&self.value).as_raw() as *const() as usize;
//        raw.into()
    }
}



// =================
// === FlowInput ===
// =================

#[derive(Clone)]
pub struct FlowInput<Input> {
    data : Rc<dyn WeakEventConsumer<Input>>
}

impl<Def:NodeDefinition,Input> From<WeakNode<Def>> for FlowInput<Input>
where Node<Def> : EventConsumer<Input> {
    fn from(node:WeakNode<Def>) -> Self {
        Self {data:Rc::new(node)}
    }
}

impl<Def:NodeDefinition,Input> From<&WeakNode<Def>> for FlowInput<Input>
    where Node<Def> : EventConsumer<Input> {
    fn from(node:&WeakNode<Def>) -> Self {
        Self {data:Rc::new(node.clone_ref())}
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

// FIXME
//impl<Out> EventEmitter for Node<NeverData<Out>>
//where NeverData<Out> : NodeDefinition {
//    fn emit_event(&self, _value:&Output<Self>) {}
//    fn register_target(&self, _tgt:FlowInput<Output<Self>>) {}
//}



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

impl<Out:Value> EventConsumer<Out> for Node<TraceData<Out>> {
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

impl<T> EventConsumer<T> for Node<ToggleData> {
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

docs_for_count! { #[derive(Debug)]
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

impl<T> EventConsumer<T> for Node<CountData> {
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

impl<Out:Value,T> EventConsumer<T> for Node<ConstantData<Out>> {
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

impl<Out:Value> EventConsumer<Out> for Node<PreviousData<Out>> {
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

impl<T,Behavior:AnyFlow> EventConsumer<T> for Node<SampleData<Behavior>> {
    fn on_event(&self, _:&T) {
        self.emit(self.definition.behavior.value());
    }
}

impl<B> InputBehaviors for SampleData<B>
    where B:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
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

impl<T,B> EventConsumer<T> for Node<GateData<T,B>>
where T:Value, B:AnyFlow<Output=bool> {
    fn on_event(&self, event:&T) {
        if self.definition.behavior.value() {
            self.emit(event)
        }
    }
}

impl<T,B> InputBehaviors for GateData<T,B>
where B:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
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
        let def         = MergeData {phantom,during_call};
        Self::construct(label,def)
    }

    /// Takes ownership of self and returns it with a new flow attached.
    pub fn with<S>(self, flow:&S) -> Self
        where S:AnyFlow<Output=Out> {
        flow.register_target(self.downgrade().into());
        self
    }

    /// Constructor for 1 input flow.
    pub fn new1<F1>(label:Label, f1:&F1) -> Self
        where F1:AnyFlow<Output=Out> {
        Self::new(label).with(f1)
    }

    /// Constructor for 2 input flows.
    pub fn new2<F1,F2>(label:Label, f1:&F1, f2:&F2) -> Self
        where F1:AnyFlow<Output=Out>,
              F2:AnyFlow<Output=Out> {
        Self::new(label).with(f1).with(f2)
    }

    /// Constructor for 3 input flows.
    pub fn new3<F1,F2,F3>(label:Label, f1:&F1, f2:&F2, f3:&F3) -> Self
        where F1:AnyFlow<Output=Out>,
              F2:AnyFlow<Output=Out>,
              F3:AnyFlow<Output=Out> {
        Self::new(label).with(f1).with(f2).with(f3)
    }

    /// Constructor for 4 input flows.
    pub fn new4<F1,F2,F3,F4>(label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4) -> Self
        where F1:AnyFlow<Output=Out>,
              F2:AnyFlow<Output=Out>,
              F3:AnyFlow<Output=Out>,
              F4:AnyFlow<Output=Out> {
        Self::new(label).with(f1).with(f2).with(f3).with(f4)
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

impl<F1,Out> Add<&F1> for &Merge<Out>
    where F1:AnyFlow<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, flow:&F1) -> Self::Output {
        flow.register_target(self.downgrade().into());
        self
    }
}

impl<F1,Out> Add<&F1> for &WeakMerge<Out>
    where F1:AnyFlow<Output=Out>, Out:Value {
    type Output = Self;
    fn add(self, flow:&F1) -> Self::Output {
        flow.register_target(self.into());
        self
    }
}

impl<Out:Value> EventConsumer<Out> for Node<MergeData<Out>> {
    fn on_event(&self, event:&Out) {
        self.emit(event);
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
pub struct Zip2Data <F1,F2> { flow1:F1, flow2:F2 }}
pub type   Zip2     <F1,F2> = Node     <Zip2Data<F1,F2>>;
pub type   WeakZip2 <F1,F2> = WeakNode <Zip2Data<F1,F2>>;

impl<F1,F2> HasOutput for Zip2Data<F1,F2>
    where F1:AnyFlow, F2:AnyFlow {
    type Output = (Output<F1>,Output<F2>);
}

impl<F1,F2> Zip2<F1,F2>
    where F1:AnyFlow, F2:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let def   = Zip2Data {flow1,flow2};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.into());
        this
    }
}

impl<F1,F2,Out> EventConsumer<Out> for Node<Zip2Data<F1,F2>>
    where F1:AnyFlow, F2:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        self.emit((value1,value2));
    }
}

impl<F1,F2> InputBehaviors for Zip2Data<F1,F2>
    where F1:AnyFlow, F2:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.flow1), Link::mixed(&self.flow2)]
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
pub struct Zip3Data <F1,F2,F3> { flow1:F1, flow2:F2, flow3:F3 }}
pub type   Zip3     <F1,F2,F3> = Node     <Zip3Data<F1,F2,F3>>;
pub type   WeakZip3 <F1,F2,F3> = WeakNode <Zip3Data<F1,F2,F3>>;

impl<F1,F2,F3> HasOutput for Zip3Data<F1,F2,F3>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
    type Output = (Output<F1>,Output<F2>,Output<F3>);
}

impl<F1,F2,F3> Zip3<F1,F2,F3>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let def   = Zip3Data {flow1,flow2,flow3};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.clone_ref().into());
        f3.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,Out> EventConsumer<Out> for Node<Zip3Data<F1,F2,F3>>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        self.emit((value1,value2,value3));
    }
}

impl<F1,F2,F3> InputBehaviors for Zip3Data<F1,F2,F3>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.flow1), Link::mixed(&self.flow2), Link::mixed(&self.flow3)]
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
pub struct Zip4Data <F1,F2,F3,F4> { flow1:F1, flow2:F2, flow3:F3, flow4:F4 }}
pub type   Zip4     <F1,F2,F3,F4> = Node     <Zip4Data<F1,F2,F3,F4>>;
pub type   WeakZip4 <F1,F2,F3,F4> = WeakNode <Zip4Data<F1,F2,F3,F4>>;

impl<F1,F2,F3,F4> HasOutput for Zip4Data<F1,F2,F3,F4>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
    type Output = (Output<F1>,Output<F2>,Output<F3>,Output<F4>);
}

impl<F1,F2,F3,F4> Zip4<F1,F2,F3,F4>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let flow4 = f4.clone_ref();
        let def   = Zip4Data {flow1,flow2,flow3,flow4};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.clone_ref().into());
        f3.register_target(weak.clone_ref().into());
        f4.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,F4,Out> EventConsumer<Out> for Node<Zip4Data<F1,F2,F3,F4>>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
    fn on_event(&self, _:&Out) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        self.emit((value1,value2,value3,value4));
    }
}

impl<F1,F2,F3,F4> InputBehaviors for Zip4Data<F1,F2,F3,F4>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![ Link::mixed(&self.flow1)
            , Link::mixed(&self.flow2)
            , Link::mixed(&self.flow3)
            , Link::mixed(&self.flow4)
            ]
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
pub struct MapData <F1,F> { flow:F1, function:F }}
pub type   Map     <F1,F> = Node     <MapData<F1,F>>;
pub type   WeakMap <F1,F> = WeakNode <MapData<F1,F>>;

impl<F1,F,Out> HasOutput for MapData<F1,F>
where F1:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>)->Out {
    type Output = Out;
}

impl<F1,F,Out> Map<F1,F>
where F1:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, function:F) -> Self {
        let flow       = f1.clone_ref();
        let definition = MapData {flow,function};
        Self::construct_and_connect(label,f1,definition)
    }
}

impl<F1,F,Out> EventConsumer<Output<F1>> for Node<MapData<F1,F>>
where F1:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>)->Out {
    fn on_event(&self, value:&Output<F1>) {
        let out = (self.definition.function)(value);
        self.emit(out);
    }
}

impl<F1,F> Debug for MapData<F1,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapData")
    }
}



// ============
// === Map2 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map2Data <F1,F2,F> { flow1:F1, flow2:F2, function:F }}
pub type   Map2     <F1,F2,F> = Node     <Map2Data<F1,F2,F>>;
pub type   WeakMap2 <F1,F2,F> = WeakNode <Map2Data<F1,F2,F>>;

impl<F1,F2,F,Out> HasOutput for Map2Data<F1,F2,F>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    type Output = Out;
}

impl<F1,F2,F,Out> Map2<F1,F2,F>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let def   = Map2Data {flow1,flow2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.into());
        this
    }
}

impl<F1,F2,F,Out> EventConsumer<Output<F1>> for Node<Map2Data<F1,F2,F>>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    fn on_event(&self, value1:&Output<F1>) {
        let value2 = self.definition.flow2.value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<F1,F2,F> Debug for Map2Data<F1,F2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map2Data")
    }
}

impl<F1,F2,F> InputBehaviors for Map2Data<F1,F2,F>
    where F1:AnyFlow, F2:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.flow2)]
    }
}



// ============
// === Map3 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map3Data <F1,F2,F3,F> { flow1:F1, flow2:F2, flow3:F3, function:F }}
pub type   Map3     <F1,F2,F3,F> = Node     <Map3Data<F1,F2,F3,F>>;
pub type   WeakMap3 <F1,F2,F3,F> = WeakNode <Map3Data<F1,F2,F3,F>>;

impl<F1,F2,F3,F,Out> HasOutput for Map3Data<F1,F2,F3,F>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    type Output = Out;
}

impl<F1,F2,F3,F,Out> Map3<F1,F2,F3,F>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let def   = Map3Data {flow1,flow2,flow3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,F,Out> EventConsumer<Output<F1>> for Node<Map3Data<F1,F2,F3,F>>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    fn on_event(&self, value1:&Output<F1>) {
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<F1,F2,F3,F> Debug for Map3Data<F1,F2,F3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map3Data")
    }
}

impl<F1,F2,F3,F> InputBehaviors for Map3Data<F1,F2,F3,F>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.flow2), Link::behavior(&self.flow3)]
    }
}



// ============
// === Map4 ===
// ============

docs_for_map! {
#[derive(Clone)]
pub struct Map4Data <F1,F2,F3,F4,F> { flow1:F1, flow2:F2, flow3:F3, flow4:F4, function:F }}
pub type   Map4     <F1,F2,F3,F4,F> = Node     <Map4Data<F1,F2,F3,F4,F>>;
pub type   WeakMap4 <F1,F2,F3,F4,F> = WeakNode <Map4Data<F1,F2,F3,F4,F>>;

impl<F1,F2,F3,F4,F,Out> HasOutput for Map4Data<F1,F2,F3,F4,F>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    type Output = Out;
}

impl<F1,F2,F3,F4,F,Out> Map4<F1,F2,F3,F4,F>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let flow4 = f4.clone_ref();
        let def   = Map4Data {flow1,flow2,flow3,flow4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,F4,F,Out> EventConsumer<Output<F1>> for Node<Map4Data<F1,F2,F3,F4,F>>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    fn on_event(&self, value1:&Output<F1>) {
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<F1,F2,F3,F4,F> InputBehaviors for Map4Data<F1,F2,F3,F4,F>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.flow2), Link::behavior(&self.flow3), Link::behavior(&self.flow4)]
    }
}

impl<F1,F2,F3,F4,F> Debug for Map4Data<F1,F2,F3,F4,F> {
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
pub struct Apply2Data <F1,F2,F> { flow1:F1, flow2:F2, function:F }}
pub type   Apply2     <F1,F2,F> = Node     <Apply2Data<F1,F2,F>>;
pub type   WeakApply2 <F1,F2,F> = WeakNode <Apply2Data<F1,F2,F>>;

impl<F1,F2,F,Out> HasOutput for Apply2Data<F1,F2,F>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    type Output = Out;
}

impl<F1,F2,F,Out> Apply2<F1,F2,F>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let def   = Apply2Data {flow1,flow2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.into());
        this
    }
}

impl<F1,F2,F,Out,T> EventConsumer<T> for Node<Apply2Data<F1,F2,F>>
where F1:AnyFlow, F2:AnyFlow, Out:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let out    = (self.definition.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<F1,F2,F> Debug for Apply2Data<F1,F2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply2Data")
    }
}



// ==============
// === Apply3 ===
// ==============

docs_for_apply! { #[derive(Clone)]
pub struct Apply3Data <F1,F2,F3,F> { flow1:F1, flow2:F2, flow3:F3, function:F }}
pub type   Apply3     <F1,F2,F3,F> = Node     <Apply3Data<F1,F2,F3,F>>;
pub type   WeakApply3 <F1,F2,F3,F> = WeakNode <Apply3Data<F1,F2,F3,F>>;

impl<F1,F2,F3,F,Out> HasOutput for Apply3Data<F1,F2,F3,F>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    type Output = Out;
}

impl<F1,F2,F3,F,Out> Apply3<F1,F2,F3,F>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let def   = Apply3Data {flow1,flow2,flow3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.clone_ref().into());
        f3.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,F,Out,T> EventConsumer<T> for Node<Apply3Data<F1,F2,F3,F>>
where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, Out:Value,
      F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let out    = (self.definition.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<F1,F2,F3,F> Debug for Apply3Data<F1,F2,F3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply3Data")
    }
}



// ==============
// === Apply4 ===
// ==============

docs_for_apply! { #[derive(Clone)]
pub struct Apply4Data <F1,F2,F3,F4,F> {flow1:F1, flow2:F2, flow3:F3, flow4:F4, function:F}}
pub type   Apply4     <F1,F2,F3,F4,F> = Node     <Apply4Data<F1,F2,F3,F4,F>>;
pub type   WeakApply4 <F1,F2,F3,F4,F> = WeakNode <Apply4Data<F1,F2,F3,F4,F>>;

impl<F1,F2,F3,F4,F,Out> HasOutput for Apply4Data<F1,F2,F3,F4,F>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    type Output = Out;
}

impl<F1,F2,F3,F4,F,Out> Apply4<F1,F2,F3,F4,F>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    /// Constructor.
    pub fn new(label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4, function:F) -> Self {
        let flow1 = f1.clone_ref();
        let flow2 = f2.clone_ref();
        let flow3 = f3.clone_ref();
        let flow4 = f4.clone_ref();
        let def   = Apply4Data {flow1,flow2,flow3,flow4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        f1.register_target(weak.clone_ref().into());
        f2.register_target(weak.clone_ref().into());
        f3.register_target(weak.clone_ref().into());
        f4.register_target(weak.into());
        this
    }
}

impl<F1,F2,F3,F4,F,Out,T> EventConsumer<T> for Node<Apply4Data<F1,F2,F3,F4,F>>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, Out:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.definition.flow1.value();
        let value2 = self.definition.flow2.value();
        let value3 = self.definition.flow3.value();
        let value4 = self.definition.flow4.value();
        let out    = (self.definition.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<F1,F2,F3,F4,F> Debug for Apply4Data<F1,F2,F3,F4,F> {
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

pub trait Anyyy : HasId + HasLabel + HasTypeLabel {}
impl<T> Anyyy for T where T : HasId + HasLabel + HasTypeLabel {}

#[derive(Derivative)]
#[derivative(Debug)]
pub struct NetworkData {
    #[derivative(Debug="ignore")]
    nodes : RefCell<Vec<Box<dyn Anyyy>>>,
    links : RefCell<HashMap<Id,Link>>,
}

#[derive(Debug,Clone)]
pub struct Link {
    pub source : Id,
    pub tp     : LinkType,
}

impl Link {
    pub fn event<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Event;
        Self {source,tp}
    }

    pub fn behavior<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Behavior;
        Self {source,tp}
    }

    pub fn mixed<T:HasId>(t:&T) -> Link {
        let source = t.id();
        let tp     = LinkType::Mixed;
        Self {source,tp}
    }
}

#[derive(Debug,Clone,Copy)]
pub enum LinkType {Event,Behavior,Mixed}



// === API ===

impl NetworkData {
    /// Constructor.
    pub fn new() -> Self {
        let nodes = default();
        let links = default();
        Self {nodes,links}
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
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        flow
    }

    // TODO to nie dziala. mozna zrobic merge a potem metodami dokladac rzeczy. To musi byc wbudowane w nody
    // TODO moznaby zrobic referencje do obecnego grafu w nodach ...
//    pub fn register2<Def:NodeDefinition>(&self, node:Node<Def>, links:Vec<Link>) -> Flow<Output<Def>> {
//        let flow : Flow<Output<Def>> = node.clone_ref().into();
//        let node = Box::new(node);
//        self.data.nodes.borrow_mut().push(node);
//        let target = flow.id();
//        links.into_iter().for_each(|link| self.register_link(target,link));
//        flow
//    }

    pub fn register_link(&self, target:Id, link:Link) {
        self.data.links.borrow_mut().insert(target,link);
    }

    pub fn draw(&self) {
        let mut viz = debug::Graphviz::default();
        self.data.nodes.borrow().iter().for_each(|node| {
            viz.add_node(node.id().into(),node.type_label(),node.label());
            println!(">>> {:?}",node.id())
        });
        debug::display_graphviz(viz);
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
        self.register(Never::new(label))
    }}

    docs_for_source! {
    pub fn source<T:Value>(&self, label:Label) -> Flow<T> {
        self.register(Source::new(label))
    }}

    docs_for_source! {
    pub fn source_(&self, label:Label) -> Flow<()> {
        self.register(Source::new(label))
    }}

    docs_for_trace! {
    pub fn trace<M,S,T>(&self, label:Label, message:M, flow:&S) -> Flow<T>
    where M:Into<String>, S:AnyFlow<Output=T>, T:Value {
        self.register(Trace::new(label,message,flow))
    }}

    docs_for_toggle! {
    pub fn toggle<S:AnyFlow>(&self, label:Label, flow:&S) -> Flow<bool> {
        self.register(Toggle::new(label,flow))
    }}

    docs_for_count! {
    pub fn count<S:AnyFlow>(&self, label:Label, flow:&S) -> Flow<usize> {
        self.register(Count::new(label,flow))
    }}

    docs_for_constant! {
    pub fn constant<S,T> (&self, label:Label, flow:&S, value:T) -> Flow<T>
    where S:AnyFlow, T:Value {
        self.register(Constant::new(label,flow,value))
    }}

    docs_for_previous! {
    pub fn previous<S,T> (&self, label:Label, flow:&S) -> Flow<T>
    where S:AnyFlow<Output=T>, T:Value {
        self.register(Previous::new(label,flow))
    }}

    docs_for_sample! {
    pub fn sample<E:AnyFlow,B:AnyFlow>
    (&self, label:Label, event:&E, behavior:&B) -> Flow<Output<B>> {
        self.register(Sample::new(label,event,behavior))
    }}

    docs_for_gate! {
    pub fn gate<T,E,B>(&self, label:Label, event:&E, behavior:&B) -> Flow<Output<E>>
    where T:Value, E:AnyFlow<Output=T>, B:AnyFlow<Output=bool> {
        self.register(Gate::new(label,event,behavior))
    }}


    // === Merge ===

    docs_for_merge! {
    /// Please note that this function does output a more specific type than just `Flow<T>`. It is
    /// left on purpose so you could use the `add` method to build recursive data-flow networks.
    pub fn merge_<T:Value>(&self, label:Label) -> WeakMerge<T> {
        self.register_raw(Merge::new(label))
    }}

    docs_for_merge! {
    pub fn merge<F1,F2,T:Value>(&self, label:Label, f1:&F1, f2:&F2) -> Flow<T>
    where F1:AnyFlow<Output=T>, F2:AnyFlow<Output=T> {
        self.register(Merge::new2(label,f1,f2))
    }}

    docs_for_merge! {
    pub fn merge1<F1,T:Value>(&self, label:Label, f1:&F1) -> Flow<T>
    where F1:AnyFlow<Output=T> {
        self.register(Merge::new1(label,f1))
    }}

    docs_for_merge! {
    pub fn merge2<F1,F2,T:Value>(&self, label:Label, f1:&F1, f2:&F2) -> Flow<T>
    where F1:AnyFlow<Output=T>, F2:AnyFlow<Output=T> {
        self.register(Merge::new2(label,f1,f2))
    }}

    docs_for_merge! {
    pub fn merge3<F1,F2,F3,T:Value>(&self, label:Label, f1:&F1, f2:&F2, f3:&F3) -> Flow<T>
    where F1:AnyFlow<Output=T>, F2:AnyFlow<Output=T>, F3:AnyFlow<Output=T> {
        self.register(Merge::new3(label,f1,f2,f3))
    }}

    docs_for_merge! {
    pub fn merge4<F1,F2,F3,F4,T:Value>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4) -> Flow<T>
    where F1:AnyFlow<Output=T>, F2:AnyFlow<Output=T>, F3:AnyFlow<Output=T>, F4:AnyFlow<Output=T> {
        self.register(Merge::new4(label,f1,f2,f3,f4))
    }}


    // === Zip ===

    docs_for_zip2! {
    pub fn zip<F1,F2>(&self, label:Label, f1:&F1, f2:&F2) -> Flow<(Output<F1>,Output<F2>)>
    where F1:AnyFlow, F2:AnyFlow {
        self.register(Zip2::new(label,f1,f2))
    }}

    docs_for_zip2! {
    pub fn zip2<F1,F2>(&self, label:Label, f1:&F1, f2:&F2) -> Flow<(Output<F1>,Output<F2>)>
    where F1:AnyFlow, F2:AnyFlow {
        self.register(Zip2::new(label,f1,f2))
    }}

    docs_for_zip3! {
    pub fn zip3<F1,F2,F3>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3) -> Flow<(Output<F1>,Output<F2>,Output<F3>)>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow {
        self.register(Zip3::new(label,f1,f2,f3))
    }}

    docs_for_zip4! {
    pub fn zip4<F1,F2,F3,F4>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4)
    -> Flow<(Output<F1>,Output<F2>,Output<F3>,Output<F4>)>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow {
        self.register(Zip4::new(label,f1,f2,f3,f4))
    }}


    // === Map ===

    docs_for_map! {
    pub fn map<S,F,T>(&self, label:Label, source:&S, f:F) -> Flow<T>
    where S:AnyFlow, T:Value, F:'static+Fn(&Output<S>)->T {
        self.register(Map::new(label,source,f))
    }}

    docs_for_map! {
    pub fn map2<F1,F2,F,T>(&self, label:Label, f1:&F1, f2:&F2, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, T:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->T {
        self.register(Map2::new(label,f1,f2,f))
    }}

    docs_for_map! {
    pub fn map3<F1,F2,F3,F,T>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, T:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->T {
        self.register(Map3::new(label,f1,f2,f3,f))
    }}

    docs_for_map! {
    pub fn map4<F1,F2,F3,F4,F,T>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, T:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->T {
        self.register(Map4::new(label,f1,f2,f3,f4,f))
    }}


    // === Apply ===

    docs_for_apply! {
    pub fn apply2<F1,F2,F,T>(&self, label:Label, f1:&F1, f2:&F2, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, T:Value, F:'static+Fn(&Output<F1>,&Output<F2>)->T {
        self.register(Apply2::new(label,f1,f2,f))
    }}

    docs_for_apply! {
    pub fn apply3<F1,F2,F3,F,T>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, T:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>)->T {
        self.register(Apply3::new(label,f1,f2,f3,f))
    }}

    docs_for_apply! {
    pub fn apply4<F1,F2,F3,F4,F,T>
    (&self, label:Label, f1:&F1, f2:&F2, f3:&F3, f4:&F4, f:F) -> Flow<T>
    where F1:AnyFlow, F2:AnyFlow, F3:AnyFlow, F4:AnyFlow, T:Value,
          F:'static+Fn(&Output<F1>,&Output<F2>,&Output<F3>,&Output<F4>)->T {
        self.register(Apply4::new(label,f1,f2,f3,f4,f))
    }}
}

///////////////////////////////////



#[allow(unused_variables)]
pub fn test() {
    println!("hello");

//    new_network! { network
//        def source  = source::<f32>();
//        def source2 = source::<()>();
//        def tg      = toggle(&source);
//        def fff     = map(&tg,|t| { println!("{:?}",t) });
//        def bb      = sample(&source2,&tg);
//
//        let bb2 : Flow<bool> = bb.into();
//
//        def fff2   = map(&bb2,|t| { println!(">> {:?}",t) });
//        def m      = merge_::<usize>();
//        def c      = count(&m);
//        def t      = trace("t",&c);
//    }
//
//    m.add(&c);
//
//    println!("{:?}",tg);
//
//    source.emit(&5.0);
//    source2.emit(&());
//    source.emit(&5.0);
//    source2.emit(&());
//    source.emit(&5.0);
//
//    m.emit(&0);
//    m.emit(&0);
//    m.emit(&0);

//    network.draw();

    new_network! { network1
        def source = source();
        def count  = source.count();
        def t      = trace("source",&source);
        def t2     = trace("count",&count);
    }

    source.emit(());
    source.emit(());
    source.emit(());

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
