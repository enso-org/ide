//! This module contains implementation of various dirty flags. A dirty flag is
//! a structure which remembers that something was changed, but not updated yet.
//! For example, dirty flags are useful when defining OpenGL buffer management.
//! When a data in CPU-buffer changes, dirty flags can keep a set of changed
//! indexes and bulk-update the GPU-buffers every animation frame. You can think
//! of dirty flags like about a way to introduce laziness to the program
//! evaluation mechanisms.

use crate::prelude::*;

use crate::data::function::callback::*;
use crate::system::web::group;
use crate::system::web::Logger;
use rustc_hash::FxHashSet;
use std::hash::Hash;
use std::ops;


// ================
// === Wrappers ===
// ================
// TODO: Refactor all wrappers using Shapely generics

// === Logger Wrapper ===

/// Struct-wrapper adding a logger to the underlying structure. See the
/// documentation of `shapely` to learn more about struct-wrappers.
#[derive(Shrinkwrap,Debug)]
#[shrinkwrap(mutable)]
pub struct WithLogger<T>
    ( pub Logger
    , #[shrinkwrap(main_field)] pub T
    );

pub trait HasLogger {
    fn logger    (&    self) -> &    Logger;
    fn logger_mut(&mut self) -> &mut Logger;
}

impl<T> HasLogger for WithLogger<T> {
    fn logger    (&    self) -> &    Logger { &    self.0 }
    fn logger_mut(&mut self) -> &mut Logger { &mut self.0 }
}

// === OnSet Wrapper ===

/// Struct-wrapper adding a callback to the underlying structure. See the
/// documentation of `shapely` to learn more about struct-wrappers.
#[derive(Derivative,Shrinkwrap)]
#[derivative(Debug(bound = "T:Debug"))]
#[shrinkwrap(mutable)]
pub struct WithOnSet<OnSet, T>
    ( pub Callback<OnSet>
    , #[shrinkwrap(main_field)] pub T
    );

pub trait HasOnSet<OnSet> {
    fn on_set    (&    self) -> &    Callback<OnSet>;
    fn on_set_mut(&mut self) -> &mut Callback<OnSet>;
}

impl<OnSet,T> HasOnSet<OnSet> for WithOnSet<OnSet, T> {
    fn on_set    (&    self) -> &    Callback<OnSet> { &    self.0 }
    fn on_set_mut(&mut self) -> &mut Callback<OnSet> { &mut self.0 }
}

impl<OnSet,T:HasOnSet<OnSet>> HasOnSet<OnSet> for WithLogger<T> {
    fn on_set(&self) -> &Callback<OnSet> {
        self.deref().on_set()
    }
    fn on_set_mut(&mut self) -> &mut Callback<OnSet> {
        self.deref_mut().on_set_mut()
    }
}


// =====================
// === Smart Methods ===
// =====================

/// All of these smart methods implement the same method but in different
/// variants depending on the super-constraint. We cannot automatically import
/// it, so the users of this module need to manually import all `traits::*`.
pub mod traits {
    pub trait SmartSet0                   { fn set(&mut self); }
    pub trait SmartSet1<A1>               { fn set(&mut self, a1:A1); }
    pub trait SmartSet2<A1,A2>            { fn set(&mut self, a1:A1, a2:A2); }
    pub trait SmartSet3<A1,A2,A3>         { fn set(&mut self, a1:A1, a2:A2, a3:A3); }

    pub trait SharedSmartSet0             { fn set(&self); }
    pub trait SharedSmartSet1<A1>         { fn set(&self, a1:A1); }
    pub trait SharedSmartSet2<A1,A2>      { fn set(&self, a1:A1, a2:A2); }
    pub trait SharedSmartSet3<A1,A2,A3>   { fn set(&self, a1:A1, a2:A2, a3:A3); }

    pub trait SmartUnset0                 { fn unset(&mut self); }
    pub trait SmartUnset1<A1>             { fn unset(&mut self, a1:A1); }
    pub trait SmartUnset2<A1,A2>          { fn unset(&mut self, a1:A1, a2:A2); }
    pub trait SmartUnset3<A1,A2,A3>       { fn unset(&mut self, a1:A1, a2:A2, a3:A3); }

    pub trait SharedSmartUnset0           { fn unset(&self); }
    pub trait SharedSmartUnset1<A1>       { fn unset(&self, a1:A1); }
    pub trait SharedSmartUnset2<A1,A2>    { fn unset(&self, a1:A1, a2:A2); }
    pub trait SharedSmartUnset3<A1,A2,A3> { fn unset(&self, a1:A1, a2:A2, a3:A3); }

