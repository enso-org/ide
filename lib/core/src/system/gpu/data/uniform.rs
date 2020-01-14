#![allow(missing_docs)]

use crate::prelude::*;

use enum_dispatch::*;
use nalgebra::Matrix4;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use shapely::shared;
use web_sys::WebGlUniformLocation;

use crate::display::render::webgl::Context;
use crate::system::gpu::data::texture::*;
use crate::system::web::Logger;



// =====================
// === UniformUpload ===
// =====================

/// Abstraction for uploading uniforms to GPU based on their types.
pub trait UniformUpload {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation);
}

impl UniformUpload for i32 {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform1i(Some(location),*self);
    }
}

impl UniformUpload for f32 {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform1f(Some(location),*self);
    }
}

impl UniformUpload for Vector2<f32> {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform_matrix2fv_with_f32_array(Some(location),false,self.data.as_slice());
    }
}

impl UniformUpload for Vector3<f32> {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform_matrix3fv_with_f32_array(Some(location),false,self.data.as_slice());
    }
}

impl UniformUpload for Vector4<f32> {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform_matrix4fv_with_f32_array(Some(location),false,self.data.as_slice());
    }
}

impl UniformUpload for Matrix4<f32> {
    fn upload_uniform(&self, context:&Context, location:&WebGlUniformLocation) {
        context.uniform_matrix4fv_with_f32_array(Some(location),false,self.data.as_slice());
    }
}



// ====================
// === UniformValue ===
// ====================

/// Describes every value which can be kept inside an Uniform.
pub trait UniformValue = UniformUpload;

/// Some values need to be initialized before they can be used as uniforms. Textures, for example,
/// need to allocate memory on GPU and if used with remote source, need to download images.
/// For primitive types, like numbers or matrices, the binding operation does nothing.
pub trait IntoUniformValue = IntoUniformValueImpl where
    Uniform<AsUniformValue<Self>>: Into<AnyUniform>;

/// Internal helper for `IntoUniformValue`.
pub trait IntoUniformValueImpl {
    type Result;
    fn into_uniform_value(self, context:&Context) -> Self::Result;
}

/// Result of the binding operation.
pub type AsUniformValue<T> = <T as IntoUniformValueImpl>::Result;

// TODO[wd]: Make those generic with marcos:

impl IntoUniformValueImpl for f32 {
    type Result = f32;
    fn into_uniform_value(self, _context:&Context) -> Self::Result {
        self
    }
}

impl IntoUniformValueImpl for i32 {
    type Result = i32;
    fn into_uniform_value(self, _context:&Context) -> Self::Result {
        self
    }
}

impl IntoUniformValueImpl for Matrix4<f32> {
    type Result = Matrix4<f32>;
    fn into_uniform_value(self, _context:&Context) -> Self::Result {
        self
    }
}


// ====================
// === UniformScope ===
// ====================

shared! { UniformScope

/// A scope containing set of uniform values.
#[derive(Debug)]
pub struct UniformScopeData {
    map     : HashMap<String,AnyUniform>,
    logger  : Logger,
    context : Context,
}

impl {
    /// Constructor.
    pub fn new(logger:Logger, context:&Context) -> Self {
        let map     = default();
        let context = context.clone();
        Self {map,logger,context}
    }

    /// Look up uniform by name.
    pub fn get<Name:Str>(&self, name:Name) -> Option<AnyUniform> {
        self.map.get(name.as_ref()).cloned()
    }

    /// Checks if uniform of a given name was defined in this scope.
    pub fn contains<Name:Str>(&self, name:Name) -> bool {
        self.map.contains_key(name.as_ref())
    }

    /// Add a new uniform with a given name and initial value. Returns `None` if the name is in use.
    /// Please note that the value will be bound to the context before it becomes the uniform.
    /// Refer to the docs of `IntoUniformValue` to learn more.
    pub fn add<Name:Str, Value:IntoUniformValue>
    (&mut self, name:Name, value:Value) -> Option<Uniform<AsUniformValue<Value>>> {
        self.add_or_else(name,value,Some,|_|None)
    }

    /// Add a new uniform with a given name and initial value. Panics if the name is in use.
    /// Please note that the value will be bound to the context before it becomes the uniform.
    /// Refer to the docs of `IntoUniformValue` to learn more.
    pub fn add_or_panic<Name:Str, Value:IntoUniformValue>
    (&mut self, name:Name, value:Value) -> Uniform<AsUniformValue<Value>> {
        self.add_or_else(name,value,|t|{t},|name| {
            panic!("Trying to override uniform '{}'.", name.as_ref())
        })
    }
}}

impl UniformScopeData {
    /// Adds a new uniform with a given name and initial value. In case the name was already in use,
    /// it fires the `fail` function. Otherwise, it fires the `ok` function on the newly created
    /// uniform.
    pub fn add_or_else<Name:Str,Value:IntoUniformValue,Ok,Fail,T>
    (&mut self, name:Name, value:Value, ok:Ok, fail:Fail) -> T
    where Ok   : Fn(Uniform<AsUniformValue<Value>>)->T,
          Fail : Fn(Name)->T {
        if self.map.contains_key(name.as_ref()) { fail(name) } else {
            let bound_value = value.into_uniform_value(&self.context);
            let uniform     = Uniform::new(bound_value);
            let any_uniform = uniform.clone().into();
            self.map.insert(name.into(),any_uniform);
            ok(uniform)
        }
    }
}



