//! Implementation of display objects, elements that have visual representation and can form
//! hierarchical layouts. The implementation is very careful about performance, it tracks the
//! transformation changes and updates only the needed subset of the display object tree on demand.

use crate::prelude::*;
use logger::enabled::Logger;

use super::transform;

use crate::control::callback::DynEvent;
use crate::control::callback::DynEventDispatcher;
use crate::data::dirty::traits::*;
use crate::data::dirty;
use crate::display::scene::Scene;

use data::opt_vec::OptVec;
use nalgebra::Matrix4;
use nalgebra::Vector3;
use transform::CachedTransform;



// ==================
// === ParentBind ===
// ==================

/// Description of parent-child relation. It contains reference to parent node and information
/// about the child index there. It is used when a child is reconnected to different parent to
/// update the old parent with the information that the child was removed.
#[derive(Derivative)]
//#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
#[allow(missing_docs)]
pub struct ParentBind<Host> {
    pub parent : WeakInstance<Host>,
    pub index  : usize
}

impl<Host> ParentBind<Host> {
    pub fn parent(&self) -> Option<Instance<Host>> {
        self.parent.upgrade()
    }
}

impl<Host> Drop for ParentBind<Host> {
    fn drop(&mut self) {
        self.parent().for_each(|p|p.remove_child_by_index(self.index));
    }
}



// =================
// === Callbacks ===
// =================

/// Callbacks manager for display objects. Callbacks can be set only once. Panics if you try set
/// another callback to field with an already assigned callback. This design was chosen because it
/// is very lightweight and is not confusing (setting a callback unregisters previous one). We may
/// want to switch to a real callback registry in the future if there will be suitable use cases for
/// it.
#[derive(Derivative)]
#[derivative(Default(bound=""))]
#[allow(clippy::type_complexity)]
pub struct Callbacks<Host> {
    pub on_updated : RefCell<Option<Box<dyn Fn(&Model<Host>)>>>,
    pub on_show    : RefCell<Option<Box<dyn Fn(&Host)>>>,
    pub on_hide    : RefCell<Option<Box<dyn Fn(&Host)>>>,
}

impl<Host> Callbacks<Host> {
    fn on_updated(&self, model:&Model<Host>) {
        if let Some(f) = &*self.on_updated.borrow() { f(model) }
    }

    fn on_show(&self, host:&Host) {
        if let Some(f) = &*self.on_show.borrow() { f(host) }
    }

    fn on_hide(&self, host:&Host) {
        if let Some(f) = &*self.on_hide.borrow() { f(host) }
    }
}

impl<Host> Debug for Callbacks<Host> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Callbacks")
    }
}



// ==================
// === DirtyFlags ===
// ==================

// === Types ===

pub type ChildrenDirty   = dirty::SharedSet<usize,Option<Box<dyn Fn()>>>;
pub type RemovedChildren<Host> = dirty::SharedVector<WeakInstance<Host>,Option<Box<dyn Fn()>>>;
pub type NewParentDirty  = dirty::SharedBool<()>;
pub type TransformDirty  = dirty::SharedBool<Option<Box<dyn Fn()>>>;


// === Definition ===

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct DirtyFlags<Host> {
    parent           : NewParentDirty,
    children         : ChildrenDirty,
    removed_children : RemovedChildren<Host>,
    transform        : TransformDirty,
    #[derivative(Debug="ignore")]
    callback         : Rc<RefCell<Box<dyn Fn()>>>,
}

impl<Host> DirtyFlags<Host> {
    #![allow(trivial_casts)]
    pub fn new(logger:impl AnyLogger) -> Self {
        let parent           = NewParentDirty  :: new(logger::disabled::Logger::sub(&logger,"dirty.parent"),());
        let children         = ChildrenDirty   :: new(logger::disabled::Logger::sub(&logger,"dirty.children"),None);
        let removed_children = RemovedChildren :: new(logger::disabled::Logger::sub(&logger,"dirty.removed_children"),None);
        let transform        = TransformDirty  :: new(logger::disabled::Logger::sub(&logger,"dirty.transform"),None);
        let callback         = Rc::new(RefCell::new(Box::new(||{}) as Box<dyn Fn()>));

        let on_mut = enclose!((callback) move || (callback.borrow())() );
        transform        . set_callback(Some(Box::new(on_mut.clone())));
        children         . set_callback(Some(Box::new(on_mut.clone())));
        removed_children . set_callback(Some(Box::new(on_mut)));

        Self {parent,children,removed_children,transform,callback}
    }