    pub trait SmartCheck0                 { fn check(&self) -> bool; }
    pub trait SmartCheck1<A1>             { fn check(&self, a1:A1) -> bool; }
    pub trait SmartCheck2<A1,A2>          { fn check(&self, a1:A1, a2:A2) -> bool; }
    pub trait SmartCheck3<A1,A2,A3>       { fn check(&self, a1:A1, a2:A2, a3:A3) -> bool; }

    pub trait SharedSmartCheck0           { fn check(&self) -> bool; }
    pub trait SharedSmartCheck1<A1>       { fn check(&self, a1:A1) -> bool; }
    pub trait SharedSmartCheck2<A1,A2>    { fn check(&self, a1:A1, a2:A2) -> bool; }
    pub trait SharedSmartCheck3<A1,A2,A3> { fn check(&self, a1:A1, a2:A2, a3:A3) -> bool; }
}

use traits::*;


// =====================
// === DirtyFlagData ===
// =====================

/// The abstraction for dirty flag underlying structure. You would rather not
/// need to ever access these methods directly. They are used to implement
/// high-level methods in `DirtyFlag` and `SharedDirtyFlag` wrappers.
pub trait DirtyFlagData: Display {
    type Args: Debug;
    fn check     (&    self, args: &Self::Args) -> bool;
    fn set       (&mut self, args:  Self::Args);
    fn check_all (&self) -> bool;
    fn unset_all (&mut self);
}

pub trait DirtyFlagData0: Display {
    type Args: Debug;
    fn check (&    self) -> bool;
    fn set   (&mut self);
}

pub trait DirtyFlagData1<T1>: Display {
    fn check (&    self, t1:T1) -> bool;
    fn set   (&mut self, t1:T1);
}

//pub trait DirtyFlagData2<T1,T2>: Display {
//    fn check (&    self, t1:T1, t2:T2) -> bool;
//    fn set   (&mut self, t1:T1, t2:T2);
//}

pub trait DirtyFlagDataUnset: DirtyFlagData {
    fn unset(&mut self, args:Self::Args);
}

type Args<T> = <T as DirtyFlagData>::Args;

pub trait HasElems<'t> {
    type AsRefs;
}

impl<'t> HasElems<'t> for () { 
    type AsRefs = (); 
}

impl<'t,T1:'t> HasElems<'t> for (T1,) { 
    type AsRefs = (&'t T1,); 
}

impl<'t,T1:'t,T2:'t> HasElems<'t> for (T1,T2) { 
    type AsRefs = (&'t T1, &'t T2); 
}

impl<'t,T1:'t,T2:'t,T3:'t> HasElems<'t> for (T1,T2,T3) { 
    type AsRefs = (&'t T1, &'t T2, &'t T3); 
}

type AsRefs<'t,T> = <T as HasElems<'t>>::AsRefs;

// =================
// === DirtyFlag ===
// =================

// === Definition ===

/// Abstraction for every dirty flag implementation. It is a smart struct adding
/// logging and callback utilities to the underlying data. Moreover, it
/// implements public API for working with dirty flags.
#[derive(Derivative)]
#[derivative(Debug(bound = "T:Debug"))]
//#[shrinkwrap(mutable)]
pub struct DirtyFlag<T,OnSet> {
    pub def: WithLogger<WithOnSet<OnSet,T>>
}

// === API ===

impl<OnSet,T>
DirtyFlag<T,OnSet> {
    pub fn data(&self) -> &T { &self.def }
}

impl<OnSet,T:Default>
DirtyFlag<T,OnSet> {
    pub fn new(logger: Logger, on_set:Callback<OnSet>) -> Self {
        let def = WithLogger(logger, WithOnSet(on_set, default()));
        DirtyFlag {def}
    }
}

impl<T:DirtyFlagData, OnSet:Callback0>
DirtyFlag<T,OnSet> {
    /// Sets the dirty flag by providing explicit parameters in a tuple. You
    /// should rather not need to use this function explicitly. You can use the
    /// smart setters instead. They are just `set` function which accepts as
    /// many parameters as really needed.
    pub fn set_args(&mut self, args: Args<T>) {
        let first_set = !self.check_all();
        let is_set_for = self.check_for(&args);
        if !is_set_for {
            self.def.set(args);
            group!(self.logger(), format!("Setting to {}.", self.data()), {
                if first_set { self.on_set_mut().call() }
            })
        }
    }

    pub fn check_args(&self, args:&Args<T>) -> bool {
        self.def.check(args)
    }
}

