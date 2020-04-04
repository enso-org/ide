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

use enso_prelude::*;






// =============
// === Graph ===
// =============

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

impl GraphData {
    pub fn new() -> Self {
        let nodes = default();
        Self {nodes}
    }
}

impl Graph {
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



// =============
// === Value ===
// =============

pub trait Value = 'static + Clone + Default;



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
    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>);
}

pub trait EventConsumer<T> {
    fn on_event(&self, value:&T);
}

pub trait LastValueProvider : HasOutput {
    fn last_value(&self) -> Self::Output;
}




// ============
// === Node ===
// ============

// === Types ===

pub trait NodeDefinition = 'static + ?Sized + HasOutput;

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Node<T:NodeDefinition> {
    data : Rc<NodeData<T>>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakNode<T:NodeDefinition> {
    data : Weak<NodeData<T>>,
}

pub struct NodeData<Def:NodeDefinition> {
    targets    : RefCell<Vec<Weak<dyn EventConsumer<Output<Def>>>>>,
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
        source.register_target(weak.data);
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

    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>) {
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

    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>) {
        self.data.upgrade().for_each(|data| data.register_target(tgt))
    }
}

impl<Def:NodeDefinition> LastValueProvider for WeakNode<Def> {
    fn last_value(&self) -> Self::Output {
        self.data.upgrade().map(|data| data.last_value()).unwrap_or_default()
    }
}


// === NodeData Impls ===

impl<Def:NodeDefinition> EventEmitter for NodeData<Def> {
    fn emit(&self, value:&Self::Output) {
        *self.last_value.borrow_mut() = value.clone();
        let mut dirty = false;
        self.targets.borrow().iter().for_each(|weak| match weak.upgrade() {
            Some(tgt) => tgt.on_event(value),
            None      => dirty = true
        });
        if dirty {
            self.targets.borrow_mut().retain(|weak| weak.upgrade().is_none());
        }

    }

    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>) {
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

    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>) {
        self.data.upgrade().for_each(|t| t.register_target(tgt))
    }
}

impl<Out:Value> LastValueProvider for Stream<Out> {
    fn last_value(&self) -> Self::Output {
        self.data.upgrade().map(|t| t.last_value()).unwrap_or_default()
    }
}



// ==============
// === Source ===
// ==============

pub type   Source     <Out=()> = Node     <SourceData<Out>>;
pub type   WeakSource <Out=()> = WeakNode <SourceData<Out>>;
pub struct SourceData <Out=()> { phantom : PhantomData<Out> }

impl<Out:Value> HasOutput for SourceData<Out> {
    type Output = Out;
}

impl<Out:Value> Source<Out> {
    pub fn new() -> Self {
        let phantom    = default();
        let definition = SourceData {phantom};
        Self::construct(definition)
    }
}



// ==============
// === Toggle ===
// ==============

pub type   Toggle     = Node     <ToggleData>;
pub type   WeakToggle = WeakNode <ToggleData>;
pub struct ToggleData { value : Cell<bool> }

impl HasOutput for ToggleData {
    type Output = bool;
}

