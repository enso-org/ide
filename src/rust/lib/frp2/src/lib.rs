//! This module implements an Functional Reactive Programming system. It is an advanced event
//! handling framework which allows describing events and actions by creating declarative event
//! stream diagrams.
//!
//! Please read this document as the initial introduction to FRP concepts:
//! https://github.com/hansroland/reflex-dom-inbits/blob/master/tutorial.md
//!
//! The architecture of the FRP engine is pretty complex and it is hard to understand all the
//! dependencies by reading the code only. In order to make it easier, the following diagram was
//! prepared. It visualizes a simple FRP network of `source -> count -> map(|t| t.to_string())`:
//!
//! - StreamNode colors indicate to which network the nodes belong (nodes of that color will be dropped as
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
//!     node_data_1[label="StreamNodeData<()>",pos="1.5,-1!",color=blue]
//!     node_1 -> node_data_1
//!     stream_1[label="Stream<()>", pos="3.5,-1!"]
//!     stream_1 -> node_data_1 [style=dashed]
//!
//!     StreamInput_1[label="StreamInput<()>", pos="1.5,-2!",color=blue]
//!     StreamInput_1_[pos="1.5,-2!",label="",color=crimson, width=1.49, height=0.6]
//!     node_data_1 -> StreamInput_1 [arrowhead=none]
//!     WeakCount_[pos="1.5,-3!",label="",color=crimson, width=1.25, height=0.6]
//!     WeakCount[pos="1.5,-3!",color=blue]
//!     StreamInput_1 -> WeakCount [color=red]
//!     node_2[label="Count",pos="-0.5,-4!",color=crimson]
//!     CountData[pos="-0.5,-3!",color=crimson]
//!     node_2 -> CountData
//!     node_data_2[label="StreamNodeData<usize>",pos="1.5,-4!",color=crimson]
//!     node_2 -> node_data_2
//!     WeakCount -> node_data_2 [style=dashed]
//!     WeakCount -> CountData
//!     stream_2[label="Stream<usize>",pos="3.5,-4!"]
//!     stream_2 -> node_data_2 [style=dashed]
//!
//!     StreamInput_2[label="StreamInput<usize>", pos="1.5,-5!",color=crimson]
//!     node_data_2 -> StreamInput_2
//!     WeakMap[pos="1.5,-6!",color=crimson]
//!     StreamInput_2 -> WeakMap [color=red]
//!     node_3[label="Map<Count,[f]>",pos="-0.5,-7!",color=crimson]
//!     MapData[pos="-0.5,-6!",color=crimson]
//!     node_3 -> MapData
//!     node_data_3[label="StreamNodeData<String>",pos="1.5,-7!",color=crimson]
//!     node_3 -> node_data_3
//!     WeakMap -> node_data_3 [style=dashed] [weight=10]
//!     WeakMap -> MapData
//!     stream_3[label="Stream<String>", pos="3.5,-7!"]
//!     stream_3 -> node_data_3 [style=dashed]
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







#[derive(Debug,Shrinkwrap)]
pub struct Watched<T> {
    #[shrinkwrap(main_field)]
    target : T,
    handle : WatchHandle
}

impl<T:HasId> HasId for Watched<T> {
    fn id(&self) -> Id {
        self.target.id()
    }
}


#[derive(Debug)]
pub struct WatchHandle {
    counter : WatchCounter
}

impl WatchHandle {
    pub fn new(counter:&WatchCounter) -> Self {
        let counter = counter.clone_ref();
        counter.increase();
        Self {counter}
    }
}

impl Drop for WatchHandle {
    fn drop(&mut self) {
        self.counter.decrease()
    }
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct WatchCounter {
    count: Rc<Cell<usize>>
}

impl WatchCounter {
    pub fn new() -> Self {
        default()
    }

    pub fn is_zero(&self) -> bool {
        self.count.get() == 0
    }

    pub fn new_watch(&self) -> WatchHandle {
        WatchHandle::new(self)
    }

    fn increase(&self) {
        self.count.set(self.count.get() + 1);
    }

    fn decrease(&self) {
        self.count.set(self.count.get() - 1);
    }
}






// =================
// === TypeLabel ===
// =================

/// Label of the output type of this FRP node. Used mainly for debugging purposes.
pub trait HasOutputTypeLabel {
    /// Output type label of this object.
    fn output_type_label(&self) -> String;
}



// ======================
// === InputBehaviors ===
// ======================

pub trait InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link>;
}

impl<T> InputBehaviors for T {
    default fn input_behaviors(&self) -> Vec<Link> {
        vec![]
    }
}





// ==========
// === Id ===
// ==========

/// Identifier of FRP node. Used mainly for debug purposes.
#[derive(Debug,Clone,CloneRef,Copy,Eq,From,Hash,Into,PartialEq)]
pub struct Id {
    raw : usize
}

/// Implementors of this trait has to be assigned with an unique Id. All FRP nodes implement it.
#[allow(missing_docs)]
pub trait HasId {
    fn id(&self) -> Id;
}



// =============
// === Label ===
// =============

/// FRP node label. USed mainly for debugging purposes.
type Label = &'static str;

/// Implementors of this trait has to be assigned with a label. Each FRP node implements it.
#[allow(missing_docs)]
pub trait HasLabel {
    fn label(&self) -> Label;
}



// ============
// === Data ===
// ============

/// Data that flows trough the FRP network.
pub trait Data = 'static + Clone + Debug + Default;



// =================
// === HasOutput ===
// =================

/// Implementors of this trait has to know their output type.
#[allow(missing_docs)]
pub trait HasOutput {
    type Output : Data;
}

/// A static version of `HasOutput`.
pub trait HasOutputStatic = 'static + HasOutput;


/// Accessor of the accosiated `Output` type.
pub type Output<T> = <T as HasOutput>::Output;



