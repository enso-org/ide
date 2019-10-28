use crate::prelude::*;

use crate::system::web::fmt;
use crate::system::web::group;
use crate::system::web::Logger;
use std::ops;
use rustc_hash::FxHashSet;
use std::hash::Hash;

use crate::data::function::callback::*;

trait Builder {
    type Config: Default;

    fn new(cfg: Self::Config) -> Self;
}

trait DirtyCheck {
    fn is_set(&self) -> bool;
}

// ============================
// === TO BE REFACTORED OUT ===
// ============================

pub trait When {
    fn when<F: FnOnce() -> T, T>(&self, f: F);
}

impl When for bool {
    fn when<F: FnOnce() -> T, T>(&self, f: F) {
        if *self {
            f();
        }
    }
}

// =============================================================================
// === Bool ====================================================================
// =============================================================================

// ==============
// === Bool ===
// ==============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct Bool<OnSet = NoCallback> {
    pub is_dirty : bool,
    pub on_set   : Callback<OnSet>,
    pub logger   : Logger,
}

pub trait BoolCtx<OnSet> = where OnSet: Callback0;

// === API ===

impl<OnSet> Bool<OnSet> {
    pub fn new(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let is_dirty = false;
        Self { is_dirty, on_set, logger }
    }
}

impl<OnSet> Bool<OnSet> where (): BoolCtx<OnSet> {
    pub fn set(&mut self) {
        if !self.is_dirty {
            group!(self.logger, "Setting.", {
                self.on_set.call();
                self.is_dirty = true;
            })
        }
    }
}

impl<OnSet> DirtyCheck for Bool<OnSet> {
    fn is_set(&self) -> bool {
        self.is_dirty
    }
}

// ====================
// === SharedBool ===
// ====================

// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound = ""))]
pub struct SharedBool<OnSet = NoCallback> {
    pub data: Rc<RefCell<Bool<OnSet>>>,
}

// === API ===

impl<OnSet> SharedBool<OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let base = Bool::new(on_set, logger);
        let data = Rc::new(RefCell::new(base));
        Self { data }
    }
}

impl<OnSet: Callback0> SharedBool<OnSet> {
    pub fn set(&self) {
        self.data.borrow_mut().set();
    }
}

// =============================================================================
// === Range ===================================================================
// =============================================================================

// =============
// === Range ===
// =============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound = "Ix:Debug"))]
pub struct Range<Ix = usize, OnSet = NoCallback> {
    pub range:  Option<ops::Range<Ix>>,
    pub on_set: Callback<OnSet>,
    pub logger: Logger,
}

pub trait RangeCtx<OnSet> = where OnSet: Callback0;
pub trait RangeIx         = PartialOrd + Copy + Debug;

// === API ===

impl<Ix, OnSet> Range<Ix, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let range = None;
        Self { range, on_set, logger }
    }
}

impl<Ix: RangeIx, OnSet> Range<Ix, OnSet> where Self: RangeCtx<OnSet> {
    /// This is an semantically unsafe function as it breaks the contract of a
    /// dirty flag. Dirty flags should not either be set to track new items, or
    /// should be cleared. Arbitrary reshape could be a potential hard-to-track
    /// error.
    fn unsafe_replace_range(&mut self, range: ops::Range<Ix>) {
        self.range = Some(range);
    }

    pub fn set(&mut self, ix: Ix) {
        let range = match &self.range {
            None    => { ix .. ix },
            Some(r) => {
                if      ix < r.start { ix .. r.end   }
                else if ix > r.end   { r.start .. ix }
                else                 { r.clone()     }
            }
        };
        group!(self.logger, format!("Setting dirty range to [{:?}].", range), {
            if !self.is_set() { self.on_set.call(); }
            self.unsafe_replace_range(range);
        })
    }

    pub fn set_range(&mut self, start: Ix, end: Ix) {
        self.set(start);
        self.set(end);
    }
}

impl<Ix, OnSet> DirtyCheck for Range<Ix, OnSet> {
    fn is_set(&self) -> bool {
        self.range.is_some()
    }
}

