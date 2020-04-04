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






//outputs : Rc<RefCell<Vec<dyn >>>


//pub struct WeakNodeTemplate<T:?Sized> {
//    graph   : Graph,
//    data    : Weak<T>,
//    outputs : Rc<RefCell<Vec<dyn Any>>>,
//}






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

    pub fn register<T:NodeDefinition>(&self, node:Node<T>) {
        let node = Box::new(node);
        self.data.nodes.borrow_mut().push(node);
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

//pub trait HasEventInput {
//    type EventInput : Value;
//}



// ====================
// === EventEmitter ===
// ====================


pub trait Input = 'static + ValueProvider + EventEmitter + CloneRef;

pub trait EventEmitter : HasOutput {
    fn emit(&self, value:&Self::Output);
    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>);
}

pub trait EventConsumer<T> {
    fn on_event(&self, value:&T);
}

pub trait ValueProvider : HasOutput {
    fn value(&self) -> Self::Output;
}




// ============
// === Node ===
// ============

pub trait NodeDefinition = 'static + ?Sized + HasOutput;

pub struct NodeData<Def:NodeDefinition> {
    targets    : RefCell<Vec<Weak<dyn EventConsumer<Output<Def>>>>>,
    last_value : RefCell<Output<Def>>,
    definition : Def,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakNode<T:NodeDefinition> {
    data : Weak<NodeData<T>>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct Node<T:NodeDefinition> {
    data : Rc<NodeData<T>>,
}

#[derive(CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct WeakAnyNode<Out> {
    data : Weak<dyn EventEmitter<Output=Out>>,
}

impl<Def:NodeDefinition> From<WeakNode<Def>> for WeakAnyNode<Def::Output> {
    fn from(node:WeakNode<Def>) -> Self {
        WeakAnyNode {data:node.data}
    }
}

impl<Def:NodeDefinition> Node<Def> {
    pub fn construct(definition:Def) -> Self {
        let targets    = default();
        let last_value = default();
        let data    = Rc::new(NodeData {targets,last_value,definition});
        Self {data}
    }

    pub fn downgrade(&self) -> WeakNode<Def> {
        let data = Rc::downgrade(&self.data);
        WeakNode {data}
    }
}

impl<T:NodeDefinition> WeakNode<T> {
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.data.upgrade().map(|data| Node{data})
    }
}

impl<Def:NodeDefinition> HasOutput for Node     <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for WeakNode <Def> { type Output = Output<Def>; }
impl<Def:NodeDefinition> HasOutput for NodeData <Def> { type Output = Output<Def>; }

impl<Def:NodeDefinition> EventEmitter for WeakNode<Def> {
    fn emit(&self, value:&Output<Def>) {
        self.data.upgrade().for_each(|data| data.emit(value))
    }

    fn register_target(&self,tgt:Weak<dyn EventConsumer<Output<Self>>>) {
        self.data.upgrade().for_each(|data| data.register_target(tgt))
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

impl<Def:NodeDefinition> ValueProvider for Node<Def> {
    fn value(&self) -> Self::Output {
        self.data.value()
    }
}

impl<Def:NodeDefinition> ValueProvider for WeakNode<Def> {
    fn value(&self) -> Self::Output {
        self.data.upgrade().map(|data| data.value()).unwrap_or_default()
    }
}

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

impl<Def:NodeDefinition> ValueProvider for NodeData<Def> {
    fn value(&self) -> Self::Output {
        self.last_value.borrow().clone()
    }
}


// ==============
// === Source ===
// ==============

pub type Source     <T=()> = Node     <SourceData<T>>;
pub type WeakSource <T=()> = WeakNode <SourceData<T>>;
pub struct SourceData<T=()> {
    phantom : PhantomData<T>
}

impl<T:Value> HasOutput for SourceData<T> {
    type Output = T;
}

impl<T:Value> Source<T> {
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
    pub fn new<Src:Input>(src:&Src) -> Self {
        let value      = default();
        let definition = ToggleData {value};
        let this       = Self::construct(definition);
        let weak       = this.downgrade();
        src.register_target(weak.data);
        this
    }
}

impl<T> EventConsumer<T> for NodeData<ToggleData> {
    fn on_event(&self, _:&T) {
        let value = !self.definition.value.get();
        self.definition.value.set(value);
        self.emit(&value);
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

impl<Source:Input> Sample<Source> {
    pub fn new<E:Input>(event:&E,behavior:&Source) -> Self {
        let behavior   = behavior.clone_ref();
        let definition = SampleData {behavior};
        let this       = Self::construct(definition);
        let weak       = this.downgrade();
        event.register_target(weak.data);
        this
    }
}

impl<T,Source:Input> EventConsumer<T> for NodeData<SampleData<Source>> {
    fn on_event(&self, _:&T) {
        self.emit(&self.definition.behavior.value());
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
where S : Input, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    type Output = Out;
}

impl<S,F,Out> Map<S,F>
where S : Input, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    pub fn new(src:&S,function:F) -> Self {
        let source     = src.clone_ref();
        let definition = MapData {source,function};
        let this       = Self::construct(definition);
        let weak       = this.downgrade();
        src.register_target(weak.data);
        this
    }
}

impl<S,F,Out> EventConsumer<Output<S>> for NodeData<MapData<S,F>>
where S : Input, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
    fn on_event(&self, value:&Output<S>) {
        let out = (self.definition.function)(value);
        self.emit(&out);
    }
}








impl Graph {
    pub fn source<T:Value>(&self) -> WeakSource<T> {
        let node = Source::<T>::new();
        let weak = node.downgrade();
        self.register(node);
        weak
    }

    pub fn sample<Event:Input,Behavior:Input>(&self, event:&Event, behavior:&Behavior) -> WeakSample<Behavior> {
        let node = Sample::new(event,behavior);
        let weak = node.downgrade();
        self.register(node);
        weak
    }


    pub fn toggle<Src:Input>(&self, source:&Src) -> WeakToggle {
        let node = Toggle::new(source);
        let weak = node.downgrade();
        self.register(node);
        weak
    }

    pub fn map<S,F,Out>(&self, source:&S, function:F) -> WeakMap<S,F>
    where S : Input, F : 'static + Fn(&Output<S>) -> Out, Out : Value {
        let node = Map::<S,F>::new(source,function);
        let weak = node.downgrade();
        self.register(node);
        weak
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
    let fff2   = frp.map(&bb,|t| { println!(">> {:?}",t) });

    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
    source2.emit(&());
    source.emit(&5.0);
}
