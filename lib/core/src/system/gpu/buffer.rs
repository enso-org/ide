#![allow(missing_docs)]

use crate::prelude::*;

use crate::closure;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::data::function::callback::*;
use crate::data::seq::observable::Observable;
use crate::debug::stats::Stats;
use crate::display::render::webgl::Context;
use crate::system::gpu::data::attribute::class::Attribute;
use crate::system::gpu::data::GpuData;
use crate::system::gpu::data::Item;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use std::iter::Extend;
use std::ops::RangeInclusive;
use web_sys::WebGlBuffer;



// ==================
// === BufferData ===
// ==================

// === Definition ===

/// Please refer to the 'Buffer management pipeline' doc to learn more about
/// attributes, scopes, geometries, meshes, scenes, and other relevant concepts.
///
/// Buffers are values stored in geometry. Under the hood they are stored in
/// vectors and are synchronised with GPU buffers on demand.
#[derive(Derivative,Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derivative(Debug(bound="T:Debug"))]
pub struct BufferData<T> {
    #[shrinkwrap(main_field)]
    pub buffer       : Data<T>,
    pub buffer_dirty : BufferDirty,
    pub resize_dirty : ResizeDirty,
    pub logger       : Logger,
    pub gl_buffer    : WebGlBuffer,
    context          : Context,
    stats            : Stats,
    gpu_mem_usage    : u32,
}


// === Types ===

pub type ObservableVec<T,OnMut,OnResize> = Observable<Vec<T>,OnMut,OnResize>;
pub type Data<T> = ObservableVec<T,DataOnSet,DataOnResize>;

#[macro_export]
/// Promote relevant types to parent scope. See `promote!` macro for more information.
macro_rules! promote_buffer_types { ($callbacks:tt $module:ident) => {
    promote! { $callbacks $module [BufferData<T>,Buffer<T>,AnyBuffer] }
};}


// === Callbacks ===

pub type BufferDirty = dirty::SharedRange<usize,Box<dyn Fn()>>;
pub type ResizeDirty = dirty::SharedBool<Box<dyn Fn()>>;

closure! {
fn buffer_on_resize(dirty:ResizeDirty) -> DataOnResize {
    || dirty.set()
}}

closure! {
fn buffer_on_mut(dirty:BufferDirty) -> DataOnSet {
    |ix: usize| dirty.set(ix)
}}


// === Instances ===

impl<T> BufferData<T> {
    /// Creates a new empty buffer.
    pub fn new<OnMut:Fn()+'static,OnResize:Fn()+'static>
    (logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut, on_resize:OnResize) -> Self {
        stats.inc_buffer_count();
        logger.info(fmt!("Creating new {} buffer.", T::type_display()));
        let stats          = stats.clone_ref();
        let set_logger     = logger.sub("buffer_dirty");
        let resize_logger  = logger.sub("resize_dirty");
        let buffer_dirty   = BufferDirty::new(set_logger,Box::new(on_mut));
        let resize_dirty   = ResizeDirty::new(resize_logger,Box::new(on_resize));
        let buff_on_resize = buffer_on_resize(resize_dirty.clone_ref());
        let buff_on_mut    = buffer_on_mut(buffer_dirty.clone_ref());
        let buffer         = Data::new(buff_on_mut, buff_on_resize);
        let context        = context.clone();
        let gl_buffer      = create_gl_buffer(&context);
        let gpu_mem_usage  = default();
        Self {buffer,buffer_dirty,resize_dirty,logger,gl_buffer,context,stats,gpu_mem_usage}
    }
}

impl<T:GpuData> BufferData<T> {

    /// View the data as slice of primitive elements.
    pub fn as_prim_slice(&self) -> &[Item<T>] {
        <T as GpuData>::convert_prim_buffer(&self.buffer.data)
    }

