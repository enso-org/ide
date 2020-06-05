//! This module defines an abstraction for DOM shapes and provides utility for efficient shape
//! change tracking which does not cause a reflow.
//! Learn more: https://gist.github.com/paulirish/5d52fb081b3570c81e3a

use crate::control::callback;
use crate::prelude::*;
use crate::system::web::resize_observer::ResizeObserver;
use crate::system::web;
use crate::frp;
use nalgebra::Vector2;
use wasm_bindgen::prelude::Closure;



// =============
// === Shape ===
// =============

// === Shape ===

use shapely::shared;
use enso_frp::stream::ValueProvider;

//shared! { Shape
///// Contains information about DOM element shape and provides utils for querying the size both in
///// DOM pixel units as well as device pixel units.
//#[derive(Debug)]
//##[derive(Clone,Copy)]
//pub struct ShapeData {
//    width       : f32,
//    height      : f32,
//    pixel_ratio : f32
//}
//
//impl {
//    /// Constructor.
//    pub fn new() -> Self {
//        let width       = 0.0;
//        let height      = 0.0;
//        let pixel_ratio = web::device_pixel_ratio() as f32;
//        Self {width,height,pixel_ratio}
//    }
//
//    /// Getter.
//    pub fn width(&self) -> f32 {
//        self.width
//    }
//
//    /// Getter.
//    pub fn height(&self) -> f32 {
//        self.height
//    }
//
//    /// Getter.
//    pub fn pixel_ratio(&self) -> f32 {
//        self.pixel_ratio
//    }
//
//    /// Dimension setter in DOM pixel units. Use `device_pixels` to switch to device units.
//    pub fn set(&mut self, width:f32, height:f32) {
//        self.width  = width;
//        self.height = height;
//    }
//
//    /// Sets the size to the size of the provided element. This operation is slow as it causes
//    /// reflow.
//    pub fn set_from_element_with_reflow(&mut self, element:&web::HtmlElement) {
//        let bbox   = element.get_bounding_client_rect();
//        let width  = bbox.width()  as f32;
//        let height = bbox.height() as f32;
//        self.set(width,height);
//    }
//}}
//
//impl ShapeData {
//    /// Switched to device pixel units. On low-dpi screens device pixels map 1:1 with DOM pixels.
//    /// On high-dpi screens, a single device pixel is often mapped to 2 or 3 DOM pixels.
//    pub fn device_pixels(&self) -> Self {
//        let width  = self.width  * self.pixel_ratio;
//        let height = self.height * self.pixel_ratio;
//        Self {width,height,..*self}
//    }
//}
//
//impl Shape {
//    /// Constructor.
//    pub fn from_element_with_reflow(element:&web::HtmlElement) -> Self {
//        let this = Self::default();
//        this.set_from_element_with_reflow(element);
//        this
//    }
//
//    /// Current value of the shape.
//    pub fn current(&self) -> ShapeData {
//        *self.rc.borrow()
//    }
//}
//
//impl Default for Shape {
//    fn default() -> Self {
//        Self::new()
//    }
//}
//
//impl Default for ShapeData {
//    fn default() -> Self {
//        Self::new()
//    }
//}
//




#[derive(Clone,Copy,Debug)]
pub struct Shape {
    pub width       : f32,
    pub height      : f32,
    pub pixel_ratio : f32
}

impl Shape {

    pub fn new(width:f32, height:f32) -> Self {
        Self {width,height,..default()}
    }

    pub fn new_from_element_with_reflow(element:&web::HtmlElement) -> Self {
        let mut shape = Self::default();
        shape.set_from_element_with_reflow(element);
        shape
    }

    pub fn set_from_element_with_reflow(&mut self, element:&web::HtmlElement) {
        let bbox    = element.get_bounding_client_rect();
        self.width  = bbox.width()  as f32;
        self.height = bbox.height() as f32;
    }

    /// Switched to device pixel units. On low-dpi screens device pixels map 1:1 with DOM pixels.
    /// On high-dpi screens, a single device pixel is often mapped to 2 or 3 DOM pixels.
    pub fn device_pixels(&self) -> Self {
        let width  = self.width  * self.pixel_ratio;
        let height = self.height * self.pixel_ratio;
        Self {width,height,..*self}
    }
}

impl Default for Shape {
    fn default() -> Self {
        let width       = 100.0;
        let height      = 100.0;
        let pixel_ratio = web::device_pixel_ratio() as f32;
        Self {width,height,pixel_ratio}
    }
}

impl Into<Vector2<f32>> for Shape {
    fn into(self) -> Vector2<f32> {
        Vector2::new(self.width,self.height)
    }
}

impl Into<V2> for Shape {
    fn into(self) -> V2<f32> {
        V2(self.width,self.height)
    }
}

impl Into<V2> for &Shape {
    fn into(self) -> V2<f32> {
        V2(self.width,self.height)
    }
}



// ======================
// === WithKnownShape ===
// ======================

/// A wrapper for `HtmlElement` or anything which derefs to it. It tracks the element size without
/// causing browser reflow.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[clone_ref(bound="T:CloneRef")]
pub struct WithKnownShape<T=web_sys::HtmlElement> {
    #[shrinkwrap(main_field)]
    dom          : T,
    network      : frp::Network,
    pub shape    : frp::Sampler<Shape>,
    shape_source : frp::Source<Shape>,
    observer     : Rc<ResizeObserver>,
}

impl<T> WithKnownShape<T> {
    /// Constructor.
    pub fn new(dom:&T) -> Self
    where T : Clone + AsRef<web::JsValue> + Into<web_sys::HtmlElement> {
        let dom     = dom.clone();
        let element = dom.clone().into();
        frp::new_network! { network
            shape_source <- source();
            shape        <- shape_source.sampler();
        };
        let callback = Closure::new(f!((w,h) shape_source.emit(Shape::new(w,h))));
        let observer = Rc::new(ResizeObserver::new(dom.as_ref(),callback));
        shape_source.emit(Shape::new_from_element_with_reflow(&element));
        Self {dom,network,shape,shape_source,observer}
    }

    /// Get the current shape of the object.
    pub fn shape(&self) -> Shape {
        self.shape.value()
    }

    pub fn recompute_shape_with_reflow(&self) where T : Clone + Into<web_sys::HtmlElement> {
        self.shape_source.emit(Shape::new_from_element_with_reflow(&self.dom.clone().into()))
    }
}

impl From<WithKnownShape<web::HtmlDivElement>> for WithKnownShape<web::EventTarget> {
    fn from(t:WithKnownShape<web::HtmlDivElement>) -> Self {
        let dom          = t.dom.into();
        let network      = t.network;
        let shape        = t.shape;
        let shape_source = t.shape_source;
        let observer     = t.observer;
        Self {dom,network,shape,shape_source,observer}
    }
}

impl From<WithKnownShape<web::HtmlElement>> for WithKnownShape<web::EventTarget> {
    fn from(t:WithKnownShape<web::HtmlElement>) -> Self {
        let dom          = t.dom.into();
        let network      = t.network;
        let shape        = t.shape;
        let shape_source = t.shape_source;
        let observer     = t.observer;
        Self {dom,network,shape,shape_source,observer}
    }
}