    fn set_callback<F:'static+Fn()>(&self,f:F) {
        *self.callback.borrow_mut() = Box::new(f);
    }

    fn unset_callback(&self) {
        *self.callback.borrow_mut() = Box::new(||{});
    }
}



// =================
// === Model ===
// =================

/// A hierarchical representation of object containing a position, a scale and a rotation.
#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Model<Host=Scene> {
    host        : PhantomData <Host>,
    dirty       : DirtyFlags  <Host>,
    callbacks   : Callbacks   <Host>,
    parent_bind : RefCell     <Option<ParentBind<Host>>>,
    children    : RefCell     <OptVec<WeakInstance<Host>>>,
    transform   : Cell        <CachedTransform>,
    visible     : Cell        <bool>,
    logger      : Logger,
}

impl<Host> Model<Host> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger      = Logger::from_logger(logger);
        let parent_bind = default();
        let children    = default();
        let transform   = default();
        let dirty       = DirtyFlags::new(&logger);
        let visible     = Cell::new(true);
        let callbacks   = default();
        let host        = default();
        Self {logger,parent_bind,children,transform,visible,callbacks,dirty,host}
    }

    /// Checks whether the object is visible.
    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    /// Checks whether the object is orphan (do not have parent object attached).
    pub fn is_orphan(&self) -> bool {
        self.parent_bind.borrow().is_none()
    }

    /// Parent object getter.
    pub fn parent(&self) -> Option<Instance<Host>> {
        self.parent_bind.borrow().as_ref().and_then(|t|t.parent())
    }

    /// Count of children objects.
    pub fn children_count(&self) -> usize {
        self.children.borrow().len()
    }




    /// Removes child by a given index. Does nothing if the index was incorrect.
    fn remove_child_by_index(&self, index:usize) {
        self.children.borrow_mut().remove(index).for_each(|child| {
            child.upgrade().for_each(|child| child.raw_unset_parent());
            self.dirty.children.unset(&index);
            self.dirty.removed_children.set(child);
        });
    }

    /// Recompute the transformation matrix of this object and update all of its dirty children.
    pub fn update(&self, host:&Host) {
        let origin0 = Matrix4::identity();
        self.update_with_origin(host,origin0,false)
    }

    /// Updates object transformations by providing a new origin location. See docs of `update` to
    /// learn more.
    fn update_with_origin
    (&self, host:&Host, parent_origin:Matrix4<f32>, parent_origin_changed:bool) {
        self.update_visibility(host);
        let has_new_parent      = self.dirty.parent.check();
        let is_origin_dirty     = has_new_parent || parent_origin_changed;
        let new_parent_origin   = is_origin_dirty.as_some(parent_origin);
        let parent_origin_label = if new_parent_origin.is_some() {"new"} else {"old"};
        group!(self.logger, "Update with {parent_origin_label} parent origin.", {
            let mut transform  = self.transform.get();
            let origin_changed = transform.update(new_parent_origin);
            let new_origin     = transform.matrix;
            self.transform.set(transform);
            if origin_changed {
                self.logger.info("Self origin changed.");
                self.callbacks.on_updated(self);
                if !self.children.borrow().is_empty() {
                    group!(self.logger, "Updating all children.", {
                        self.children.borrow().iter().for_each(|child| {
                            child.upgrade().for_each(|t| t.update_with_origin(host,new_origin,true));
                        });
                    })
                }
            } else {
                self.logger.info("Self origin did not change.");
                if self.dirty.children.check_all() {
                    group!(self.logger, "Updating dirty children.", {
                        self.dirty.children.take().iter().for_each(|ix| {
                            self.children.borrow()[*ix].upgrade().for_each(|t| {
                                t.update_with_origin(host,new_origin,false);
                            })
                        });
                    })
                }
            }
            self.dirty.children.unset_all();
        });
        self.dirty.transform.unset();
        self.dirty.parent.unset();
    }

    /// Internal
    fn update_visibility(&self, host:&Host) {
        self.hide_removed_children(host);
        let parent_changed = self.dirty.parent.check();
        let foo = self.is_orphan();
        if parent_changed && !foo {
            self.show(host)
        }
    }

    pub fn hide_removed_children(&self, host:&Host) {
        if self.dirty.removed_children.check_all() {
            group!(self.logger, "Updating removed children", {
                self.dirty.removed_children.take().into_iter().for_each(|child| {
                    if let Some(child) = child.upgrade() {
                        if child.is_orphan() {
                            child.hide(host)
                        }
                    }
                });
            })
        }
    }

    pub fn hide(&self, host:&Host) {
        self.hide_removed_children(host);
        if self.visible.get() {
            self.logger.info("Hiding.");
            self.visible.set(false);
            self.callbacks.on_hide(host);
            self.children.borrow().iter().for_each(|child| {
                child.upgrade().for_each(|t| t.hide(host));
            });
        }
    }

    pub fn show(&self, host:&Host) {
       if !self.visible.get() {
           self.logger.info("Showing.");
           self.visible.set(true);
           self.callbacks.on_show(host);
           self.children.borrow().iter().for_each(|child| {
               child.upgrade().for_each(|t| t.show(host));
           });
       }
    }

    /// Unset all node's callbacks. Because the Instance structure may live longer than one's could
    /// expect (usually to the next scene refresh), it is wise to unset all callbacks when disposing
    /// object.
    // TODO[ao] Instead if this, the Instance should keep weak references to its children (at least in
    // "removed" list) and do not extend their lifetime.
    pub fn clear_callbacks(&self) {
        self.callbacks.on_updated.clear();
        self.callbacks.on_show.clear();
        self.callbacks.on_hide.clear();
    }
}


