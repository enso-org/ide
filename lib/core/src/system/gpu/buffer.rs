//! This module implements utilities for managing WebGL buffers.

use crate::prelude::*;

use crate::closure;
use crate::control::callback::Callback;
use crate::control::callback::CallbackFn;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::data::seq::observable::Observable;
use crate::debug::stats::Stats;
use crate::display::render::webgl::Context;
use crate::system::gpu::data::attribute::class::Attribute;
use crate::system::gpu::data::class::JSBufferView;
use crate::system::gpu::data::GpuData;
use crate::system::gpu::data::Item;
use crate::system::web::info;
use crate::system::web::Logger;
use crate::system::web::warning;

use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use shapely::shared;
use std::iter::Extend;
use std::ops::RangeInclusive;
use web_sys::WebGlBuffer;



// =============
// === Types ===
// =============

/// A vector which fires events whenever it is modified or resized.
pub type ObservableVec<T> = Observable<Vec<T>,OnMut,OnResize>;

/// Dirty flag keeping track of the range of modified elements.
pub type MutDirty = dirty::SharedRange<usize,Callback>;

/// Dirty flag keeping track of whether the buffer was resized.
pub type ResizeDirty = dirty::SharedBool<Callback>;

closure! {
fn on_resize_fn(dirty:ResizeDirty) -> OnResize {
    || dirty.set()
}}

closure! {
fn on_mut_fn(dirty:MutDirty) -> OnMut {
    |ix: usize| dirty.set(ix)
}}



// ==============
// === Buffer ===
// ==============

