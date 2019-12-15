use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::closure;
use crate::dirty;
use crate::dirty::traits::*;
use crate::system::web::group;

use nalgebra::{Vector3, Vector4, Matrix4, Perspective3, Matrix, U4};
use basegl_system_web::Logger;
use crate::display::symbol::material::shader::glsl::PrimType::Mat2;
use failure::_core::fmt::{Formatter, Error};



// =================
// === AxisOrder ===
// =================

/// Defines the order in which particular axis coordinates are processed. Used
/// for example to define the rotation order in `DisplayObject`.
pub enum AxisOrder { XYZ, XZY, YXZ, YZX, ZXY, ZYX }

impl Default for AxisOrder {
    fn default() -> Self { Self::XYZ }
}



// =================
// === Transform ===
// =================

/// Defines the order in which transformations (scale, rotate, translate) are
/// applied to a particular object.
pub enum TransformOrder {
    ScaleRotateTranslate,
    ScaleTranslateRotate,
    RotateScaleTranslate,
    RotateTranslateScale,
    TranslateRotateScale,
    TranslateScaleRotate
}

impl Default for TransformOrder {
    fn default() -> Self { Self::ScaleRotateTranslate }
}



// =================
// === Transform ===
// =================

pub struct Transform {
    pub position        : Vector3<f32>,
    pub scale           : Vector3<f32>,
    pub rotation        : Vector3<f32>,
    pub transform_order : TransformOrder,
    pub rotation_order  : AxisOrder,
}

impl Default for Transform {
    fn default() -> Self {
        let position        = Vector3::new(0.0,0.0,0.0);
        let scale           = Vector3::new(1.0,1.0,1.0);
        let rotation        = Vector3::new(0.0,0.0,0.0);
        let transform_order = default();
        let rotation_order  = default();
        Self {position,scale,rotation,transform_order,rotation_order}
    }
}

impl Transform {
    /// Creates a new transformation object.
    pub fn new() -> Self { default() }

    /// Computes transformation matrix from the provided scale, rotation, and
    /// translation components, based on the transformation and rotation orders.
    pub fn matrix(&self) -> Matrix4<f32> {
        let mut matrix = Matrix4::identity();
        let matrix_ref = &mut matrix;
        match self.transform_order {
            TransformOrder::ScaleRotateTranslate => {
                self.append_scale       (matrix_ref);
                self.append_rotation    (matrix_ref);
                self.append_translation (matrix_ref);
            }
            TransformOrder::ScaleTranslateRotate => {
                self.append_scale       (matrix_ref);
                self.append_translation (matrix_ref);
                self.append_rotation    (matrix_ref);
            }
            TransformOrder::RotateScaleTranslate => {
                self.append_rotation    (matrix_ref);
                self.append_scale       (matrix_ref);
                self.append_translation (matrix_ref);
            }
            TransformOrder::RotateTranslateScale => {
                self.append_rotation    (matrix_ref);
                self.append_translation (matrix_ref);
                self.append_scale       (matrix_ref);
            }
            TransformOrder::TranslateRotateScale => {
                self.append_translation (matrix_ref);
                self.append_rotation    (matrix_ref);
                self.append_scale       (matrix_ref);
            }
            TransformOrder::TranslateScaleRotate => {
                self.append_translation (matrix_ref);
                self.append_scale       (matrix_ref);
                self.append_rotation    (matrix_ref);
            }
        }
        matrix
    }

    /// Computes a rotation matrix from the provided rotation values based on
    /// the rotation order.
    pub fn rotation_matrix(&self) -> Matrix4<f32> {
        let rx = Matrix4::from_scaled_axis(&Vector3::x() * self.rotation.x);
        let ry = Matrix4::from_scaled_axis(&Vector3::y() * self.rotation.y);
        let rz = Matrix4::from_scaled_axis(&Vector3::z() * self.rotation.z);
        match self.rotation_order {
            AxisOrder::XYZ => rz * ry * rx,
            AxisOrder::XZY => ry * rz * rx,
            AxisOrder::YXZ => rz * rx * ry,
            AxisOrder::YZX => rx * rz * ry,
            AxisOrder::ZXY => ry * rx * rz,
            AxisOrder::ZYX => rx * ry * rz,
        }
    }

    fn append_translation(&self, m:&mut Matrix4<f32>) {
        m.append_translation_mut(&self.position);
    }

    fn append_rotation(&self, m:&mut Matrix4<f32>) {
        *m = self.rotation_matrix() * (*m);
    }

    fn append_scale(&self, m:&mut Matrix4<f32>) {
        m.append_nonuniform_scaling_mut(&self.scale);
    }
}



// =======================
// === CachedTransform ===
// =======================

pub struct CachedTransform<OnChange> {
    transform        : Transform,
    transform_matrix : Matrix4<f32>,
    origin           : Matrix4<f32>,
    matrix           : Matrix4<f32>,
    pub dirty        : dirty::SharedBool<OnChange>,
    pub logger       : Logger,
}

