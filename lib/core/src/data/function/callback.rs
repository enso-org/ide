#![allow(missing_docs)]

use crate::prelude::*;
use std::fmt;


// =============
// === Types ===
// =============

pub type NoCallback = ();

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Proc<Func>(pub Func);

impl<Func> Debug for Proc<Func> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Callback")
    }
}



// =================
// === Instances ===
// =================

pub trait Proc0 {
    fn call(&mut self);
}

pub trait Proc1<Arg1> {
    fn call(&mut self, arg1:Arg1);
}


// === Unit Implementations ===

impl<T: Proc0> Proc0 for Option<T> {
    fn call(&mut self) {
        self.iter_mut().for_each(|t| {
            t.call()
        })
    }
}

impl Proc0 for () {
    fn call(&mut self) {}
}

impl<Arg1> Proc1<Arg1> for () {
    fn call(&mut self, _arg1:Arg1) {}
}


// === FnMut Implementations ===

impl<F: FnMut() -> T, T> Proc0 for F {
    fn call(&mut self) {
        self();
    }
}

impl<Arg1, F:FnMut(Arg1) -> T, T> Proc1<Arg1> for F {
    fn call(&mut self, arg1:Arg1) {
        self(arg1);
    }
}
