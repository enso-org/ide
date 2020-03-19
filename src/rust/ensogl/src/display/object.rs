#![allow(missing_docs)]

#[warn(missing_docs)]
pub mod transform;

use crate::prelude::*;

use crate::closure;
use crate::data::dirty;
use crate::data::dirty::traits::*;
use data::opt_vec::OptVec;

use nalgebra::Vector3;
use nalgebra::Matrix4;
use transform::CachedTransform;

use crate::control::callback::DynEventDispatcher;
use crate::control::callback::DynEvent;
use shapely::shared;
use crate::display::scene::Scene;



// ==============
// === Traits ===
// ==============

/// Common traits.
pub mod traits {
    pub use super::Object;
    pub use super::ObjectOps;
}



// ==================
// === ParentBind ===
// ==================

/// Description of parent-child relation. It contains reference to parent node and information
/// about the child index there. It is used when a child is reconnected to different parent to
/// update the old parent with the information that the child was removed.
#[derive(Clone,Debug)]
pub struct ParentBind {
    pub parent : Node,
    pub index  : usize
}

impl ParentBind {
    pub fn dispose(&self) {
        self.parent.remove_child_by_index(self.index);
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
#[derive(Default)]
pub struct Callbacks {
    pub on_updated   : Option<Box<dyn Fn(&NodeData)>>,
    pub on_show      : Option<Box<dyn Fn()>>,
    pub on_hide      : Option<Box<dyn Fn()>>,
    pub on_show_with : Option<Rc<dyn Fn(&NodeData, &Scene)>>,
    pub on_hide_with : Option<Box<dyn Fn(&Scene)>>,
}

impl Callbacks {
    /// Setter. Warning, altering the node structure during execution of the callback may cause
    /// panic.
    pub fn set_on_updated<F:Fn(&NodeData)+'static>(&mut self, f:F) {
        if self.on_updated.is_some() { panic!("The `on_updated` callback was already set.") }
        self.on_updated = Some(Box::new(f))
    }

    /// Setter. Warning, altering the node structure during execution of the callback may cause
    /// panic.
    pub fn set_on_show<F:Fn()+'static>(&mut self, f:F) {
        if self.on_show.is_some() { panic!("The `on_show` callback was already set.") }
        self.on_show = Some(Box::new(f))
    }

    /// Setter. Warning, altering the node structure during execution of the callback may cause
    /// panic.
    pub fn set_on_hide<F:Fn()+'static>(&mut self, f:F) {
        if self.on_hide.is_some() { panic!("The `on_hide` callback was already set.") }
        self.on_hide = Some(Box::new(f))
    }

    pub fn set_on_show_with<F:Fn(&NodeData, &Scene)+'static>(&mut self, f:F) {
        if self.on_show_with.is_some() { panic!("The `on_show_with` callback was already set.") }
        self.on_show_with = Some(Rc::new(f))
    }

    pub fn set_on_hide_with<F:Fn(&Scene)+'static>(&mut self, f:F) {
        if self.on_hide_with.is_some() { panic!("The `on_hide_with` callback was already set.") }
        self.on_hide_with = Some(Box::new(f))
    }
}

impl Debug for Callbacks {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Callbacks")
    }
}



// ============
// === Node ===
// ============

// === Types ===

pub type ChildDirty      = dirty::SharedSet<usize,Option<OnChange>>;
pub type RemovedChildren = dirty::SharedVector<Node,Option<OnChange>>;
pub type NewParentDirty  = dirty::SharedBool<()>;
pub type TransformDirty  = dirty::SharedBool<Option<OnChange>>;


// === Callbacks ===

closure! {
fn fn_on_change(dirty:ChildDirty, ix:usize) -> OnChange { || dirty.set(ix) }
}


// === Definition ===

#[derive(Clone,Shrinkwrap)]
pub struct Node {
    pub rc : Rc<NodeData>
}

impl CloneRef for Node {}

impl Node {
    pub fn new<L:Into<Logger>>(logger:L) -> Self {
        let rc = Rc::new(NodeData::new(logger));
        Self {rc}
    }
}