// ===================
// === UniformData ===
// ===================

shared! { Uniform

/// An uniform value.
#[derive(Debug)]
pub struct UniformData<Value> {
    value: Value,
    dirty: bool,
}

impl<Value> {
    /// Constructor.
    pub fn new(value:Value) -> Self {
        let dirty = true;
        Self {value,dirty}
    }

    /// Sets the value of this uniform.
    pub fn set(&mut self, value:Value) {
        self.set_dirty();
        self.value = value;
    }

    /// Modifies the value stored by the uniform.
    pub fn modify<F:FnOnce(&mut Value)>(&mut self, f:F) {
        self.set_dirty();
        f(&mut self.value);
    }

    /// Checks whether the uniform was changed and not yet updated.
    pub fn check_dirty(&self) -> bool {
        self.dirty
    }

    /// Sets the dirty flag.
    pub fn set_dirty(&mut self) {
        self.dirty = true;
    }

    /// Clears the dirty flag.
    pub fn unset_dirty(&mut self) {
        self.dirty = false;
    }
}}

impl<Value:UniformValue> UniformData<Value> {
    /// Uploads the uniform data to the provided location of the currently bound shader program.
    pub fn upload(&self, context:&Context, location:&WebGlUniformLocation) {
        self.value.upload_uniform(context,location);
    }
}

impl<Value:UniformValue> Uniform<Value> {
    /// Uploads the uniform data to the provided location of the currently bound shader program.
    pub fn upload(&self, context:&Context, location:&WebGlUniformLocation) {
        self.rc.borrow().upload(context,location)
    }
}

impl<Value> Uniform<Value> where Context : ContextTextureOps<Value,Guard=TextureBindingGuard> {
    /// Bind texture in this WebGl context.
    pub fn bind_texture_unit(&self, context:&Context, unit:u32) -> TextureBindingGuard {
        let value = &self.rc.borrow().value;
        context.bind_texture_unit(value,unit)
    }
}


// ======================
// === AnyPrimUniform ===
// ======================

/// Existentially typed uniform value.
#[allow(non_camel_case_types)]
#[enum_dispatch(AnyPrimUniformOps)]
#[derive(Clone,Debug)]
pub enum AnyPrimUniform {
    Variant_i32           (Uniform<i32>),
    Variant_f32           (Uniform<f32>),
    Variant_Vector2_of_f32(Uniform<Vector2<f32>>),
    Variant_Vector3_of_f32(Uniform<Vector3<f32>>),
    Variant_Matrix4_of_f32(Uniform<Matrix4<f32>>)
}

/// Set of operations exposed by the `AnyPrimUniform` value.
#[enum_dispatch]
pub trait AnyPrimUniformOps {
    fn upload(&self, context:&Context, location:&WebGlUniformLocation);
}


pub type Identity<T> = T;

macro_rules! with_all_prim_types {
    ( $f:ident ) => {
        $f! { [Identity i32] [Identity f32] [Vector2 f32] [Vector3 f32] [Matrix4 f32] }
    }
}



// =========================
// === AnyTextureUniform ===
// =========================

macro_rules! gen_any_texture_uniform {
    ( $([$internal_format:tt $type:tt])* ) => { paste::item! {
        #[allow(missing_docs)]
        #[allow(non_camel_case_types)]
        #[enum_dispatch(AnyTextureUniformOps)]
        #[derive(Clone,Debug)]
        pub enum AnyTextureUniform {
            $( [< $internal_format _ $type >] (Uniform<BoundTexture<$internal_format,$type>>) ),*
        }
    }}
}

macro_rules! gen_prim_conversions {
    ( $([$t1:ident $t2:ident])* ) => {$(
        impl From<Uniform<$t1<$t2>>> for AnyUniform {
            fn from(t:Uniform<$t1<$t2>>) -> Self {
                Self::Prim(t.into())
            }
        }
    )*}
}

macro_rules! gen_texture_conversions {
    ( $([$internal_format:tt $type:tt])* ) => {$(
        impl From<Uniform<BoundTexture<$internal_format,$type>>> for AnyUniform {
            fn from(t:Uniform<BoundTexture<$internal_format,$type>>) -> Self {
                Self::Texture(t.into())
            }
        }
    )*}
}

crate::with_all_texture_types!(gen_any_texture_uniform);


#[enum_dispatch]
pub trait AnyTextureUniformOps {
    fn bind_texture_unit(&self, context:&Context, unit:u32) -> TextureBindingGuard;
}



// ==================
// === AnyUniform ===
// ==================

#[derive(Clone,Debug)]
pub enum AnyUniform {
    Prim(AnyPrimUniform),
    Texture(AnyTextureUniform)
}

with_all_prim_types!(gen_prim_conversions);
crate::with_all_texture_types!(gen_texture_conversions);