shared! {Buffer
/// CPU-counterpart of WebGL buffers. The buffer data is synchronised with GPU on demand, usually
/// in the update stage before drawing the frame.
#[derive(Debug)]
pub struct BufferData<T> {
    buffer        : ObservableVec<T>,
    mut_dirty     : MutDirty,
    resize_dirty  : ResizeDirty,
    gl_buffer     : WebGlBuffer,
    context       : Context,
    stats         : Stats,
    gpu_mem_usage : u32,
    logger        : Logger,
}

impl<T:GpuData> {
    /// Constructor.
    pub fn new<OnMut:CallbackFn,OnResize:CallbackFn>
    (logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut, on_resize:OnResize) -> Self {
        info!(logger,"Creating new {T::type_display()} buffer.",{
            stats.inc_buffer_count();
            let mut_dirty     = MutDirty::new(logger.sub("mut_dirty"),Callback(on_mut));
            let resize_dirty  = ResizeDirty::new(logger.sub("resize_dirty"),Callback(on_resize));
            let on_resize_fn  = on_resize_fn(resize_dirty.clone_ref());
            let on_mut_fn     = on_mut_fn(mut_dirty.clone_ref());
            let buffer        = ObservableVec::new(on_mut_fn,on_resize_fn);
            let gl_buffer     = create_gl_buffer(&context);
            let context       = context.clone();
            let stats         = stats.clone_ref();
            let gpu_mem_usage = default();
            Self {buffer,mut_dirty,resize_dirty,logger,gl_buffer,context,stats,gpu_mem_usage}
        })
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Checks if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Gets a copy of the data by its index.
    pub fn get(&self, index:usize) -> T {
        *self.buffer.index(index)
    }

    /// Sets data value at the given index.
    pub fn set(&mut self, index:usize, value:T) {
        *self.buffer.index_mut(index) = value;
    }

    /// Adds a single new element initialized to default value.
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    /// Adds multiple new elements initialized to default values.
    pub fn add_elements(&mut self, elem_count:usize) {
        self.extend(iter::repeat(T::empty()).take(elem_count));
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        info!(self.logger, "Updating.", {
            self.context.bind_buffer(Context::ARRAY_BUFFER,Some(&self.gl_buffer));
            if self.resize_dirty.check() {
                self.upload_data(&None);
            } else if self.mut_dirty.check_all() {
                self.upload_data(&self.mut_dirty.take().range);
            } else {
                warning!(self.logger,"Update requested but it was not needed.")
            }
            self.mut_dirty.unset_all();
            self.resize_dirty.unset();
        })
    }

    /// Binds the underlying WebGLBuffer to a given target.
    /// https://developer.mozilla.org/docs/Web/API/WebGLRenderingContext/bindBuffer
    pub fn bind(&self, target:u32) {
        self.context.bind_buffer(target,Some(&self.gl_buffer));
    }

    /// Binds the buffer currently bound to gl.ARRAY_BUFFER to a generic vertex attribute of the
    /// current vertex buffer object and specifies its layout. Please note that this function is
    /// more complex that a raw call to `WebGLRenderingContext.vertexAttribPointer`, as it correctly
    /// handles complex data types like `mat4`. See the following links to learn more:
    /// https://developer.mozilla.org/docs/Web/API/WebGLRenderingContext/vertexAttribPointer
    /// https://stackoverflow.com/questions/38853096/webgl-how-to-bind-values-to-a-mat4-attribute
    pub fn vertex_attrib_pointer(&self, loc:u32, instanced:bool) {
        let item_byte_size = <T as GpuData>::gpu_item_byte_size() as i32;
        let item_type      = <T as GpuData>::glsl_item_type_code();
        let rows           = <T as GpuData>::rows() as i32;
        let cols           = <T as GpuData>::cols() as i32;
        let col_byte_size  = item_byte_size * rows;
        let stride         = col_byte_size  * cols;
        let normalize      = false;
        for col in 0..cols {
            let lloc = loc + col as u32;
            let off  = col * col_byte_size;
            self.context.enable_vertex_attrib_array(lloc);
            self.context.vertex_attrib_pointer_with_i32(lloc,rows,item_type,normalize,stride,off);
            if instanced {
                self.context.vertex_attrib_divisor(lloc, 1);
            }
        }
    }
}}


// === Private API ===

impl<T:GpuData> BufferData<T> {
    /// View the data as slice of primitive elements.
    pub fn as_prim_slice(&self) -> &[Item<T>] {
        <T as GpuData>::convert_prim_buffer(&self.buffer.data)
    }

    /// View the data as slice of elements.
    pub fn as_slice(&self) -> &[T] {
        &self.buffer.data
    }

    /// Uploads the provided data to the GPU buffer.
    fn upload_data(&mut self, opt_range:&Option<RangeInclusive<usize>>) {
        // Note that `js_buffer_view` is somewhat dangerous (hence the `unsafe`!). This is creating
        // a raw view into our module's `WebAssembly.Memory` buffer, but if we allocate more pages
        // for ourself (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the resulting js array to be invalid.
        //
        // As a result, after `js_buffer_view` we have to be very careful not to do any memory
        // allocations before it's dropped.

        self.logger.info("Setting buffer data.");
        self.stats.inc_data_upload_count();

        let data           = self.as_slice();
        let item_byte_size = <T as GpuData>::gpu_item_byte_size() as u32;
        let item_count     = <T as GpuData>::item_count()         as u32;

        match opt_range {
            None => {
                unsafe {
                    let js_array = data.js_buffer_view();
                    self.context.buffer_data_with_array_buffer_view
                    (Context::ARRAY_BUFFER, &js_array, Context::STATIC_DRAW);
                }
                self.stats.mod_gpu_memory_usage(|s| s - self.gpu_mem_usage);
                self.gpu_mem_usage = self.len() as u32 * item_count * item_byte_size;
                self.stats.mod_gpu_memory_usage(|s| s + self.gpu_mem_usage);
                self.stats.mod_data_upload_size(|s| s + self.gpu_mem_usage);
            }
            Some(range) => {
                let start           = *range.start() as u32;
                let end             = *range.end()   as u32;
                let start_item      = start * item_count;
                let length          = (end - start + 1) * item_count;
                let dst_byte_offset = (item_byte_size * item_count * start) as i32;
                unsafe {
                    let js_array = data.js_buffer_view();
                    self.context.buffer_sub_data_with_i32_and_array_buffer_view_and_src_offset_and_length
                    (Context::ARRAY_BUFFER,dst_byte_offset,&js_array,start_item,length)
                }
                self.stats.mod_data_upload_size(|s| s + length * item_byte_size);
            }
        }
    }
}


// === Smart Accessors ===

impl<T:GpuData> Buffer<T> {
    /// Get the attribute pointing to a given buffer index.
    pub fn at(&self, index:usize) -> Attribute<T> {
        Attribute::new(index,self.clone_ref())
    }
}


// === Instances ===

impl<T> Deref for BufferData<T> {
    type Target = ObservableVec<T>;
    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}

impl<T> DerefMut for BufferData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buffer
    }
}