impl<OnChange> CachedTransform<OnChange> {
    pub fn new(logger:Logger, on_change:OnChange) -> Self {
        let logger_dirty     = logger.sub("dirty");
        let transform        = default();
        let transform_matrix = Matrix4::identity();
        let origin           = Matrix4::identity();
        let matrix           = Matrix4::identity();
        let dirty            = dirty::SharedBool::new(logger_dirty,on_change);
        Self {transform,transform_matrix,origin,matrix,dirty,logger}
    }

    pub fn update(&mut self, new_origin:Option<&Matrix4<f32>>) -> bool {
        let is_dirty       = self.dirty.check_all();
        let origin_changed = new_origin.is_some();
        let changed        = is_dirty || origin_changed;
        if changed {
            group!(self.logger, "Update.", {
                if is_dirty {
                    self.transform_matrix = self.transform.matrix();
                    self.dirty.unset_all();
                }
                new_origin.iter().for_each(|t| self.origin = *t.clone());
                self.matrix = self.origin * self.transform_matrix;
            })
        }
        changed
    }
}


// === Getters ===

impl<OnChange> CachedTransform<OnChange> {
    pub fn position(&self) -> &Vector3<f32> {
        &self.transform.position
    }

    pub fn rotation(&self) -> &Vector3<f32> {
        &self.transform.rotation
    }

    pub fn scale(&self) -> &Vector3<f32> {
        &self.transform.scale
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        &self.matrix
    }

    pub fn global_position(&self) -> Vector3<f32> {
        (self.matrix * Vector4::new(0.0,0.0,0.0,1.0)).xyz()
    }
}


// === Setters ===

impl<OnChange:Callback0> CachedTransform<OnChange> {
    pub fn position_mut(&mut self) -> &mut Vector3<f32> {
        self.dirty.set();
        &mut self.transform.position
    }

    pub fn rotation_mut(&mut self) -> &mut Vector3<f32> {
        self.dirty.set();
        &mut self.transform.rotation
    }

    pub fn scale_mut(&mut self) -> &mut Vector3<f32> {
        self.dirty.set();
        &mut self.transform.scale
    }

    pub fn set_position(&mut self, t:Vector3<f32>) {
        *self.position_mut() = t;
    }

    pub fn set_rotation(&mut self, t:Vector3<f32>) {
        *self.rotation_mut() = t;
    }

    pub fn set_scale(&mut self, t:Vector3<f32>) {
        *self.scale_mut() = t;
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        f(self.position_mut())
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        f(self.rotation_mut())
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        f(self.scale_mut())
    }
}



// =================================
// === HierarchicalTransformData ===
// =================================

pub struct HierarchicalTransformData {
    pub parent_bind      : Option<ParentBind>,
    pub children         : OptVec<HierarchicalTransform>,
    pub transform        : CachedTransform<Option<OnChange>>,
    pub child_dirty      : ChildDirty,
    pub new_parent_dirty : NewParentDirty,
    pub on_changed       : Option<Box<dyn FnMut()>>,
    pub logger           : Logger,
}


// === Types ===

pub type ChildDirty     = dirty::SharedSet<usize,Option<OnChange>>;
pub type NewParentDirty = dirty::SharedBool<()>;
pub type TransformDirty = dirty::SharedBool<Option<OnChange>>;


// === Callbacks ===

closure! {
fn fn_on_change(dirty:ChildDirty, ix:usize) -> OnChange { || dirty.set(ix) }
}


// === API ===

impl HierarchicalTransformData {
    pub fn new(logger:Logger) -> Self {
        let parent_bind      = default();
        let children         = default();
        let transform        = CachedTransform::new(logger.sub("transform"), None);
        let child_dirty      = ChildDirty::new(logger.sub("child_dirty"),None);
        let new_parent_dirty = NewParentDirty::new(logger.sub("new_parent_dirty"),());
        let on_changed       = None;
        Self {parent_bind,children,transform,child_dirty,new_parent_dirty,on_changed,logger}
    }

    fn set_on_change_callback(&mut self, callback:Option<OnChange>) {
        self.transform.dirty.set_callback(callback.clone());
        self.child_dirty.set_callback(callback);
    }

    pub fn parent(&self) -> Option<&HierarchicalTransform> {
        self.parent_bind.as_ref().map(|ref t| &t.parent)
    }

    pub fn set_parent(&mut self, parent:Option<ParentBind>) {
        match parent {
            None => {
                self.logger.info("Removing parent bind.");
                self.set_on_change_callback(None);
            },
            Some(ref p) => {
                self.logger.info("Adding new parent bind.");
                let dirty     = p.parent.rc.borrow().child_dirty.clone_rc();
                let on_change = fn_on_change(dirty, p.index);
                self.set_on_change_callback(Some(on_change));
            }
        }
        self.new_parent_dirty.set();
        self.parent_bind = parent;
    }

    pub fn update(&mut self) {
        let origin0 = Matrix4::identity();
        self.update_with(&origin0,false)
    }

