#![allow(missing_docs)]

pub mod upload;

use crate::prelude::*;

use enso_shapely::shared;
use enum_dispatch::*;
use upload::UniformUpload;
use web_sys::WebGlUniformLocation;

use crate::system::gpu::data::prim::*;
use crate::system::gpu::data::texture::*;
use crate::system::gpu::shader::Context;

use web_sys::WebGlTexture;



// ====================
// === UniformValue ===
// ====================

/// Describes every value which can be stored inside of an uniform.
pub trait UniformValue = Sized where Uniform<Self>: Into<AnyUniform>;



// ====================
// === UniformScope ===
// ====================

shared! { UniformScope

/// A scope containing set of uniform values.
#[derive(Debug)]
pub struct UniformScopeData {
    map     : HashMap<String,AnyUniform>,
    logger  : Logger,
}

impl {
    /// Constructor.
    pub fn new(logger:Logger) -> Self {
        let map = default();
        Self {map,logger}
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
        self.add_or_else(name,value,Some,|_,_,_|None)
    }

    /// Add a new uniform with a given name and initial value. Panics if the name is in use.
    pub fn add_or_panic<Name:Str, Value:UniformValue>
    (&mut self, name:Name, value:Value) -> Uniform<Value> {
        self.add_or_else(name,value,|t|{t},|name,_,_| {
            panic!("Trying to override uniform '{}'.", name.as_ref())
        })
    }
}}

impl UniformScopeData {
    /// Adds a new uniform with a given name and initial value. In case the name was already in use,
    /// it fires the `on_exist` function. Otherwise, it fires the `on_fresh` function on the newly
    /// created uniform.
    pub fn add_or_else<Name: Str, Value: UniformValue, OnFresh, OnExist, T>(
        &mut self,
        name: Name,
        value: Value,
        on_fresh: OnFresh,
        on_exist: OnExist,
    ) -> T
    where
        OnFresh: FnOnce(Uniform<Value>) -> T,
        OnExist: FnOnce(Name, Value, &AnyUniform) -> T,
    {
        match self.map.get(name.as_ref()) {
            Some(v) => on_exist(name, value, v),
            None => {
                let uniform = Uniform::new(value);
                let any_uniform = uniform.clone().into();
                self.map.insert(name.into(), any_uniform);
                on_fresh(uniform)
            }
        }
    }

    /// Gets an existing uniform or adds a new one in case it was missing. Returns `None` if the
    /// uniform exists but its type does not match the requested one.
    pub fn get_or_add<Name: Str, Value: UniformValue>(
        &mut self,
        name: Name,
        value: Value,
    ) -> Option<Uniform<Value>>
    where
        for<'t> &'t Uniform<Value>: TryFrom<&'t AnyUniform>,
    {
        self.add_or_else(name, value, Some, move |_, value, uniform| {
            let out: Option<&Uniform<Value>> = uniform.try_into().ok();
            let out = out.cloned();
            if let Some(t) = &out {
                t.set(value)
            }
            out
        })
    }
}

impl UniformScope {
    /// Gets an existing uniform or adds a new one in case it was missing. Returns `None` if the
    /// uniform exists but its type does not match the requested one.
    pub fn get_or_add<Name: Str, Value: UniformValue>(
        &self,
        name: Name,
        value: Value,
    ) -> Option<Uniform<Value>>
    where
        for<'t> &'t Uniform<Value>: TryFrom<&'t AnyUniform>,
    {
        self.rc.borrow_mut().get_or_add(name, value)
    }
}



// ===============
// === Uniform ===
// ===============

// TODO: Finish the dirty implementation. After uniform is changed it should be marked dirty. After
//       each frame, all uniforms should be marked as non-dirty. When rendering an object, it should
//       check for dirty unforms in its bindigs. There are few interesting cases - when object is
//       hidden and is shown, its obsolete uniforms need to be updated. This may be realized by
//       using an int or frame number instead of bool to mark dirty state and checking if the
//       object uniform was uploaded with the newest version.
//
//       Please note that currently a special uniform 'zoom' is modified in the render loop. See
//       the `scene::View` implementation to learn more.

shared! { Uniform

/// An uniform value.
#[derive(Debug)]
pub struct UniformData<Value> {
    value: Value,
    // dirty: bool,
}

impl<Value> {
    /// Constructor.
    pub fn new(value:Value) -> Self {
        // let dirty = true;
        Self {value}
    }

    /// Sets the value of this uniform.
    pub fn set(&mut self, value:Value) {
        // self.set_dirty();
        self.value = value;
    }

//    /// Checks whether the uniform was changed and not yet updated.
//    pub fn check_dirty(&self) -> bool {
//        self.dirty
//    }

//    /// Sets the dirty flag.
//    pub fn set_dirty(&mut self) {
//        self.dirty = true;
//    }
//
//    /// Clears the dirty flag.
//    pub fn unset_dirty(&mut self) {
//        self.dirty = false;
//    }
}

impl<Value:Clone> {
    /// Reads the value of this uniform.
    pub fn get(&self) -> Value {
        self.value.clone()
    }
}}

impl<Value> Uniform<Value> {
    pub fn swap(&self, that: &Self) {
        self.rc.borrow_mut().swap(&mut *that.rc.borrow_mut())
    }
}

impl<Value> UniformData<Value> {
    pub fn swap(&mut self, that: &mut Self) {
        mem::swap(self, that)
    }
}



// ========================
// === Texture Uniforms ===
// ========================

impl<T> HasContent for Uniform<T> {
    type Content = T;
}

