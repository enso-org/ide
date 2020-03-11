//! This module contains the implementation of Css3dObject, a struct used to represent CSS3D
//! elements.

use crate::prelude::*;

use crate::display;
use crate::system::web;
use crate::system::web::StyleSetter;
use crate::system::web::NodeInserter;

use nalgebra::Vector2;
use nalgebra::Vector3;
use web_sys::HtmlDivElement;

use super::css3d_renderer::set_object_transform;



// =============================
// === Css3dObjectProperties ===
// =============================

#[derive(Debug)]
struct Css3dObjectProperties {
    display_object : display::object::Node,
    dom            : HtmlDivElement,
    dimensions     : Vector2<f32>,
}

impl Drop for Css3dObjectProperties {
    fn drop(&mut self) {
        self.dom.remove();
        self.display_object.unset_parent();
    }
}



// =======================
// === Css3dObjectData ===
// =======================

#[derive(Clone,Debug)]
pub(super) struct Css3dObjectData {
    properties : Rc<RefCell<Css3dObjectProperties>>
}

impl Css3dObjectData {
    fn new
    ( display_object : display::object::Node
    , dom            : HtmlDivElement
    , dimensions     : Vector2<f32>
    ) -> Self {
        let properties = Css3dObjectProperties {display_object,dom,dimensions};
        let properties = Rc::new(RefCell::new(properties));
        Self {properties}
    }

    fn position(&self) -> Vector3<f32> {
        self.properties.borrow().display_object.position()
    }

    fn set_dimensions(&self, dimensions:Vector2<f32>) {
        let mut properties = self.properties.borrow_mut();
        properties.dimensions = dimensions;
        properties.display_object.with_logger(|logger| {
            properties.dom.set_style_or_warn("width",  format!("{}px", dimensions.x), logger);
            properties.dom.set_style_or_warn("height", format!("{}px", dimensions.y), logger);
        });
    }

    fn dimensions(&self) -> Vector2<f32> {
        self.properties.borrow().dimensions
    }

    fn dom(&self) -> HtmlDivElement {
        self.properties.borrow().dom.clone()
    }

    fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        let mut position = self.position();
        f(&mut position);
        self.properties.borrow().display_object.set_position(position);
    }

    fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.properties.borrow().display_object.mod_scale(f);
    }
}



// ===================
// === Css3dObject ===
// ===================

/// A structure for representing a HtmlElement in the 3d world.
#[derive(Debug,Clone)]
pub struct Css3dObject {
    pub(super) data : Css3dObjectData
}

impl Css3dObject {
    /// Creates a Css3dObject from element name.
    pub fn new(dom:&web_sys::Node) -> Self {
        let div    = web::create_div();
        let logger = Logger::new("DomObject");
        div.set_style_or_warn("position", "absolute", &logger);
        div.set_style_or_warn("width"   , "0px"     , &logger);
        div.set_style_or_warn("height"  , "0px"     , &logger);
        div.append_or_panic(dom);
        let display_object = display::object::Node::new(logger);
        let dimensions     = Vector2::new(0.0,0.0);
        display_object.set_on_updated(enclose!((dom) move |t| {
            let mut transform = t.matrix();
            transform.iter_mut().for_each(|a| *a = eps(*a));
            set_object_transform(&dom, &transform);
        }));
        let data = Css3dObjectData::new(display_object,div,dimensions);
        Self {data}
    }

    /// Sets the underlying HtmlElement dimension.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.data.set_dimensions(dimensions)
    }

    /// Gets the underlying HtmlElement dimension.
    pub fn dimensions(&self) -> Vector2<f32> {
        self.data.dimensions()
    }

    /// Gets Css3dObject's dom.
    pub fn dom(&self) -> HtmlDivElement {
        self.data.dom()
    }

    /// Gets object's position.
    pub fn position(&self) -> Vector3<f32> {
        self.data.position()
    }

    /// Modifies the position of the object.
    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.data.mod_position(f);
    }

    /// Modifies the scale of the object.
    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.data.mod_scale(f);
    }
}

impl From<&Css3dObject> for display::object::Node {
    fn from(t:&Css3dObject) -> Self {
        t.data.properties.borrow().display_object.clone_ref()
    }
}






// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}