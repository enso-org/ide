// This file contains the implementation of DOMContainer. A struct that aids us to handle html
// elements, get its position and dimension avoiding style reflow.
//
// It relies on Resize Observer and Intersection Observer, which notifies us when the element's
// rect and visibility rect is updated.

// FIXME: ResizeCallback can be completely substituted by IntersectionCallback

use crate::prelude::*;

use crate::system::web::get_element_by_id;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::StyleSetter;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web::intersection_observer::IntersectionObserver;

use wasm_bindgen::prelude::Closure;
use web_sys::HtmlElement;
use nalgebra::Vector2;
use std::cell::RefCell;
use std::rc::Rc;



// ============================
// === IntersectionCallback ===
// ============================

type IntersectionCallback        = Box<dyn Fn(&Vector2<f32>, &Vector2<f32>)>;
pub trait IntersectionCallbackFn = Fn(&Vector2<f32>, &Vector2<f32>) + 'static;



// ======================
// === ResizeCallback ===
// ======================

type ResizeCallback        = Box<dyn Fn(&Vector2<f32>)>;
pub trait ResizeCallbackFn = Fn(&Vector2<f32>) + 'static;



// ========================
// === DOMContainerData ===
// ========================

#[derive(Derivative)]
#[derivative(Debug)]
pub struct DOMContainerData {
    position   : Vector2<f32>,
    dimensions : Vector2<f32>,
    #[derivative(Debug="ignore")]
    resize_callbacks       : Vec<ResizeCallback>,
    #[derivative(Debug="ignore")]
    intersection_callbacks : Vec<IntersectionCallback>
}

impl DOMContainerData {
    pub fn new(position:Vector2<f32>, dimensions:Vector2<f32>) -> Self {
        let resize_callbacks       = default();
        let intersection_callbacks = default();
        Self { position, dimensions, resize_callbacks, intersection_callbacks }
    }
}



// ====================
// === DOMContainer ===
// ====================

/// A collection for holding 3D `Object`s.
#[derive(Debug)]
pub struct DOMContainer {
    pub dom               : HtmlElement,
    resize_observer       : Option<ResizeObserver>,
    intersection_observer : Option<IntersectionObserver>,
    data                  : Rc<RefCell<DOMContainerData>>,
}

impl Clone for DOMContainer {
    fn clone(&self) -> Self {
        DOMContainer::from_element(self.dom.clone())
    }
}

impl DOMContainer {
    pub fn from_element(dom:HtmlElement) -> Self {
        let rect                  = dom.get_bounding_client_rect();
        let x                     = rect.x()      as f32;
        let y                     = rect.y()      as f32;
        let width                 = rect.width()  as f32;
        let height                = rect.height() as f32;
        let dimensions            = Vector2::new(width, height);
        let position              = Vector2::new(x    , y);
        let data                  = DOMContainerData::new(position, dimensions);
        let data                  = Rc::new(RefCell::new(data));
        let resize_observer       = None;
        let intersection_observer = None;
        let mut ret = Self { dom,resize_observer,intersection_observer,data };

        ret.init_listeners();
        ret
    }
    pub fn from_id(dom_id:&str) -> Result<Self> {
        let dom : HtmlElement = dyn_into(get_element_by_id(dom_id)?)?;
        Ok(Self::from_element(dom))
    }

    fn init_listeners(&mut self) {
        self.init_resize_listener();
        self.init_intersection_listener();
    }

    fn init_intersection_listener(&mut self) {
        let data = self.data.clone();
        let closure = Closure::new(move |x, y, width, height| {
            let mut data = data.borrow_mut();
            data.position   = Vector2::new(x as f32, y as f32);
            data.dimensions = Vector2::new(width as f32, height as f32);
            for callback in &data.intersection_callbacks {
                callback(&data.position, &data.dimensions);
            }
        });
        let observer = IntersectionObserver::new(&self.dom, closure);
        self.intersection_observer = Some(observer);
    }

    fn init_resize_listener(&mut self) {
        let data = self.data.clone();
        let closure = Closure::new(move |width, height| {
            let mut data = data.borrow_mut();
            data.dimensions = Vector2::new(width as f32, height as f32);
            for callback in &data.resize_callbacks {
                callback(&data.dimensions);
            }
        });
        let observer = ResizeObserver::new(&self.dom, closure);
        self.resize_observer = Some(observer);
    }

    /// Sets the Scene DOM's dimensions.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.dom.set_property_or_panic("width" , format!("{}px", dimensions.x));
        self.dom.set_property_or_panic("height", format!("{}px", dimensions.y));
        self.data.borrow_mut().dimensions = dimensions;
    }

    /// Gets the Scene DOM's position.
    pub fn position(&self) -> Vector2<f32> {
        self.data.borrow().position
    }

    /// Gets the Scene DOM's dimensions.
    pub fn dimensions(&self) -> Vector2<f32> {
        self.data.borrow().dimensions
    }

    /// Adds a ResizeCallback.
    pub fn add_resize_callback<T>(&mut self, callback:T)
        where T : ResizeCallbackFn {
        self.data.borrow_mut().resize_callbacks.push(Box::new(callback));
    }
}