impl<T> Drop for BufferData<T> {
    fn drop(&mut self) {
        self.context.delete_buffer(Some(&self.gl_buffer));
        self.stats.mod_gpu_memory_usage(|s| s - self.gpu_mem_usage);
        self.stats.dec_buffer_count();
    }
}


// === Utils ===

fn create_gl_buffer(context:&Context) -> WebGlBuffer {
    let buffer = context.create_buffer();
    buffer.ok_or("failed to create buffer").unwrap()
}



// ========================
// === TO BE REFACTORED ===
// ========================

// TODO The following code should be refactored to use the new macro `eval-tt`
// TODO engine. Some utils, like `cartesian` macro should also be refactored
// TODO out.

macro_rules! cartesian_impl {
    ($out:tt [] $b:tt $init_b:tt, $f:ident) => {
        $f!{ $out }
    };
    ($out:tt [$a:ident, $($at:tt)*] [] $init_b:tt, $f:ident) => {
        cartesian_impl!{ $out [$($at)*] $init_b $init_b, $f }
    };
    ([$($out:tt)*] [$a:ident, $($at:tt)*] [$b:ident, $($bt:tt)*] $init_b:tt
    ,$f:ident) => {
        cartesian_impl!{
            [$($out)* ($a, $b),] [$a, $($at)*] [$($bt)*] $init_b, $f
        }
    };
}

macro_rules! cartesian {
    ([$($a:tt)*], [$($b:tt)*], $f:ident) => {
        cartesian_impl!{ [] [$($a)*,] [$($b)*,] [$($b)*,], $f }
    };
}



// =================
// === AnyBuffer ===
// =================

use enum_dispatch::*;

// === Macros ===

#[derive(Debug)]
pub struct BadVariant;

macro_rules! mk_any_buffer_impl {
([$(($base:ident, $param:ident)),*,]) => { paste::item! {

    /// An enum with a variant per possible buffer type (i32, f32, Vector<f32>,
    /// and many, many more). It provides a faster alternative to dyn trait one:
    /// `Buffer<dyn GpuData, OnMut, OnResize>`.
    #[enum_dispatch(IsBuffer)]
    #[derive(Debug)]
    pub enum AnyBuffer {
        $(  [<Variant $base For $param>]
                (Buffer<$base<$param>>),
        )*
    }

    $( // ======================================================================

    impl<'t>
    TryFrom<&'t AnyBuffer>
    for &'t Buffer<$base<$param>> {
        type Error = BadVariant;
        fn try_from(v: &'t AnyBuffer)
        -> Result <&'t Buffer<$base<$param>>, Self::Error> {
            match v {
                AnyBuffer::[<Variant $base For $param>](a) => Ok(a),
                _ => Err(BadVariant)
            }
        }
    }

    impl<'t>
    TryFrom<&'t mut AnyBuffer>
    for &'t mut Buffer<$base<$param>> {
        type Error = BadVariant;
        fn try_from(v: &'t mut AnyBuffer)
        -> Result <&'t mut Buffer<$base<$param>>, Self::Error> {
            match v {
                AnyBuffer::[<Variant $base For $param>](a) => Ok(a),
                _ => Err(BadVariant)
            }
        }
    }

    )* // ======================================================================
}
}}

macro_rules! mk_any_buffer {
    ($bases:tt, $params:tt) => {
        cartesian!($bases, $params, mk_any_buffer_impl);
    }
}


// === Definition ===

type Identity<T> = T;
mk_any_buffer!([Identity,Vector2,Vector3,Vector4,Matrix4], [f32]);

/// Collection of all methods common to every buffer variant.
#[enum_dispatch]
pub trait IsBuffer {
    fn add_element(&self);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn update(&self);
    fn bind(&self, target:u32);
    fn vertex_attrib_pointer(&self, index:u32, instanced:bool);
}
