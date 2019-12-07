use crate::prelude::*;

use crate::data::function::callback::*;
use crate::data::opt_vec::OptVec;
use crate::closure;
use crate::dirty;
use crate::dirty::traits::*;
use crate::system::web::group;

use nalgebra::{Vector3, Vector4, Matrix4};
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


// =============================
// === HierarchicalTransform ===
// =============================

pub struct HierarchicalTransform<OnChange> {
    pub transform        : Transform,
    transform_matrix : Matrix4<f32>,
    origin           : Matrix4<f32>,
    matrix           : Matrix4<f32>,
    pub dirty        : dirty::SharedBool<OnChange>,
    pub logger       : Logger,
}

impl<OnChange> HierarchicalTransform<OnChange> {
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
                    println!(">>");
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

impl<OnChange> HierarchicalTransform<OnChange> {
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

impl<OnChange:Callback0> HierarchicalTransform<OnChange> {
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


// =====================
// === DisplayObject ===
// =====================

pub struct ParentBind {
    pub parent : DisplayObject,
    pub index  : usize
}

impl ParentBind {
    pub fn dispose(&self) {
        self.parent.remove_child(self.index);
    }
}

#[derive(Clone)]
pub struct DisplayObject {
    rc: Rc<RefCell<DisplayObjectData>>,
}

impl DisplayObject {
    pub fn new(logger:Logger) -> Self {
        let data = DisplayObjectData::new(logger);
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }

    pub fn set_on_change_callback(&self, callback:Option<OnChange>) {
        self.rc.borrow_mut().set_on_change_callback(callback);
    }

    pub fn update_with(&self, new_origin:Option<&Matrix4<f32>>) {
        self.rc.borrow_mut().update_with(new_origin);
    }

    pub fn update(&self) {
        self.rc.borrow_mut().update();
    }

    pub fn set_parent(&self, parent:Option<ParentBind>) {
        self.rc.borrow_mut().set_parent(parent);
    }

    pub fn remove_parent_bind(&self) -> Option<ParentBind> {
        self.rc.borrow_mut().remove_parent_bind()
    }

    pub fn add_child(&self, child:&DisplayObject) {
        let index = self.rc.borrow_mut().insert_child_raw(child);
        group!(self.rc.borrow().logger, "Adding child at index {}.", index, {
            let parent      = self.clone();
            let parent_bind = ParentBind {parent,index};
            let child_bind  = child.remove_parent_bind();
            child_bind.iter().for_each(|t| t.dispose());
            child.set_parent(Some(parent_bind));
        })
    }

    pub fn remove_child(&self, index:usize) {
        let opt_child = self.rc.borrow_mut().remove_child(index);
    }
}

// === Getters ===

impl DisplayObject {
    pub fn global_position(&self) -> Vector3<f32> {
        self.rc.borrow().global_position()
    }

    pub fn position(&self) -> Vector3<f32> {
        self.rc.borrow().position().clone()
    }

    pub fn scale(&self) -> Vector3<f32> {
        self.rc.borrow().scale().clone()
    }

    pub fn rotation(&self) -> Vector3<f32> {
        self.rc.borrow().rotation().clone()
    }

    pub fn matrix(&self) -> Matrix4<f32> {
        self.rc.borrow().matrix().clone()
    }
}

pub struct RefGuard<'t,Base,Data> {
    data   : &'t Data,
    borrow : Ref<'t,Base>,
}

impl<'t,Base,Data> Deref for RefGuard<'t,Base,Data> {
    type Target = Data;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

impl<'t,Base,Data> RefGuard<'t,Base,Data> {
    pub fn new<F:FnOnce(&'t Base) -> &'t Data>(base:&'t RefCell<Base>, f:F) -> Self {
        let borrow    = base.borrow();
        let reference = unsafe { drop_lifetime(&borrow) };
        let data      = f(reference);
        RefGuard {data,borrow}
    }
}

impl<'t,Base,Data:Debug> Debug for RefGuard<'t,Base,Data> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.data.fmt(f)
    }
}

// === Setters ===

impl DisplayObject {
    pub fn set_position(&self, t:Vector3<f32>) {
        self.rc.borrow_mut().set_position(t);
    }

    pub fn set_scale(&self, t:Vector3<f32>) {
        self.rc.borrow_mut().set_scale(t);
    }

