#![allow(missing_docs)]

use crate::prelude::*;

use crate::display::render::webgl::Context;
use crate::display::render::webgl::glsl;

use nalgebra::*;
use web_sys::WebGlUniformLocation;
use code_builder::HasCodeRepr;


// =============
// === Types ===
// =============

pub trait MatrixCtx<T,R,C> = where
    T:Scalar, R:DimName, C:DimName,
    DefaultAllocator: nalgebra::allocator::Allocator<T,R,C>,
    <DefaultAllocator as nalgebra::allocator::Allocator<T,R,C>>::Buffer:Copy;






// =============
// === Empty ===
// =============

/// Trait for types which have empty value.
pub trait Empty {
    fn empty() -> Self;
    fn is_empty(&self) -> bool where Self:Sized+PartialEq {
        *self == Self::empty()
    }
}

impl Empty for i32          { fn empty() -> Self { 0   } }
impl Empty for f32          { fn empty() -> Self { 0.0 } }
impl Empty for Vector2<f32> { fn empty() -> Self { Self::new(0.0,0.0)         } }
impl Empty for Vector3<f32> { fn empty() -> Self { Self::new(0.0,0.0,0.0)     } }
impl Empty for Vector4<f32> { fn empty() -> Self { Self::new(0.0,0.0,0.0,1.0) } }
impl Empty for Matrix4<f32> { fn empty() -> Self { Self::identity()           } }



// =================
// === IsUniform ===
// =================

pub type UniformLocation = WebGlUniformLocation;

pub trait ContextUniformOps<T> {
    fn set_uniform(&self, location:&UniformLocation, value:&T);
}

impl ContextUniformOps<i32> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&i32) {
        self.uniform1i(Some(location),*value);
    }
}

impl ContextUniformOps<f32> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&f32) {
        self.uniform1f(Some(location),*value);
    }
}

impl ContextUniformOps<Vector2<f32>> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&Vector2<f32>) {
        self.uniform_matrix2fv_with_f32_array(Some(location),false,value.data.as_slice());
    }
}

impl ContextUniformOps<Vector3<f32>> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&Vector3<f32>) {
        self.uniform_matrix3fv_with_f32_array(Some(location),false,value.data.as_slice());
    }
}

impl ContextUniformOps<Vector4<f32>> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&Vector4<f32>) {
        self.uniform_matrix4fv_with_f32_array(Some(location),false,value.data.as_slice());
    }
}

impl ContextUniformOps<Matrix4<f32>> for Context {
    fn set_uniform(&self, location:&UniformLocation, value:&Matrix4<f32>) {
        self.uniform_matrix4fv_with_f32_array(Some(location),false,value.data.as_slice());
    }
}



// ===============
// === GpuData ===
// ===============

// === Definition ===

pub trait JsBufferViewArr = Sized where [Self]:JsBufferView;

pub trait T1 = where glsl::PrimType: From<PhantomData<Self>>;

/// Class for buffer items, like `f32` or `Vector<f32>`. It defines utils
/// for mapping the item to WebGL buffer and vice versa.
pub trait BufferItem: Copy + Empty + JsBufferViewArr + T1 {

    // === Types ===

    /// The primitive type which this type is build of. In case of the most primitive types, like
    /// `f32` this type may be set to itself.
    type Item: BufferItem;

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

    /// Conversion from slice of a buffer to the item. Buffers contain primitive
    /// values only, so two `Vector3<f32>` are represented there as six `f32`
    /// values. This allows us to view the buffers using desired types.
    fn from_buffer(buffer: &[Self::Item]) -> &[Self];

    /// Mutable conversion from slice of a buffer to the item. See the docs for
    /// `from_buffer` to learn more.
    fn from_buffer_mut(buffer: &mut [Self::Item]) -> &mut [Self];

    // TODO: simplify when this gets resolved: https://github.com/rustsim/nalgebra/issues/687
    fn convert_prim_buffer(buffer: &[Self]) -> &[Self::Item];
    fn convert_prim_buffer_mut(buffer: &mut [Self]) -> &mut [Self::Item];


    // === GLSL ===

    /// Returns the WebGL enum code representing the item type, like Context::FLOAT.
    fn glsl_item_type_code() -> u32 {
        Self::Item::glsl_item_type_code()
    }

    /// Returns the GLSL type name, like `"float"` for `f32`.
    fn glsl_type_name() -> String {
        glsl::PrimType::from_phantom::<Self>().to_code()
    }

    /// Converts the data to GLSL value.
    fn to_glsl(&self) -> String;
}


// === Type Families ===