/// A hierarchical representation of object containing a position, a scale and a rotation.
#[derive(Debug)]
pub struct NodeData {
    parent_bind      : CloneCell<Option<ParentBind>>,
    children         : RefCell<OptVec<Node>>,
    removed_children : RemovedChildren,
    child_dirty      : ChildDirty,
    new_parent_dirty : NewParentDirty,
    transform        : Cell<CachedTransform>,
    event_dispatcher : DynEventDispatcher,
    visible          : Cell<bool>,
    callbacks        : RefCell<Callbacks>,
    logger           : Logger,
}

impl NodeData {
    pub fn new<L:Into<Logger>>(logger:L) -> Self {
        let logger           = logger.into();
        let parent_bind      = default();
        let children         = default();
        let event_dispatcher = default();
        let transform        = default();
        let child_dirty      = ChildDirty      :: new(logger.sub("child_dirty")      , None);
        let removed_children = RemovedChildren :: new(logger.sub("removed_children") , None);
        let new_parent_dirty = NewParentDirty  :: new(logger.sub("new_parent_dirty") , ());
        let visible          = Cell::new(true);
        let callbacks        = default();
        Self {logger,parent_bind,children,removed_children,event_dispatcher,transform,child_dirty
             ,new_parent_dirty,visible,callbacks}
    }

    pub fn is_visible(&self) -> bool {
        self.visible.get()
    }

    pub fn parent(&self) -> Option<Node> {
        self.parent_bind.get().map(|t| t.parent)
    }

    pub fn is_orphan(&self) -> bool {
        self.parent_bind.get().is_none()
    }

    pub fn dispatch_event(&mut self, event:&DynEvent) {
        self.event_dispatcher.dispatch(event);
        self.parent_bind.get().map_ref(|bind| bind.parent.dispatch_event(event));
    }

    pub fn child_count(&self) -> usize {
        self.children.borrow().len()
    }

    /// Removes child by a given index. Does nothing if the index was incorrect. In general, it is a
    /// better idea to use `remove_child` instead. Storing and using index explicitly is error
    /// prone.
    pub fn remove_child_by_index(&self, index:usize) {
        self.children.borrow_mut().remove(index).for_each(|child| {
            child.raw_unset_parent();
            self.child_dirty.unset(&index);
            self.removed_children.set(child);
        });
    }

    /// Recompute the transformation matrix of this object and update all of its dirty children.
    pub fn update(&self) {
        let origin0 = Matrix4::identity();
        self.update_origin(&None,origin0,false)
    }

    /// Recompute the transformation matrix of this object and update all of its dirty children.
    pub fn update_with(&self, scene:&Scene) {
        let origin0 = Matrix4::identity();
        self.update_origin(&Some(scene),origin0,false)
    }

    /// Updates object transformations by providing a new origin location. See docs of `update` to
    /// learn more.
    fn update_origin(&self, scene:&Option<&Scene>, parent_origin:Matrix4<f32>, force:bool) {
        self.update_visibility(scene);
        let parent_changed = self.new_parent_dirty.check();
        let use_origin     = force || parent_changed;
        let new_origin     = use_origin.as_some(parent_origin);
        let msg            = match new_origin {
            Some(_) => "Update with new parent origin.",
            None    => "Update with old parent origin."
        };
        group!(self.logger, "{msg}", {
            let mut transform  = self.transform.get();
            let origin_changed = transform.update(new_origin);
            let origin         = transform.matrix;
            self.transform.set(transform);
            if origin_changed {
                self.logger.info("Self origin changed.");
                if let Some(f) = &self.callbacks.borrow().on_updated { f(self) }
                if !self.children.borrow().is_empty() {
                    group!(self.logger, "Updating all children.", {
                        self.children.borrow().iter().for_each(|child| {
                            child.update_origin(scene,origin,true);
                        });
                    })
                }
            } else {
                self.logger.info("Self origin did not change.");
                if self.child_dirty.check_all() {
                    group!(self.logger, "Updating dirty children.", {
                        self.child_dirty.take().iter().for_each(|ix| {
                            self.children.borrow()[*ix].update_origin(scene,origin,false)
                        });
                    })
                }
            }
            self.child_dirty.unset_all();
        });
        self.new_parent_dirty.unset();
    }

