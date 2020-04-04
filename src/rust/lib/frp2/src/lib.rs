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

    pub fn register<T:'static>(&self, node:Node<T>) {
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

pub trait Value = Debug;



// =================
// === HasOutput ===
// =================

pub trait HasOutput {
    type Output : Value;
}

pub type Output<T> = <T as HasOutput>::Output;

pub trait HasEventInput {
    type EventInput : Value;
}



// ====================
// === EventEmitter ===
// ====================


pub trait EventEmitter : HasOutput {
    fn emit(&self, value:&Self::Output);
}

pub trait EventConsumer : HasEventInput {
    fn on_event(&self, value:&Self::EventInput);
}




// ============
// === Node ===
// ============

pub trait NodeDefinition = ?Sized+HasOutput;

pub struct NodeData<Def:?Sized+HasOutput> {
    targets    : Vec<WeakNode<dyn EventConsumer<EventInput=Output<Def>>>>,
    definition : Def,
}

pub struct WeakNode<T:?Sized+HasOutput> {
    data : Weak<NodeData<T>>,
}

pub struct Node<T:?Sized> {
    data : Rc<NodeData<T>>,
}

impl<T> Node<T> {
    pub fn construct(definition:T) -> Self {
        let targets = default();
        let data    = Rc::new(NodeData {definition,targets});
        Self {data}
    }

    pub fn downgrade(&self) -> WeakNode<T> {
        let data = Rc::downgrade(&self.data);
        WeakNode {data}
    }
}

impl<T> WeakNode<T> {
    pub fn upgrade(&self) -> Option<Node<T>> {
        self.data.upgrade().map(|data| Node{data})
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

impl<T:'static> Source<T> {
    pub fn new() -> Self {
        let phantom    = PhantomData;
        let definition = SourceData {phantom};
        Self::construct(definition)
    }
}


impl Graph {
    pub fn source<T:'static>(&self) -> WeakSource<T> {
        let node = Source::<T>::new();
        let weak = node.downgrade();
        self.register(node);
        weak
    }
}


// ==============
// === Source ===
// ==============

pub type Lambda<F> = WeakNode<LambdaData<F>>;
pub struct LambdaData<F> {
    function : F
}

//impl<T:'static> Lambda<T> {
//    pub fn new(graph:&Graph) -> Self {
//        let phantom = PhantomData;
//        let data    = Rc::new(LambdaData {phantom});
//        let strong  = Node {data};
//        let weak    = strong.downgrade();
//        graph.register(strong);
//        weak
//    }
//}





pub fn test() {
    println!("hello");
    let graph  = Graph::new();
    let source = graph.source::<()>();
}