impl<T:DirtyFlagDataUnset,OnSet>
DirtyFlag<T,OnSet> where Args<T>:Debug {
    pub fn unset_args(&mut self, args: Args<T>) {
        self.logger().info(|| format!("Unsetting {:?}.", args));
        self.def.unset(args);
    }
}

impl<T:DirtyFlagData,OnSet>
DirtyFlag<T,OnSet> {
    /// Unset the dirty flag. This function resets the state of the flag, so it
    /// is exactly as it was after fresh flag creation.
    pub fn unset_all(&mut self) {
        if self.check_all() {
            group!(self.logger(), "Resetting.", { self.def.unset_all() })
        }
    }

    /// Check if the flag was dirty, unset it, and result the check status.
    pub fn check_and_unset_all(&mut self) -> bool {
        let is_set = self.check_all();
        self.unset_all();
        is_set
    }

    /// Check if the flag is dirty.
    pub fn check_all(&self) -> bool {
        self.def.check_all()
    }

    /// Check if the flag is dirty for a specific input argument.
    pub fn check_for(&self, args: &Args<T>) -> bool {
        self.def.check(args)
    }
}

// === Instances ===

// TODO: Remove after refactoring HasOnSet to Shapely
impl <OnSet,T> HasOnSet<OnSet> for DirtyFlag<T,OnSet> {
    fn on_set(&self) -> &Callback<OnSet> {
        self.def.on_set()
    }
    fn on_set_mut(&mut self) -> &mut Callback<OnSet> {
        self.def.deref_mut().on_set_mut()
    }
}

// TODO: Remove after refactoring HasOnSet to Shapely
impl<OnSet,T> HasLogger for DirtyFlag<T,OnSet> {
    fn logger    (&    self) -> &    Logger { self.def.logger    () }
    fn logger_mut(&mut self) -> &mut Logger { self.def.logger_mut() }
}

// === Smart Methods ===

impl<T,OnSet>
SmartSet0 for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=()>, OnSet:Callback0 {
    fn set(&mut self) { self.set_args(()) }
}

impl<T,OnSet,A1> SmartSet1<A1> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&mut self, a1:A1) { self.set_args((a1,)) }
}

impl<T,OnSet,A1,A2> SmartSet2<A1,A2> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&mut self, a1:A1, a2:A2) { self.set_args((a1, a2)) }
}

impl<T,OnSet,A1,A2,A3> SmartSet3<A1,A2,A3> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&mut self, a1:A1, a2:A2, a3:A3) { self.set_args((a1, a2, a3)) }
}

impl<T, OnSet> SmartUnset0 for DirtyFlag<T,OnSet>
where T:DirtyFlagDataUnset<Args=()>, OnSet:Callback0 {
    fn unset(&mut self) { self.unset_args(()) }
}

impl<T,OnSet,A1> SmartUnset1<A1> for DirtyFlag<T,OnSet>
where T:DirtyFlagDataUnset<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&mut self, a1:A1) { self.unset_args((a1,)) }
}

impl<T,OnSet,A1,A2> SmartUnset2<A1,A2> for DirtyFlag<T,OnSet>
where T:DirtyFlagDataUnset<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&mut self, a1:A1, a2:A2) { self.unset_args((a1, a2)) }
}

impl<T,OnSet,A1,A2,A3> SmartUnset3<A1,A2,A3> for DirtyFlag<T,OnSet>
where T:DirtyFlagDataUnset<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&mut self, a1:A1, a2:A2, a3:A3) { self.unset_args((a1, a2, a3)) }
}

impl<T, OnSet> SmartCheck0 for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=()>, OnSet:Callback0 {
    fn check(&self) -> bool { self.check_args(&()) }
}

impl<T,OnSet,A1> SmartCheck1<A1> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1) -> bool { self.check_args(&(a1,)) }
}

impl<T,OnSet,A1,A2> SmartCheck2<A1,A2> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1, a2:A2) -> bool { self.check_args(&(a1, a2)) }
}

impl<T,OnSet,A1,A2,A3> SmartCheck3<A1,A2,A3> for DirtyFlag<T,OnSet>
where T:DirtyFlagData<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1, a2:A2, a3:A3) -> bool { self.check_args(&(a1, a2, a3)) }
}


