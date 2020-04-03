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


// ============
// === Node ===
// ============

pub struct WeakNode<T:?Sized> {
    graph : WeakGraph,
    data  : Weak<T>,
}

pub struct StrongNode<T:?Sized> {
    graph : WeakGraph,
    data  : Rc<T>,
}

impl<T> StrongNode<T> {
    pub fn downgrade(&self) -> WeakNode<T> {
        let graph = self.graph.clone_ref();
        let data  = Rc::downgrade(&self.data);
        WeakNode {graph,data}
    }
}

impl<T> WeakNode<T> {
    pub fn upgrade(&self) -> Option<StrongNode<T>> {
        self.data.upgrade().map(|data| {
            let graph = self.graph.clone_ref();
            StrongNode{graph,data}
        })
    }
}



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

    pub fn register<T:'static>(&self, node:StrongNode<T>) {
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
    fn emit(value:&Self::Output);
}

pub trait EventConsumer : HasEventInput {
    fn on_event(value:&Self::EventInput);
}






// ==============
// === Source ===
// ==============

pub type Source<T=()> = WeakNode<SourceData<T>>;
pub struct SourceData<T=()> {
    phantom : PhantomData<T>
}

impl<T:'static> Source<T> {
    pub fn new(g:&Graph) -> Self {
        let phantom = PhantomData;
        let data    = Rc::new(SourceData {phantom});
        let graph   = g.downgrade();
        let strong  = StrongNode {graph,data};
        let weak    = strong.downgrade();
        g.register(strong);
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
//        let strong  = StrongNode {data};
//        let weak    = strong.downgrade();
//        graph.register(strong);
//        weak
//    }
//}





pub fn test() {
    println!("hello");
    let graph  = Graph::new();
    let source = Source::<()>::new(&graph);
}