    /// View the data as slice of elements.
    pub fn as_slice(&self) -> &[T] {
        &self.buffer.data
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            self.context.bind_buffer(Context::ARRAY_BUFFER, Some(&self.gl_buffer));
            if self.resize_dirty.check() {
                self.upload_data(&None);
            } else if self.buffer_dirty.check_all() {
                let range = &self.buffer_dirty.take().range;
                self.upload_data(range);
            }
            self.buffer_dirty.unset_all();
            self.resize_dirty.unset();
        })
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
            None => unsafe {
                let js_array = data.js_buffer_view();
                self.context.buffer_data_with_array_buffer_view
                (Context::ARRAY_BUFFER, &js_array, Context::STATIC_DRAW);

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
}

impl<T> BufferData<T> {
    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Checks if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Binds the underlying WebGLBuffer to a given target.
    /// https://developer.mozilla.org/docs/Web/API/WebGLRenderingContext/bindBuffer
    pub fn bind(&self, target:u32) {
        self.context.bind_buffer(target, Some(&self.gl_buffer));
    }
}

pub trait AddElementCtx<T> = where
    T: GpuData + Clone;

impl<T>
BufferData<T> where Self: AddElementCtx<T> {
    /// Adds a single new element initialized to default value.
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    /// Adds multiple new elements initialized to default values.
    pub fn add_elements(&mut self, elem_count: usize) {
        self.extend(iter::repeat(T::empty()).take(elem_count));
    }
}

impl<T>
Index<usize> for BufferData<T> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        self.buffer.index(index)
    }
}

impl<T>
IndexMut<usize> for BufferData<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.buffer.index_mut(index)
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



// ==============
// === Buffer ===
// ==============

/// Shared view for `Buffer`.
#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
#[derivative(Clone(bound=""))]
pub struct Buffer<T> {
    pub rc: Rc<RefCell<BufferData<T>>>
}

impl<T> Buffer<T> {
    /// Creates a new empty buffer.
    pub fn new<OnMut:Fn()+'static,OnResize:Fn()+'static>
    (logger:Logger, stats:&Stats, context:&Context, on_mut:OnMut, on_resize:OnResize) -> Self {
        let data = BufferData::new(logger,stats,context,Box::new(on_mut),Box::new(on_resize));
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl<T:GpuData> Buffer<T> {
    /// Check dirty flags and update the state accordingly.
    pub fn update(&self) {
        self.rc.borrow_mut().update()
    }

    /// binds the buffer currently bound to gl.ARRAY_BUFFER to a generic vertex
    /// attribute of the current vertex buffer object and specifies its layout.
    /// https://developer.mozilla.org/docs/Web/API/WebGLRenderingContext/vertexAttribPointer
    pub fn vertex_attrib_pointer(&self, index:u32, instanced:bool) {
        self.rc.borrow().vertex_attrib_pointer(index,instanced)
    }
}

impl<T> Buffer<T> {
    // FIXME: Rethink if buffer should know about Attribute.
    /// Get the variable by given index.
    pub fn get(&self, index:usize) -> Attribute<T> {
        Attribute::new(index, self.clone())
    }

    /// Returns the number of elements in the buffer.
    pub fn len(&self) -> usize {
        self.rc.borrow().len()
    }

    /// Checks if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.rc.borrow().is_empty()
    }

    /// Binds the underlying WebGLBuffer to a given target.
    /// https://developer.mozilla.org/docs/Web/API/WebGLRenderingContext/bindBuffer
    pub fn bind(&self, target:u32) {
        self.rc.borrow().bind(target)
    }
}

impl<T> Buffer<T> where (): AddElementCtx<T> {
    /// Adds a single new element initialized to default value.
    pub fn add_element(&self){
        self.rc.borrow_mut().add_element()
    }
}

impl <T>
From<Rc<RefCell<BufferData<T>>>> for Buffer<T> {
    fn from(rc: Rc<RefCell<BufferData<T>>>) -> Self {
        Self {rc}
    }
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