// =======================
// === SharedDirtyFlag ===
// =======================

// === Definition ===

/// A version of `DirtyFlag` which uses internal mutability pattern. It is meant
/// to expose the same API but without requiring `self` reference to be mutable.
#[derive(Derivative,Shrinkwrap)]
#[derivative(Debug(bound = "T:Debug"))]
#[derivative(Clone(bound = ""))]
pub struct SharedDirtyFlag<T,OnSet> {
    rc: Rc<RefCell<DirtyFlag<T,OnSet>>>
}

// === API ===

impl<T:Copy,OnSet>
SharedDirtyFlag<T,OnSet> {
    pub fn data(&self) -> T {
        *self.rc.borrow().data()
    }
}

impl<T,OnSet>
SharedDirtyFlag<T,OnSet> {
    pub fn set_callback(&self, on_set:OnSet) {
        *self.rc.borrow_mut().on_set_mut() = Callback(on_set);
    }
}

impl<T:Default,OnSet>
SharedDirtyFlag<T,OnSet> {
    pub fn new(logger: Logger, on_set: OnSet) -> Self {
        let callback = Callback(on_set);
        let rc       = Rc::new(RefCell::new(DirtyFlag::new(logger,callback)));
        Self { rc }
    }
}

impl<T:DirtyFlagData, OnSet:Callback0>
SharedDirtyFlag<T,OnSet> {
    pub fn set_with(&self, args: Args<T>) {
        self.rc.borrow_mut().set_args(args)
    }
}

impl<T:DirtyFlagData,OnSet>
SharedDirtyFlag<T,OnSet> {
    pub fn unset_all(&mut self) {
        self.rc.borrow_mut().unset_all()
    }

    pub fn check_and_unset_all(&mut self) -> bool {
        self.rc.borrow_mut().check_and_unset_all()
    }

    pub fn check_all(&self) -> bool {
        self.rc.borrow().check_all()
    }

    pub fn check_args(&self, args: &Args<T>) -> bool {
        self.rc.borrow().check_for(args)
    }
}

impl<T,OnSet>
From<Rc<RefCell<DirtyFlag<T,OnSet>>>> for SharedDirtyFlag<T,OnSet> {
    fn from(rc: Rc<RefCell<DirtyFlag<T,OnSet>>>) -> Self {
        Self {rc}
    }
}


// === Smart SmartSets ===

impl<T,OnSet> SharedSmartSet0 for SharedDirtyFlag<T,OnSet>
where T: DirtyFlagData<Args=()>, OnSet:Callback0 {
    fn set(&self) { self.rc.borrow_mut().set_args(()) }
}

impl<T,OnSet,A1> SharedSmartSet1<A1> for SharedDirtyFlag<T,OnSet>
where T: DirtyFlagData<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&self, a1:A1) { self.rc.borrow_mut().set_args((a1,)) }
}

impl<T,OnSet,A1,A2> SharedSmartSet2<A1,A2> for SharedDirtyFlag<T,OnSet>
where T: DirtyFlagData<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&self, a1:A1, a2:A2) { self.rc.borrow_mut().set_args((a1,a2)) }
}

impl<T,OnSet,A1,A2,A3> SharedSmartSet3<A1,A2,A3> for SharedDirtyFlag<T,OnSet>
where T: DirtyFlagData<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn set(&self, a1:A1, a2:A2, a3:A3) {
        self.rc.borrow_mut().set_args((a1,a2,a3))
    }
}

impl<T,OnSet> SharedSmartUnset0 for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=()>, OnSet:Callback0 {
    fn unset(&self) { self.rc.borrow_mut().unset_args(()) }
}

impl<T,OnSet,A1> SharedSmartUnset1<A1> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&self, a1:A1) { self.rc.borrow_mut().unset_args((a1,)) }
}

impl<T,OnSet,A1,A2> SharedSmartUnset2<A1,A2> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&self, a1:A1, a2:A2) { self.rc.borrow_mut().unset_args((a1,a2)) }
}

impl<T,OnSet,A1,A2,A3> SharedSmartUnset3<A1,A2,A3> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn unset(&self, a1:A1, a2:A2, a3:A3) {
        self.rc.borrow_mut().unset_args((a1,a2,a3))
    }
}

impl<T,OnSet> SharedSmartCheck0 for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=()>, OnSet:Callback0 {
    fn check(&self) -> bool { self.rc.borrow().check_args(&()) }
}