// === Private API ===

impl<Host> Model<Host> {
    fn register_child<T:Object<Host>>(&self, child:&T) -> usize {
        let child = child.display_object().clone();
        let index = self.children.borrow_mut().insert(child.downgrade());
        self.dirty.children.set(index);
        index
    }

    /// Removes and returns the parent bind. Please note that the parent is not updated.
    fn take_parent_bind(&self) -> Option<ParentBind<Host>> {
        self.parent_bind.borrow_mut().take()
    }

    /// Removes the binding to the parent object. This is internal operation. Parent is not updated.
    fn raw_unset_parent(&self) {
        self.logger.info("Removing parent bind.");
        self.dirty.unset_callback();
        self.dirty.parent.set();
    }

    /// Set parent of the object. If the object already has a parent, the parent would be replaced.
    fn set_parent_bind(&self, bind:ParentBind<Host>) {
        self.logger.info("Adding new parent bind.");
        if let Some(parent) = bind.parent() {
            let index = bind.index;
            let dirty = parent.dirty.children.clone_ref();
            self.dirty.set_callback(move || dirty.set(index));
            self.dirty.parent.set();
            *self.parent_bind.borrow_mut() = Some(bind);
        }
    }
}


// === Getters ===

impl<Host> Model<Host> {
    // pub fn parent_bind(&self) -> Option<ParentBind<Host>> {
    //     self.parent_bind.get()
    // }

    pub fn global_position(&self) -> Vector3<f32> {
        self.transform.get().global_position()
    }

    pub fn position(&self) -> Vector3<f32> {
        self.transform.get().position()
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.transform.get().scale()
    }

    pub fn rotation(&self) -> Vector3<f32> {
        self.transform.get().rotation()
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.transform.get().matrix()
    }
}


// === Setters ===