    /// Internal
    fn update_visibility(&self, scene:&Option<&Scene>) {
        if self.removed_children.check_all() {
            group!(self.logger, "Updating removed children", {
                self.removed_children.take().into_iter().for_each(|child| {
                    if child.is_orphan() {
                        child.hide();
                        match scene {
                            Some(s) => child.hide_with(s),
                            _ => {}
                        }
                    }
                });
            })
        }

        let parent_changed = self.new_parent_dirty.check();
        if parent_changed && !self.is_orphan() {
            self.show();
            match scene {
                Some(s) => self.show_with(s),
                _ => {}
            }
        }
    }

    /// Hide this node and all of its children. This function is called automatically when updating
    /// a node with a disconnected parent.
    pub fn hide(&self) {
        if self.visible.get() {
            self.logger.info("Hiding.");
            self.visible.set(false);
            if let Some(f) = &self.callbacks.borrow().on_hide { f() }
            self.children.borrow().iter().for_each(|child| {
                child.hide();
            });
        }
    }

    pub fn hide_with(&self, scene:&Scene) {
//        if self.visible {
            self.logger.info("Hiding.");
            if let Some(f) = &self.callbacks.borrow().on_hide_with { f(scene) }
            self.children.borrow().iter().for_each(|child| {
                child.hide_with(scene);
            });
//        }
    }

    /// Show this node and all of its children. This function is called automatically when updating
    /// a node with a newly attached parent.
    pub fn show(&self) {
        if !self.visible.get() {
            self.logger.info("Showing.");
            self.visible.set(true);
            if let Some(f) = &self.callbacks.borrow().on_show { f() }
            self.children.borrow().iter().for_each(|child| {
                child.show();
            });
        }
    }

    pub fn show_with(&self, scene:&Scene) {
//        if !self.visible {
            self.logger.info("Showing.");
            let cb = self.callbacks.borrow().on_show_with.clone();
            if let Some(f) = cb { f(self,scene) }
            self.children.borrow().iter().for_each(|child| {
                child.show_with(scene);
            });
//        }
    }

    /// Unset all node's callbacks. Because the Node structure may live longer than one's could
    /// expect (usually to the next scene refresh), it is wise to unset all callbacks when disposing
    /// object.
    // TODO[ao] Instead if this, the Node should keep weak references to its children (at least in
    // "removed" list) and do not extend their lifetime.
    pub fn clear_callbacks(&self) {
        self.callbacks.borrow_mut().on_updated   = default();
        self.callbacks.borrow_mut().on_show      = default();
        self.callbacks.borrow_mut().on_hide      = default();
        self.callbacks.borrow_mut().on_show_with = default();
        self.callbacks.borrow_mut().on_hide_with = default();
    }
}


// === Private API ===

impl NodeData {
    pub fn register_child<T:Object>(&self, child:&T) -> usize {
        let child = child.display_object().clone();
        let index = self.children.borrow_mut().insert(child);
        self.child_dirty.set(index);
        index
    }

    /// Removes and returns the parent bind. Please note that the parent is not updated.
    pub fn take_parent_bind(&self) -> Option<ParentBind> {
        self.parent_bind.take()
    }

    /// Removes the binding to the parent object. This is internal operation. Parent is not updated.
    pub fn raw_unset_parent(&self) {
        self.logger.info("Removing parent bind.");
        self.child_dirty.set_callback(None);
        self.removed_children.set_callback(None);
        self.new_parent_dirty.set();
    }

    /// Set parent of the object. If the object already has a parent, the parent would be replaced.
    pub fn set_parent_bind(&self, bind:ParentBind) {
        self.logger.info("Adding new parent bind.");
        let dirty  = bind.parent.child_dirty.clone_ref();
        let index  = bind.index;
        let on_mut = move || {dirty.set(index)};
        self.child_dirty.set_callback(Some(Box::new(on_mut.clone())));
        self.removed_children.set_callback(Some(Box::new(on_mut)));
        self.new_parent_dirty.set();
        self.parent_bind.set(Some(bind));
    }

    pub fn set_parent_bind2(&self, bind:ParentBind, dirty:ChildDirty) {
        self.logger.info("Adding new parent bind.");
        let index  = bind.index;
        let on_mut = move || {dirty.set(index)};
        self.child_dirty.set_callback(Some(Box::new(on_mut.clone())));
        self.removed_children.set_callback(Some(Box::new(on_mut)));
        self.new_parent_dirty.set();
        self.parent_bind.set(Some(bind));
    }

