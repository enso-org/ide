#![allow(missing_docs)]

pub mod upload;

use crate::prelude::*;

use enum_dispatch::*;
use shapely::shared;
use upload::UniformUpload;
use web_sys::WebGlUniformLocation;

use crate::system::gpu::shader::Context;
use crate::system::gpu::data::texture::*;
use crate::system::gpu::data::prim::*;



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


// === Instances ===

macro_rules! define_identity_uniform_value_impl {
    ( [] [$([$t1:ident $t2:ident])*] ) => {$(
        impl IntoUniformValueImpl for $t1<$t2> {
            type Result = $t1<$t2>;
            fn into_uniform_value(self, _context:&Context) -> Self::Result {
                self
            }
        }
    )*}
}
crate::with_all_prim_types!([[define_identity_uniform_value_impl][]]);

impl<S:StorageRelation<I,T>,I,T> IntoUniformValueImpl for Texture<S,I,T> {
    type Result = Texture<S,I,T>;
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
        self.add_or_else(name,value,Some,|_,_,_|None)
    }

    /// Add a new uniform with a given name and initial value. Panics if the name is in use.
    /// Please note that the value will be bound to the context before it becomes the uniform.
    /// Refer to the docs of `IntoUniformValue` to learn more.
    pub fn add_or_panic<Name:Str, Value:IntoUniformValue>
    (&mut self, name:Name, value:Value) -> Uniform<AsUniformValue<Value>> {
        self.add_or_else(name,value,|t|{t},|name,_,_| {
            panic!("Trying to override uniform '{}'.", name.as_ref())
        })
    }


}}

impl UniformScopeData {
    /// Adds a new uniform with a given name and initial value. In case the name was already in use,
    /// it fires the `on_exist` function. Otherwise, it fires the `on_fresh` function on the newly
    /// created uniform.
    pub fn add_or_else<Name:Str,Value:IntoUniformValue,OnFresh,OnExist,T>
    (&mut self, name:Name, value:Value, on_fresh:OnFresh, on_exist:OnExist) -> T
    where OnFresh : FnOnce(Uniform<AsUniformValue<Value>>)->T,
          OnExist : FnOnce(Name,Value,&AnyUniform)->T {
        match self.map.get(name.as_ref()) {
            Some(v) => on_exist(name,value,v),
            None => {
                let bound_value = value.into_uniform_value(&self.context);
                let uniform     = Uniform::new(bound_value);
                let any_uniform = uniform.clone().into();
                self.map.insert(name.into(),any_uniform);
                on_fresh(uniform)
            }
        }
    }

    /// Gets an existing uniform or adds a new one in case it was missing. Returns `None` if the
    /// uniform exists but its type does not match the requested one.
    pub fn get_or_add<Name:Str, Value:IntoUniformValue>
    (&mut self, name:Name, value:Value) -> Option<Uniform<AsUniformValue<Value>>>
    where for<'t> &'t Uniform<AsUniformValue<Value>> : TryFrom<&'t AnyUniform> {
        let context = self.context.clone();
        self.add_or_else(name,value,Some,move |_,value,uniform| {
            let out:Option<&Uniform<AsUniformValue<Value>>> = uniform.try_into().ok();
            let out = out.cloned();
            match &out {
                Some(t) => {
                    let bound_value = value.into_uniform_value(&context);
                    t.set(bound_value);
                }
                None => {}
            }
            out
        })
    }
}

impl UniformScope {
    /// Gets an existing uniform or adds a new one in case it was missing. Returns `None` if the
    /// uniform exists but its type does not match the requested one.
    pub fn get_or_add<Name:Str, Value:IntoUniformValue>
    (&self, name:Name, value:Value) -> Option<Uniform<AsUniformValue<Value>>>
    where for<'t> &'t Uniform<AsUniformValue<Value>> : TryFrom<&'t AnyUniform> {
        self.rc.borrow_mut().get_or_add(name,value)
    }
}



// ===============
// === Uniform ===
// ===============

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
}

