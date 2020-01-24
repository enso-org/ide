


use crate::prelude::*;


shared! { Switch
pub struct SwitchData {
    prev: Vec<Switch>
}}



pub struct Event<T> {
    value: T
}



// ============
// === Node ===
// ============


pub trait Node {
    type Output;

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

    pub fn get(&self) -> Event<T> {
        todo!()
    }
}}

impl<T> Node for EventEmitter<T> {
    type Output = T;
}



// ===========
// === Map ===
// ===========

shared! { Map

pub struct MapData<A,B> {
    source : Rc<dyn Node<Output=A>>,
    func   : Rc<dyn Fn(&A) -> B>,
}

impl<A,B> {
    pub fn new<F:'static + Fn(&A) -> B, Source:Node<Output=A>+'static>
    (source:&Source, f:F) -> Self {
        let source = Rc::new(source.clone_ref());
        let func   = Rc::new(f);
        Self {source,func}
    }
}}


impl<A,B> Node for Map<A,B> {
    type Output = B;
}







//////////////////////////////////////////////////////

pub fn test () {

    let e1 = EventEmitter::<i32>::new();

    let n1 = Map::new(&e1, |i| {i+1});
//    let n2 = Map::new(&e1, |i| {i+2});

}