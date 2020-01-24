#![allow(missing_docs)]


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}



// ============
// === Node ===
// ============


pub trait HasOutput {
    type Output;
}

pub struct Node<T> {
    raw: Rc<dyn HasOutput<Output=T>>,
}

impl<T> Node<T> {
    pub fn new<A:HasOutput<Output=T>+'static>(a:A) -> Self {
        let raw = Rc::new(a);
        Self {raw}
    }

    pub fn clone_ref(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<A:HasOutput<Output=T>+CloneRef+'static,T> From<&A> for Node<T> {
    fn from(a:&A) -> Self {
        Self::new(a.clone_ref())
    }
}



// ====================
// === EventEmitter ===
// ====================

shared! { EventEmitter

pub struct EventEmitterData<T> {
    callbacks: Vec<Rc<dyn Fn(T)>>
}

impl<T> {
    pub fn new() -> Self {
        let callbacks = default();
        Self {callbacks}
    }

    pub fn emit(&self, value:&T) {

    }
}}

impl<T> HasOutput for EventEmitter<T> {
    type Output = T;
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


impl<A,T> HasOutput for Lambda1<A,T> {
    type Output = T;
}



// ===============
// === Lambda2 ===
// ===============

shared! { Lambda2

pub struct LambdaData2<A,B,T> {
    source1 : Node<A>,
    source2 : Node<B>,
    func    : Rc<dyn Fn(&A,&B) -> T>,
}

impl<A,B,T> {
    pub fn new<F:'static + Fn(&A,&B) -> T, Source1:Into<Node<A>>, Source2:Into<Node<B>>>
    (source1:Source1, source2:Source2, f:F) -> Self {
        let source1 = source1.into();
        let source2 = source2.into();
        let func    = Rc::new(f);
        Self {source1,source2,func}
    }
}}


impl<A,B,T> HasOutput for Lambda2<A,B,T> {
    type Output = T;
}





//////////////////////////////////////////////////////

pub fn test () {

    let e1 = EventEmitter::<i32>::new();

    let n1 = Lambda1::new(&e1, |i| {i+1});
    let n2 = Lambda1::new(&e1, |i| {i+1});

    let n3 = Lambda2::new(&n1,&n2,|i,j| {i * j});


    e1.emit(&7);

}