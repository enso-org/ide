#![allow(missing_docs)]

use crate::prelude::*;

use crate::system::gpu::buffer::Buffer;
use crate::system::gpu::data::GpuData;


// =================
// === Attribute ===
// =================

/// View for a particular buffer. Allows reading and writing buffer data
/// via the internal mutability pattern. It is implemented as a view on
/// a selected `Buffer` element under the hood.
#[derive(Clone,Debug,Derivative)]
pub struct Attribute<T> {
    index  : usize,
    buffer : Buffer<T>
}

impl<T> Attribute<T> {
    /// Creates a new variable as an indexed view over provided buffer.
    pub fn new(index:usize, buffer:Buffer<T>) -> Self {
        Self {index, buffer}
    }
}

impl<T:GpuData> Attribute<T> {
    /// Gets a copy of the data this attribute points to.
    pub fn get(&self) -> T {
        self.buffer.get(self.index)
    }

    /// Sets the data this attribute points to.
    pub fn set(&self, value:T) {
        self.buffer.set(self.index,value);
    }

    /// Modifies the data this attribute points to.
    pub fn modify<F:FnOnce(&mut T)>(&self, f:F) {
        let mut value = self.get();
        f(&mut value);
        self.set(value);
    }
}