// ===================
// === SharedRange ===
// ===================

// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound = "Ix:Debug"))]
pub struct SharedRange<Ix = usize, OnSet = NoCallback> {
    pub data: Rc<RefCell<Range<Ix, OnSet>>>,
}

// === API ===

impl<Ix, OnSet> SharedRange<Ix, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let base = Range::new_raw(on_set, logger);
        let data = Rc::new(RefCell::new(base));
        Self { data }
    }
}

impl<Ix: RangeIx, OnSet> SharedRange<Ix, OnSet> where Self: RangeCtx<OnSet> {
    pub fn set(&self, ix: Ix) {
        self.data.borrow_mut().set(ix);
    }
}

// =============================================================================
// === Set =====================================================================
// =============================================================================

// ===========
// === Set ===
// ===========

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound = "Item:Debug"))]
pub struct Set<Item: Eq + Hash + Debug, OnSet = NoCallback> {
    pub set    : FxHashSet<Item>,
    pub on_set : Callback<OnSet>,
    pub logger : Logger,
}

pub trait SetCtx<Item, OnSet> = where 
    Item  : Eq + Hash + Debug,
    OnSet : Callback0;

// === API ===

impl<Item, OnSet> Set<Item, OnSet> where (): SetCtx<Item, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let set = default();
        Self { set, on_set, logger }
    }
}

impl<Item, OnSet> Set<Item, OnSet> where (): SetCtx<Item, OnSet> {
    pub fn set(&mut self, item: Item) {
        if !self.set.contains(&item) {
            group!(self.logger, format!("Setting item {:?}.", item), {
                if !self.is_set() {
                    self.on_set.call();
                }
                self.set.insert(item);
            })
        }
    }

    pub fn unset(&mut self) {
        self.logger.info("Unsetting.");
        self.set.clear()
    }
}

impl<Item, OnSet> DirtyCheck for Set<Item, OnSet> 
where (): SetCtx<Item, OnSet> {
    fn is_set(&self) -> bool {
        !self.set.is_empty()
    }
}

// =================
// === SharedSet ===
// =================

// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound = "Item:Debug"))]
pub struct SharedSet<Item: Eq + Hash + Debug, OnSet = NoCallback> {
    pub data: Rc<RefCell<Set<Item, OnSet>>>,
}

// === API ===

impl<Item, OnSet> SharedSet<Item, OnSet> where (): SetCtx<Item, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let base = Set::new_raw(on_set, logger);
        let data = Rc::new(RefCell::new(base));
        Self { data }
    }
}

impl<Item, OnSet> SharedSet<Item, OnSet> where (): SetCtx<Item, OnSet> {
    pub fn set(&self, item: Item) {
        self.data.borrow_mut().set(item);
    }

    pub fn is_set(&self) -> bool {
        self.data.borrow().is_set()
    }

    pub fn unset(&self) {
        self.data.borrow_mut().unset()
    }
}

// ====================
// === RangeBuilder ===
// ====================

// TODO: This section could be auto-generated with macros. It is intentionally
// written without syntax sugar to mimic a possible shape of the codegen.

pub struct RangeBuilder<_Builder_, OnSet = NoCallback> {
    _builder_:   _Builder_,
    pub _on_set: Option<Callback<OnSet>>,
    pub _logger: Option<Logger>,
}

impl<_Builder_, OnSet> RangeBuilder<_Builder_, OnSet> {
    pub fn new(builder: _Builder_) -> Self {
        Self { _builder_: builder, _on_set: None, _logger: None }
    }

    pub fn on_set(self, val: OnSet) -> RangeBuilder<_Builder_, OnSet> {
        RangeBuilder {
            _builder_: self._builder_,
            _on_set:   Some(Callback(val)),
            _logger:   self._logger,
        }
    }

    pub fn logger(self, val: Logger) -> RangeBuilder<_Builder_, OnSet> {
        RangeBuilder { _builder_: self._builder_, _on_set: self._on_set, _logger: Some(val) }
    }
}

