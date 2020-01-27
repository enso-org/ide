//! Definitions of callback handling utilities.

use crate::prelude::*;

use std::any::TypeId;



// ================
// === Callback ===
// ================

/// Immutable callback type.
pub trait CallbackFn = Fn() + 'static;

/// Immutable callback object.
pub type Callback = Box<dyn CallbackFn>;

/// Callback object smart constructor.
#[allow(non_snake_case)]
pub fn Callback<F:CallbackFn>(f:F) -> Callback {
    Box::new(f)
}

/// Mutable callback type.
pub trait CallbackMutFn = FnMut() + 'static;

/// Mutable callback object.
pub type CallbackMut = Box<dyn CallbackMutFn>;

/// Mutable callback type with one parameter.
pub trait CallbackMut1Fn<T> = FnMut(T) + 'static;

/// Mutable callback object with one parameter.
pub type CallbackMut1<T> = Box<dyn CallbackMut1Fn<T>>;


/// Mutable callback type with one parameter.
pub trait XCallbackMut1Fn<T> = FnMut(&T) + 'static;

/// Mutable callback object with one parameter.
pub type XCallbackMut1<T> = Box<dyn XCallbackMut1Fn<T>>;



// ======================
// === CallbackHandle ===
// ======================

/// Handle to a callback. When the handle is dropped, the callback is removed.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackHandle {
    rc: Rc<()>
}

impl CallbackHandle {

    /// Create a new handle.
    pub fn new() -> Self {
        default()
    }

    /// Create guard for this handle.
    pub fn guard(&self) -> Guard {
        let weak = Rc::downgrade(&self.rc);
        Guard {weak}
    }

    /// Forget the handle. Warning! You would not be able to stop the callback after performing this
    /// operation.
    pub fn forget(self) {
        std::mem::forget(self)
    }
}

/// CallbackHandle's guard. Used to check if the handle is still valid.
pub struct Guard {
    weak: Weak<()>
}

impl Guard {
    /// Checks if the handle is still valid.
    pub fn exists(&self) -> bool {
        self.weak.upgrade().is_some()
    }
}



// ========================
// === CallbackRegistry ===
// ========================

/// Registry gathering callbacks. Each registered callback is assigned with a handle. Callback and
/// handle lifetimes are strictly connected. As soon a handle is dropped, the callback is removed
/// as well.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry {
    #[derivative(Debug="ignore")]
    callback_list: Vec<(Guard, CallbackMut)>
}

impl CallbackRegistry {

    /// Adds new callback and returns a new handle for it.
    pub fn add<F:CallbackMutFn>(&mut self, callback:F) -> CallbackHandle {
        let callback = Box::new(callback);
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.callback_list.push((guard, callback));
        handle
    }

    /// Fires all registered callbacks.
    pub fn run_all(&mut self) {
        self.clear_unused_callbacks();
        self.callback_list.iter_mut().for_each(|(_,callback)| callback());
    }

    /// Checks all registered callbacks and removes the ones which got dropped.
    fn clear_unused_callbacks(&mut self) {
        self.callback_list.retain(|(guard,_)| guard.exists());
    }
}

/// Registry gathering callbacks. Each registered callback is assigned with a handle. Callback and
/// handle lifetimes are strictly connected. As soon a handle is dropped, the callback is removed
/// as well.
#[derive(Derivative)]
#[derivative(Debug, Default)]
pub struct CallbackRegistry1<T:Copy> {
    #[derivative(Debug="ignore")]
    callback_list: Vec<(Guard, CallbackMut1<T>)>
}

impl<T:Copy> CallbackRegistry1<T> {
    /// Adds new callback and returns a new handle for it.
    pub fn add<F:CallbackMut1Fn<T>>(&mut self, callback:F) -> CallbackHandle {
        let callback = Box::new(callback);
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.callback_list.push((guard, callback));
        handle
    }

    /// Fires all registered callbacks.
    pub fn run_all(&mut self, t:T) {
        self.clear_unused_callbacks();
        self.callback_list.iter_mut().for_each(move |(_,callback)| callback(t));
    }

    /// Checks all registered callbacks and removes the ones which got dropped.
    fn clear_unused_callbacks(&mut self) {
        self.callback_list.retain(|(guard,_)| guard.exists());
    }
}



// ========================
// === CallbackRegistry ===
// ========================

// TODO CallbackRegistry1 implementation is broken. It requires `T` to be `Copy` which does not
//      make sense in general. This implementation is a correct one. All usages of the old one
//      should be replaced in subsequent PRs.

/// Registry gathering callbacks. Each registered callback is assigned with a handle. Callback and
/// handle lifetimes are strictly connected. As soon a handle is dropped, the callback is removed
/// as well.
#[derive(Derivative)]
#[derivative(Debug,Default(bound=""))]
pub struct XCallbackRegistry1<T> {
    #[derivative(Debug="ignore")]
    callback_list: Vec<(Guard,XCallbackMut1<T>)>
}

impl<T> XCallbackRegistry1<T> {
    /// Adds new callback and returns a new handle for it.
    pub fn add<F:XCallbackMut1Fn<T>>(&mut self, callback:F) -> CallbackHandle {
        let callback = Box::new(callback);
        let handle   = CallbackHandle::new();
        let guard    = handle.guard();
        self.callback_list.push((guard,callback));
        handle
    }

    /// Fires all registered callbacks.
    pub fn run_all(&mut self, t:&T) {
        self.clear_unused_callbacks();
        self.callback_list.iter_mut().for_each(move |(_,callback)| callback(t));
    }

    /// Checks all registered callbacks and removes the ones which got dropped.
    fn clear_unused_callbacks(&mut self) {
        self.callback_list.retain(|(guard,_)| guard.exists());
    }
}



// ==========================
// === DynEventDispatcher ===
// ==========================

/// A dynamic event wrapper. Dynamic events can be pattern matched by their types. See docs of
/// `DynEventDispatcher` to learn more.
#[derive(Debug,Clone)]
pub struct DynEvent {
    any: Rc<dyn Any>
}

impl DynEvent {
    /// Constructor.
    pub fn new<T:'static>(t:T) -> Self {
        let any = Rc::new(t);
        DynEvent {any}
    }
}

/// A dynamic event dispatcher. Allows dispatching an event of any type and registering listeners
/// for a particular type.
#[derive(Derivative,Default)]
#[derivative(Debug)]
pub struct DynEventDispatcher {
    #[derivative(Debug="ignore")]
    listener_map: HashMap<TypeId,Vec<(Guard,XCallbackMut1<DynEvent>)>>
}

impl DynEventDispatcher {
    /// Registers a new listener for a given type.
    pub fn add_listener<F:XCallbackMut1Fn<T>,T:'static>(&mut self, mut f:F) -> CallbackHandle {
        let callback = Box::new(move |event:&DynEvent| {
            event.any.downcast_ref::<T>().iter().for_each(|t| { f(t) })
        });
        let type_id   = (&PhantomData::<T>).type_id();
        let handle    = CallbackHandle::new();
        let guard     = handle.guard();
        let listeners = self.listener_map.entry(type_id).or_insert_with(default);
        listeners.push((guard,callback));
        handle
    }

    /// Dispatch an event to all listeners registered for that particular event type.
    pub fn dispatch(&mut self, event:&DynEvent) {
        let type_id = event.any.type_id();
        self.listener_map.get_mut(&type_id).iter_mut().for_each(|listeners| {
            listeners.retain(|(guard,_)| guard.exists());
            listeners.iter_mut().for_each(move |(_,callback)| callback(event));
        });
    }
}