    fn remove_parent_bind(&mut self) -> Option<ParentBind> {
        self.parent_bind.take()
    }

    pub fn insert_child_raw(&mut self, child:&HierarchicalTransform) -> usize {
        let child_rc = child.clone();
        let index    = self.children.insert(child_rc);
        self.child_dirty.set(index);
        index
    }

    pub fn remove_child(&mut self, index:usize) {
        group!(self.logger, "Removing child at index {}.", index, {
            let opt_child = self.children.remove(index);
            opt_child.iter().for_each(|t| t.set_parent(None));
            self.child_dirty.unset(&index);
        })
    }

    pub fn update_with(&mut self, parent_origin:&Matrix4<f32>, force:bool) {
        let use_origin = force || self.new_parent_dirty.check();
        let new_origin = use_origin.as_some(parent_origin);
        let msg        = match new_origin {
            Some(_) => "Update with new parent origin.",
            None    => "Update with old parent origin."
        };
        group!(self.logger, msg, {
            let origin_changed = self.transform.update(new_origin);
            let origin         = &self.transform.matrix;
            if origin_changed {
                self.logger.info("Self origin changed.");
                if !self.children.is_empty() {
                    group!(self.logger, "Updating all children.", {
                        self.children.iter().for_each(|child| {
                            child.update_with(origin,true);
                        });
                    })
                }
            } else {
                self.logger.info("Self origin did not change.");
                if self.child_dirty.check_all() {
                    group!(self.logger, "Updating dirty children.", {
                        self.child_dirty.iter().for_each(|ix| {
                            self.children[*ix].update_with(origin,false)
                        });
                    })
                }
            }
            self.child_dirty.unset_all();
        });
        self.new_parent_dirty.unset();
        self.on_changed.iter_mut().for_each(|f| f());
    }


}


// === Getters ===

impl HierarchicalTransformData {
    pub fn global_position(&self) -> Vector3<f32> {
        self.transform.global_position()
    }

    pub fn position(&self) -> &Vector3<f32> {
        self.transform.position()
    }

    pub fn scale(&self) -> &Vector3<f32> {
        self.transform.scale()
    }

    pub fn rotation(&self) -> &Vector3<f32> {
        self.transform.rotation()
    }

    pub fn matrix(&self) -> &Matrix4<f32> {
        self.transform.matrix()
    }
}


// === Setters ===

impl HierarchicalTransformData {
    pub fn position_mut(&mut self) -> &mut Vector3<f32> {
        self.transform.position_mut()
    }

    pub fn scale_mut(&mut self) -> &mut Vector3<f32> {
        self.transform.scale_mut()
    }

    pub fn rotation_mut(&mut self) -> &mut Vector3<f32> {
        self.transform.rotation_mut()
    }

    pub fn set_position(&mut self, t:Vector3<f32>) {
        self.transform.set_position(t);
    }

    pub fn set_scale(&mut self, t:Vector3<f32>) {
        self.transform.set_scale(t);
    }

    pub fn set_rotation(&mut self, t:Vector3<f32>) {
        self.transform.set_rotation(t);
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        self.transform.mod_position(f)
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        self.transform.mod_rotation(f)
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&mut self, f:F) {
        self.transform.mod_scale(f)
    }

    pub fn set_on_changed<F:FnMut()+'static>(&mut self, f:F) {
        self.on_changed = Some(Box::new(f))
    }
}



// =============================
// === HierarchicalTransform ===
// =============================

/// A hierarchical representation of object containing a position, a scale and a rotation.
///
/// # Safety
/// Please note that you will get runtime crash when running the `update` function if your object
/// hierarchy forms a loop, for example, `obj2` is child of `obj1`, while `obj1` is child of `obj2`.
/// It is not easy to discover such situations, but maybe it will be worth to add some additional
/// safety on top of that in the future.
#[derive(Clone,Shrinkwrap)]
pub struct HierarchicalTransform {
    rc: Rc<RefCell<HierarchicalTransformData>>,
}

impl HierarchicalTransform {
    /// Creates a new object instance.
    pub fn new(logger:Logger) -> Self {
        let data = HierarchicalTransformData::new(logger);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }

    /// Set callback which will be fired after the object gets dirty for the first time.
    pub fn set_on_change_callback(&self, callback:Option<OnChange>) {
        self.borrow_mut().set_on_change_callback(callback);
    }

    /// Recompute the transformation matrix of this object and update all of its dirty children.
    pub fn update(&self) {
        self.borrow_mut().update();
    }

    /// Updates object transformations by providing a new origin location. See docs of `update` to
    /// learn more.
    pub fn update_with(&self, new_origin:&Matrix4<f32>, force:bool) {
        self.borrow_mut().update_with(new_origin,force);
    }

    /// Gets a reference to a parent object, if exists.
    pub fn parent(&self) -> Option<HierarchicalTransform> {
        self.borrow().parent().map(|t| t.clone_rc())
    }

    /// Gets index at which the object was registered in its parent object.
    pub fn index(&self) -> Option<usize> {
        self.parent_bind().map(|t| t.index)
    }