// ====================
// === EventEmitter ===
// ====================

/// Any type which can be used as FRP stream output.
pub trait StreamOutput = 'static + ValueProvider + EventEmitter + CloneRef + HasId;

/// Implementors of this trait have to know how to emit events to subsequent nodes and how to
/// register new event receivers.
pub trait EventEmitter : HasOutput {
    fn emit_event(&self , value:&Self::Output);
    fn register_target(&self , target:StreamInput<Output<Self>>);
    fn register_watch(&self) -> WatchHandle;
}

impl<T:EventEmitter> EventEmitterPoly for T {}
pub trait EventEmitterPoly : EventEmitter {
    fn ping(&self) where Self : HasOutput<Output=()> {
        self.emit_event(&())
    }

    fn emit<T:ToRef<Output<Self>>>(&self, value:T) {
        self.emit_event(value.to_ref())
    }
}



// ======================
// === Event Consumer ===
// ======================

/// Implementors of this trait have to know how to consume incoming events.
pub trait EventConsumer<T> {
    /// Callback for a new incoming event.
    fn on_event(&self, value:&T);
}

/// Implementors of this trait have to know how to consume incoming events. However, it is allowed
/// for them not to consume an event if they were already dropped.
pub trait WeakEventConsumer<T> {
    /// Callback for a new incoming event. Returns true if the event was consumed or false if it was
    /// not. Not consuming an event means that the event receiver was already dropped.
    fn on_event_if_exists(&self, value:&T) -> bool;
}



// =====================
// === ValueProvider ===
// =====================

/// Implementors of this trait have to be able to return their current output value.
pub trait ValueProvider : HasOutput {
    /// The current output value of the FRP node.
    fn value(&self) -> Self::Output;
}




// ===================
// === StreamInput ===
// ===================

/// A generalization of any stream input which consumes events of the provided type. This is the
/// slowest bit of the whole FRP network as it uses an trait object, however, we can refactor it
/// in the future to an enum-based trait if needed.
#[derive(Clone)]
pub struct StreamInput<Input> {
    data : Rc<dyn WeakEventConsumer<Input>>
}

impl<Def,Input> From<WeakStreamNode<Def>> for StreamInput<Input>
where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<Input> {
    fn from(node:WeakStreamNode<Def>) -> Self {
        Self {data:Rc::new(node)}
    }
}

impl<Def,Input> From<&WeakStreamNode<Def>> for StreamInput<Input>
where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<Input> {
    fn from(node:&WeakStreamNode<Def>) -> Self {
        Self {data:Rc::new(node.clone_ref())}
    }
}

impl<Input> Debug for StreamInput<Input> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"StreamInput")
    }
}




// ======================
// === StreamNodeData ===
// ======================

/// Internal structure of every stream FRP node.
///
/// A few important design decisions are worth mentioning. The `during_call` field is set to `true`
/// after a new event is emitted from this structure and is set to false after the event stops
/// propagating. It is used for preventing events to loop indefinitely. It is especially useful
/// in recursive FRP network. The `watch_counter` field counts the amount of nodes which are not
/// event targets (the `targets` field), but are watching this node and can ask it for the last
/// value any time. If the number of such nodes is zero, the value propagated trough this node does
/// not need to be cached, and it will not be cloned. This minimizes the amount of clones in FRP
/// networks drastically.
#[derive(Debug)]
pub struct StreamNodeData<Out=()> {
    label         : Label,
    targets       : RefCell<Vec<StreamInput<Out>>>,
    value_cache   : RefCell<Out>,
    during_call   : Cell<bool>,
    watch_counter : WatchCounter,
}

impl<Out:Default> StreamNodeData<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let targets       = default();
        let value_cache   = default();
        let during_call   = default();
        let watch_counter = default();
        Self {label,targets,value_cache,during_call,watch_counter}
    }

    fn use_caching(&self) -> bool {
        !self.watch_counter.is_zero()
    }
}

impl<Out:Data> HasOutput for StreamNodeData<Out> {
    type Output = Out;
}

impl<Out:Data> EventEmitter for StreamNodeData<Out> {
    fn emit_event(&self, value:&Out) {
        if !self.during_call.get() {
            self.during_call.set(true);
            if self.use_caching() {
                *self.value_cache.borrow_mut() = value.clone();
            }
            self.targets.borrow_mut().retain(|target| target.data.on_event_if_exists(value));
            self.during_call.set(false);
        }
    }

    fn register_target(&self,target:StreamInput<Out>) {
        self.targets.borrow_mut().push(target)
    }

    fn register_watch(&self) -> WatchHandle {
        self.watch_counter.new_watch()
    }
}

impl<Out:Data> ValueProvider for StreamNodeData<Out> {
    fn value(&self) -> Out {
        self.value_cache.borrow().clone()
    }
}


// ====================
// === Event Stream ===
// ====================

// === Types ===

/// Weak reference to FRP stream node with limited functionality and parametrized only by the
/// output type. This should be the main type used in public FRP APIs.
/// See the docs of `StreamNodeData` to learn more about its internal design.
#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Stream<Out=()> {
    data : Weak<StreamNodeData<Out>>,
}

/// A strong reference to FRP stream node. See the docs of `StreamNodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct StreamNode<Def:HasOutputStatic> {
    data       : Rc<StreamNodeData<Output<Def>>>,
    definition : Rc<Def>,
}

/// Weak reference to FRP stream node. See the docs of `StreamNodeData` to learn more about its
/// internal design.
#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakStreamNode<Def:HasOutputStatic> {
    stream     : Stream<Output<Def>>,
    definition : Rc<Def>,
}


// === Output ===

impl<Out:Data>            HasOutput for Stream         <Out> { type Output = Out; }
impl<Def:HasOutputStatic> HasOutput for StreamNode     <Def> { type Output = Output<Def>; }
impl<Def:HasOutputStatic> HasOutput for WeakStreamNode <Def> { type Output = Output<Def>; }


