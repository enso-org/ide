//! This module contains the implementation of HTMLObject, a struct used to represent CSS3D
//! elements.

use crate::prelude::*;

use crate::display::object::DisplayObjectData;
use crate::system::web::create_element;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::Error;
use crate::system::web::StyleSetter;
use crate::system::web::NodeInserter;
use crate::system::web::NodeRemover;
use crate::system::gpu::data::JsBufferView;
use super::eps;

use nalgebra::Vector2;
use nalgebra::Matrix4;
use web_sys::HtmlElement;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use js_sys::Object;



// ===================
// === Js Bindings ===
// ===================

mod js {
    use super::*;

    #[wasm_bindgen(module = "/src/system/web/dom/html/snippets.js")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn set_object_transform(dom: &JsValue, matrix_array: &Object);
    }
}

#[allow(unsafe_code)]
fn set_object_transform(dom: &JsValue, matrix: &Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array =  matrix.js_buffer_view();
        js::set_object_transform(&dom, &matrix_array);
    }
}



// ==================
// === HtmlObject ===
// ==================

/// A structure for representing a 3D HTMLElement in a `HTMLScene`.
#[derive(Shrinkwrap, Debug, Clone)]
#[shrinkwrap(mutable)]
pub struct Css3dObject {
    #[shrinkwrap(main_field)]
    /// HTMLObject's hierarchical transforms.
    pub display_object : DisplayObjectData,

    /// The DOM to be rendered with CSS3D.
    pub dom     : HtmlElement,
    dimensions  : Vector2<f32>,
    camera_node : HtmlElement
}

impl Drop for Css3dObject {
    fn drop(&mut self) {
        self.dom.remove_from_parent_or_panic();
        self.display_object.unset_parent()
    }
}

impl Css3dObject {
    /// Creates a HTMLObject from element name.
    pub fn new
    <L:Into<Logger>,S:AsRef<str>>(logger:L, dom_name:S, camera_node:HtmlElement) -> Result<Self> {
        let dom = dyn_into(create_element(dom_name.as_ref())?)?;
        Ok(Self::from_element(logger,dom,camera_node))
    }

    /// Creates a HTMLObject from a web_sys::HtmlElement.
    pub fn from_element
    <L:Into<Logger>>(logger:L, element:HtmlElement, camera_node:HtmlElement) -> Self {
        let logger = logger.into();
        element.set_property_or_panic("position", "absolute");
        element.set_property_or_panic("width"   , "0px");
        element.set_property_or_panic("height"  , "0px");
        let dom            = element;
        let display_object = DisplayObjectData::new(logger.clone());
        let dimensions     = Vector2::new(0.0, 0.0);
        let object = Self {display_object,dom,dimensions,camera_node};
        let object_clone = object.clone();
        object.display_object.set_on_render(move || {
            object_clone.render_dom();
        });
        object
    }

    /// Creates a HTMLObject from a HTML string.
    pub fn from_html_string<L,T>(logger:L, html_string:T, camera_node:HtmlElement) -> Result<Self>
    where L:Into<Logger>, T:AsRef<str> {
        let element = create_element("div")?;
        element.set_inner_html(html_string.as_ref());
        match element.first_element_child() {
            Some(element) => Ok(Self::from_element(logger,dyn_into(element)?,camera_node)),
            None          => Err(Error::missing("valid HTML")),
        }
    }

    /// Sets the underlying HtmlElement dimension.
    pub fn set_dimensions(&mut self, dimensions:Vector2<f32>) {
        self.dimensions = dimensions;
        self.dom.set_property_or_panic("width",  format!("{}px", dimensions.x));
        self.dom.set_property_or_panic("height", format!("{}px", dimensions.y));
    }

    /// Gets the underlying HtmlElement dimension.
    pub fn dimensions(&self) -> &Vector2<f32> {
        &self.dimensions
    }

    /// Renders the object's dom.
    pub fn render_dom(&self) {
        let mut transform = self.matrix();
        transform.iter_mut().for_each(|a| *a = eps(*a));

        let camera_node = &self.camera_node;
        let parent_node = self.dom.parent_node();
        if !camera_node.is_same_node(parent_node.as_ref()) {
            camera_node.append_or_panic(&self.dom);
        }

        set_object_transform(&self.dom, &transform);
    }
}

impl From<&Css3dObject> for DisplayObjectData {
    fn from(t:&Css3dObject) -> Self {
        t.display_object.clone_ref()
    }
}