impl<OnSet, _Builder_: Fn(Callback<OnSet>, Logger) -> T, T> RangeBuilder<_Builder_, OnSet>
where Callback<OnSet>: Default
{
    pub fn build(self) -> T {
        let on_set = self._on_set.unwrap_or_else(|| Default::default());
        let logger = self._logger.unwrap_or_else(|| Default::default());
        (self._builder_)(on_set, logger)
    }
}

// === Instances ===

impl<Ix: RangeIx, OnSet> Range<Ix, OnSet> {
    pub fn builder() -> RangeBuilder<fn(Callback<OnSet>, Logger) -> Range<Ix, OnSet>, OnSet> {
        RangeBuilder::new(Self::new_raw)
    }
}

impl<Ix: RangeIx, OnSet> SharedRange<Ix, OnSet> {
    pub fn builder() -> RangeBuilder<fn(Callback<OnSet>, Logger) -> SharedRange<Ix, OnSet>, OnSet> {
        RangeBuilder::new(Self::new_raw)
    }
}

// =================================================================================================
// === BitField ====================================================================================
// =================================================================================================

use bit_field::BitField as BF;
use std::ops::RangeBounds;

// ================
// === BitField ===
// ================

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct BitField<T = u32, OnSet = NoCallback> {
    pub bits:   T,
    pub on_set: Callback<OnSet>,
    pub logger: Logger,
}

pub trait BitFieldBase = Default + PartialEq + Copy + From<u32> + BF;
//
// === API ===

impl<T: BitFieldBase, OnSet> BitField<T, OnSet> {
    pub fn new(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let bits = Default::default();
        Self { bits, on_set, logger }
    }
}

impl<T: BitFieldBase, OnSet: Callback0> BitField<T, OnSet> {
    pub fn set(&mut self, ix: usize) {
        if self.bits == Default::default() {
            self.on_set.call();
        }
        self.bits.set_bit(ix, true);
    }

    pub fn set_range<R: RangeBounds<usize>>(&mut self, range: R) {
        self.bits.set_bits(range, From::from(1:u32));
    }
}

impl<T: BitFieldBase, OnSet> DirtyCheck for BitField<T, OnSet> {
    fn is_set(&self) -> bool {
        self.bits == Default::default()
    }
}

// ======================
// === SharedBitField ===
// ======================

// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound="T:Debug"))]
pub struct SharedBitField<T = u32, OnSet = NoCallback> {
    pub data: Rc<RefCell<BitField<T, OnSet>>>,
}

// === API ===

impl<T: BitFieldBase, OnSet> SharedBitField<T, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let base = BitField::new(on_set, logger);
        let data = Rc::new(RefCell::new(base));
        Self { data }
    }
}

impl<T: BitFieldBase, OnSet: Callback0> SharedBitField<T, OnSet> {
    pub fn set(&self, ix: usize) {
        self.data.borrow_mut().set(ix);
    }
}

// =============================================================================
// === Custom ==================================================================
// =============================================================================

// ==============
// === Custom ===
// ==============

// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound="T:Debug"))]
pub struct Custom<T, OnSet = NoCallback> {
    pub data     : T,
    pub is_dirty : bool,
    pub on_set   : Callback<OnSet>,
    pub logger   : Logger,
}

// === API ===

impl<T: Default, OnSet> Custom<T, OnSet> {
    pub fn new(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let data     = Default::default();
        let is_dirty = false;
        Self { data, is_dirty, on_set, logger }
    }
}

impl<T: Default, OnSet: Callback0> Custom<T, OnSet> {
    pub fn set(&mut self, f: fn(&mut T)) {
        group!(self.logger, "Setting.", {
            if !self.is_dirty {
                self.on_set.call();
                self.is_dirty = true;
            }
            f(&mut self.data);
        })
    }

    pub fn set_to(&mut self, t: T) {
        group!(self.logger, "Setting.", {
            if !self.is_dirty {
                self.on_set.call();
                self.is_dirty = true;
            }
            self.data = t;
        })
    }