// === Derefs ===

impl<Def> Deref for StreamNode<Def>
where Def:HasOutputStatic {
    type Target = Def;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}


// === Constructors ===

impl<Def:HasOutputStatic> StreamNode<Def> {
    /// Constructor.
    pub fn construct(label:Label, definition:Def) -> Self {
        let data       = Rc::new(StreamNodeData::new(label));
        let definition = Rc::new(definition);
        Self {data,definition}
    }

    /// Constructor which registers the newly created node as the event target of the argument.
    pub fn construct_and_connect<S>(label:Label, stream:&S, definition:Def) -> Self
    where S:StreamOutput, Self:EventConsumer<Output<S>> {
        let this = Self::construct(label,definition);
        let weak = this.downgrade();
        stream.register_target(weak.into());
        this
    }

    /// Downgrades to the weak version.
    pub fn downgrade(&self) -> WeakStreamNode<Def> {
        let stream     = Stream {data:Rc::downgrade(&self.data)};
        let definition = self.definition.clone_ref();
        WeakStreamNode {stream,definition}
    }
}

impl<T:HasOutputStatic> WeakStreamNode<T> {
    /// Upgrades to the strong version.
    pub fn upgrade(&self) -> Option<StreamNode<T>> {
        self.stream.data.upgrade().map(|data| {
            let definition = self.definition.clone_ref();
            StreamNode{data,definition}
        })
    }
}

impl<Def> From<StreamNode<Def>> for Stream<Def::Output>
where Def:HasOutputStatic {
    fn from(node:StreamNode<Def>) -> Self {
        let data = Rc::downgrade(&node.data);
        Stream {data}
    }
}


// === EventEmitter ===

impl<Out:Data> EventEmitter for Stream<Out> {
    fn emit_event(&self, value:&Self::Output) {
        self.data.upgrade().for_each(|t| t.emit_event(value))
    }

    fn register_target(&self,target:StreamInput<Output<Self>>) {
        self.data.upgrade().for_each(|t| t.register_target(target))
    }

    fn register_watch(&self) -> WatchHandle {
        self.data.upgrade().map(|t| t.register_watch()).unwrap() // FIXME
    }
}

impl<Def:HasOutputStatic> EventEmitter for StreamNode<Def>  {
    fn emit_event      (&self, value:&Output<Def>)           { self.data.emit_event(value) }
    fn register_target (&self,tgt:StreamInput<Output<Self>>) { self.data.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.data.register_watch() }
}

impl<Def:HasOutputStatic> EventEmitter for WeakStreamNode<Def> {
    fn emit_event      (&self, value:&Output<Def>)           { self.stream.emit_event(value) }
    fn register_target (&self,tgt:StreamInput<Output<Self>>) { self.stream.register_target(tgt) }
    fn register_watch  (&self) -> WatchHandle                { self.stream.register_watch() }
}


// === WeakEventConsumer ===

impl<Def,T> WeakEventConsumer<T> for WeakStreamNode<Def>
    where Def:HasOutputStatic, StreamNode<Def>:EventConsumer<T> {
    fn on_event_if_exists(&self, value:&T) -> bool {
        self.upgrade().map(|node| {node.on_event(value);}).is_some()
    }
}


// === ValueProvider ===

impl<Out:Data> ValueProvider for Stream<Out> {
    fn value(&self) -> Self::Output {
        self.data.upgrade().map(|t| t.value()).unwrap_or_default()
    }
}

impl<Def:HasOutputStatic> ValueProvider for StreamNode<Def> {
    fn value(&self) -> Self::Output {
        self.data.value_cache.borrow().clone()
    }
}

impl<Def:HasOutputStatic> ValueProvider for WeakStreamNode<Def> {
    fn value(&self) -> Self::Output {
        self.stream.value()
    }
}


// === HasId ===

impl<Out> HasId for Stream<Out> {
    fn id(&self) -> Id {
        let raw = self.data.as_raw() as *const() as usize;
        raw.into()
    }
}

impl<Def:HasOutputStatic> HasId for StreamNode<Def> {
    fn id(&self) -> Id {
        self.downgrade().id()
    }
}

impl<Def:HasOutputStatic> HasId for WeakStreamNode<Def> {
    fn id(&self) -> Id {
        self.stream.id()
    }
}


// === HasLabel ===

impl<Def:HasOutputStatic> HasLabel for StreamNode<Def>
    where Def:InputBehaviors {
    fn label(&self) -> Label {
        self.data.label
    }
}

// FIXME code quality below:
impl<Def> HasOutputTypeLabel for StreamNode<Def>
    where Def:HasOutputStatic+InputBehaviors {
    fn output_type_label(&self) -> String {
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


// === InputBehaviors ===

impl<Def:HasOutputStatic> InputBehaviors for StreamNode<Def>
where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![] // FIXME
//        self.data.input_behaviors()
    }
}

impl<Def:HasOutputStatic> InputBehaviors for WeakStreamNode<Def>
    where Def:InputBehaviors {
    fn input_behaviors(&self) -> Vec<Link> {
        self.stream.input_behaviors()
    }
}


// === Debug ===

impl<Out> Debug for Stream<Out> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        match self.data.upgrade() {
            None    => write!(f,"Stream(Dropped)"),
            Some(_) => write!(f,"Stream"),
        }

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
pub type   Never     <Out=()> = StreamNode     <NeverData<Out>>;
pub type   WeakNever <Out=()> = WeakStreamNode <NeverData<Out>>;

impl<Out:Data> HasOutput for NeverData<Out> {
    type Output = Out;
}

impl<Out:Data> Never<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom    = default();
        let definition = NeverData {phantom};
        Self::construct(label,definition)
    }
}