impl<Host> Model<Host> {
    fn with_transform<F,T>(&self, f:F) -> T
    where F : FnOnce(&mut CachedTransform) -> T {
//        if let Some(bind) = self.parent_bind.get() {
//            bind.parent.dirty.children.set(bind.index);
//        }
        self.dirty.transform.set();

        let mut transform = self.transform.get();
        let out = f(&mut transform);
        self.transform.set(transform);
        out
    }

    pub fn set_position(&self, v:Vector3<f32>) {
        self.with_transform(|t| t.set_position(v));
    }

    pub fn set_scale(&self, v:Vector3<f32>) {
        self.with_transform(|t| t.set_scale(v));
    }

    pub fn set_rotation(&self, v:Vector3<f32>) {
        self.with_transform(|t| t.set_rotation(v));
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|t| t.mod_position(f));
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|t| t.mod_rotation(f));
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|t| t.mod_scale(f));
    }

    pub fn set_on_updated<F:Fn(&Model<Host>)+'static>(&self, f:F) {
        self.callbacks.on_updated.set(Box::new(f))
    }

    /// Sets a callback which will be called with a reference to scene when the object will be
    /// shown (attached to visible display object graph).
    pub fn set_on_show<F:Fn(&Host)+'static>(&self, f:F) {
        self.callbacks.on_show.set(Box::new(f))
    }

    /// Sets a callback which will be called with a reference to scene when the object will be
    /// hidden (detached from display object graph).
    pub fn set_on_hide<F:Fn(&Host)+'static>(&self, f:F) {
        self.callbacks.on_hide.set(Box::new(f))
    }
}



// ==========
// === Id ===
// ==========

#[derive(Clone,CloneRef,Copy,Debug,Default,Display,Eq,From,Hash,Into,PartialEq)]
pub struct Id(usize);



// ================
// === Instance ===
// ================

#[derive(Derivative,Shrinkwrap)]
#[derive(CloneRef)]
#[derivative(Clone(bound=""))]
pub struct Instance<Host=Scene> {
    pub rc : Rc<Model<Host>>
}

impl<Host> Instance<Host> {
    pub fn clone2(&self) -> Self {
        Self {rc:self.rc.clone()}
    }

    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        Self {rc:Rc::new(Model::new(logger))}
    }

    pub fn downgrade(&self) -> WeakInstance<Host> {
        let weak = Rc::downgrade(&self.rc);
        WeakInstance{weak}
    }
}


// === Public API ==

impl<Host> Instance<Host> {
    pub fn with_logger<F:FnOnce(&Logger)>(&self, f:F) {
        f(&self.rc.logger)
    }

    pub fn _id(&self) -> Id {
        Id(Rc::downgrade(&self.rc).as_raw() as *const() as usize)
    }

    /// Adds a new `Object` as a child to the current one.
    pub fn _add_child<T:Object<Host>>(&self, child:&T) {
        self.clone2().add_child_take(child);
    }

    /// Adds a new `Object` as a child to the current one. This is the same as `add_child` but takes
    /// the ownership of `self`.
    pub fn add_child_take<T:Object<Host>>(self, child:&T) {
        self.rc.logger.info("Adding new child.");
        let child = child.display_object();
        child.unset_parent();
        let index = self.register_child(child);
        self.rc.logger.info(|| format!("Child index is {}.", index));
        let parent_bind = ParentBind {parent:self.downgrade(),index};
        child.set_parent_bind(parent_bind);
    }

    /// Removes the provided object reference from child list of this object. Does nothing if the
    /// reference was not a child of this object.
    pub fn _remove_child<T:Object<Host>>(&self, child:&T) {
        let child = child.display_object();
        if self.has_child(child) {
            child.unset_parent()
        }
    }

    /// Replaces the parent binding with a new parent.
    pub fn set_parent<T:Object<Host>>(&self, parent:&T) {
        parent.display_object().add_child(self);
    }

    /// Removes the current parent binding.
    pub fn _unset_parent(&self) {
        self.take_parent_bind();
    }

