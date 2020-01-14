//! This module defines abstraction for items in buffers stored on GPU.

use crate::prelude::*;

use crate::display::render::webgl::glsl;
use crate::system::gpu::data::gl_enum::*;
use crate::system::gpu::data::ShaderDefault;
use crate::display::render::webgl::glsl::Glsl;

use nalgebra::*;
use code_builder::HasCodeRepr;


// =============
// === Types ===
// =============

pub trait MatrixCtx<T,R,C> = where
    T:Scalar, R:DimName, C:DimName,
    DefaultAllocator: nalgebra::allocator::Allocator<T,R,C>,
    <DefaultAllocator as nalgebra::allocator::Allocator<T,R,C>>::Buffer:Copy;



// =================
// === IsUniform ===
// =================


//pub trait ContextUniformOps<T> {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&T);
//}
//
//impl ContextUniformOps<i32> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&i32) {
//        self.uniform1i(Some(location),*value);
//    }
//}
//
//impl ContextUniformOps<f32> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&f32) {
//        self.uniform1f(Some(location),*value);
//    }
//}
//
//impl ContextUniformOps<Vector2<f32>> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&Vector2<f32>) {
//        self.uniform_matrix2fv_with_f32_array(Some(location),false,value.data.as_slice());
//    }
//}
//
//impl ContextUniformOps<Vector3<f32>> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&Vector3<f32>) {
//        self.uniform_matrix3fv_with_f32_array(Some(location),false,value.data.as_slice());
//    }
//}
//
//impl ContextUniformOps<Vector4<f32>> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&Vector4<f32>) {
//        self.uniform_matrix4fv_with_f32_array(Some(location),false,value.data.as_slice());
//    }
//}
//
//impl ContextUniformOps<Matrix4<f32>> for Context {
//    fn set_uniform(&self, location:&WebGlUniformLocation, value:&Matrix4<f32>) {
//        self.uniform_matrix4fv_with_f32_array(Some(location),false,value.data.as_slice());
//    }
//}



// ==================
// === BufferItem ===
// ==================

// === Definition ===

pub trait JsBufferViewArr = Sized where [Self]:JsBufferView;

/// Super bounds of the `BufferItem::Item` type;
pub trait ItemBounds = BufferItem + PhantomInto<GlEnum>;

/// Super bounds of the `BufferItem` trait.
pub trait BufferItemBounds =
    Copy + ShaderDefault + JsBufferViewArr + PhantomInto<glsl::PrimType> + Into<Glsl>;

/// Class for buffer items, like `f32` or `Vector<f32>`.
///
/// WebGL buffers contain primitive values only, so for example, two `Vector3<f32>` are represented
/// as six `f32` values. This trait defines fast conversions (views) for the underlying flat data
/// storage.
pub trait BufferItem: BufferItemBounds {

    // === Types ===

    /// The primitive type which this type is build of. In case of the most primitive types, like
    /// `f32` this type may be set to itself.
    type Item: ItemBounds;

    /// The number of rows of the type encoded as 2d matrix.
    type Rows: DimName;

    /// The number of columns of the type encoded as 2d matrix.
    type Cols: DimName;


    // === Size ===

    /// Returns the number of rows of the type encoded as 2d matrix.
    fn rows() -> usize {
        <Self::Rows as DimName>::dim()
    }

    /// Returns the number of columns of the type encoded as 2d matrix.
    fn cols() -> usize {
        <Self::Cols as DimName>::dim()
    }

    /// Count of primitives of the item. For example, `Vector3<f32>` contains
    /// three primitives (`f32` values).
    fn item_count() -> usize {
        Self::rows() * Self::cols()
    }

    /// Returns the size in bytes in GPU memory of the type.
    fn gpu_byte_size() -> usize {
        Self::gpu_item_byte_size() * Self::item_count()
    }

    /// Returns the size in bytes in GPU memory of the primitive type of this type.
    fn gpu_item_byte_size() -> usize {
        Self::Item::gpu_byte_size()
    }


    // === Conversions ===

    /// Conversion from a slice of items to a buffer slice.
    fn slice_from_items(buffer: &[Self::Item]) -> &[Self];

    /// Conversion from a mutable slice of items to a mutable buffer slice.
    fn slice_from_items_mut(buffer: &mut [Self::Item]) -> &mut [Self];

    /// Converts from a buffer slice to a slice of items.
    fn slice_to_items(buffer: &[Self]) -> &[Self::Item];