    /// Gets a reference to a parent bind description, if exists.
    pub fn parent_bind(&self) -> Option<ParentBind> {
        self.borrow().parent_bind.clone()
    }

    /// Set parent of the object. If the object already has a parent, the parent would be replaced.
    pub fn set_parent(&self, parent:Option<ParentBind>) {
        self.borrow_mut().set_parent(parent);
    }

    /// Adds a new `DisplayObject` as a child to the current one.
    pub fn add_child(&self, child:&HierarchicalTransform) {
        group!(self.borrow().logger, "Adding child.", {
            let child_bind = child.remove_parent_bind();
            child_bind.iter().for_each(|t| t.dispose());
            let index = self.borrow_mut().insert_child_raw(child);
            self.borrow().logger.info(|| format!("Child index is {}.", index));
            let parent      = self.clone();
            let parent_bind = ParentBind {parent,index};
            child.set_parent(Some(parent_bind));
        })
    }

    /// Removes the provided object reference from child list of this object. Does nothing if the
    /// reference was not a child of this object.
    pub fn remove_child(&self, child:&HierarchicalTransform) {
        child.parent_bind().iter().for_each(|bind| {
            if self == &bind.parent { self.remove_child_by_index(bind.index) }
        })
    }

    /// Removes child by a given index. Does nothing if the index was incorrect. Please use the
    /// `remove_child` method unless you are 100% sure that the index is correct.
    pub fn remove_child_by_index(&self, index:usize) {
        let opt_child = self.borrow_mut().remove_child(index);
    }


    // === Private API ===

    fn remove_parent_bind(&self) -> Option<ParentBind> {
        self.borrow_mut().remove_parent_bind()
    }
}


// === Getters ===

impl HierarchicalTransform {
    pub fn global_position(&self) -> Vector3<f32> {
        self.borrow().global_position()
    }

    pub fn position(&self) -> Vector3<f32> {
        self.borrow().position().clone()
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.borrow().scale().clone()
    }

    pub fn rotation(&self) -> Vector3<f32> {
        self.borrow().rotation().clone()
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.borrow().matrix().clone()
    }
}


// === Setters ===

impl HierarchicalTransform {
    pub fn set_position(&self, t:Vector3<f32>) {
        self.borrow_mut().set_position(t);
    }

    pub fn set_scale(&self, t:Vector3<f32>) {
        self.borrow_mut().set_scale(t);
    }

    pub fn set_rotation(&self, t:Vector3<f32>) {
        self.borrow_mut().set_rotation(t);
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.borrow_mut().mod_position(f)
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.borrow_mut().mod_rotation(f)
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.borrow_mut().mod_scale(f)
    }

    pub fn set_on_changed<F:FnMut()+'static>(&self, f:F) {
        self.borrow_mut().set_on_changed(f)
    }
}


// === Instances ===

impl From<Rc<RefCell<HierarchicalTransformData>>> for HierarchicalTransform {
    fn from(rc: Rc<RefCell<HierarchicalTransformData>>) -> Self {
        Self {rc}
    }
}

impl PartialEq for HierarchicalTransform {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.rc,&other.rc)
    }
}


// === ParentBind ===

#[derive(Clone)]
pub struct ParentBind {
    pub parent : HierarchicalTransform,
    pub index  : usize
}

impl ParentBind {
    pub fn dispose(&self) {
        self.parent.remove_child_by_index(self.index);
    }
}



use std::f32::consts::{PI};
use crate::dirty::{SharedDirtyFlag, SetData};


// ==========================================================


// === CloneRef ===

pub trait CloneRef {
    fn clone_ref(&self) -> Self;
}





// === ParentBind ===

#[derive(Clone)]
pub struct ParentBind2<T> {
    pub parent : T,
    pub index  : usize
}

impl<T: IsHierarchy> ParentBind2<T> {
    pub fn dispose(&self) {
        self.parent.remove_child_by_index(self.index);
    }
}

// === HasLogger ===

pub trait HasLogger {
    fn logger(&self) -> &Logger;
}


// ==============
// === Shared ===
// ==============

#[derive(Derivative)]
#[derivative(Clone(bound=""))]
pub struct Shared<T> {
    rc: Rc<RefCell<T>>
}

