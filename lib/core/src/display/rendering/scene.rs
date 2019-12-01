use wasm_bindgen::prelude::Closure;
use crate::prelude::*;
use crate::system::web::get_element_by_id;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::StyleSetter;
use crate::system::web::resize_observer::ResizeObserver;
use web_sys::HtmlElement;
use nalgebra::Vector2;
use std::cell::RefCell;
use std::rc::Rc;
use std::fmt;

// ======================
// === ResizeCallback ===
// ======================

pub type ResizeCallback = Box<dyn Fn(&Vector2<f32>)>;

// =================
// === SceneData ===
// =================

#[derive(Derivative)]
#[derivative(Debug)]
struct SceneData {
    dimensions : Vector2<f32>,
    #[derivative(Debug="ignore")]
    resize_callbacks : Vec<ResizeCallback>
}

impl SceneData {
    pub fn new(dimensions : Vector2<f32>) -> Self {
        let resize_callbacks = default();
        Self { dimensions, resize_callbacks }
    }
}

// =============
// === Scene ===
// =============

/// A collection for holding 3D `Object`s.
//#[derive(Debug)]
pub struct Scene {
    pub dom          : HtmlElement,
    _resize_observer : ResizeObserver,
    data             : Rc<RefCell<SceneData>>
}

impl fmt::Debug for Scene {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.dom)?;
        write!(f, "{:?}", self._resize_observer)?;
        write!(f, "{:?}", self.data.borrow())
    }
}

impl Scene {
    /// Searches for a HtmlElement identified by id and appends to it.
    pub fn new(dom_id: &str) -> Result<Self> {
        let dom : HtmlElement = dyn_into(get_element_by_id(dom_id)?)?;
        dom.set_property_or_panic("overflow", "hidden");

        let width  = dom.client_width()  as f32;
        let height = dom.client_height() as f32;
        let dimensions = Vector2::new(width, height);
        let data = Rc::new(RefCell::new(SceneData::new(dimensions)));

        let data_clone = data.clone();
        let resize_closure = Closure::new(move |width, height| {
            let mut data = data_clone.borrow_mut();
            data.dimensions = Vector2::new(width as f32, height as f32);
            for callback in &data.resize_callbacks {
                callback(&data.dimensions);
            }
        });
        let _resize_observer = ResizeObserver::new(&dom, resize_closure);

        Ok(Self { dom, _resize_observer, data })
    }

    /// Sets the Scene DOM's dimensions.
    pub fn set_dimensions(&mut self, dimensions : Vector2<f32>) {
        self.dom.set_property_or_panic("width" , format!("{}px", dimensions.x));
        self.dom.set_property_or_panic("height", format!("{}px", dimensions.y));
        self.data.borrow_mut().dimensions = dimensions;
    }

    /// Gets the Scene DOM's dimensions.
    pub fn get_dimensions(&self) -> Vector2<f32> {
        self.data.borrow().dimensions
    }

    /// Adds a ResizeCallback.
    pub fn add_resize_callback<T>(&mut self, callback : T)
        where T : Fn(&Vector2<f32>) + 'static {
        self.data.borrow_mut().resize_callbacks.push(Box::new(callback));
    }
}