    /// Checks if the provided object is child of the current one.
    pub fn has_child<T:Object<Host>>(&self, child:&T) -> bool {
        self.child_index(child).is_some()
    }

    /// Checks if the object has a parent.
    pub fn _has_parent(&self) -> bool {
        self.rc.parent_bind.borrow().is_some()
    }

    /// Returns the index of the provided object if it was a child of the current one.
    pub fn child_index<T:Object<Host>>(&self, child:&T) -> Option<usize> {
        let child = child.display_object();
        child.parent_bind.borrow().as_ref().and_then(|bind| {
            if bind.parent().as_ref() == Some(self) { Some(bind.index) } else { None }
        })
    }
}


// === Getters ===

impl<Host> Instance<Host> {
    pub fn index(&self) -> Option<usize> {
        self.parent_bind.borrow().as_ref().map(|t| t.index)
    }
}


// === Instances ===

impl<Host> PartialEq for Instance<Host> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.rc,&other.rc)
    }
}

impl<Host> Display for Instance<Host> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Instance")
    }
}

impl<Host> Debug for Instance<Host> {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Instance")
    }
}



// ================
// === WeakInstance ===
// ================

#[derive(Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Debug(bound=""))]
pub struct WeakInstance<Host> {
    pub weak : Weak<Model<Host>>
}

impl<Host> WeakInstance<Host> {
    pub fn upgrade(&self) -> Option<Instance<Host>> {
        self.weak.upgrade().map(|rc| Instance {rc})
    }
}

// FIXME: This is not a valid implementation as the pointers may alias when one instance was dropped
//        and other was created in the same location.
impl<Host> PartialEq for WeakInstance<Host> {
    fn eq(&self, other:&Self) -> bool {
        self.weak.ptr_eq(&other.weak)
    }
}



// ==============
// === Object ===
// ==============

pub trait Object<Host=Scene> {
    fn display_object(&self) -> &Instance<Host>;
}

impl<Host> Object<Host> for Instance<Host> {
    fn display_object(&self) -> &Instance<Host> {
        self
    }
}

impl<Host,T:Object<Host>> Object<Host> for &T {
    fn display_object(&self) -> &Instance<Host> {
        let t : &T = *self;
        t.display_object()
    }
}



// =================
// === ObjectOps ===
// =================

// FIXME
// We are using names with underscores in order to fix this bug
// https://github.com/rust-lang/rust/issues/70727
impl<Host,T:Object<Host>> ObjectOps<Host> for T {}
pub trait ObjectOps<Host=Scene> : Object<Host> {
    fn add_child<T:Object<Host>>(&self, child:&T) {
        self.display_object()._add_child(child.display_object());
    }

    fn remove_child<T:Object<Host>>(&self, child:&T) {
        self.display_object()._remove_child(child.display_object());
    }

    fn id(&self) -> Id {
        self.display_object()._id()
    }

    fn unset_parent(&self) {
        self.display_object()._unset_parent();
    }

    fn has_parent(&self) -> bool {
        self.display_object()._has_parent()
    }

    fn transform_matrix(&self) -> Matrix4<f32> {
        self.display_object().rc.matrix()
    }

    fn global_position(&self) -> Vector3<f32> {
        self.display_object().rc.global_position()
    }

    fn position(&self) -> Vector3<f32> {
        self.display_object().rc.position()
    }

    fn scale(&self) -> Vector3<f32> {
        self.display_object().rc.scale()
    }

    fn rotation(&self) -> Vector3<f32> {
        self.display_object().rc.rotation()
    }

    fn set_position(&self, t:Vector3<f32>) {
        self.display_object().rc.set_position(t);
    }

    fn set_position_xy(&self, t:Vector2<f32>) {
        self.mod_position(|p| {
            p.x = t.x;
            p.y = t.y;
        })
    }

    fn set_position_x(&self, t:f32) {
        self.mod_position(|p| p.x = t)
    }

    fn set_position_y(&self, t:f32) {
        self.mod_position(|p| p.y = t)
    }