pub type Item <T> = <T as BufferItem>::Item;
pub type Rows <T> = <T as BufferItem>::Rows;
pub type Cols <T> = <T as BufferItem>::Cols;


// === Instances ===

impl BufferItem for i32 {
    type Item = Self;
    type Rows = U1;
    type Cols = U1;

    fn gpu_byte_size           () -> usize { 4 }
    fn from_buffer             (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn from_buffer_mut         (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
    fn convert_prim_buffer     (buffer: &    [Self]) -> &    [Self::Item] { buffer }
    fn convert_prim_buffer_mut (buffer: &mut [Self]) -> &mut [Self::Item] { buffer }
    fn glsl_item_type_code     () -> u32            { Context::INT }
    fn to_glsl                 (&self) -> String    { self.to_string() }
}

impl BufferItem for f32 {
    type Item = Self;
    type Rows = U1;
    type Cols = U1;

    fn gpu_byte_size           () -> usize { 4 }
    fn from_buffer             (buffer: &    [Self::Item]) -> &    [Self] { buffer }
    fn from_buffer_mut         (buffer: &mut [Self::Item]) -> &mut [Self] { buffer }
    fn convert_prim_buffer     (buffer: &    [Self]) -> &    [Self::Item] { buffer }
    fn convert_prim_buffer_mut (buffer: &mut [Self]) -> &mut [Self::Item] { buffer }
    fn glsl_item_type_code     ()      -> u32            { Context::FLOAT }
    fn to_glsl                 (&self) -> String {
        let is_int = self.fract() == 0.0;
        if is_int { format!("{}.0" , self) }
        else      { format!("{}"   , self) }
    }
}


impl<T: BufferItem<Item=T>,R,C> BufferItem for MatrixMN<T,R,C>
    where T:Default, Self:MatrixCtx<T,R,C>, Self:Empty, Self:T1 {
    type Item = T;
    type Rows = R;
    type Cols = C;

    fn from_buffer(buffer: &[Self::Item]) -> &[Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts(buffer.as_ptr().cast(), len)
        }
    }

    fn from_buffer_mut(buffer: &mut [Self::Item]) -> &mut [Self] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() / Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }

    fn convert_prim_buffer(buffer: &[Self]) -> &[Self::Item] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        let len = buffer.len() * Self::item_count();
        unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast(), len) }
    }

    fn convert_prim_buffer_mut(buffer: &mut [Self]) -> &mut [Self::Item] {
        // This code casts slice to matrix. This is safe because `MatrixMN`
        // uses `nalgebra::Owned` allocator, which resolves to array defined as
        // `#[repr(C)]` under the hood.
        unsafe {
            let len = buffer.len() * Self::item_count();
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr().cast(), len)
        }
    }

    fn to_glsl(&self) -> String {
        let vals:Vec<String> = self.as_slice().iter().cloned().map(|t|format!("{:?}",t)).collect();
        format!("{}({})",Self::glsl_type_name(),vals.join(","))
    }
}


// =================================
// === GLSL PrimType Conversions ===
// =================================

macro_rules! define_prim_type_conversions {
    ($($ty:ty => $name:ident),* $(,)?) => {$(
        impl From<PhantomData<$ty>> for glsl::PrimType {
            fn from(_:PhantomData<$ty>) -> Self {
                Self::$name
            }
        }
    )*}
}

define_prim_type_conversions! {
    i32            => Int,
    f32            => Float,
    Vector2<f32>   => Vec2,
    Vector3<f32>   => Vec3,
    Vector4<f32>   => Vec4,
    Matrix2<f32>   => Mat2,
    Matrix3<f32>   => Mat3,
    Matrix4<f32>   => Mat4,
    Matrix2x3<f32> => Mat2x3,
    Matrix2x4<f32> => Mat2x4,
    Matrix3x2<f32> => Mat3x2,
    Matrix3x4<f32> => Mat3x4,
    Matrix4x2<f32> => Mat4x2,
    Matrix4x3<f32> => Mat4x3,
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

impl<T: BufferItem<Item=T>,R,C> JsBufferView for [MatrixMN<T,R,C>]
    where Self                    : MatrixCtx<T,R,C>,
          T                       : Default,
          MatrixMN<T,R,C>         : BufferItem,
          [Item<MatrixMN<T,R,C>>] : JsBufferView {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        <MatrixMN<T,R,C> as BufferItem>::convert_prim_buffer(self).js_buffer_view()
    }
}

impl<T: BufferItem<Item=T>,R,C> JsBufferView for MatrixMN<T,R,C>
    where Self:MatrixCtx<T,R,C> {
    unsafe fn js_buffer_view(&self) -> js_sys::Object {
        self.as_slice().js_buffer_view()
    }
}