// FIXME
//impl<Out> EventEmitter for StreamNode<NeverData<Out>>
//where NeverData<Out> : HasOutputStatic {
//    fn emit_event(&self, _value:&Output<Self>) {}
//    fn register_target(&self, _tgt:StreamInput<Output<Self>>) {}
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
pub type   Source     <Out=()> = StreamNode     <SourceData<Out>>;
pub type   WeakSource <Out=()> = WeakStreamNode <SourceData<Out>>;

impl<Out:Data> HasOutput for SourceData<Out> {
    type Output = Out;
}

impl<Out:Data> Source<Out> {
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
pub type   Trace     <Out> = StreamNode     <TraceData<Out>>;
pub type   WeakTrace <Out> = WeakStreamNode <TraceData<Out>>;

impl<Out:Data> HasOutput for TraceData<Out> {
    type Output = Out;
}

impl<Out:Data> Trace<Out> {
    /// Constructor.
    pub fn new<M,S>(label:Label, message:M, stream:&S) -> Self
    where M:Into<String>, S:StreamOutput<Output=Out> {
        let phantom = default();
        let message = message.into();
        let def     = TraceData {phantom,message};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data> EventConsumer<Out> for Trace<Out> {
    fn on_event(&self, event:&Out) {
        println!("[FRP] {}: {:?}", self.message, event);
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
pub type   Toggle     = StreamNode     <ToggleData>;
pub type   WeakToggle = WeakStreamNode <ToggleData>;

impl HasOutput for ToggleData {
    type Output = bool;
}

impl Toggle {
    /// Constructor.
    pub fn new<S:StreamOutput>(label:Label, stream:&S) -> Self {
        Self::new_with(label,stream,default())
    }

    /// Constructor with explicit start value.
    pub fn new_with<S:StreamOutput>(label:Label, stream:&S, init:bool) -> Self {
        let value = Cell::new(init);
        let def   = ToggleData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<T> EventConsumer<T> for Toggle {
    fn on_event(&self, _:&T) {
        let value = !self.value.get();
        self.value.set(value);
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
pub type   Count     = StreamNode     <CountData>;
pub type   WeakCount = WeakStreamNode <CountData>;

impl HasOutput for CountData {
    type Output = usize;
}

impl Count {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S) -> Self
    where S:StreamOutput {
        let value = default();
        let def   = CountData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<T> EventConsumer<T> for Count {
    fn on_event(&self, _:&T) {
        let value = self.value.get() + 1;
        self.value.set(value);
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
pub type   Constant     <Out=()> = StreamNode     <ConstantData<Out>>;
pub type   WeakConstant <Out=()> = WeakStreamNode <ConstantData<Out>>;

impl<Out:Data> HasOutput for ConstantData<Out> {
    type Output = Out;
}

impl<Out:Data> Constant<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S, value:Out) -> Self
    where S:StreamOutput {
        let def = ConstantData {value};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data,T> EventConsumer<T> for Constant<Out> {
    fn on_event(&self, _:&T) {
        self.emit(&self.value);
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
pub type   Previous     <Out=()> = StreamNode     <PreviousData<Out>>;
pub type   WeakPrevious <Out=()> = WeakStreamNode <PreviousData<Out>>;

impl<Out:Data> HasOutput for PreviousData<Out> {
    type Output = Out;
}

impl<Out:Data> Previous<Out> {
    /// Constructor.
    pub fn new<S>(label:Label, stream:&S) -> Self
        where S:StreamOutput<Output=Out> {
        let previous = default();
        let def      = PreviousData {previous};
        Self::construct_and_connect(label,stream,def)
    }
}

impl<Out:Data> EventConsumer<Out> for Previous<Out> {
    fn on_event(&self, event:&Out) {
        let previous = mem::replace(&mut *self.previous.borrow_mut(),event.clone());
        self.emit(previous);
    }
}





pub fn watch_stream<T:StreamOutput>(target:&T) -> Watched<T> {
    let target = target.clone_ref();
    let handle = target.register_watch();
    Watched {target,handle}
}


// ==============
// === Sample ===
// ==============

macro_rules! docs_for_sample { ($($tt:tt)*) => { #[doc="
Samples the first stream (behavior) on every incoming event of the second stream. The incoming event
is dropped and a new event with the behavior's value is emitted.
"]$($tt)* }}

docs_for_sample! { #[derive(Debug)]
pub struct SampleData <T1> { behavior:Watched<T1> }}
pub type   Sample     <T1> = StreamNode     <SampleData<T1>>;
pub type   WeakSample <T1> = WeakStreamNode <SampleData<T1>>;

impl<T1:HasOutput> HasOutput for SampleData<T1> {
    type Output = Output<T1>;
}

impl<T1:StreamOutput> Sample<T1> {
    /// Constructor.
    pub fn new<Event:StreamOutput>(label:Label, event:&Event, behavior:&T1) -> Self {
        let behavior   = watch_stream(behavior);
        let definition = SampleData {behavior};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T,T1:StreamOutput> EventConsumer<T> for Sample<T1> {
    fn on_event(&self, _:&T) {
        self.emit(self.behavior.value());
    }
}

impl<B> InputBehaviors for SampleData<B>
where B:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
    }
}



// ============
// === Gate ===
// ============

macro_rules! docs_for_gate { ($($tt:tt)*) => { #[doc="
Passes the incoming event of the fisr stream only if the value of the second stream is `true`.
"]$($tt)* }}

docs_for_gate! { #[derive(Debug)]
pub struct GateData <T1,Out=()> { behavior:Watched<T1>, phantom:PhantomData<Out> }}
pub type   Gate     <T1,Out=()> = StreamNode     <GateData<T1,Out>>;
pub type   WeakGate <T1,Out=()> = WeakStreamNode <GateData<T1,Out>>;

impl<T1,Out:Data> HasOutput for GateData<T1,Out> {
    type Output = Out;
}

impl<T1,Out> Gate<T1,Out>
where Out:Data, T1:StreamOutput<Output=bool> {
    /// Constructor.
    pub fn new<E>(label:Label, event:&E, behavior:&T1) -> Self
    where E:StreamOutput<Output=Out> {
        let behavior   = watch_stream(behavior);
        let phantom    = default();
        let definition = GateData {behavior,phantom};
        Self::construct_and_connect(label,event,definition)
    }
}

impl<T1,Out> EventConsumer<Out> for Gate<T1,Out>
where Out:Data, T1:StreamOutput<Output=bool> {
    fn on_event(&self, event:&Out) {
        if self.behavior.value() {
            self.emit(event)
        }
    }
}

impl<T1,Out> InputBehaviors for GateData<T1,Out>
where T1:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.behavior)]
    }
}



// =============
// === Merge ===
// =============

macro_rules! docs_for_merge { ($($tt:tt)*) => { #[doc="
Merges multiple input streams into a single output stream. All input streams have to share the same
output data type. Please note that `Merge` can be used to create recursive FRP networks by creating
an empty merge and using the `add` method to attach new streams to it. When a recursive network is
created, `Merge` breaks the cycle. After passing the first event, no more events will be passed
till the end of the current FRP network resolution.
"]$($tt)* }}

docs_for_merge! { #[derive(Clone,Debug)]
pub struct MergeData <Out=()> { phantom:PhantomData<Out>, during_call:Cell<bool> }}
pub type   Merge     <Out=()> = StreamNode     <MergeData<Out>>;
pub type   WeakMerge <Out=()> = WeakStreamNode <MergeData<Out>>;

impl<Out:Data> HasOutput for MergeData<Out> {
    type Output = Out;
}

impl<Out:Data> Merge<Out> {
    /// Constructor.
    pub fn new(label:Label) -> Self {
        let phantom     = default();
        let during_call = default();
        let def         = MergeData {phantom,during_call};
        Self::construct(label,def)
    }

    /// Takes ownership of self and returns it with a new stream attached.
    pub fn with<S>(self, stream:&S) -> Self
        where S:StreamOutput<Output=Out> {
        stream.register_target(self.downgrade().into());
        self
    }

    /// Constructor for 1 input stream.
    pub fn new1<T1>(label:Label, t1:&T1) -> Self
        where T1:StreamOutput<Output=Out> {
        Self::new(label).with(t1)
    }

    /// Constructor for 2 input streams.
    pub fn new2<T1,T2>(label:Label, t1:&T1, t2:&T2) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2)
    }

    /// Constructor for 3 input streams.
    pub fn new3<T1,T2,T3>(label:Label, t1:&T1, t2:&T2, t3:&T3) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out>,
              T3:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2).with(t3)
    }

    /// Constructor for 4 input streams.
    pub fn new4<T1,T2,T3,T4>(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Self
        where T1:StreamOutput<Output=Out>,
              T2:StreamOutput<Output=Out>,
              T3:StreamOutput<Output=Out>,
              T4:StreamOutput<Output=Out> {
        Self::new(label).with(t1).with(t2).with(t3).with(t4)
    }
}

impl<Out:Data> WeakMerge<Out> {
    /// Takes ownership of self and returns it with a new stream attached.
    pub fn with<S>(self, stream:&S) -> Self
    where S:StreamOutput<Output=Out> {
        stream.register_target(self.clone_ref().into());
        self
    }
}

impl<T1,Out> Add<&T1> for &Merge<Out>
    where T1:StreamOutput<Output=Out>, Out:Data {
    type Output = Self;
    fn add(self, stream:&T1) -> Self::Output {
        stream.register_target(self.downgrade().into());
        self
    }
}

impl<T1,Out> Add<&T1> for &WeakMerge<Out>
    where T1:StreamOutput<Output=Out>, Out:Data {
    type Output = Self;
    fn add(self, stream:&T1) -> Self::Output {
        stream.register_target(self.into());
        self
    }
}

impl<Out:Data> EventConsumer<Out> for Merge<Out> {
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

docs_for_zip2! { #[derive(Debug)]
pub struct Zip2Data <T1,T2> { stream1:Watched<T1>, stream2:Watched<T2> }}
pub type   Zip2     <T1,T2> = StreamNode     <Zip2Data<T1,T2>>;
pub type   WeakZip2 <T1,T2> = WeakStreamNode <Zip2Data<T1,T2>>;

impl<T1,T2> HasOutput for Zip2Data<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    type Output = (Output<T1>,Output<T2>);
}

impl<T1,T2> Zip2<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Zip2Data {stream1,stream2};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.into());
        this
    }
}

impl<T1,T2,Out> EventConsumer<Out> for Zip2<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        self.emit((value1,value2));
    }
}

impl<T1,T2> InputBehaviors for Zip2Data<T1,T2>
    where T1:StreamOutput, T2:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.stream1), Link::mixed(&self.stream2)]
    }
}



// ============
// === Zip3 ===
// ============

macro_rules! docs_for_zip3 { ($($tt:tt)*) => { #[doc="
Merges three input streams into a stream containing values from all of them. On event from any of
the streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip3! { #[derive(Debug)]
pub struct Zip3Data <T1,T2,T3> { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3> }}
pub type   Zip3     <T1,T2,T3> = StreamNode     <Zip3Data<T1,T2,T3>>;
pub type   WeakZip3 <T1,T2,T3> = WeakStreamNode <Zip3Data<T1,T2,T3>>;

impl<T1,T2,T3> HasOutput for Zip3Data<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    type Output = (Output<T1>,Output<T2>,Output<T3>);
}

impl<T1,T2,T3> Zip3<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Zip3Data {stream1,stream2,stream3};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,Out> EventConsumer<Out> for Zip3<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        self.emit((value1,value2,value3));
    }
}

impl<T1,T2,T3> InputBehaviors for Zip3Data<T1,T2,T3>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::mixed(&self.stream1), Link::mixed(&self.stream2), Link::mixed(&self.stream3)]
    }
}