impl<T> CloneRef for Shared<T> {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

impl<T> Shared<T> {
    pub fn new(data:T) -> Self {
        Self {rc: Rc::new(RefCell::new(data))}
    }
}


// ============
// === Item ===
// ============

// === Type Family ===

pub trait HasItem { type Item; }
pub type Item<T> = <T as HasItem>::Item;


// === Shared ===

impl<T:HasItem> HasItem for Shared<T> {
    type Item = Item<T>;
}



// =================
// === Hierarchy ===
// =================

// === Definition ===

pub struct Hierarchy<T> {
    pub parent_bind : Option<ParentBind2<T>>,
    pub children    : OptVec<T>,
    pub logger      : Logger,
}

impl<T> HasItem for Hierarchy<T> {
    type Item = T;
}


// === Interface ===

pub trait IsHierarchyMut: HasItem + HasLogger {
    fn set_parent            (&mut self, parent:ParentBind2<Self::Item>);
    fn remove_parent         (&mut self);
    fn add_child_raw         (&mut self, child:&Self::Item) -> usize;
    fn remove_child_by_index (&mut self, index:usize);
    fn index                 (&self) -> Option<usize>;
    fn children              (&self) -> &OptVec<Self::Item>;
}

pub trait IsHierarchy
where Self: CloneRef + HasItem {
    fn set_parent            (&self, parent:ParentBind2<Self::Item>);
    fn remove_parent         (&self);
    fn add_child             (&self, child:&Self::Item) -> usize;
    fn remove_child_by_index (&self, index:usize);
    fn index                 (&self) -> Option<usize>;
}

trait IsHierarchyHelperMut: IsHierarchyMut {
    fn remove_parent_bind (&mut self) -> Option<ParentBind2<Self::Item>>;
}

trait IsHierarchyHelper: IsHierarchy {
    fn remove_parent_bind (&self) -> Option<ParentBind2<Self::Item>>;
}


// === Instances ===

impl<T> Hierarchy<T> {
    pub fn new(logger:Logger) -> Self {
        let parent_bind = default();
        let children    = default();
        Self {parent_bind,children,logger}
    }

    pub fn parent(&self) -> Option<&T> {
        self.parent_bind.as_ref().map(|ref t| &t.parent)
    }
}

impl<T> HasLogger for Hierarchy<T> {
    fn logger(&self) -> &Logger {
        &self.logger
    }
}

impl<T:IsHierarchy> IsHierarchyMut for Hierarchy<T> {
    fn set_parent(&mut self, parent:ParentBind2<Self::Item>) {
        group!(self.logger, "Setting new parent bind.", {
            self.remove_parent();
            self.parent_bind = Some(parent);
        })
    }

    fn remove_parent(&mut self) {
        self.logger.info("Removing parent bind.");
        self.parent_bind.iter().for_each(|p| p.dispose());
        self.parent_bind = None;
    }

    fn add_child_raw(&mut self, child:&Self::Item) -> usize {
        let child_ref = child.clone_ref();
        let index     = self.children.insert(child_ref);
        index
    }

    fn remove_child_by_index(&mut self, index:usize) {
        group!(self.logger, "Removing child at index {}.", index, {
            let opt_child = self.children.remove(index);
            opt_child.map(|t| t.remove_parent());
        })
    }

    fn index(&self) -> Option<usize> {
        self.parent_bind.as_ref().map(|t| t.index)
    }

    fn children(&self) -> &OptVec<Self::Item> {
        &self.children
    }
}

impl<T:IsHierarchy> IsHierarchyHelperMut for Hierarchy<T> {
    fn remove_parent_bind(&mut self) -> Option<ParentBind2<Self::Item>> {
        self.parent_bind.take()
    }
}


// === Shared ===

trait HierarchyCtx = where
    Self: HasItem + IsHierarchyHelperMut + Sized,
    Item<Self>: HasItem<Item=Item<Self>>,
    Item<Self>: IsHierarchyHelper + From<Shared<Self>>;

impl<T:HierarchyCtx> IsHierarchy for Shared<T> {
    fn add_child(&self, child: &Item<Self>) -> usize {
        let child_bind = child.remove_parent_bind();
        child_bind.map(|t| t.dispose());
        let index = self.rc.borrow_mut().add_child_raw(child);
//        self.borrow().logger.info(|| format!("Child index is {}.", index));
        let parent      = self.clone_ref().into();
        let bind   = ParentBind2 {parent,index};
        child.set_parent(bind);
        index
    }

    fn set_parent(&self, parent:ParentBind2<Item<Self>>) {
        self.rc.borrow_mut().set_parent(parent)
    }

    fn remove_parent(&self) {
        self.rc.borrow_mut().remove_parent()
    }

    fn remove_child_by_index(&self, index:usize) {
        self.rc.borrow_mut().remove_child_by_index(index)
    }