    fn set_position_z(&self, t:f32) {
        self.mod_position(|p| p.z = t)
    }

    fn set_scale(&self, t:Vector3<f32>) {
        self.display_object().rc.set_scale(t);
    }

    fn set_rotation(&self, t:Vector3<f32>) {
        self.display_object().rc.set_rotation(t);
    }

    fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.display_object().rc.mod_position(f)
    }

    fn mod_position_xy<F:FnOnce(Vector2<f32>)->Vector2<f32>>(&self, f:F) {
        self.set_position_xy(f(self.position().xy()));
    }

    fn mod_position_x<F:FnOnce(f32)->f32>(&self, f:F) {
        self.set_position_x(f(self.position().x));
    }

    fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.display_object().rc.mod_rotation(f)
    }

    fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.display_object().rc.mod_scale(f)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn hierarchy_test() {
        let node1 = Instance::<()>::new(Logger::new("node1"));
        let node2 = Instance::<()>::new(Logger::new("node2"));
        let node3 = Instance::<()>::new(Logger::new("node3"));
        node1.add_child(&node2);
        assert_eq!(node2.index(),Some(0));

        node1.add_child(&node2);
        assert_eq!(node2.index(),Some(0));

        node1.add_child(&node3);
        assert_eq!(node3.index(),Some(1));

        node1.remove_child(&node3);
        assert_eq!(node3.index(),None);
    }

    #[test]
    fn transformation_test() {
        let node1 = Instance::<()>::new(Logger::new("node1"));
        let node2 = Instance::<()>::new(Logger::new("node2"));
        let node3 = Instance::<()>::new(Logger::new("node3"));
        assert_eq!(node1.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node1.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(0.0,0.0,0.0));

        node1.mod_position(|t| t.x += 7.0);
        node1.add_child(&node2);
        node2.add_child(&node3);
        assert_eq!(node1.position()        , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node1.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(0.0,0.0,0.0));

        node1.update(&());
        assert_eq!(node1.position()        , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,0.0,0.0));

        node2.mod_position(|t| t.y += 5.0);
        node1.update(&());
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,5.0,0.0));

        node3.mod_position(|t| t.x += 1.0);
        node1.update(&());
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(8.0,5.0,0.0));

        node2.mod_rotation(|t| t.z += PI/2.0);
        node1.update(&());
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));

        node1.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.global_position() , Vector3::new(8.0,0.0,0.0));

        node1.remove_child(&node3);
        node3.update(&());
        assert_eq!(node3.global_position() , Vector3::new(1.0,0.0,0.0));

        node2.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));

        node1.remove_child(&node3);
        node1.update(&());
        node2.update(&());
        node3.update(&());
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));
    }

    #[test]
    fn parent_test() {
        let node1 = Instance::<()>::new(Logger::new("node1"));
        let node2 = Instance::<()>::new(Logger::new("node2"));
        let node3 = Instance::<()>::new(Logger::new("node3"));
        node1.add_child(&node2);
        node1.add_child(&node3);
        node2.unset_parent();
        node3.unset_parent();
        assert_eq!(node1.children_count(),0);
    }


    #[test]
    fn visibility_test() {
        let node1 = Instance::<()>::new(Logger::new("node1"));
        let node2 = Instance::<()>::new(Logger::new("node2"));
        let node3 = Instance::<()>::new(Logger::new("node3"));
        assert_eq!(node3.is_visible(),true);
        node3.update(&());
        assert_eq!(node3.is_visible(),true);
        node1.add_child(&node2);
        node2.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.is_visible(),true);
        node3.unset_parent();
        assert_eq!(node3.is_visible(),true);
        node1.update(&());
        assert_eq!(node3.is_visible(),false);
        node1.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.is_visible(),true);
        node2.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.is_visible(),true);
        node3.unset_parent();
        node1.update(&());
        assert_eq!(node3.is_visible(),false);
        node2.add_child(&node3);
        node1.update(&());
        assert_eq!(node3.is_visible(),true);
    }
}