impl<T,OnSet,A1> SharedSmartCheck1<A1> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1,)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1) -> bool { self.rc.borrow().check_args(&(a1,)) }
}

impl<T,OnSet,A1,A2> SharedSmartCheck2<A1,A2> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1, A2)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1, a2:A2) -> bool { self.rc.borrow().check_args(&(a1,a2)) }
}

impl<T,OnSet,A1,A2,A3> SharedSmartCheck3<A1,A2,A3> for SharedDirtyFlag<T,OnSet>
    where T: DirtyFlagDataUnset<Args=(A1, A2, A3)>, OnSet:Callback0, Args<T>:Debug {
    fn check(&self, a1:A1, a2:A2, a3:A3) -> bool {
        self.rc.borrow().check_args(&(a1,a2,a3))
    }
}

// === Iterators ===

// FIXME: This is very error prone. Fix it after this gets resolved:
// https://github.com/rust-lang/rust/issues/66505

// [1] Please refer to `Prelude::drop_lifetime` docs to learn why it is safe to
// use it here.
impl<T, OnSet> SharedDirtyFlag<T,OnSet>
where for<'t> &'t T: IntoIterator {
    pub fn iter(&self) -> SharedDirtyFlagIter<T, OnSet> {
        let _borrow   = self.rc.borrow();
        let reference = unsafe { drop_lifetime(&_borrow) }; // [1]
        let iter      = reference.def.into_iter();
        SharedDirtyFlagIter { iter, _borrow }
    }
}

/// Iterator guard for SharedDirtyFlag. It exposes the iterator of original
/// structure behind the shared reference.
pub struct SharedDirtyFlagIter<'t,T,OnSet>
where &'t T: IntoIterator {
    pub iter : <&'t T as IntoIterator>::IntoIter,
    _borrow  : Ref<'t,DirtyFlag<T,OnSet>>
}

impl<'t,T,OnSet> Deref for SharedDirtyFlagIter<'t,T,OnSet>
where &'t T: IntoIterator {
    type Target = <&'t T as IntoIterator>::IntoIter;
    fn deref(&self) -> &Self::Target { &self.iter }
}

impl<'t,T,OnSet> DerefMut for SharedDirtyFlagIter<'t,T,OnSet>
where &'t T: IntoIterator {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.iter }
}

impl<'t,T,OnSet> Iterator for SharedDirtyFlagIter<'t,T,OnSet>
where &'t T: IntoIterator {
    type Item = <&'t T as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}


// =============================================================================
// === Flags ===================================================================
// =============================================================================

// ============
// === Bool ===
// ============

/// The on / off dirty flag. If you need a simple dirty / clean switch, this one
/// is the right choice.

pub type  Bool       <OnSet> = DirtyFlag       <BoolData,OnSet>;
pub type  SharedBool <OnSet> = SharedDirtyFlag <BoolData,OnSet>;
pub trait BoolCtx    <OnSet> = where OnSet: Callback0;

#[derive(Debug,Display,Default)]
pub struct BoolData { is_dirty: bool }
impl DirtyFlagData for BoolData {
    type Args = ();
    fn check_all (&self) -> bool        { self.is_dirty }
    fn check     (&self, _:&()) -> bool { self.is_dirty }
    fn set       (&mut self, _:())      { self.is_dirty = true  }
    fn unset_all (&mut self)            { self.is_dirty = false }
}

impl DirtyFlagDataUnset for BoolData {
    fn unset(&mut self, _:()) { self.is_dirty = false }
}


// =============
// === Range ===
// =============

/// Dirty flag which keeps information about a range of dirty items. It does not
/// track items separately, nor you are allowed to keep multiple ranges in it.
/// Just a single value range.

pub type  Range       <Ix,OnSet> = DirtyFlag       <RangeData<Ix>,OnSet>;
pub type  SharedRange <Ix,OnSet> = SharedDirtyFlag <RangeData<Ix>,OnSet>;
pub trait RangeCtx       <OnSet> = where OnSet: Callback0;
pub trait RangeIx                = PartialOrd + Copy + Debug;

#[derive(Debug,Default)]
pub struct RangeData<Ix=usize> { pub range: Option<ops::RangeInclusive<Ix>> }
impl<Ix:RangeIx> DirtyFlagData for RangeData<Ix> {
    type Args = (Ix,);
    fn unset_all(&mut self) {
        self.range = None
    }

