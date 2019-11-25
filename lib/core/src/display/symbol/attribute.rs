pub mod item;

use crate::prelude::*;

use crate::data::function::callback::*;
use crate::dirty;
use crate::dirty::traits::*;
use crate::system::web::Logger;
use crate::system::web::fmt;
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;
use crate::tp::debug::TypeDebugName;
use std::iter::Extend;
use crate::dirty::traits::*;
use crate::data::seq::observable::Observable;

use nalgebra;
use nalgebra::dimension::{U1, U2, U3};
use nalgebra::dimension::DimName;
use nalgebra::Scalar;
use nalgebra::Vector2;
use nalgebra::Vector3;
use nalgebra::Vector4;
use nalgebra::Matrix;
use nalgebra::MatrixMN;

use crate::closure;
use crate::system::web::group;
use item::Item;


// =================
// === Callbacks ===
// =================

pub type SetDirty    <Callback> = dirty::SharedRange<usize,Callback>;
pub type ResizeDirty <Callback> = dirty::SharedBool<Callback>;

closure! {
fn buffer_on_resize<C:Callback0> (dirty:ResizeDirty<C>) ->
    BufferOnResize { || dirty.set() }
}

closure! {
fn buffer_on_set<C:Callback0> (dirty:SetDirty<C>) ->
    BufferOnSet { |ix: usize| dirty.set(ix) }
}


// ==============
// === Buffer ===
// ==============

/// Vector with attached callbacks listening for changes.
pub type Buffer<T,OnSet,OnResize> =
    Observable<Vec<T>, BufferOnSet<OnSet>, BufferOnResize<OnResize>>;

/// The `Buffer` behind a shared reference with internal mutability.
pub type SharedBuffer<T,OnSet,OnResize> =
    Rc<RefCell<Buffer<T,OnSet,OnResize>>>;


// ============
// === View ===
// ============

/// View for a particular attribute. Allows reading and writing attribute data
/// via the internal mutability pattern. It is implemented as a view on
/// a selected `SharedBuffer` element under the hood.
pub struct View<T,OnSet,OnResize> {
    index  : usize,
    buffer : SharedBuffer <T,OnSet,OnResize>
}

impl<T,OnSet:'static,OnResize> View<T,OnSet,OnResize> {

    // [1] Please refer to `Prelude::drop_lifetime` docs to learn why it is safe
    // to use it here.
    pub fn get(&self) -> IndexGuard<Buffer<T,OnSet,OnResize>> {
        let _borrow = self.buffer.borrow();
        let target  = _borrow.index(self.index);
        let target  = unsafe { drop_lifetime(target) }; // [1]
        IndexGuard { target, _borrow }
    }

    // [1] Please refer to `Prelude::drop_lifetime` docs to learn why it is safe
    // to use it here.
    pub fn get_mut(&self) -> IndexGuardMut<Buffer<T,OnSet,OnResize>> {
        let mut _borrow = self.buffer.borrow_mut();
        let target      = _borrow.index_mut(self.index);
        let target      = unsafe { drop_lifetime_mut(target) }; // [1]
        IndexGuardMut { target, _borrow }
    }

    pub fn modify<F: FnOnce(&mut T)>(&self, f:F) {
        f(&mut self.buffer.borrow_mut()[self.index]);
    }
}

#[derive(Shrinkwrap)]
pub struct IndexGuard<'t,T> where
    T:Index<usize> {
    #[shrinkwrap(main_field)]
    pub target : &'t <T as Index<usize>>::Output,
    _borrow    : Ref<'t,T>
}

#[derive(Shrinkwrap)]
pub struct IndexGuardMut<'t,T> where
    T:Index<usize> {
    #[shrinkwrap(main_field)]
    pub target : &'t mut <T as Index<usize>>::Output,
    _borrow    : RefMut<'t,T>
}


// =================
// === Attribute ===
// =================

// === Definition ===

/// Please refer to the 'Buffer management pipeline' doc to learn more about
/// attributes, scopes, geometries, meshes, scenes, and other relevant concepts.
///
/// Attributes are values stored in geometry. Under the hood they are stored in
/// vectors and are synchronised with GPU buffers on demand.
#[derive(Derivative,Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Attribute<T:Item,OnSet,OnResize> {
    #[shrinkwrap(main_field)]
    pub buffer       : SharedBuffer <T, OnSet, OnResize>,
    pub set_dirty    : SetDirty     <OnSet>,
    pub resize_dirty : ResizeDirty  <OnResize>,
    pub logger       : Logger
}

// === Types ===

pub trait SetDirtyCtx    <Callback> = dirty::RangeCtx <Callback>;
pub trait ResizeDirtyCtx <Callback> = dirty::BoolCtx  <Callback>;

// === Instances ===

impl<T:Item, OnSet:Callback0, OnResize:Callback0>
Attribute<T,OnSet,OnResize> {

    /// Creates new attribute by providing explicit buffer object.
    pub fn new_from
    (vec:Vec<T>, logger:Logger, on_set:OnSet, on_resize:OnResize) -> Self {
        logger.info(fmt!("Creating new {} attribute.", T::type_debug_name()));
        let set_logger     = logger.sub("set_dirty");
        let resize_logger  = logger.sub("resize_dirty");
        let set_dirty      = SetDirty::new(set_logger,on_set);
        let resize_dirty   = ResizeDirty::new(resize_logger,on_resize);
        let buff_on_resize = buffer_on_resize(resize_dirty.clone_rc());
        let buff_on_set    = buffer_on_set(set_dirty.clone_rc());
        let buffer         = Buffer::new_from(vec,buff_on_set,buff_on_resize);
        let buffer         = Rc::new(RefCell::new(buffer));
        Self {buffer,set_dirty,resize_dirty,logger}
    }

    /// Creates a new empty attribute.
    pub fn new(logger:Logger, on_set:OnSet, on_resize:OnResize) -> Self {
        Self::new_from(default(),logger,on_set,on_resize)
    }

    /// Build the attribute based on the provider configuration builder.
    pub fn build(bldr:Builder<T>, on_set:OnSet, on_resize:OnResize) -> Self {
        let buffer = bldr._buffer.unwrap_or_else(default);
        let logger = bldr._logger.unwrap_or_else(default);
        Self::new_from(buffer,logger,on_set,on_resize)
    }
}

