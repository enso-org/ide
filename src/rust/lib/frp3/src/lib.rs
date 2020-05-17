#![feature(test)]

extern crate test;


use std::os::raw::c_void;


pub trait EventConsumer<T> {
    fn on_event(&self, data:&T);
}

pub struct Node<Def> {
    pub definition : Def,
    pub targets    : Vec<usize>,
    pub watchers   : Vec<usize>,
}


pub enum AnyDef {

}



pub struct Inc {}

impl EventConsumer<usize> for Inc {
    fn on_event(&self, data:&usize) {}
}





pub struct Network {
    nodes: Vec<Node<AnyDef>>,
}


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