    pub fn unset(&mut self) {
        self.logger.info("Unsetting.");
        self.data     = default();
        self.is_dirty = false;
    }
}

impl<T, OnSet> DirtyCheck for Custom<T, OnSet> {
    fn is_set(&self) -> bool {
        self.is_dirty
    }
}

// ====================
// === SharedCustom ===
// ====================

// === Definition ===

#[derive(Derivative)]
#[derivative(Clone(bound = ""))]
#[derivative(Debug(bound="T:Debug"))]
pub struct SharedCustom<T = u32, OnSet = NoCallback> {
    pub data_ref: Rc<RefCell<Custom<T, OnSet>>>,
}

// === API ===

impl<T: Default, OnSet> SharedCustom<T, OnSet> {
    pub fn new(on_set: OnSet, logger: Logger) -> Self {
        Self::new_raw(Callback(on_set), logger)
    }

    pub fn new_raw(on_set: Callback<OnSet>, logger: Logger) -> Self {
        let base     = Custom::new(on_set, logger);
        let data_ref = Rc::new(RefCell::new(base));
        Self { data_ref }
    }
}

impl<T: Default, OnSet: Callback0> SharedCustom<T, OnSet> {
    pub fn set(&self, f: fn(&mut T)) {
        self.data_ref.borrow_mut().set(f);
    }

    pub fn set_to(&self, t: T) {
        self.data_ref.borrow_mut().set_to(t);
    }

    pub fn is_set(&self) -> bool {
        self.data_ref.borrow().is_set()
    }

    pub fn unset(&self) {
        self.data_ref.borrow_mut().unset()
    }

    pub fn data(&self) -> SharedCustomDataGuard<'_, T, OnSet> {
        SharedCustomDataGuard(self.data_ref.borrow())
    }
}

pub struct SharedCustomDataGuard<'t, T, OnSet> (Ref<'t, Custom<T, OnSet>>);

impl<'t, T, OnSet> Deref for SharedCustomDataGuard<'t, T, OnSet> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0.data
    }
}

// =================================================================================================
// === Simple ======================================================================================
// =================================================================================================

// ====================
// === SharedSimple ===
// ====================

#[derive(Clone, Debug)]
pub struct SharedSimple {
    pub data: Rc<RefCell<Simple>>,
}

impl SharedSimple {
    pub fn new(logger: &Logger) -> Self {
        let data = Rc::new(RefCell::new(Simple::new(logger)));
        Self { data }
    }

    pub fn new_child(&self, logger: &Logger) -> Self {
        let child = Self::new(logger);
        child.set_parent(Some(self.clone()));
        child
    }

    pub fn set_parent(&self, new_parent: Option<SharedSimple>) {
        self.data.borrow_mut().parent = new_parent;
    }

    pub fn is_set(&self) -> bool {
        self.data.borrow().is_set()
    }

    pub fn change(&self, new_state: bool) {
        self.data.borrow_mut().change(new_state)
    }

    pub fn set(&self) {
        self.change(true)
    }

    pub fn unset(&self) {
        self.change(false)
    }
}

// ==============
// === Simple ===
// ==============

#[derive(Clone, Debug)]
pub struct Simple {
    state:  bool,
    parent: Option<SharedSimple>,
    logger: Logger,
}

impl Simple {
    pub fn new(logger: &Logger) -> Self {
        Self { state: false, parent: None, logger: logger.clone() }
    }

    pub fn is_set(&self) -> bool {
        self.state
    }

    pub fn change(&mut self, new_state: bool) {
        if self.state != new_state {
            self.state = new_state;
            self.logger.group(fmt!("Setting dirty to {}.", new_state), || {
                new_state.when(|| self.parent.iter().for_each(|p| p.set()))
            });
        }
    }

    pub fn set(&mut self) {
        self.change(true)
    }

    pub fn unset(&mut self) {
        self.logger.info("Unsetting.");
        self.change(false)
    }
}