    fn index(&self) -> Option<usize> {
        self.rc.borrow().index()
    }
}

impl<T:HierarchyCtx> IsHierarchyHelper for Shared<T> {
    fn remove_parent_bind(&self) -> Option<ParentBind2<Self::Item>> {
        self.rc.borrow_mut().remove_parent_bind()
    }
}



// =========================
// === WithLazyTransform ===
// =========================

// === Definition ===

pub struct WithLazyTransform<T> {
    pub wrapped          : T,
    pub transform        : CachedTransform<Option<OnChange>>,
    pub child_dirty      : ChildDirty,
    pub new_parent_dirty : NewParentDirty,
}

impl<T:HasItem> HasItem for WithLazyTransform<T> {
    type Item = Item<T>;
}


// === Interface ===

pub trait HasLazyTransformMut {
    fn child_dirty(&self) -> ChildDirty;
    fn update     (&mut self);
    fn update_with(&mut self, parent_origin:&Matrix4<f32>, force:bool);
}

pub trait HasLazyTransform {
    fn child_dirty(&self) -> ChildDirty;
    fn update     (&self);
    fn update_with(&self, parent_origin:&Matrix4<f32>, force:bool);
}


// === Instances ===

impl<T> WithLazyTransform<T>
where T: HasLogger {
    pub fn new(wrapped:T) -> Self {
        let logger           = wrapped.logger();
        let transform        = CachedTransform :: new(logger.sub("transform")       ,None);
        let child_dirty      = ChildDirty      :: new(logger.sub("child_dirty")     ,None);
        let new_parent_dirty = NewParentDirty  :: new(logger.sub("new_parent_dirty"),());
        Self {wrapped,transform,child_dirty,new_parent_dirty}
    }
}

impl<T:HasLogger> HasLogger for WithLazyTransform<T> {
    fn logger(&self) -> &Logger {
        self.wrapped.logger()
    }
}

impl<T> IsHierarchyMut for WithLazyTransform<T>
where T: IsHierarchyHelperMut, Item<T>:HasLazyTransform {
    fn set_parent(&mut self, bind:ParentBind2<Self::Item>) {
        let dirty     = bind.parent.child_dirty();
        let on_change = fn_on_change(dirty, bind.index);
        self.transform.dirty.set_callback(Some(on_change.clone()));
        self.child_dirty.set_callback(Some(on_change));
        self.new_parent_dirty.set();
    }

    fn remove_parent(&mut self) {
        self.wrapped.remove_parent();
        self.transform.dirty.set_callback(None);
        self.child_dirty.set_callback(None);
        self.new_parent_dirty.set();
    }

    fn add_child_raw(&mut self, child:&Self::Item) -> usize {
        let index = self.wrapped.add_child_raw(child);
        self.child_dirty.set(index);
        index
    }

    fn remove_child_by_index(&mut self, index:usize) {
        self.child_dirty.unset(&index);
        self.wrapped.remove_child_by_index(index)
    }

    fn index(&self) -> Option<usize> {
        self.wrapped.index()
    }

    fn children(&self) -> &OptVec<Self::Item> {
        self.wrapped.children()
    }
}

impl<T> IsHierarchyHelperMut for WithLazyTransform<T>
where T: IsHierarchyHelperMut, Item<T>:HasLazyTransform {
    fn remove_parent_bind(&mut self) -> Option<ParentBind2<Self::Item>> {
        self.wrapped.remove_parent_bind()
    }
}

impl<T> HasLazyTransformMut for WithLazyTransform<T>
where T: IsHierarchyHelperMut, Item<T>:HasLazyTransform {
    fn child_dirty(&self) -> ChildDirty {
        self.child_dirty.clone_rc()
    }

    fn update(&mut self) {
        let origin0 = Matrix4::identity();
        self.update_with(&origin0,false)
    }

    fn update_with(&mut self, parent_origin: &Matrix4<f32>, force: bool) {
        let use_origin = force || self.new_parent_dirty.check();
        let new_origin = use_origin.as_some(parent_origin);
        let msg        = match new_origin {
            Some(_) => "Update with new parent origin.",
            None    => "Update with old parent origin."
        };
        group!(self.logger(), msg, {
            let origin_changed = self.transform.update(new_origin);
            let origin         = &self.transform.matrix;
            if origin_changed {
                self.logger().info("Self origin changed.");
                if !self.children().is_empty() {
                    group!(self.logger(), "Updating all children.", {
                        self.children().iter().for_each(|child| {
                            child.update_with(origin,true);
                        });
                    })
                }
            } else {
                self.logger().info("Self origin did not change.");
                if self.child_dirty.check_all() {
                    group!(self.logger(), "Updating dirty children.", {
                        self.child_dirty.iter().for_each(|ix| {
                            self.children()[*ix].update_with(origin,false)
                        });
                    })
                }
            }
            self.child_dirty.unset_all();
        });
        self.new_parent_dirty.unset();
    }
}


// === Shared ===

impl<T:HasLazyTransformMut> HasLazyTransform for Shared<T> {
    fn child_dirty(&self) -> ChildDirty {
        self.rc.borrow().child_dirty()
    }

    fn update_with(&self, parent_origin:&Matrix4<f32>, force:bool) {
        self.rc.borrow_mut().update_with(parent_origin,force);
    }

    fn update(&self) {
        self.rc.borrow_mut().update()
    }
}




// ==========================
// === HierarchicalObject ===
// ==========================

#[derive(Clone,Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HierarchicalObject {
    pub data: Shared<WithLazyTransform<Hierarchy<HierarchicalObject>>>
}

impl HasItem for HierarchicalObject {
    type Item = Self;
}

impl CloneRef for HierarchicalObject {
    fn clone_ref(&self) -> Self {
        self.clone()
    }
}