impl<T:Item,OnSet,OnResize>
Attribute<T,OnSet,OnResize> {
    /// Returns a new attribute `Builder` object.
    pub fn builder() -> Builder<T> {
        default()
    }

    /// Returns the number of elements in the attribute buffer.
    pub fn len(&self) -> usize {
        self.buffer.borrow().len()
    }

    pub fn view(&self, index:usize) -> View<T,OnSet,OnResize> {
        let buffer = self.buffer.clone_rc();
        View {index,buffer}
    }
}

impl<T: Item, OnSet: Callback0, OnResize: Callback0>
Attribute<T, OnSet, OnResize> {
    pub fn update(&mut self) {
        group!(self.logger, "Updating.", {
            self.set_dirty.unset();
            self.resize_dirty.unset();
            // TODO finish
        })
    }
}


pub trait AddElementCtx = Item + Clone;
impl<T: AddElementCtx, OnSet, OnResize: Callback0> 
Attribute<T, OnSet, OnResize> {
    pub fn add_element(&mut self) {
        self.add_elements(1);
    }

    pub fn add_elements(&mut self, elem_count: usize) {
        self.borrow_mut().extend(iter::repeat(T::empty()).take(elem_count));
    }
}

// =================
// === Promotion ===
// =================

#[macro_export]
macro_rules! promote_attribute_types { ($callbacks:tt $module:ident) => {
    promote! { $callbacks $module [View<T>,Attribute<T>,AnyAttribute] }
};}

// ====================
// === AnyAttribute ===
// ====================

use enum_dispatch::*;

#[derive(Debug)]
pub struct BadVariant;


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

macro_rules! mk_any_shape_impl {
    ([$(($base:ident, $param:ident)),*,]) => { paste::item! {
        #[enum_dispatch(IsAttribute)]
        #[derive(Derivative)]
        #[derivative(Debug(bound=""))]
        pub enum AnyAttribute<OnSet, OnResize> {
            $(  [<Variant $base For $param>]
                    (Attribute<$base<$param>, OnSet, OnResize>),
            )*
        } 

        $( /////////////////////////////////////////////////////////////////////

        impl<'t, T, S> 
        TryFrom<&'t AnyAttribute<T, S>> 
        for &'t Attribute<$base<$param>, T, S> {
            type Error = BadVariant;
            fn try_from(v: &'t AnyAttribute<T, S>) 
            -> Result <&'t Attribute<$base<$param>, T, S>, Self::Error> { 
                match v {
                    AnyAttribute::[<Variant $base For $param>](a) => Ok(a),
                    _ => Err(BadVariant)
                }
            }
        }
        
        impl<'t, T, S> 
        TryFrom<&'t mut AnyAttribute<T, S>> 
        for &'t mut Attribute<$base<$param>, T, S> {
            type Error = BadVariant;
            fn try_from(v: &'t mut AnyAttribute<T, S>) 
            -> Result <&'t mut Attribute<$base<$param>, T, S>, Self::Error> { 
                match v {
                    AnyAttribute::[<Variant $base For $param>](a) => Ok(a),
                    _ => Err(BadVariant)
                }
            }
        }

        )* /////////////////////////////////////////////////////////////////////
    }
}}

macro_rules! mk_any_shape {
    ($bases:tt, $params:tt) => {
        cartesian!($bases, $params, mk_any_shape_impl);
    }
}

type Identity<T> = T;
mk_any_shape!([Identity, Vector2, Vector3, Vector4], [f32, i32]);


#[enum_dispatch]
pub trait IsAttribute<OnSet: Callback0, OnResize: Callback0> {
    fn add_element(&mut self);
    fn len(&self) -> usize;
    fn update(&mut self);
}








// // mk_any_shape!([(Vector2,f32),(Vector3,f32),]);

// pub trait IsAttribute<OnDirty> {
//     fn add_element(&self);
//     fn len(&self) -> usize;
// }

// pub struct AnyAttribute<OnDirty> (pub Box<dyn IsAttribute<OnDirty>>);

// pub trait IsAttributeCtx = AddElementCtx;
// impl<T: IsAttributeCtx, OnDirty> IsAttribute<OnDirty> for SharedAttribute<T, OnDirty> {
//     fn add_element(&self) {
//         self.add_element()
//     }
//     fn len(&self) -> usize {
//         self.len()
//     }
// }

// impl<T> std::fmt::Debug for AnyAttribute<T> {
//     fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
//         write!(fmt, "AnyAttribute")
//     }
// }

// impl<OnDirty> AnyAttribute<OnDirty> {
//     pub fn add_element(&self) {
//         self.0.add_element()
//     }
//     pub fn len(&self) -> usize {
//         self.0.len()
//     }
// }

// ===============
// === Builder ===
// ===============

#[derive(Derivative)]
#[derivative(Default(bound = ""))]
pub struct Builder<T: Item> {
    pub _buffer : Option <Vec <T>>,
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