    fn check_all(&self) -> bool {
        self.range.is_some()
    }

    fn check(&self, (ix,): &Self::Args) -> bool {
        self.range.as_ref().map(|r| r.contains(ix)) == Some(true)
    }

    fn set(&mut self, (ix,): Self::Args) {
        self.range = match &self.range {
            None    => Some(ix ..= ix),
            Some(r) => {
                if      ix < *r.start() { Some (ix ..= *r.end())   }
                else if ix > *r.end()   { Some (*r.start() ..= ix) }
                else                    { Some (r.clone())         }
            }
        };
    }
}

impl<Ix:RangeIx> Display for RangeData<Ix> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.range.as_ref().map(|t|
            format!("[{:?}...{:?}]",t.start(),t.end()))
            .unwrap_or_else(|| "false".into())
        )
    }
}

// ===========
// === Set ===
// ===========

/// Dirty flag which keeps a set of dirty values. The `HashSet` dirty flag
/// counterpart. Please note that it uses `FxHashSet` under the hood, so there
/// are no guarantees regarding attack-proof hashing algorithm here.

pub type  Set       <Ix,OnSet=()> = DirtyFlag       <SetData<Ix>,OnSet>;
pub type  SharedSet <Ix,OnSet=()> = SharedDirtyFlag <SetData<Ix>,OnSet>;
pub trait SetCtx       <OnSet>    = where OnSet: Callback0;
pub trait SetItem                 = Eq + Hash + Debug;

#[derive(Debug,Default,Shrinkwrap)]
pub struct SetData<Item:SetItem> { pub set: FxHashSet<Item> }
impl<Item:SetItem> DirtyFlagData for SetData<Item> {
    type Args = (Item,);
    fn check_all(&self) -> bool {
        !self.set.is_empty()
    }

    fn unset_all(&mut self) {
        self.set.clear();
    }

    fn check(&self, (a,): &Self::Args) -> bool {
        self.set.contains(a)
    }

    fn set (&mut self, (a,): Self::Args) {
        self.set.insert(a);
    }
}

impl<Item:SetItem> DirtyFlagDataUnset for SetData<Item> {
    fn unset (&mut self, (a,): Self::Args) {
        self.set.remove(&a);
    }
}

impl<Ix:SetItem> Display for SetData<Ix> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}",self.set)
    }
}

impl<'t,Item:SetItem> IntoIterator for &'t SetData<Item> {
    type Item = &'t Item;
    type IntoIter = <&'t FxHashSet<Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        (&self.set).iter()
    }
}


// ================
// === BitField ===
// ================

use bit_field::BitField as BF;

/// Dirty flag which keeps information about a set of enumerator values. The
/// items must be a plain enumerator implementing `Into<usize>`. The data is
/// stored as an efficient `BitField` under the hood.

pub type  Enum       <Prim,T,OnSet> = DirtyFlag       <EnumData<Prim,T>,OnSet>;
pub type  SharedEnum <Prim,T,OnSet> = SharedDirtyFlag <EnumData<Prim,T>,OnSet>;
pub trait EnumCtx           <OnSet> = where OnSet: Callback0;
pub trait EnumBase                  = Default + PartialEq + Copy + BF;

/// Dirty flag which keeps dirty indexes in a `BitField` under the hood.

pub type  BitField        <Prim,OnSet> = Enum       <Prim,usize,OnSet>;
pub type  SharedBitField  <Prim,OnSet> = SharedEnum <Prim,usize,OnSet>;


#[derive(Derivative)]
#[derivative(Debug(bound="Prim:Debug"))]
#[derivative(Default(bound="Prim:Default"))]
pub struct EnumData<Prim=u32,T=usize> {
    pub bits : Prim,
    phantom  : PhantomData<T>
}

impl<Prim:EnumBase,T:Copy+Debug+Into<usize>> DirtyFlagData for EnumData<Prim,T> {
    type Args = (T,);
    fn unset_all(&mut self) {
        self.bits = default()
    }

    fn check_all(&self) -> bool {
        self.bits != default()
    }

    fn check(&self, (t,): &Self::Args) -> bool {
        self.bits.get_bit((*t).into())
    }

    fn set(&mut self, (t,): Self::Args) {
        self.bits.set_bit(t.into(), true);
    }
}

impl<Prim:EnumBase,T:Copy+Debug+Into<usize>> Display for EnumData<Prim,T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.check_all())
    }
}