impl HierarchicalObject {
    pub fn new(logger:Logger) -> Self {
        let data = Shared::new(WithLazyTransform::new(Hierarchy::new(logger)));
        Self {data}
    }
}

impl From<Shared<WithLazyTransform<Hierarchy<HierarchicalObject>>>> for HierarchicalObject {
    fn from(data: Shared<WithLazyTransform<Hierarchy<HierarchicalObject>>>) -> Self {
        Self {data}
    }
}

impl IsHierarchy for HierarchicalObject {
    fn set_parent(&self, parent: ParentBind2<Self::Item>) {
        self.deref().set_parent(parent);
    }

    fn remove_parent(&self) {
        self.deref().remove_parent();
    }

    fn add_child(&self, child:&Self::Item) -> usize {
        self.deref().add_child(child)
    }

    fn remove_child_by_index(&self, index:usize) {
        self.deref().remove_child_by_index(index)
    }

    fn index(&self) -> Option<usize> {
        self.deref().index()
    }
}

impl IsHierarchyHelper for HierarchicalObject {
    fn remove_parent_bind(&self) -> Option<ParentBind2<Self::Item>> {
        self.deref().remove_parent_bind()
    }
}

impl HasLazyTransform for HierarchicalObject {
    fn child_dirty(&self) -> ChildDirty {
        self.deref().child_dirty()
    }

    fn update_with(&self, parent_origin:&Matrix4<f32>, force:bool) {
        self.deref().update_with(parent_origin,force)
    }

    fn update(&self) {
        self.deref().update()
    }
}


// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hierarchical_test() {
        let obj1 = HierarchicalObject::new(Logger::new("obj1"));
        let obj2 = HierarchicalObject::new(Logger::new("obj2"));
        let obj3 = HierarchicalObject::new(Logger::new("obj3"));

//
        obj1.add_child(&obj2);
        assert_eq!(obj2.index(), Some(0));
        obj1.add_child(&obj2);
        assert_eq!(obj2.index(), Some(10));
//        obj1.add_child(&obj3);
//        assert_eq!(obj3.index(), Some(1));
//        obj1.remove_child(&obj3);
//        assert_eq!(obj3.index(), None);
    }

    #[test]
    fn hierarchy_test() {
        let obj1 = HierarchicalTransform::new(Logger::new("obj1"));
        let obj2 = HierarchicalTransform::new(Logger::new("obj2"));
        let obj3 = HierarchicalTransform::new(Logger::new("obj3"));

        obj1.add_child(&obj2);
        assert_eq!(obj2.index(), Some(0));
        obj1.add_child(&obj2);
        assert_eq!(obj2.index(), Some(0));
        obj1.add_child(&obj3);
        assert_eq!(obj3.index(), Some(1));
        obj1.remove_child(&obj3);
        assert_eq!(obj3.index(), None);
    }

    #[test]
    fn transformation_test() {
        let obj1 = HierarchicalTransform::new(Logger::new("obj1"));
        let obj2 = HierarchicalTransform::new(Logger::new("obj2"));
        let obj3 = HierarchicalTransform::new(Logger::new("obj3"));

        assert_eq!(obj1.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj1.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(0.0,0.0,0.0));
        obj1.mod_position(|t| t.x += 7.0);
        obj1.add_child(&obj2);
        obj2.add_child(&obj3);
        assert_eq!(obj1.position()        , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj1.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(0.0,0.0,0.0));
        obj1.update();
        assert_eq!(obj1.position()        , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj3.position()        , Vector3::new(0.0,0.0,0.0));
        assert_eq!(obj1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(7.0,0.0,0.0));
        obj2.mod_position(|t| t.y += 5.0);
        obj1.update();
        assert_eq!(obj1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(7.0,5.0,0.0));
        obj3.mod_position(|t| t.x += 1.0);
        obj1.update();
        assert_eq!(obj1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(8.0,5.0,0.0));
        obj2.mod_rotation(|t| t.z += PI/2.0);
        obj1.update();
        assert_eq!(obj1.global_position() , Vector3::new(7.0,0.0,0.0));
        assert_eq!(obj2.global_position() , Vector3::new(7.0,5.0,0.0));
        assert_eq!(obj3.global_position() , Vector3::new(7.0,6.0,0.0));
        obj1.add_child(&obj3);
        obj1.update();
        assert_eq!(obj3.global_position() , Vector3::new(8.0,0.0,0.0));
        obj1.remove_child(&obj3);
        obj3.update();
        assert_eq!(obj3.global_position() , Vector3::new(1.0,0.0,0.0));
        obj2.add_child(&obj3);
        obj1.update();
        assert_eq!(obj3.global_position() , Vector3::new(7.0,6.0,0.0));
    }
}


// ==============
// === Camera ===
// ==============

pub enum Projection {
    Perspective  (Perspective),
    Orthographic (Orthographic)
}

pub struct Perspective  {
    pub aspect : f32,
    pub fov    : f32
}

pub struct Orthographic {
    pub left   : f32,
    pub right  : f32,
    pub top    : f32,
    pub bottom : f32
}

