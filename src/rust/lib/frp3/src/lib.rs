#![feature(test)]
#![feature(trait_alias)]

extern crate test;


use enso_prelude::*;

use std::os::raw::c_void;


pub trait Data = Clone;

pub trait HasOutput {
    type Output;
}

pub type Output<T> = <T as HasOutput>::Output;



pub trait EventConsumer<T> {
    fn on_event(&self, data:&T);
}

pub type Node<Def> = NodeTemplate<Def,Output<Def>>;

pub struct NodeTemplate<Def,Out> {
    pub definition : Def,
    pub targets    : Vec<usize>,
    pub watchers   : Vec<usize>,
    pub value      : Out,
    pub active     : bool,
}

impl<Def,Out> Deref for NodeTemplate<Def,Out> {
    type Target = Def;
    fn deref(&self) -> &Self::Target {
        &self.definition
    }
}

impl<Def,Out> HasOutput for NodeTemplate<Def,Out> {
    type Output = Out;
}

impl<Def,Out> ValueProvider for NodeTemplate<Def,Out> {
    fn value(&self) -> &Self::Output {
        &self.value
    }
}


pub trait ValueProvider : HasOutput {
    fn value(&self) -> &Self::Output;
}

pub trait OutputData = HasOutput where Output<Self>:Data;

pub trait Foo               : ValueProvider + OutputData {}
impl<T>   Foo for T where T : ValueProvider + OutputData {}



// ============
// === Gate ===
// ============

pub struct GateData<T1,T2> { source:T1, condition:T2 }
pub type Gate<T1,T2> = Node<GateData<T1,T2>>;

impl<T1,T2> HasOutput for GateData<T1,T2>
where T1:HasOutput {
    type Output = Output<T1>;
}

impl<T1,T2> Gate<T1,T2>
where T1:Foo, T2:Foo<Output=bool> {
    pub fn update(&mut self) -> bool {
        let condition = *self.condition.value();
        if  condition { self.value = self.source.value().clone() }
        condition
    }
}

//
//
//pub struct Network {
//    nodes: Vec<Node<AnyDef>>,
//}


pub fn add(i:usize) -> usize {
    i + 4
}

pub fn add_(i: *const c_void) -> usize {
    let j = unsafe { *(i as *const usize) };
    add(j)
}

pub fn test () {
    println!("Hello world");

    let mut state : usize = 20;
    let state_ptr: *const c_void = &state as *const _ as *const c_void;
    println!(">> {}", add_(state_ptr));
}

pub trait Adder {
    fn adder(&self) -> usize;
}

impl Adder for usize {
    fn adder(&self) -> usize {
        add(*self)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;


    #[bench]
    fn bench_1(b: &mut Bencher) {
        let state : usize = test::black_box(10);
        let n = test::black_box(1000000);
        let state_ptr: *const c_void = test::black_box(&state as *const _ as *const c_void);

        b.iter(|| {
            (0..n).for_each(|_| {
                add_(state_ptr);
            })
        });
    }

    #[bench]
    fn bench_2(b: &mut Bencher) {
        let state : usize = test::black_box(10);
        let n = test::black_box(1000000);

        let add_x = test::black_box(Box::new(state) as Box<dyn Adder>);

        b.iter(move || {
            (0..n).for_each(|_| {
                add_x.adder();
            })
        });
    }
}