    pub fn add_child_tmp<T:Object>(&self, this:&Node, child:&T) {
        self.logger.info("Adding new child.");
        let child = child.display_object();
        child.unset_parent();
        let index = self.register_child(child);
        self.logger.info(|| format!("Child index is {}.", index));
        let parent_bind = ParentBind {parent:this.clone(),index};
        child.set_parent_bind2(parent_bind,self.child_dirty.clone_ref());
    }
}

// === Getters ===

impl NodeData {
    /// Gets a clone of parent bind.
    pub fn parent_bind(&self) -> Option<ParentBind> {
        self.parent_bind.get()
    }

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

impl NodeData {
    fn with_transform<F,T>(&self, f:F) -> T
    where F : FnOnce(&mut CachedTransform) -> T {
        if let Some(bind) = self.parent_bind.get() {
            bind.parent.child_dirty.set(bind.index);
        }
        let mut transform = self.transform.get();
        let out = f(&mut transform);
        self.transform.set(transform);
        out
    }

    pub fn set_position(&self, t:Vector3<f32>) {
        self.with_transform(|transform| transform.set_position(t));
    }

    pub fn set_scale(&self, t:Vector3<f32>) {
        self.with_transform(|transform| transform.set_scale(t));
    }

    pub fn set_rotation(&self, t:Vector3<f32>) {
        self.with_transform(|transform| transform.set_rotation(t));
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|transform| transform.mod_position(f));
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|transform| transform.mod_rotation(f));
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.with_transform(|transform| transform.mod_scale(f));
    }

    pub fn set_on_updated<F:Fn(&NodeData)+'static>(&self, f:F) {
        self.callbacks.borrow_mut().set_on_updated(f)
    }

    pub fn set_on_show<F:Fn()+'static>(&self, f:F) {
        self.callbacks.borrow_mut().set_on_show(f)
    }

    pub fn set_on_hide<F:Fn()+'static>(&self, f:F) {
        self.callbacks.borrow_mut().set_on_hide(f)
    }

    pub fn set_on_show_with<F:Fn(&NodeData, &Scene)+'static>(&self, f:F) {
        self.callbacks.borrow_mut().set_on_show_with(f)
    }

    pub fn set_on_hide_with<F:Fn(&Scene)+'static>(&self, f:F) {
        self.callbacks.borrow_mut().set_on_hide_with(f)
    }
}

impl Display for Node {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Node")
    }
}

impl Debug for Node {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Node")
    }
}



// ============
// === Node ===
// ============

// === Public API ==

impl Node {
    pub fn with_logger<F:FnOnce(&Logger)>(&self, f:F) {
        f(&self.rc.logger)
    }

    /// Adds a new `Object` as a child to the current one.
    pub fn _add_child<T:Object>(&self, child:&T) {
        self.clone_ref().add_child_take(child);
    }

    /// Adds a new `Object` as a child to the current one. This is the same as `add_child` but takes
    /// the ownership of `self`.
    pub fn add_child_take<T:Object>(self, child:&T) {
        self.rc.logger.info("Adding new child.");
        let child = child.display_object();
        child.unset_parent();
        let index = self.register_child(child);
        self.rc.logger.info(|| format!("Child index is {}.", index));
        let parent_bind = ParentBind {parent:self,index};
        child.set_parent_bind(parent_bind);
    }

    /// Removes the provided object reference from child list of this object. Does nothing if the
    /// reference was not a child of this object.
    pub fn remove_child<T:Object>(&self, child:&T) {
        let child = child.display_object();
        if self.has_child(child) {
            child.unset_parent()
        }
    }

    /// Replaces the parent binding with a new parent.
    pub fn set_parent<T:Object>(&self, parent:&T) {
        parent.display_object().add_child(self);
    }

    /// Removes the current parent binding.
    pub fn _unset_parent(&self) {
        self.take_parent_bind().for_each(|t| t.dispose());
    }

    /// Checks if the provided object is child of the current one.
    pub fn has_child<T:Object>(&self, child:&T) -> bool {
        self.child_index(child).is_some()
    }

    /// Returns the index of the provided object if it was a child of the current one.
    pub fn child_index<T:Object>(&self, child:&T) -> Option<usize> {
        let child = child.display_object();
        child.parent_bind().and_then(|bind| {
            if self == &bind.parent { Some(bind.index) } else { None }
        })
    }
}


