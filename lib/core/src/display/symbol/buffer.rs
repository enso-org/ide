pub mod item;
pub mod data;
pub mod var;

use crate::prelude::*;

use crate::closure;
use crate::data::function::callback::*;
use crate::dirty;
use crate::dirty::traits::*;
use crate::system::web::Logger;
use crate::system::web::fmt;
use crate::system::web::group;
use crate::tp::debug::TypeDebugName;
use item::Item;
use nalgebra;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use std::iter::Extend;


// ==============
// === Buffer ===
// ==============

// === Definition ===

/// Please refer to the 'Buffer management pipeline' doc to learn more about
/// attributes, scopes, geometries, meshes, scenes, and other relevant concepts.
///
/// Buffers are values stored in geometry. Under the hood they are stored in
/// vectors and are synchronised with GPU buffers on demand.
#[derive(Derivative,Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Buffer<T:Item,OnSet,OnResize> {
    #[shrinkwrap(main_field)]
    pub buffer       : SharedData  <T,OnSet,OnResize>,
    pub set_dirty    : SetDirty    <OnSet>,
    pub resize_dirty : ResizeDirty <OnResize>,
    pub logger       : Logger
}

// === Types ===

pub type Var        <T,S,R> = var ::Var        <T,DataOnSet<S>,DataOnResize<R>>;
pub type Data       <T,S,R> = data::Data       <T,DataOnSet<S>,DataOnResize<R>>;
pub type SharedData <T,S,R> = data::SharedData <T,DataOnSet<S>,DataOnResize<R>>;

#[macro_export]
macro_rules! promote_buffer_types { ($callbacks:tt $module:ident) => {
    promote! { $callbacks $module [Var<T>,Buffer<T>,AnyBuffer] }
};}

// === Callbacks ===

pub type SetDirty    <Callback> = dirty::SharedRange<usize,Callback>;
pub type ResizeDirty <Callback> = dirty::SharedBool<Callback>;

closure! {
fn buffer_on_resize<C:Callback0> (dirty:ResizeDirty<C>) ->
    DataOnResize { || dirty.set() }
}

closure! {
fn buffer_on_set<C:Callback0> (dirty:SetDirty<C>) ->
    DataOnSet { |ix: usize| dirty.set(ix) }
}

// === Instances ===

impl<T:Item, OnSet:Callback0, OnResize:Callback0>
Buffer<T,OnSet,OnResize> {

    /// Creates new buffer by providing explicit buffer object.
    pub fn new_from
    (vec:Vec<T>, logger:Logger, on_set:OnSet, on_resize:OnResize) -> Self {
        logger.info(fmt!("Creating new {} buffer.", T::type_debug_name()));
        let set_logger     = logger.sub("set_dirty");
        let resize_logger  = logger.sub("resize_dirty");
        let set_dirty      = SetDirty::new(set_logger,on_set);
        let resize_dirty   = ResizeDirty::new(resize_logger,on_resize);
        let buff_on_resize = buffer_on_resize(resize_dirty.clone_rc());
        let buff_on_set    = buffer_on_set(set_dirty.clone_rc());
        let buffer         = Data::new_from(vec, buff_on_set, buff_on_resize);
        let buffer         = Rc::new(RefCell::new(buffer));
        Self {buffer,set_dirty,resize_dirty,logger}
    }

    /// Creates a new empty buffer.
    pub fn new(logger:Logger, on_set:OnSet, on_resize:OnResize) -> Self {
        Self::new_from(default(),logger,on_set,on_resize)
    }

    /// Build the buffer based on the provider configuration builder.
    pub fn build(bldr:Builder<T>, on_set:OnSet, on_resize:OnResize) -> Self {
        let buffer = bldr._buffer.unwrap_or_else(default);
        let logger = bldr._logger.unwrap_or_else(default);
        Self::new_from(buffer,logger,on_set,on_resize)
    }
}

impl<T:Item,OnSet,OnResize>
Buffer<T,OnSet,OnResize> {
    /// Returns a new buffer `Builder` object.
    pub fn builder() -> Builder<T> {
        default()
    }

    /// Returns the number of elements in the buffer buffer.
    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }

    /// Get the variable by given index.
    pub fn get(&self, index:usize) -> Var<T,OnSet,OnResize> {
        Var::new(index,self.buffer.clone_rc())
    }

    /// Check dirty flags and update the state accordingly.
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            self.set_dirty.unset();
            self.resize_dirty.unset();
            // TODO finish
        })
    }
}

pub trait AddElementCtx<T,OnResize> = where
    T: Item + Clone,
    OnResize: Callback0;

impl<T,OnSet,OnResize>
Buffer<T,OnSet,OnResize> where Self: AddElementCtx<T,OnResize> {
    /// Adds a single new element initialized to default value.
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    /// Adds multiple new elements initialized to default values.
    pub fn add_elements(&mut self, elem_count: usize) {
        self.borrow_mut().extend(iter::repeat(T::empty()).take(elem_count));
    }
}

// ===============
// === Builder ===
// ===============

/// Buffer builder.
#[derive(Derivative)]
#[derivative(Default(bound=""))]
pub struct Builder<T: Item> {
    pub _buffer : Option <Vec<T>>,
    pub _logger : Option <Logger>
}

impl<T: Item> Builder<T> {
    pub fn new() -> Self {
        default()
    }

    pub fn buffer(self, val: Vec <T>) -> Self {
        Self { _buffer: Some(val), _logger: self._logger }
    }

    pub fn logger(self, val: Logger) -> Self {
        Self { _buffer: self._buffer, _logger: Some(val) }
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
    /// `Buffer<dyn Item, OnSet, OnResize>`.
    #[enum_dispatch(IsBuffer)]
    #[derive(Derivative)]
    #[derivative(Debug(bound=""))]
    pub enum AnyBuffer<OnSet, OnResize> {
        $(  [<Variant $base For $param>]
                (Buffer<$base<$param>, OnSet, OnResize>),
        )*
    }

    $( // ======================================================================

    impl<'t, T, S>
    TryFrom<&'t AnyBuffer<T, S>>
    for &'t Buffer<$base<$param>, T, S> {
        type Error = BadVariant;
        fn try_from(v: &'t AnyBuffer<T, S>)
        -> Result <&'t Buffer<$base<$param>, T, S>, Self::Error> {
            match v {
                AnyBuffer::[<Variant $base For $param>](a) => Ok(a),
                _ => Err(BadVariant)
            }
        }
    }

    impl<'t, T, S>
    TryFrom<&'t mut AnyBuffer<T, S>>
    for &'t mut Buffer<$base<$param>, T, S> {
        type Error = BadVariant;
        fn try_from(v: &'t mut AnyBuffer<T, S>)
        -> Result <&'t mut Buffer<$base<$param>, T, S>, Self::Error> {
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
mk_any_buffer!([Identity, Vector2, Vector3, Vector4], [f32, i32]);

/// Collection of all methods common to every buffer variant.
#[enum_dispatch]
pub trait IsBuffer<OnSet: Callback0, OnResize: Callback0> {
    fn add_element(&mut self);
    fn len(&self) -> usize;
    fn update(&mut self);
}