    pub fn set_rotation(&self, t:Vector3<f32>) {
        self.rc.borrow_mut().set_rotation(t);
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.rc.borrow_mut().mod_position(f)
    }

    pub fn mod_rotation<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.rc.borrow_mut().mod_rotation(f)
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.rc.borrow_mut().mod_scale(f)
    }
}

// === Instances ===

impl From<Rc<RefCell<DisplayObjectData>>> for DisplayObject {
    fn from(rc: Rc<RefCell<DisplayObjectData>>) -> Self {
        Self {rc}
    }
}


// =========================
// === DisplayObjectData ===
// =========================

pub struct DisplayObjectData {
    pub parent_bind     : Option<ParentBind>,
    pub children        : OptVec<DisplayObject>,
    pub transform       : HierarchicalTransform<Option<OnChange>>,
    pub child_dirty     : ChildDirty,
    pub new_child_dirty : NewChildDirty,
    pub logger          : Logger,
}

// === Types ===

pub type ChildDirty     = dirty::SharedSet<usize,Option<OnChange>>;
pub type NewChildDirty  = dirty::SharedSet<usize,Option<OnChange>>;
pub type TransformDirty = dirty::SharedBool<Option<OnChange>>;

// === Callbacks ===

closure! {
fn fn_on_change(dirty:ChildDirty, ix:usize) -> OnChange { || dirty.set(ix) }
}

// === API ===

impl DisplayObjectData {
    pub fn new(logger:Logger) -> Self {
        let parent_bind     = default();
        let children        = default();
        let transform       = HierarchicalTransform::new(logger.sub("transform"),None);
        let child_dirty     = ChildDirty::new(logger.sub("child_dirty"),None);
        let new_child_dirty = NewChildDirty::new(logger.sub("child_dirty"),None);
        Self {parent_bind,children,transform,child_dirty,new_child_dirty,logger}
    }

    fn set_on_change_callback(&mut self, callback:Option<OnChange>) {
        self.transform.dirty.set_callback(callback.clone());
        self.child_dirty.set_callback(callback);
    }

    pub fn set_parent(&mut self, parent:Option<ParentBind>) {
        println!("set parent borrowed");

//        self.parent_bind.iter().for_each(|t| t.dispose());

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
        self.parent_bind = parent;
    }

    pub fn update(&mut self) {
        self.update_with(None)
    }

    pub fn remove_parent_bind(&mut self) -> Option<ParentBind> {
        self.parent_bind.take()
    }

    pub fn insert_child_raw(&mut self, child:&DisplayObject) -> usize {
        let child_rc = child.clone();
        let index    = self.children.insert(child_rc);
        self.new_child_dirty.set(index);
        index
    }

    pub fn remove_child(&mut self, index:usize) {
        group!(self.logger, "Removing child at index {}.", index, {
            let opt_child = self.children.remove(index);
            opt_child.iter().for_each(|t| t.set_parent(None));
            self.child_dirty.unset(index);
            self.new_child_dirty.unset(index);
        })
    }

    pub fn update_with(&mut self, new_origin:Option<&Matrix4<f32>>) {
        let msg = match new_origin {
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
                            child.update_with(Some(origin));
                        });
                    })
                }
            } else {
                self.logger.info("Self origin did not change.");
                if self.child_dirty.check_all() {
                    group!(self.logger, "Updating dirty children.", {
                        self.child_dirty.iter().for_each(|ix| {
                            self.children[*ix].update_with(None)
                        });
                    })
                }
                if self.new_child_dirty.check_all() {
                    group!(self.logger, "Updating new children", {
                        self.new_child_dirty.iter().for_each(|ix| {
                            self.children[*ix].update_with(Some(origin))
                        });
                    })
                }
            }
            self.child_dirty.unset_all();
            self.new_child_dirty.unset_all();
        })
    }


}

// === Getters ===

impl DisplayObjectData {
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

impl DisplayObjectData {
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
}

use std::f32::consts::{PI};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let obj1 = DisplayObject::new(Logger::new("obj1"));
        let obj2 = DisplayObject::new(Logger::new("obj2"));
        let obj3 = DisplayObject::new(Logger::new("obj3"));

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


        // remove child
    }
}