impl Toggle {
    pub fn new<Src:AnyStream>(src:&Src) -> Self {
        let value      = default();
        let definition = ToggleData {value};
        Self::construct_and_connect(src,definition)
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
// === Merge ===
// =============

pub type   Merge     <T> = Node     <MergeData<T>>;
pub type   WeakMerge <T> = WeakNode <MergeData<T>>;
pub struct MergeData <T> { phantom : PhantomData<T> }

impl<T:Value> HasOutput for MergeData<T> {
    type Output = T;
}

impl<T:Value> Merge<T> {
    pub fn new<Src1,Src2>(src1:&Src1, src2:&Src2) -> Self
        where Src1 : AnyStream<Output=T>,
              Src2 : AnyStream<Output=T> {
        let phantom    = default();
        let definition = MergeData {phantom};
        let this       = Self::construct(definition);
        let weak       = this.downgrade();
        src1.register_target(weak.data.clone_ref());
        src1.register_target(weak.data);
        this
    }
}

impl<T:Value> EventConsumer<T> for NodeData<MergeData<T>> {
    fn on_event(&self, event:&T) {
        self.emit(event);
    }
}



// ================
// === Previous ===
// ================

pub type   Previous     <T> = Node     <PreviousData<T>>;
pub type   WeakPrevious <T> = WeakNode <PreviousData<T>>;
pub struct PreviousData <T> { previous : RefCell<T> }

impl<T:Value> HasOutput for PreviousData<T> {
    type Output = T;
}

impl<T:Value> Previous<T> {
    pub fn new<Src>(src:&Src) -> Self
    where Src : AnyStream<Output=T> {
        let previous   = default();
        let definition = PreviousData {previous};
        Self::construct_and_connect(src,definition)
    }
}

impl<T:Value> EventConsumer<T> for NodeData<PreviousData<T>> {
    fn on_event(&self, event:&T) {
        let previous = mem::replace(&mut *self.definition.previous.borrow_mut(),event.clone());
        self.emit(&previous);
    }
}



// ==============
// === Sample ===
// ==============

pub type   Sample     <Source> = Node     <SampleData<Source>>;
pub type   WeakSample <Source> = WeakNode <SampleData<Source>>;
pub struct SampleData <Source> { behavior : Source }

impl<Source:HasOutput> HasOutput for SampleData<Source> {
    type Output = Output<Source>;
}

impl<Source:AnyStream> Sample<Source> {
    pub fn new<E:AnyStream>(event:&E,behavior:&Source) -> Self {
        let behavior   = behavior.clone_ref();
        let definition = SampleData {behavior};
        Self::construct_and_connect(event,definition)
    }
}

impl<T,Source:AnyStream> EventConsumer<T> for NodeData<SampleData<Source>> {
    fn on_event(&self, _:&T) {
        self.emit(&self.definition.behavior.last_value());
    }
}



// ============
// === Gate ===
// ============

pub type   Gate     <T,B> = Node     <GateData<T,B>>;
pub type   WeakGate <T,B> = WeakNode <GateData<T,B>>;
pub struct GateData <T,B> { behavior : B, phantom : PhantomData<T> }

impl<T:Value,B> HasOutput for GateData<T,B> {
    type Output = T;
}

impl<T,B> Gate<T,B>
where T:Value, B:AnyStream<Output=bool> {
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



// ===========
// === Map ===
// ===========

pub type Map     <S,F> = Node     <MapData<S,F>>;
pub type WeakMap <S,F> = WeakNode <MapData<S,F>>;
pub struct MapData<S,F> {
    source   : S,
    function : F,
}

impl<S,F,Out> HasOutput for MapData<S,F>
where S : AnyStream, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    type Output = Out;
}

impl<S,F,Out> Map<S,F>
where S : AnyStream, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    pub fn new(src:&S,function:F) -> Self {
        let source     = src.clone_ref();
        let definition = MapData {source,function};
        Self::construct_and_connect(src,definition)
    }
}

impl<S,F,Out> EventConsumer<Output<S>> for NodeData<MapData<S,F>>
where S : AnyStream, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    fn on_event(&self, value:&Output<S>) {
        let out = (self.definition.function)(value);
        self.emit(&out);
    }
}








impl Graph {
    pub fn source<T:Value>(&self) -> WeakSource<T> {
        self.register(Source::<T>::new())
    }

    pub fn gate<T,E,B>(&self, event:&E, behavior:&B) -> WeakGate<T,B>
    where T:Value, E:AnyStream<Output=T>, B:AnyStream<Output=bool> {
        self.register(Gate::new(event,behavior))
    }

    pub fn sample<E:AnyStream,B:AnyStream>(&self, event:&E, behavior:&B) -> WeakSample<B> {
        self.register(Sample::new(event,behavior))
    }

    pub fn toggle<Src:AnyStream>(&self, source:&Src) -> WeakToggle {
        self.register(Toggle::new(source))
    }

    pub fn map<S,F,Out>(&self, source:&S, function:F) -> WeakMap<S,F>
    where S : AnyStream, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
        self.register(Map::<S,F>::new(source,function))
    }
}



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

    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
}
