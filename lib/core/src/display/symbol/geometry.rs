use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::dirty;
use crate::dirty::traits::*;
use crate::display::symbol::scope;
use crate::display::symbol::scope::Scope;
use crate::system::web::Logger;
use crate::system::web::group;
use crate::system::web::fmt;
use std::slice::SliceIndex;
use crate::closure;
use paste;
use num_enum::IntoPrimitive;


// ================
// === Geometry ===
// ================

// === Definition ===

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Geometry<OnDirty> {
    #[shrinkwrap(main_field)]
    pub scopes       : Scopes      <OnDirty>,
    pub scopes_dirty : ScopesDirty <OnDirty>,
    pub logger       : Logger,
}

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scopes<OnDirty> {
    pub point     : AttributeScope <OnDirty>,
    pub vertex    : AttributeScope <OnDirty>,
    pub primitive : AttributeScope <OnDirty>,
    pub instance  : AttributeScope <OnDirty>,
    pub object    : UniformScope   <OnDirty>,
    pub global    : GlobalScope    <OnDirty>,
}

//#[derive(Derivative)]
//#[derivative(Debug)]
//#[derivative(Default)]
//pub struct ScopesDirtyStatus {
//    pub point     : bool,
//    pub vertex    : bool,
//    pub primitive : bool,
//    pub instance  : bool,
//    pub object    : bool,
//    pub global    : bool,
//}

#[derive(Copy,Clone,Debug,IntoPrimitive)]
#[repr(u8)]
pub enum ScopesDirtyStatus {
    point,
    vertex,
    primitive,
    instance,
    object,
    global,
}

impl From<ScopesDirtyStatus> for usize {
    fn from(t: ScopesDirtyStatus) -> Self {
        Into::<u8>::into(t).into()
    }
}


// === Types ===

pub type AttributeIndex <T, Callback> = scope::AttributeIndex<T, Closure_scope_on_change<Callback>>;
pub type ScopesDirty    <Callback> = dirty::SharedEnum<u8,ScopesDirtyStatus, Callback>;
pub type AttributeScope <Callback> = Scope<Closure_scope_on_change<Callback>>;
pub type UniformScope   <Callback> = Scope<Closure_scope_on_change<Callback>>; // FIXME
pub type GlobalScope    <Callback> = Scope<Closure_scope_on_change<Callback>>; // FIXME
pub type AnyAttribute   <Callback> = scope::AnyAttribute<Closure_scope_on_change<Callback>>;
pub type Attribute      <T, Callback> = scope::Attribute<T, Closure_scope_on_change<Callback>>;

// === Callbacks ===


closure!(scope_on_change<Callback: Callback0>
    (dirty: ScopesDirty<Callback>, item: ScopesDirtyStatus)
        || { dirty.set_with((item,)) });

// === Implementation ===

impl<OnDirty: Callback0> Geometry<OnDirty> {
    pub fn new(logger: Logger, on_dirty: OnDirty) -> Self {
        let scopes_logger = logger.sub("scopes_dirty");
        let scopes_dirty  = ScopesDirty::new(scopes_logger,on_dirty);
        let scopes        = group!(logger, "Initializing.", {
            macro_rules! new_scope { ($cls:ident { $($name:ident),* } ) => {$(
                let sub_logger = logger.sub(stringify!($name));
                let status_mod = ScopesDirtyStatus::$name;
                let scs_dirty  = scopes_dirty.clone();
                let callback   = scope_on_change(scs_dirty, status_mod);
                let $name      = $cls::new(sub_logger, callback);
            )*}}

            new_scope!(AttributeScope { point, vertex, primitive, instance });
            new_scope!(AttributeScope { object });
            new_scope!(AttributeScope { global });

            Scopes { point, vertex, primitive, instance, object, global }
        });
        Self { scopes, scopes_dirty, logger }
    }

    pub fn update(&mut self) {
//        if self.check_dirty() {
//            group!(self.logger, "Updating.", {
//                if self.geometry_dirty.check_and_unset() {
//                    self.geometry.update()
//                }
//            })
//        }
    }

    pub fn check_dirty(&self) -> bool {
        self.scopes_dirty.check()
    }
}



