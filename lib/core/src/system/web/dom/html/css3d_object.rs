//! This module contains the implementation of DomSymbol, a struct used to represent CSS3D
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
// === DomSymbolData ===
// =============================

#[derive(Debug)]
struct DomSymbolData {
    display_object : display::object::Node,
    dom            : HtmlDivElement,
    size           : Vector2<f32>,
}

impl Drop for DomSymbolData {
    fn drop(&mut self) {
        self.dom.remove();
        self.display_object.unset_parent();
    }
}



// =======================
// === DomSymbolData ===
// =======================

#[derive(Clone,Debug)]
pub struct DomSymbol {
    rc : Rc<RefCell<DomSymbolData>>
}

impl DomSymbol {

    pub fn position(&self) -> Vector3<f32> {
        self.rc.borrow().display_object.position()
    }

    pub fn set_size(&self, size:Vector2<f32>) {
        let mut properties = self.rc.borrow_mut();
        properties.size = size;
        properties.display_object.with_logger(|logger| {
            properties.dom.set_style_or_warn("width",  format!("{}px", size.x), logger);
            properties.dom.set_style_or_warn("height", format!("{}px", size.y), logger);
        });
    }

    pub fn size(&self) -> Vector2<f32> {
        self.rc.borrow().size
    }

    pub fn dom(&self) -> HtmlDivElement {
        self.rc.borrow().dom.clone()
    }

    pub fn mod_position<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        let mut position = self.position();
        f(&mut position);
        self.rc.borrow().display_object.set_position(position);
    }

    pub fn mod_scale<F:FnOnce(&mut Vector3<f32>)>(&self, f:F) {
        self.rc.borrow().display_object.mod_scale(f);
    }
}



// ===================
// === DomSymbol ===
// ===================

impl DomSymbol {
    /// Creates a DomSymbol from element name.
    pub fn new(content:&web_sys::Node) -> Self {
        let dom    = web::create_div();
        let logger = Logger::new("DomSymbol");
        dom.set_style_or_warn("position", "absolute", &logger);
        dom.set_style_or_warn("width"   , "0px"     , &logger);
        dom.set_style_or_warn("height"  , "0px"     , &logger);
        dom.append_or_panic(content);
        let display_object = display::object::Node::new(logger);
        let size     = Vector2::new(0.0,0.0);
        display_object.set_on_updated(enclose!((dom) move |t| {
            let mut transform = t.matrix();
            transform.iter_mut().for_each(|a| *a = eps(*a));
            set_object_transform(&dom,&transform);
        }));

        let data = DomSymbolData {display_object,dom,size};
        let rc   = Rc::new(RefCell::new(data));
        Self {rc}
    }
}

impl From<&DomSymbol> for display::object::Node {
    fn from(obj:&DomSymbol) -> Self {
        obj.rc.borrow().display_object.clone_ref()
    }
}






// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}