// ============
// === Zip4 ===
// ============

macro_rules! docs_for_zip4 { ($($tt:tt)*) => { #[doc="
Merges four input streams into a stream containing values from all of them. On event from any of the
streams, all streams are sampled and the final event is produced.
"]$($tt)* }}

docs_for_zip4! { #[derive(Debug)]
pub struct Zip4Data <T1,T2,T3,T4>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4> }}
pub type   Zip4     <T1,T2,T3,T4> = StreamNode     <Zip4Data<T1,T2,T3,T4>>;
pub type   WeakZip4 <T1,T2,T3,T4> = WeakStreamNode <Zip4Data<T1,T2,T3,T4>>;

impl<T1,T2,T3,T4> HasOutput for Zip4Data<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    type Output = (Output<T1>,Output<T2>,Output<T3>,Output<T4>);
}

impl<T1,T2,T3,T4> Zip4<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def   = Zip4Data {stream1,stream2,stream3,stream4};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.clone_ref().into());
        t4.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,Out> EventConsumer<Out> for Zip4<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn on_event(&self, _:&Out) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        self.emit((value1,value2,value3,value4));
    }
}

impl<T1,T2,T3,T4> InputBehaviors for Zip4Data<T1,T2,T3,T4>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![ Link::mixed(&self.stream1)
            , Link::mixed(&self.stream2)
            , Link::mixed(&self.stream3)
            , Link::mixed(&self.stream4)
            ]
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
pub struct MapData <T1,F> { stream:Watched<T1>, function:F }}
pub type   Map     <T1,F> = StreamNode     <MapData<T1,F>>;
pub type   WeakMap <T1,F> = WeakStreamNode <MapData<T1,F>>;

