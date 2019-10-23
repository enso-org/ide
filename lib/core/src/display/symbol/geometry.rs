use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::dirty::SharedCustom;
use crate::display::symbol::scope;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use paste;



// =============
// === Dirty === 
// =============

#[derive(Derivative)]
#[derivative(Debug)]
#[derivative(Default)]
pub struct ScopesDirtyStatus {
    pub point     : bool,
    pub vertex    : bool,
    pub primitive : bool,
    pub instance  : bool,
    pub object    : bool,
    pub global    : bool,
}

pub type Dirty         <OnDirty> = SharedCustom<ScopesDirtyStatus, OnDirty>;
pub type OnScopeChange <OnDirty> = impl Fn() + Clone;

pub fn scope_on_change<OnDirty: Callback0>(
    dirty  : &Dirty<OnDirty>,
    action : fn(&mut ScopesDirtyStatus)
) -> OnScopeChange<OnDirty> {
    let dirty = dirty.clone();
    move || dirty.set(action)
}

// ===========
// === Geo ===
// ===========

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Geo<OnDirty = NoCallback> {
    #[shrinkwrap(main_field)]
    pub scopes : Scopes <OnDirty>,
    pub dirty  : Dirty  <OnDirty>,
    pub logger : Logger,
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scopes<OnDirty> {
    pub point     : AttributeScope     <OnDirty>,
    pub vertex    : AttributeScope     <OnDirty>,
    pub primitive : AttributeScope     <OnDirty>,
    pub instance  : AttributeScope     <OnDirty>,
    pub object    : UniformScope       <OnDirty>,
    pub global    : SharedUniformScope <OnDirty>,
}


// === Types ===
type AttributeScope     <OnDirty> = scope::Scope<OnScopeChange<OnDirty>>;
type UniformScope       <OnDirty> = scope::Scope<OnScopeChange<OnDirty>>; // FIXME
type SharedUniformScope <OnDirty> = scope::Scope<OnScopeChange<OnDirty>>; // FIXME

// === Implementation ===

impl<OnDirty: Callback0> Geo<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let dirty  = Dirty::new(on_dirty, logger.sub("dirty"));
        let scopes = group!(logger, "Initializing.", {
            macro_rules! new_scope { ($cls:ident { $($name:ident),* } ) => {$(
                let sub_logger = logger.sub(stringify!($name));
                let callback   = scope_on_change(&dirty, |x| {x.$name = true});
                let $name      = $cls::new(sub_logger, callback);
            )*}}

            new_scope!(AttributeScope { point, vertex, primitive, instance });
            new_scope!(AttributeScope { object });
            new_scope!(AttributeScope { global });

            Scopes { point, vertex, primitive, instance, object, global }
        });
        Self { scopes, dirty, logger }
    }
}

// impl<T> Geo<T> {
//     pub fn get(&self, ix: ScopeIndex) -> &AttributeScope<T> { self.child_by_ix(ix) }
//     pub fn get_mut(&mut self, ix: ScopeIndex) -> &mut AttributeScope<T> { self.child_by_ix_mut(ix) }

//     //    scope_getter!(point);
//     pub fn vertex    (&self) -> &AttributeScope<T> { self.get(self.vertex_ix)    }
//     pub fn primitive (&self) -> &AttributeScope<T> { self.get(self.primitive_ix) }
//     pub fn instance  (&self) -> &AttributeScope<T> { self.get(self.instance_ix)  }
//     pub fn object    (&self) -> &AttributeScope<T> { self.get(self.object_ix)    }

//     pub fn primitive_mut (&mut self) -> &mut AttributeScope<T> {
//         let ix = self.primitive_ix;
//         self.child_by_ix_mut(ix)
//     }

// }


// // // ===========
// // // === Geo ===
// // // ===========

// // // === Types ===

// // // type ScopeIndex       = nested::Index;
// // // type Scope  <OnDirty> = scope::Scope<OnChildChange<OnDirty>>;
// // // type Scopes <OnDirty> = Nested<Scope<OnDirty>, OnDirty>;

// // // === Definition ===

// // macro_rules! struct_geo {
// // ($($vtx_scope:ident),*) => {paste::item! {


// // #[derive(Shrinkwrap)]
// // #[shrinkwrap(mutable)]
// // #[derive(Derivative)]
// // #[derivative(Debug(bound=""))]
// // pub struct Geo2<OnDirty = NoCallback> {
// //     #[shrinkwrap(main_field)]
// //     pub scopes: Scopes<OnDirty>,
// //     $(pub [<$vtx_scope _scope_ix>]: ScopeIndex,)*
// // }

// // // === Implementation ===

// // impl<OnDirty: Callback0> Geo2<OnDirty> {
// //     pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
// //         let mut scopes                   = Nested::new(logger, on_dirty);
// //         $(  let name                     = stringify!($vtx_scope);
// //             let [<$vtx_scope _scope_ix>] = Self::add_scope(&mut scopes, name);
// //         )*
// //         Self {scopes, $([<$vtx_scope _scope_ix>],)*}
// //     }

// //     fn add_scope(scopes: &mut Scopes<OnDirty>, name: &str) -> nested::Index {
// //         let logger = scopes.logger.sub(name);
// //         scopes.add(|f| scope::Scope::new(logger, f))
// //     }
// // }

// // impl<OnDirty> Geo2<OnDirty> {$(
// //     /// Immutable scopes accessors
// //     pub fn $vtx_scope(&self) -> &Scope<OnDirty> {
// //         self.child_by_ix(self.[<$vtx_scope _scope_ix>])
// //     }

// //     /// Mutable scopes accessor
// //     pub fn [<$vtx_scope _mut>](&mut self) -> &mut Scope<OnDirty> {
// //         let ix = self.[<$vtx_scope _scope_ix>];
// //         self.child_by_ix_mut(ix)
// //     }
// // )*}


// // }}}
// // struct_geo!(point, vertex, primitive, instance, object);


