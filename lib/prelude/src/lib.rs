#![feature(trait_alias)]

pub use core::any::type_name;
pub use core::fmt::Debug;
pub use derivative::Derivative;
pub use derive_more::*;
pub use failure::Fail;
pub use itertools::Itertools;
pub use shrinkwraprs::Shrinkwrap;
pub use std::cell::RefCell;
pub use std::collections::HashMap;
pub use std::fmt::Display;
pub use std::ops::Deref;
pub use std::rc::Rc;
pub use std::iter;

pub trait Str = AsRef<str>;

pub fn default<T: Default>() -> T {
    Default::default()
}