// === Getters ===

impl Node {
    pub fn index(&self) -> Option<usize> {
        self.parent_bind().map(|t| t.index)
    }
}

// === Instances ===

impl From<&Node> for Node {
    fn from(t:&Node) -> Self { t.clone_ref() }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.rc,&other.rc)
    }
}



// ==============
// === Object ===
// ==============

pub trait Object {
    fn display_object(&self) -> &Node;
}

impl<T> Object for T where for<'t> &'t T:Into<&'t Node> {
    fn display_object(&self) -> &Node {
        self.into()
    }
}

impl<T:Object> ObjectOps for T {}
pub trait ObjectOps : Object {
    fn add_child<T:Object>(&self, child:&T) {
        self.display_object()._add_child(child.display_object());
    }

    fn unset_parent(&self) {
        self.display_object()._unset_parent();
    }

    fn dispatch_event(&self, event:&DynEvent) {
//        self.display_object().rc.dispatch_event(event)
    }

    fn dispatch_event2(&self, event:&DynEvent) {
//        self.display_object().rc.dispatch_event(event)
    }

    fn transform_matrix(&self) -> Matrix4<f32> {
        self.display_object().rc.matrix()
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

    fn set_scale(&self, t:Vector3<f32>) {
        self.display_object().rc.set_scale(t);
    }

    fn set_rotation(&self, t:Vector3<f32>) {
        self.display_object().rc.set_rotation(t);
    }

    fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.display_object().rc.mod_position(f)
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
        let node1 = Node::new(Logger::new("node1"));
        let node2 = Node::new(Logger::new("node2"));
        let node3 = Node::new(Logger::new("node3"));
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
        let node1 = Node::new(Logger::new("node1"));
        let node2 = Node::new(Logger::new("node2"));
        let node3 = Node::new(Logger::new("node3"));
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

        node1.update();
        assert_eq!(node1.position()        , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,0.0,0.0));

        node2.mod_position(|t| t.y += 5.0);
        node1.update();
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,5.0,0.0));

        node3.mod_position(|t| t.x += 1.0);
        node1.update();
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(8.0,5.0,0.0));

        node2.mod_rotation(|t| t.z += PI/2.0);
        node1.update();
        assert_eq!(node1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(node2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));

        node1.add_child(&node3);
        node1.update();
        assert_eq!(node3.global_position() , Vector3::new(8.0,0.0,0.0));

        node1.remove_child(&node3);
        node3.update();
        assert_eq!(node3.global_position() , Vector3::new(1.0,0.0,0.0));

        node2.add_child(&node3);
        node1.update();
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));

        node1.remove_child(&node3);
        node1.update();
        node2.update();
        node3.update();
        assert_eq!(node3.global_position() , Vector3::new(7.0,6.0,0.0));
    }

    #[test]
    fn parent_test() {
        let node1 = Node::new(Logger::new("node1"));
        let node2 = Node::new(Logger::new("node2"));
        let node3 = Node::new(Logger::new("node3"));
        node1.add_child(&node2);
        node1.add_child(&node3);
        node2.unset_parent();
        node3.unset_parent();
        assert_eq!(node1.child_count(),0);
    }


    #[test]
    fn visibility_test() {
        let node1 = Node::new(Logger::new("node1"));
        let node2 = Node::new(Logger::new("node2"));
        let node3 = Node::new(Logger::new("node3"));
        assert_eq!(node3.is_visible(),true);
        node3.update();
        assert_eq!(node3.is_visible(),true);
        node1.add_child(&node2);
        node2.add_child(&node3);
        node1.update();
        assert_eq!(node3.is_visible(),true);
        node3.unset_parent();
        assert_eq!(node3.is_visible(),true);
        node1.update();
        assert_eq!(node3.is_visible(),false);
        node1.add_child(&node3);
        node1.update();
        assert_eq!(node3.is_visible(),true);
        node2.add_child(&node3);
        node1.update();
        assert_eq!(node3.is_visible(),true);
        node3.unset_parent();
        node1.update();
        assert_eq!(node3.is_visible(),false);
        node2.add_child(&node3);
        node1.update();
        assert_eq!(node3.is_visible(),true);
    }
}