impl<T> WithContent for Uniform<T> {
    fn with_content<F: FnOnce(&Self::Content) -> R, R>(&self, f: F) -> R {
        f(&self.rc.borrow().value)
    }
}



// ======================
// === AnyPrimUniform ===
// ======================

#[derive(Clone, Copy, Debug)]
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
    /// Uploads the uniform data to the provided location of the currently bound shader program.
    fn upload(&self, context: &Context, location: &WebGlUniformLocation);
}

impl<Value: UniformUpload> AnyPrimUniformOps for Uniform<Value> {
    fn upload(&self, context: &Context, location: &WebGlUniformLocation) {
        self.rc.borrow().upload(context, location)
    }
}

impl<Value: UniformUpload> AnyPrimUniformOps for UniformData<Value> {
    fn upload(&self, context: &Context, location: &WebGlUniformLocation) {
        self.value.upload_uniform(context, location)
    }
}



// =========================
// === AnyTextureUniform ===
// =========================

macro_rules! define_any_texture_uniform {
    ( [ $([$storage:ident $internal_format:ident $item_type:ident])* ] ) => { paste::item! {
        #[allow(non_camel_case_types)]
        #[derive(Clone,CloneRef,Debug)]
        pub enum AnyTextureUniform {
            $([<$storage _ $internal_format _ $item_type >]
                (Uniform<Texture<$storage,$internal_format,$item_type>>)),*
        }

        impl AnyTextureUniform {
            pub fn try_swap(&self, that:&Self) -> bool {
                match (self,that) {
                    $(
                        ( Self::[<$storage _ $internal_format _ $item_type >](a)
                        , Self::[<$storage _ $internal_format _ $item_type >](b)
                        ) => {a.swap(b); true},
                    )*
                    _ => false
                }
            }
        }

        impl TextureOps for AnyTextureUniform {
            fn bind_texture_unit
            (&self, context:&crate::display::Context, unit:TextureUnit) -> TextureBindGuard {
                match self {
                    $(
                        Self::[<$storage _ $internal_format _ $item_type >](t) =>
                            t.bind_texture_unit(context,unit)
                    ),*
                }
            }

            fn gl_texture(&self) -> WebGlTexture {
                match self {
                    $(Self::[<$storage _ $internal_format _ $item_type >](t) => t.gl_texture()),*
                }
            }

            fn get_format(&self) -> AnyFormat {
                match self {
                    $(Self::[<$storage _ $internal_format _ $item_type >](t) => t.get_format()),*
                }
            }

            fn get_item_type(&self) -> AnyItemType {
                match self {
                    $(Self::[<$storage _ $internal_format _ $item_type >](t) => t.get_item_type()),*
                }
            }
        }

        $(
            impl From<Uniform<Texture<$storage,$internal_format,$item_type>>>
            for AnyTextureUniform {
                fn from(t:Uniform<Texture<$storage,$internal_format,$item_type>>) -> Self {
                    Self::[<$storage _ $internal_format _ $item_type >](t)
                }
            }

            impl<'t> TryFrom<&'t AnyTextureUniform>
            for &'t Uniform<Texture<$storage,$internal_format,$item_type>> {
                type Error = TypeMismatch;
                fn try_from(value:&'t AnyTextureUniform) -> Result<Self,Self::Error> {
                    match value {
                        AnyTextureUniform::[<$storage _ $internal_format _ $item_type >](t) => Ok(t),
                        _ => Err(TypeMismatch),
                    }
                }
            }
        )*
    }}
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
                _ => panic!("Invalid internal format and item type combination ({:?},{:?}).",
                    internal_format,item_type)
            }
        }
    }
}

macro_rules! generate {
    ( [ $([$internal_format:ident $item_type:ident])* ] ) => {
        define_any_texture_uniform!{[ $([GpuOnly $internal_format $item_type])* ]}
        define_get_or_add_gpu_texture_dyn!{[ $([$internal_format $item_type])* ]}
    }
}

crate::with_all_texture_types! ([generate _]);



// ==================
// === AnyUniform ===
// ==================

#[derive(Clone, Debug)]
pub enum AnyUniform {
    Prim(AnyPrimUniform),
    Texture(AnyTextureUniform),
}


// === Conversions ===

impl<T> From<Uniform<T>> for AnyUniform
where Uniform<T>: IntoAnyUniform
{
    fn from(t: Uniform<T>) -> Self {
        t.into_any_uniform()
    }
}

pub trait IntoAnyUniform: Sized {
    fn into_any_uniform(self) -> AnyUniform;
}

impl<T: Into<AnyPrimUniform>> IntoAnyUniform for T {
    default fn into_any_uniform(self) -> AnyUniform {
        AnyUniform::Prim(self.into())
    }
}

impl<S, I, T> IntoAnyUniform for Uniform<Texture<S, I, T>>
where
    S: StorageRelation<I, T>,
    I: InternalFormat,
    T: ItemType,
    Uniform<Texture<S, I, T>>: Into<AnyTextureUniform>,
{
    fn into_any_uniform(self) -> AnyUniform {
        AnyUniform::Texture(self.into())
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


impl<'t, S: StorageRelation<I, T>, I: InternalFormat, T: ItemType> TryFrom<&'t AnyUniform>
    for &'t Uniform<Texture<S, I, T>>
where &'t Uniform<Texture<S, I, T>>: TryFrom<&'t AnyTextureUniform, Error = TypeMismatch>
{
    type Error = TypeMismatch;
    fn try_from(value: &'t AnyUniform) -> Result<Self, Self::Error> {
        match value {
            AnyUniform::Texture(t) => t.try_into(),
            _ => Err(TypeMismatch),
        }
    }
}
