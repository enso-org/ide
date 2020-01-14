//! This module defines attributes and related utilities.

use crate::prelude::*;

use crate::closure;
use crate::data::dirty;
use crate::debug::Stats;
use crate::system::gpu::shader::Context;
use data::opt_vec::OptVec;

use crate::data::dirty::traits::*;
use crate::system::gpu::types::*;




// ======================
// === AttributeScope ===
// ======================

/// Scope defines a view for geometry structure. For example, there is point
/// scope or instance scope. Scope contains buffer of data for each item it
/// describes.
#[derive(Debug)]
pub struct AttributeScope {
    buffers      : OptVec<AnyBuffer>,
    buffer_dirty : BufferDirty,
    shape_dirty  : ShapeDirty,
    name_map     : HashMap<String,BufferIndex>,
    logger       : Logger,
    free_ids     : Vec<InstanceIndex>,
    size         : usize,
    context      : Context,
    stats        : Stats,
}


// === Types ===

pub type InstanceIndex = usize;
pub type BufferIndex   = usize;
pub type BufferDirty   = dirty::SharedBitField<u64,Box<dyn Fn()>>;
pub type ShapeDirty    = dirty::SharedBool<Box<dyn Fn()>>;


// === Callbacks ===

closure! {
fn buffer_on_set(dirty:BufferDirty, ix:usize) -> BufferOnSet {
    || dirty.set(ix)
}}

closure! {
fn buffer_on_resize(dirty:ShapeDirty) -> BufferOnResize {
    || dirty.set()
}}


// === Implementation ===

impl AttributeScope {
    /// Create a new scope with the provided dirty callback.
    pub fn new<OnMut:Fn()+Clone+'static>(logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut) -> Self {
        logger.info("Initializing.");
        let stats         = stats.clone_ref();
        let buffer_logger = logger.sub("buffer_dirty");
        let shape_logger  = logger.sub("shape_dirty");
        let buffer_dirty  = BufferDirty::new(buffer_logger,Box::new(on_mut.clone()));
        let shape_dirty   = ShapeDirty::new(shape_logger,Box::new(on_mut));
        let buffers       = default();
        let name_map      = default();
        let free_ids      = default();
        let size          = default();
        let context       = context.clone();
        Self {context,buffers,buffer_dirty,shape_dirty,name_map,logger,free_ids,size,stats}
    }
}

impl AttributeScope {
    /// Adds a new named buffer to the scope.
    pub fn add_buffer<Name:Str, T: BufferItem>(&mut self, name:Name) -> Buffer<T>
    where AnyBuffer: From<Buffer<T>> {
        let name         = name.as_ref().to_string();
        let buffer_dirty = self.buffer_dirty.clone();
        let shape_dirty  = self.shape_dirty.clone();
        let ix           = self.buffers.reserve_ix();
        group!(self.logger, "Adding buffer '{}' at index {}.", name, ix, {
            let on_set     = buffer_on_set(buffer_dirty, ix);
            let on_resize  = buffer_on_resize(shape_dirty);
            let logger     = self.logger.sub(&name);
            let context    = &self.context;
            let buffer     = Buffer::new(logger,&self.stats,context,on_set,on_resize);
            let buffer_ref = buffer.clone();
            self.buffers.set(ix, AnyBuffer::from(buffer));
            self.name_map.insert(name, ix);
            self.shape_dirty.set();
            buffer_ref
        })
    }

    /// Lookups buffer by a given name.
    pub fn buffer(&self, name:&str) -> Option<&AnyBuffer> {
        self.name_map.get(name).map(|i| &self.buffers[*i])
    }

    /// Checks if a buffer with the given name was created in this scope.
    pub fn contains<S:Str>(&self, name:S) -> bool {
        self.name_map.contains_key(name.as_ref())
    }

    /// Adds a new instance to every buffer in the scope.
    pub fn add_instance(&mut self) -> InstanceIndex {
        group!(self.logger, "Adding {} instance(s).", 1, {
            match self.free_ids.pop() {
                Some(ix) => ix,
                None     => {
                    let ix = self.size;
                    self.size += 1;
                    self.buffers.iter_mut().for_each(|t| t.add_element());
                    ix
                }
            }
        })
    }

    /// Disposes instance for reuse in the future. Please note that the disposed data still
    /// exists in the buffer and will be used when rendering. It is yours responsibility to hide
    /// id, fo example by degenerating vertices.
    pub fn dispose(&mut self, id:InstanceIndex) {
        group!(self.logger, "Disposing instance {}.", id, {
            self.free_ids.push(id);
        })
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            if self.shape_dirty.check() {
                for i in 0..self.buffers.len() {
                    self.buffers[i].update()
                }
            } else {
                for i in 0..self.buffers.len() {
                    if self.buffer_dirty.check(&i) {
                        self.buffers[i].update()
                    }
                }
            }
            self.shape_dirty.unset();
            self.buffer_dirty.unset_all();
        })
    }

    /// Returns the size of buffers in this scope.
    pub fn size(&self) -> usize {
        self.size
    }
}



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

impl<T: BufferItem> Attribute<T> {
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