impl<T1,F,Out> HasOutput for MapData<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    type Output = Out;
}

impl<T1,F,Out> Map<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, function:F) -> Self {
        let stream     = watch_stream(t1);
        let definition = MapData {stream,function};
        Self::construct_and_connect(label,t1,definition)
    }
}

impl<T1,F,Out> EventConsumer<Output<T1>> for Map<T1,F>
where T1:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>)->Out {
    fn on_event(&self, value:&Output<T1>) {
        let out = (self.function)(value);
        self.emit(out);
    }
}

impl<T1,F> Debug for MapData<T1,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"MapData")
    }
}



// ============
// === Map2 ===
// ============

docs_for_map! {
pub struct Map2Data <T1,T2,F> { stream1:Watched<T1>, stream2:Watched<T2>, function:F }}
pub type   Map2     <T1,T2,F> = StreamNode     <Map2Data<T1,T2,F>>;
pub type   WeakMap2 <T1,T2,F> = WeakStreamNode <Map2Data<T1,T2,F>>;

impl<T1,T2,F,Out> HasOutput for Map2Data<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    type Output = Out;
}

impl<T1,T2,F,Out> Map2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Map2Data {stream1,stream2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,F,Out> EventConsumer<Output<T1>> for Map2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let out    = (self.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<T1,T2,F> Debug for Map2Data<T1,T2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map2Data")
    }
}

impl<T1,T2,F> InputBehaviors for Map2Data<T1,T2,F>
    where T1:StreamOutput, T2:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2)]
    }
}



// ============
// === Map3 ===
// ============

docs_for_map! {
pub struct Map3Data <T1,T2,T3,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, function:F }}
pub type   Map3     <T1,T2,T3,F> = StreamNode     <Map3Data<T1,T2,T3,F>>;
pub type   WeakMap3 <T1,T2,T3,F> = WeakStreamNode <Map3Data<T1,T2,T3,F>>;

impl<T1,T2,T3,F,Out> HasOutput for Map3Data<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,F,Out> Map3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Map3Data {stream1,stream2,stream3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,F,Out> EventConsumer<Output<T1>> for Map3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let out    = (self.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<T1,T2,T3,F> Debug for Map3Data<T1,T2,T3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Map3Data")
    }
}

impl<T1,T2,T3,F> InputBehaviors for Map3Data<T1,T2,T3,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2), Link::behavior(&self.stream3)]
    }
}



// ============
// === Map4 ===
// ============

docs_for_map! {
pub struct Map4Data <T1,T2,T3,T4,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4>, function:F }}
pub type   Map4     <T1,T2,T3,T4,F> = StreamNode     <Map4Data<T1,T2,T3,T4,F>>;
pub type   WeakMap4 <T1,T2,T3,T4,F> = WeakStreamNode <Map4Data<T1,T2,T3,T4,F>>;

impl<T1,T2,T3,T4,F,Out> HasOutput for Map4Data<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,T4,F,Out> Map4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def   = Map4Data {stream1,stream2,stream3,stream4,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,F,Out> EventConsumer<Output<T1>> for Map4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    fn on_event(&self, value1:&Output<T1>) {
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        let out    = (self.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<T1,T2,T3,T4,F> InputBehaviors for Map4Data<T1,T2,T3,T4,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
    fn input_behaviors(&self) -> Vec<Link> {
        vec![Link::behavior(&self.stream2), Link::behavior(&self.stream3), Link::behavior(&self.stream4)]
    }
}

impl<T1,T2,T3,T4,F> Debug for Map4Data<T1,T2,T3,T4,F> {
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

docs_for_apply! {
pub struct Apply2Data <T1,T2,F> { stream1:Watched<T1>, stream2:Watched<T2>, function:F }}
pub type   Apply2     <T1,T2,F> = StreamNode     <Apply2Data<T1,T2,F>>;
pub type   WeakApply2 <T1,T2,F> = WeakStreamNode <Apply2Data<T1,T2,F>>;

impl<T1,T2,F,Out> HasOutput for Apply2Data<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    type Output = Out;
}

impl<T1,T2,F,Out> Apply2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let def   = Apply2Data {stream1,stream2,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.into());
        this
    }
}

