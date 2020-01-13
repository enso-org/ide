#![allow(missing_docs)]

use crate::prelude::*;

use enum_dispatch::*;
use nalgebra::Matrix4;
use nalgebra::Vector3;
use shapely::shared;
use web_sys::WebGlUniformLocation;

use crate::display::render::webgl::Context;
use crate::system::gpu::data::ContextUniformOps;
use crate::system::web::Logger;
use crate::system::gpu::data::texture::*;


// ================
// === Bindable ===
// ================
//
//trait Bindable {
//    type Result;
//    fn bind(self, context:&Context) -> Self::Result;
//}
//
//default impl<T> Bindable for T {
//    type Result = T;
//    fn bind(self, context:&Context) -> Self::Result {
//        self
//    }
//}
//
//impl<I,T> Bindable for Texture<I,T> {
//    type Result = BoundTexture<I,T>;
//    fn bind(self, context:&Context) -> Self::Result {
//        BoundTexture::new(self,context)
//    }
//}
//


// =============
// === Types ===
// =============

/// A set of constraints that every uniform has to met.
pub trait UniformValue = Sized where
    AnyUniform : From<Uniform<Self>>;
//    Context    : ContextUniformOps<Self>;


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
    pub fn add<Name:Str, Value:UniformValue>
    (&mut self, name:Name, value:Value) -> Option<Uniform<Value>> {
        self.add_or_else(name,value,Some,|_|None)
    }

    /// Add a new uniform with a given name and initial value. Panics if the name is in use.
    pub fn add_or_panic<Name:Str, Value:UniformValue>
    (&mut self, name:Name, value:Value) -> Uniform<Value> {
        self.add_or_else(name,value,|t|{t},|name| {
            panic!("Trying to override uniform '{}'.", name.as_ref())
        })
    }
}}


impl UniformScopeData {
    pub fn add_or_panic2 <Name:Str,I:InternalFormat,T:PrimType>
    (&mut self, name:Name, value:Texture<I,T>) -> Uniform<BoundTexture<I,T>>
        where AnyUniform: From<Uniform<BoundTexture<I,T>>> {
        let uniform = Uniform::new(BoundTexture::new(value,&self.context));
        let any_uniform = uniform.clone().into();
        self.map.insert(name.into(), any_uniform);
        uniform
    }
}

impl UniformScope {
    pub fn add_or_panic2 <Name:Str,I:InternalFormat,T:PrimType>
    (&self, name:Name, value:Texture<I,T>) -> Uniform<BoundTexture<I,T>>
    where AnyUniform: From<Uniform<BoundTexture<I,T>>> {
        self.rc.borrow_mut().add_or_panic2(name,value)
    }
}

impl UniformScopeData {
    /// Adds a new uniform with a given name and initial value. In case the name was already in use,
    /// it fires the `fail` function. Otherwise, it fires the `ok` function on the newly created
    /// uniform.
    pub fn add_or_else<Name:Str, Value:UniformValue, Ok:Fn(Uniform<Value>)->T, Fail:Fn(Name)->T, T>
    (&mut self, name:Name, value:Value, ok:Ok, fail:Fail) -> T {
        if self.map.contains_key(name.as_ref()) { fail(name) } else {
            let uniform     = Uniform::new(value);
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

impl<Value:UniformValue> {
    /// Constructor.
    pub fn new(value:Value) -> Self {
        let dirty = false;
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

impl<Value:UniformValue> UniformData<Value> where Context: ContextUniformOps<Value> {
    /// Uploads the uniform data to the provided location of the currently bound shader program.
    pub fn upload(&self, context:&Context, location:&WebGlUniformLocation) {
        context.set_uniform(location,&self.value);
    }
}

impl<Value:UniformValue> Uniform<Value> where Context: ContextUniformOps<Value> {
    /// Uploads the uniform data to the provided location of the currently bound shader program.
    pub fn upload(&self, context:&Context, location:&WebGlUniformLocation) {
        self.rc.borrow().upload(context,location)
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
        $f! { [Identity i32] [Identity f32] [Vector3 f32] [Matrix4 f32] }
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