impl<Value:Clone> {
    /// Reads the value of this uniform.
    pub fn get(&self) -> Value {
        self.value.clone()
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


// ========================
// === Texture Uniforms ===
// ========================

impl<T> HasContent for Uniform<T> {
    type Content = T;
}

impl<T> WithContent for Uniform<T> {
    fn with_content<F:FnOnce(&Self::Content)->R,R>(&self, f:F) -> R {
        f(&self.rc.borrow().value)
    }
}



// ======================
// === AnyPrimUniform ===
// ======================

#[derive(Clone,Copy,Debug)]
pub struct TypeMismatch;

macro_rules! define_any_prim_uniform {
    ( [] [$([$t1:ident $t2:ident])*] ) => { paste::item! {
        /// Existentially typed uniform value.
        #[allow(non_camel_case_types)]
        #[enum_dispatch(AnyPrimUniformOps)]
        #[derive(Clone,Debug)]
        pub enum AnyPrimUniform {
            $([<Variant_ $t1 _ $t2>](Uniform<$t1<$t2>>)),*
        }

        $(impl<'t> TryFrom<&'t AnyPrimUniform> for &'t Uniform<$t1<$t2>> {
            type Error = TypeMismatch;
            fn try_from(value:&'t AnyPrimUniform) -> Result<Self,Self::Error> {
                match value {
                    AnyPrimUniform::[<Variant_ $t1 _ $t2>](t) => Ok(t),
                    _ => Err(TypeMismatch)
                }
            }
        })*
    }}
}
crate::with_all_prim_types!([[define_any_prim_uniform][]]);

/// Set of operations exposed by the `AnyPrimUniform` value.
#[enum_dispatch]
pub trait AnyPrimUniformOps {
    fn upload(&self, context:&Context, location:&WebGlUniformLocation);
}



// =========================
// === AnyTextureUniform ===
// =========================

// Note, we could do it using static dispatch instead (like in the AnyPrimUniform case) if this
// gets fixed: https://github.com/rust-lang/rust/issues/68324 .

#[derive(Clone,Debug,Shrinkwrap)]
pub struct AnyTextureUniform {
    pub raw: Box<dyn AnyTextureUniformOps>
}

clone_boxed!(AnyTextureUniformOps);
pub trait AnyTextureUniformOps:CloneBoxedForAnyTextureUniformOps + TextureOps + Debug {
    fn as_any(&self) -> &dyn Any;
}

impl<T:TextureOps+Debug+'static> AnyTextureUniformOps for Uniform<T>
where Uniform<T>: TextureOps {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<T:AnyTextureUniformOps + 'static> From<T> for AnyTextureUniform {
    fn from(t:T) -> Self {
        let raw = Box::new(t);
        Self {raw}
    }
}

impl<'t,S:StorageRelation<I,T>,I:InternalFormat,T:ItemType>
TryFrom<&'t AnyTextureUniform> for &'t Uniform<Texture<S,I,T>> {
    type Error = TypeMismatch;
    fn try_from(value:&'t AnyTextureUniform) -> Result<Self,Self::Error> {
        value.as_any().downcast_ref().ok_or(TypeMismatch)
    }
}


macro_rules! define_get_or_add_gpu_texture_dyn {
    ( [ $([$internal_format:ident $item_type:ident])* ] ) => {
        pub fn get_or_add_gpu_texture_dyn<P:Into<GpuOnlyData>>
        ( context         : &Context
        , scope           : &UniformScope
        , name            : &str
        , internal_format : AnyInternalFormat
        , item_type       : AnyItemType
        , provider        : P
        , parameters      : Option<Parameters>
        ) -> AnyTextureUniform {
            let provider = provider.into();
            match (internal_format,item_type) {
                $((AnyInternalFormat::$internal_format, AnyItemType::$item_type) => {
                    let mut texture = Texture::<GpuOnly,$internal_format,$item_type>
                                ::new(&context,provider);
                    if let Some(parameters) = parameters {
                        texture.set_parameters(parameters);
                    }
                    let uniform = scope.get_or_add(name,texture).unwrap();
                    uniform.into()
                })*
                _ => panic!("Invalid (internal format, item type) combination.")
            }
        }
    }
}


crate::with_all_texture_types! ([define_get_or_add_gpu_texture_dyn _]);



// ==================
// === AnyUniform ===
// ==================

#[derive(Clone,Debug)]
pub enum AnyUniform {
    Prim(AnyPrimUniform),
    Texture(AnyTextureUniform)
}


// === Conversions ===

impl<T> From<Uniform<T>> for AnyUniform where Uniform<T>:IntoAnyUniform {
    fn from(t:Uniform<T>) -> Self {
        t.into_any_uniform()
    }
}

pub trait IntoAnyUniform: Sized {
    fn into_any_uniform(self) -> AnyUniform;
}

impl<T:Into<AnyPrimUniform>> IntoAnyUniform for T {
    default fn into_any_uniform(self) -> AnyUniform {
        AnyUniform::Prim(self.into())
    }
}

impl<S:StorageRelation<I,T>,I:InternalFormat,T:ItemType>
IntoAnyUniform for Uniform<Texture<S,I,T>> {
    fn into_any_uniform(self) -> AnyUniform {
        AnyUniform::Texture(AnyTextureUniform {raw: Box::new(self)})
    }
}

macro_rules! generate_prim_type_downcasts {
    ( [] [$([$t1:ident $t2:ident])*] ) => {
        $(impl<'t> TryFrom<&'t AnyUniform> for &'t Uniform<$t1<$t2>> {
            type Error = TypeMismatch;
            fn try_from(value:&'t AnyUniform) -> Result<Self,Self::Error> {
                match value {
                    AnyUniform::Prim(t) => t.try_into(),
                    _ => Err(TypeMismatch)
                }
            }
        })*
    }
}
crate::with_all_prim_types!([[generate_prim_type_downcasts][]]);


impl<'t,S:StorageRelation<I,T>,I:InternalFormat,T:ItemType>
TryFrom<&'t AnyUniform> for &'t Uniform<Texture<S,I,T>>
where &'t Uniform<Texture<S,I,T>> : TryFrom<&'t AnyTextureUniform, Error=TypeMismatch> {
    type Error = TypeMismatch;
    fn try_from(value:&'t AnyUniform) -> Result<Self,Self::Error> {
        match value {
            AnyUniform::Texture(t) => t.try_into(),
            _ => Err(TypeMismatch)
        }
    }
}
