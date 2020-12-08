#![allow(missing_docs)]

use crate::prelude::*;
use std::fmt;



// =============
// === Types ===
// =============

pub type NoCallback = ();

pub struct Function<F>(pub F);

impl<F> Deref for Function<F> {
    type Target = F;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<F> DerefMut for Function<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<F> Debug for Function<F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Callback")
    }
}



// =================
// === Instances ===
// =================


pub trait Function0 {
    fn call(&self);
}

pub trait Function1<Arg1> {
    fn call(&self, arg1:Arg1);
}


pub trait FunctionMut0 {
    fn call(&mut self);
}

pub trait FunctionMut1<Arg1> {
    fn call(&mut self, arg1:Arg1);
}


// === Unit Implementations ===

impl<T:FunctionMut0> FunctionMut0 for Option<T> {
    fn call(&mut self) {
        if let Some(f) = self {
            f.call()
        }
    }
}

impl FunctionMut0 for () {
    fn call(&mut self) {}
}

impl<Arg1> FunctionMut1<Arg1> for () {
    fn call(&mut self, _arg1:Arg1) {}
}


impl<T:Function0> Function0 for Option<T> {
    fn call(&self) {
        if let Some(f) = self {
            f.call()
        }
    }
}

impl Function0 for () {
    fn call(&self) {}
}

impl<Arg1> Function1<Arg1> for () {
    fn call(&self, _arg1:Arg1) {}
}


// === FnMut Implementations ===

impl<F: FnMut() -> T, T> FunctionMut0 for F {
    fn call(&mut self) {
        self();
    }
}

impl<Arg1, F:FnMut(Arg1) -> T, T> FunctionMut1<Arg1> for F {
    fn call(&mut self, arg1:Arg1) {
        self(arg1);
    }
}

impl<F: Fn() -> T, T> Function0 for F {
    fn call(&self) {
        self();
    }
}

impl<Arg1, F:Fn(Arg1) -> T, T> Function1<Arg1> for F {
    fn call(&self, arg1:Arg1) {
        self(arg1);
    }
}