impl<T1,T2,F,Out,T> EventConsumer<T> for Apply2<T1,T2,F>
where T1:StreamOutput, T2:StreamOutput, Out:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let out    = (self.function)(&value1,&value2);
        self.emit(out);
    }
}

impl<T1,T2,F> Debug for Apply2Data<T1,T2,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply2Data")
    }
}



// ==============
// === Apply3 ===
// ==============

docs_for_apply! {
pub struct Apply3Data <T1,T2,T3,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, function:F }}
pub type   Apply3     <T1,T2,T3,F> = StreamNode     <Apply3Data<T1,T2,T3,F>>;
pub type   WeakApply3 <T1,T2,T3,F> = WeakStreamNode <Apply3Data<T1,T2,T3,F>>;

impl<T1,T2,T3,F,Out> HasOutput for Apply3Data<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,F,Out> Apply3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let def   = Apply3Data {stream1,stream2,stream3,function};
        let this  = Self::construct(label,def);
        let weak  = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,F,Out,T> EventConsumer<T> for Apply3<T1,T2,T3,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let out    = (self.function)(&value1,&value2,&value3);
        self.emit(out);
    }
}

impl<T1,T2,T3,F> Debug for Apply3Data<T1,T2,T3,F> {
    fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Apply3Data")
    }
}



// ==============
// === Apply4 ===
// ==============

docs_for_apply! {
pub struct Apply4Data <T1,T2,T3,T4,F>
    { stream1:Watched<T1>, stream2:Watched<T2>, stream3:Watched<T3>, stream4:Watched<T4>, function:F }}
pub type   Apply4     <T1,T2,T3,T4,F> = StreamNode     <Apply4Data<T1,T2,T3,T4,F>>;
pub type   WeakApply4 <T1,T2,T3,T4,F> = WeakStreamNode <Apply4Data<T1,T2,T3,T4,F>>;

impl<T1,T2,T3,T4,F,Out> HasOutput for Apply4Data<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    type Output = Out;
}

impl<T1,T2,T3,T4,F,Out> Apply4<T1,T2,T3,T4,F>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    /// Constructor.
    pub fn new(label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, function:F) -> Self {
        let stream1 = watch_stream(t1);
        let stream2 = watch_stream(t2);
        let stream3 = watch_stream(t3);
        let stream4 = watch_stream(t4);
        let def     = Apply4Data {stream1,stream2,stream3,stream4,function};
        let this    = Self::construct(label,def);
        let weak    = this.downgrade();
        t1.register_target(weak.clone_ref().into());
        t2.register_target(weak.clone_ref().into());
        t3.register_target(weak.clone_ref().into());
        t4.register_target(weak.into());
        this
    }
}

impl<T1,T2,T3,T4,F,Out,T> EventConsumer<T> for Apply4<T1,T2,T3,T4,F>
where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, Out:Data,
      F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->Out {
    fn on_event(&self, _:&T) {
        let value1 = self.stream1.value();
        let value2 = self.stream2.value();
        let value3 = self.stream3.value();
        let value4 = self.stream4.value();
        let out    = (self.function)(&value1,&value2,&value3,&value4);
        self.emit(out);
    }
}

impl<T1,T2,T3,T4,F> Debug for Apply4Data<T1,T2,T3,T4,F> {
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

pub trait Anyyy : HasId + HasLabel + HasOutputTypeLabel {}
impl<T> Anyyy for T where T : HasId + HasLabel + HasOutputTypeLabel {}

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

    pub fn register_raw<T:HasOutputStatic>(&self, node:StreamNode<T>) -> WeakStreamNode<T> {
        let weak = node.downgrade();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        weak
    }

    pub fn register<Def:HasOutputStatic>(&self, node:StreamNode<Def>) -> Stream<Output<Def>> {
        let stream = node.clone_ref().into();
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
        stream
    }

    // TODO to nie dziala. mozna zrobic merge a potem metodami dokladac rzeczy. To musi byc wbudowane w nody
    // TODO moznaby zrobic referencje do obecnego grafu w nodach ...
//    pub fn register2<Def:HasOutputStatic>(&self, node:StreamNode<Def>, links:Vec<Link>) -> Stream<Output<Def>> {
//        let stream : Stream<Output<Def>> = node.clone_ref().into();
//        let node = Box::new(node);
//        self.data.nodes.borrow_mut().push(node);
//        let target = stream.id();
//        links.into_iter().for_each(|link| self.register_link(target,link));
//        stream
//    }

    pub fn register_link(&self, target:Id, link:Link) {
        self.data.links.borrow_mut().insert(target,link);
    }