    /// Converts from a mutable buffer slice to a mutable slice of items.
    fn slice_to_items_mut(buffer: &mut [Self]) -> &mut [Self::Item];


    // === GLSL ===

    // TODO: Remove when it gets resolved: https://github.com/rust-lang/rust/issues/68210
    /// Returns the WebGL enum code representing the item type, like Context::FLOAT.
    fn glsl_item_type_code() -> GlEnum {
        Self::Item::gl_enum()
    }

    /// Returns the GLSL type name, like `"float"` for `f32`.
    fn glsl_type_name() -> String {
        Self::phantom_to::<glsl::PrimType>().to_code()
    }
}


// === Type Families ===

/// Item accessor.
pub type Item <T> = <T as BufferItem>::Item;

/// Rows accessor.
pub type Rows <T> = <T as BufferItem>::Rows;

/// Cols accessor.
pub type Cols <T> = <T as BufferItem>::Cols;


// === Instances ===

impl BufferItem for i32 {
    type Item = Self;
    type Rows = U1;
    type Cols = U1;

    fn gpu_byte_size        () -> usize { 4 }
    fn slice_from_items     (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn slice_from_items_mut (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
    fn slice_to_items       (buffer: &    [Self]) -> &    [Self::Item] { buffer }
    fn slice_to_items_mut   (buffer: &mut [Self]) -> &mut [Self::Item] { buffer }
}

impl BufferItem for f32 {
    type Item = Self;
    type Rows = U1;
    type Cols = U1;

    fn gpu_byte_size           () -> usize { 4 }
    fn slice_from_items             (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn slice_from_items_mut         (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
    fn slice_to_items     (buffer: &    [Self]) -> &    [Self::Item] { buffer }
    fn slice_to_items_mut (buffer: &mut [Self]) -> &mut [Self::Item] { buffer }
}


impl<T: BufferItem<Item=T>,R,C> BufferItem for MatrixMN<T,R,C>
    where T:ItemBounds, Self:MatrixCtx<T,R,C>, Self:ShaderDefault + PhantomInto<glsl::PrimType> {
    type Item = T;
    type Rows = R;
    type Cols = C;

    fn slice_from_items(buffer: &[Self::Item]) -> &[Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
        }
    }

    fn slice_from_items_mut(buffer: &mut [Self::Item]) -> &mut [Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }

    fn slice_to_items(buffer: &[Self]) -> &[Self::Item] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        let len = buffer.len() * Self::item_count();
        unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast(), len) }
    }

    fn slice_to_items_mut(buffer: &mut [Self]) -> &mut [Self::Item] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() * Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }
}



// ====================
// === JsBufferView ===
// ====================

pub trait JsBufferView {
    /// Creates a JS typed array which is a view into wasm's linear memory at the slice specified.
    ///
    /// This function returns a new typed array which is a view into wasm's memory. This view does
    /// not copy the underlying data.
    ///
    /// # Safety
    ///
    /// Views into WebAssembly memory are only valid so long as the backing buffer isn't resized in
    /// JS. Once this function is called any future calls to `Box::new` (or malloc of any form) may
    /// cause the returned value here to be invalidated. Use with caution!
    ///
    /// Additionally the returned object can be safely mutated but the input slice isn't guaranteed
    /// to be mutable.
    ///
    /// Finally, the returned object is disconnected from the input slice's lifetime, so there's no
    /// guarantee that the data is read at the right time.
    unsafe fn js_buffer_view(&self) -> js_sys::Object;
}


// === Instances ===

impl JsBufferView for [i32] {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        js_sys::Int32Array::view(self).into()
    }
}

impl JsBufferView for [f32] {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        js_sys::Float32Array::view(self).into()
    }
}

impl JsBufferView for [u8] {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        js_sys::Uint8Array::view(self).into()
    }
}

impl<T: BufferItem<Item=T>,R,C> JsBufferView for [MatrixMN<T,R,C>]
    where Self                    : MatrixCtx<T,R,C>,
          T                       : ItemBounds,
          MatrixMN<T,R,C>         : BufferItem,
          [Item<MatrixMN<T,R,C>>] : JsBufferView {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        <MatrixMN<T,R,C> as BufferItem>::slice_to_items(self).js_buffer_view()
    }
}

impl<T: BufferItem<Item=T>,R,C> JsBufferView for MatrixMN<T,R,C>
    where Self:MatrixCtx<T,R,C>, T:ItemBounds {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        self.as_slice().js_buffer_view()
    }
}