impl Default for Perspective {
    fn default() -> Self {
        let aspect = 1.0;
        let fov    = 45.0;
        Self {aspect,fov}
    }
}

impl Default for Orthographic {
    fn default() -> Self {
        let left   = -100.0;
        let right  =  100.0;
        let top    =  100.0;
        let bottom = -100.0;
        Self {left,right,top,bottom}
    }
}

impl Default for Projection {
    fn default() -> Self {
        Self::Perspective(default())
    }
}

pub struct Clipping {
    pub near : f32,
    pub far  : f32
}

impl Default for Clipping {
    fn default() -> Self {
        let near = 0.0;
        let far  = 1000.0;
        Self {near,far}
    }
}

#[derive(Shrinkwrap)]
pub struct Camera {
    #[shrinkwrap(main_field)]
    pub transform          : HierarchicalTransform,
    projection             : Projection,
    clipping               : Clipping,
    view_matrix            : Matrix4<f32>,
    projection_matrix      : Matrix4<f32>,
    view_projection_matrix : Matrix4<f32>,
    projection_dirty       : ProjectionDirty,
    transform_dirty        : TransformDirty2
}

type ProjectionDirty = dirty::SharedBool<()>;
type TransformDirty2 = dirty::SharedBool<()>;

impl Camera {
    pub fn new(logger: Logger) -> Self {
        let projection             = default();
        let clipping               = default();
        let view_matrix            = Matrix4::identity();
        let projection_matrix      = Matrix4::identity();
        let view_projection_matrix = Matrix4::identity();
        let projection_dirty       = ProjectionDirty::new(logger.sub("projection_dirty"),());
        let transform_dirty        = TransformDirty2::new(logger.sub("transform_dirty"),());
        let transform_dirty_copy   = transform_dirty.clone_rc();
        let transform              = HierarchicalTransform::new(logger);
        transform.set_on_changed(move || { transform_dirty_copy.set(); });
        transform.mod_position(|t| t.z = 1.0);
        Self {transform,projection,clipping,view_matrix,projection_matrix,view_projection_matrix,projection_dirty,transform_dirty}
    }

    pub fn recompute_view_matrix(&mut self) {
        self.view_matrix = self.transform.matrix().try_inverse().unwrap()
    }

    pub fn recompute_projection_matrix(&mut self) {
        self.projection_matrix = match &self.projection {
            Projection::Perspective(p) => {
                let fov_radians = p.fov * std::f32::consts::PI / 180.0;
                let near        = self.clipping.near;
                let far         = self.clipping.far;
                *Perspective3::new(p.aspect,fov_radians,near,far).as_matrix()
            }
            _ => unimplemented!()
        };
    }

    pub fn update(&mut self) {
        let mut changed = false;
        if self.transform_dirty.check() {
            self.recompute_view_matrix();
            self.transform_dirty.unset();
            changed = true;
        }
        if self.projection_dirty.check() {
            self.recompute_projection_matrix();
            self.projection_dirty.unset();
            changed = true;
        }
        if changed {
            self.view_projection_matrix = self.projection_matrix * self.view_matrix;
        }
    }
}

// === Getters ===

//impl Camera {
//    pub fn aspect     (&self) -> &f32          { &self.aspect     }
//    pub fn fov        (&self) -> &f32          { &self.fov        }
//    pub fn near       (&self) -> &f32          { &self.near       }
//    pub fn far        (&self) -> &f32          { &self.far        }
//    pub fn projection (&self) -> &Matrix4<f32> { &self.projection }
//    pub fn view       (&self) -> &Matrix4<f32> { &self.view       }
//}

// === Setters ===

impl Camera {
    pub fn projection_mut(&mut self) -> &mut Projection {
        self.projection_dirty.set();
        &mut self.projection
    }

    pub fn clipping_mut(&mut self) -> &mut Clipping {
        self.projection_dirty.set();
        &mut self.clipping
    }
}

//ar viewMatrix = m4.inverse(cameraMatrix);



//pub trait WidgetData {
//    type Value;
//
//    fn value     (&    self) -> &    Self::Value;
//    fn value_mut (&mut self) -> &mut Self::Value;
//
//    fn draw(&self);
//}
//
//struct Slider {
//
//}
//
//impl Slider {
//
//}
//
//
//struct SymbolRegistry {
//    pub vec: Vec<Symbol>
//}
//
//struct SymbolInstanceRegistry {
//    pub instances: Vec<SymbolInstance>
//}
//
//struct Symbol {
//    pub mesh   : Mesh,
//
//}
//
//struct SymbolInstance {
//    pub object   : DisplayObject,
//    pub position : Var<Vector3<f32>>,
//}
//
//
//pub fn main() {
//    let symbol_def = SymbolDef::new(EDSL...);
//    let symbol     = scene.register_symbol(symbol_def);
//
//    let s1 = symbol.new_instance();
//    let s2 = symbol.new_instance();
//
//
//    mouse().position().with(|p| s1.set_position(p));
//
//}