    pub fn draw(&self) {
        let mut viz = debug::Graphviz::default();
        self.data.nodes.borrow().iter().for_each(|node| {
            viz.add_node(node.id().into(),node.output_type_label(),node.label());
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
    pub fn never<T:Data>(&self, label:Label) -> Stream<T> {
        self.register(Never::new(label))
    }}

    docs_for_source! {
    pub fn source<T:Data>(&self, label:Label) -> Stream<T> {
        self.register(Source::new(label))
    }}

    docs_for_source! {
    pub fn source_(&self, label:Label) -> Stream<()> {
        self.register(Source::new(label))
    }}

    docs_for_trace! {
    pub fn trace<M,S,T>(&self, label:Label, message:M, stream:&S) -> Stream<T>
    where M:Into<String>, S:StreamOutput<Output=T>, T:Data {
        self.register(Trace::new(label,message,stream))
    }}

    docs_for_toggle! {
    pub fn toggle<S:StreamOutput>(&self, label:Label, stream:&S) -> Stream<bool> {
        self.register(Toggle::new(label,stream))
    }}

    docs_for_count! {
    pub fn count<S:StreamOutput>(&self, label:Label, stream:&S) -> Stream<usize> {
        self.register(Count::new(label,stream))
    }}

    docs_for_constant! {
    pub fn constant<S,T> (&self, label:Label, stream:&S, value:T) -> Stream<T>
    where S:StreamOutput, T:Data {
        self.register(Constant::new(label,stream,value))
    }}

    docs_for_previous! {
    pub fn previous<S,T> (&self, label:Label, stream:&S) -> Stream<T>
    where S:StreamOutput<Output=T>, T:Data {
        self.register(Previous::new(label,stream))
    }}

    docs_for_sample! {
    pub fn sample<E:StreamOutput,B:StreamOutput>
    (&self, label:Label, event:&E, behavior:&B) -> Stream<Output<B>> {
        self.register(Sample::new(label,event,behavior))
    }}

    docs_for_gate! {
    pub fn gate<T,E,B>(&self, label:Label, event:&E, behavior:&B) -> Stream<Output<E>>
    where T:Data, E:StreamOutput<Output=T>, B:StreamOutput<Output=bool> {
        self.register(Gate::new(label,event,behavior))
    }}


    // === Merge ===

    docs_for_merge! {
    /// Please note that this function does output a more specific type than just `Stream<T>`. It is
    /// left on purpose so you could use the `add` method to build recursive data-stream networks.
    pub fn merge_<T:Data>(&self, label:Label) -> WeakMerge<T> {
        self.register_raw(Merge::new(label))
    }}

    docs_for_merge! {
    pub fn merge<T1,T2,T:Data>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T> {
        self.register(Merge::new2(label,t1,t2))
    }}

    docs_for_merge! {
    pub fn merge1<T1,T:Data>(&self, label:Label, t1:&T1) -> Stream<T>
    where T1:StreamOutput<Output=T> {
        self.register(Merge::new1(label,t1))
    }}

    docs_for_merge! {
    pub fn merge2<T1,T2,T:Data>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T> {
        self.register(Merge::new2(label,t1,t2))
    }}

    docs_for_merge! {
    pub fn merge3<T1,T2,T3,T:Data>(&self, label:Label, t1:&T1, t2:&T2, t3:&T3) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T>, T3:StreamOutput<Output=T> {
        self.register(Merge::new3(label,t1,t2,t3))
    }}

    docs_for_merge! {
    pub fn merge4<T1,T2,T3,T4,T:Data>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4) -> Stream<T>
    where T1:StreamOutput<Output=T>, T2:StreamOutput<Output=T>, T3:StreamOutput<Output=T>, T4:StreamOutput<Output=T> {
        self.register(Merge::new4(label,t1,t2,t3,t4))
    }}


    // === Zip ===

    docs_for_zip2! {
    pub fn zip<T1,T2>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<(Output<T1>,Output<T2>)>
    where T1:StreamOutput, T2:StreamOutput {
        self.register(Zip2::new(label,t1,t2))
    }}

    docs_for_zip2! {
    pub fn zip2<T1,T2>(&self, label:Label, t1:&T1, t2:&T2) -> Stream<(Output<T1>,Output<T2>)>
    where T1:StreamOutput, T2:StreamOutput {
        self.register(Zip2::new(label,t1,t2))
    }}

    docs_for_zip3! {
    pub fn zip3<T1,T2,T3>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3) -> Stream<(Output<T1>,Output<T2>,Output<T3>)>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput {
        self.register(Zip3::new(label,t1,t2,t3))
    }}

    docs_for_zip4! {
    pub fn zip4<T1,T2,T3,T4>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4)
    -> Stream<(Output<T1>,Output<T2>,Output<T3>,Output<T4>)>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput {
        self.register(Zip4::new(label,t1,t2,t3,t4))
    }}


    // === Map ===

    docs_for_map! {
    pub fn map<S,F,T>(&self, label:Label, source:&S, f:F) -> Stream<T>
    where S:StreamOutput, T:Data, F:'static+Fn(&Output<S>)->T {
        self.register(Map::new(label,source,f))
    }}

    docs_for_map! {
    pub fn map2<T1,T2,F,T>(&self, label:Label, t1:&T1, t2:&T2, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->T {
        self.register(Map2::new(label,t1,t2,f))
    }}

    docs_for_map! {
    pub fn map3<T1,T2,T3,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->T {
        self.register(Map3::new(label,t1,t2,t3,f))
    }}

    docs_for_map! {
    pub fn map4<T1,T2,T3,T4,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->T {
        self.register(Map4::new(label,t1,t2,t3,t4,f))
    }}


    // === Apply ===

    docs_for_apply! {
    pub fn apply2<T1,T2,F,T>(&self, label:Label, t1:&T1, t2:&T2, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T:Data, F:'static+Fn(&Output<T1>,&Output<T2>)->T {
        self.register(Apply2::new(label,t1,t2,f))
    }}

    docs_for_apply! {
    pub fn apply3<T1,T2,T3,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>)->T {
        self.register(Apply3::new(label,t1,t2,t3,f))
    }}

    docs_for_apply! {
    pub fn apply4<T1,T2,T3,T4,F,T>
    (&self, label:Label, t1:&T1, t2:&T2, t3:&T3, t4:&T4, f:F) -> Stream<T>
    where T1:StreamOutput, T2:StreamOutput, T3:StreamOutput, T4:StreamOutput, T:Data,
          F:'static+Fn(&Output<T1>,&Output<T2>,&Output<T3>,&Output<T4>)->T {
        self.register(Apply4::new(label,t1,t2,t3,t4,f))
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
//        let bb2 : Stream<bool> = bb.into();
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
        assert_eq!(count.value(),0);
    }
}
