#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}



pub struct Event<T> {
    phantom: PhantomData<T>,
}


pub struct Behavior<T> {
    phantom: PhantomData<T>,
}



// ============
// === Node ===
// ============


pub trait HasOutput {
    type Output;
}

pub trait HasInput {
    type Input;
}

pub trait NodeOps {
//    fn send_event()
}

pub trait IsNode:  HasOutput + NodeOps {}

impl<T:HasInput+HasOutput+NodeOps> IsNode for T {}


#[derive(Shrinkwrap)]
pub struct Node<T> {
    raw: Rc<dyn IsNode<Output=T>>,
}

impl<T> Node<T> {
    pub fn new<A:IsNode<Output=T>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }

//    pub fn to_any(&self) -> AnyNode {
//
//    }
}

impl<A:IsNode<Output=T>+CloneRef+'static,T> From<&A> for Node<T> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}



#[derive(Shrinkwrap)]
pub struct AnyNode {
    raw: Rc<dyn NodeOps>,
}






// ====================
// === EventEmitter ===
// ====================

shared! { EventEmitterShape

pub struct EventEmitterData<T> {
    callbacks: Vec<Rc<dyn Fn(T)>>
}

impl<T> {
    pub fn new() -> Self {
        let callbacks = default();
        Self {callbacks}
    }
}}

impl<T> HasOutput for EventEmitter<T> {
    type Output = T;
}

impl<T> HasInput for EventEmitter<T> {
    type Input = ();
}

impl<T> NodeOps for EventEmitter<T> {}



type EventEmitter<T> = NodeTemplate<EventEmitterShape<T>>;


impl<T> EventEmitter<T> {
    pub fn new() -> Self {
        let shape   = EventEmitterShape::new();
        let targets = default();
        Self {shape,targets}
    }

    pub fn emit(&self, value:&T) {
        self.targets.borrow().iter().for_each(|target| {

        })
    }
}





// ===============
// === Lambda1 ===
// ===============

shared! { Lambda1

pub struct LambdaData1<A,T> {
    source : Node<A>,
    func   : Rc<dyn Fn(&A) -> T>,
}

impl<A,T> {
    pub fn new<F:'static + Fn(&A) -> T, Source:Into<Node<A>>>
    (source:Source, f:F) -> Self {
        let source = source.into();
        let func   = Rc::new(f);
        Self {source,func}
    }
}}


impl<A,T> HasInput for Lambda1<A,T> {
    type Input = A;
}

impl<A,T> HasOutput for Lambda1<A,T> {
    type Output = T;
}

impl<A,T> NodeOps for Lambda1<A,T> {}



//// ===============
//// === Lambda2 ===
//// ===============
//
//shared! { Lambda2
//
//pub struct LambdaData2<A,B,T> {
//    source1 : Node<A>,
//    source2 : Node<B>,
//    func    : Rc<dyn Fn(&A,&B) -> T>,
//}
//
//impl<A,B,T> {
//    pub fn new<F:'static + Fn(&A,&B) -> T, Source1:Into<Node<A>>, Source2:Into<Node<B>>>
//    (source1:Source1, source2:Source2, f:F) -> Self {
//        let source1 = source1.into();
//        let source2 = source2.into();
//        let func    = Rc::new(f);
//        Self {source1,source2,func}
//    }
//}}
//
//
//impl<A,B,T> HasOutput for Lambda2<A,B,T> {
//    type Output = T;
//}
//
//impl<A,B,T> NodeOps for Lambda2<A,B,T> {}


#[derive(Clone)]
pub struct NodeTemplate<T> {
    pub shape   : T,
    pub targets : Rc<RefCell<Vec<AnyNode>>>
}


impl<T:CloneRef> CloneRef for NodeTemplate<T> {
    fn clone_ref(&self) -> Self {
        let shape   = self.shape.clone_ref();
        let targets = self.targets.clone();
        Self {shape,targets}
    }
}



//////////////////////////////////////////////////////

pub fn test () {

    let e1 = EventEmitter::<i32>::new();

    let n1 = Lambda1::new(&e1, |i| {i+1});
    let n2 = Lambda1::new(&e1, |i| {i+1});

//    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit(&7);

}