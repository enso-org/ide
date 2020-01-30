//! This module contains the Css3dRenderer, a struct used to render CSS3D elements.

use crate::prelude::*;

use crate::display::object::DisplayObjectData;
use crate::display::camera::Camera2d;
use crate::display::camera::camera2d::Projection;
use crate::system::web::dom::html::Css3dObject;
use crate::system::gpu::data::JsBufferView;
use crate::system::web::Result;
use crate::system::web::create_element;
use crate::system::web::dyn_into;
use crate::system::web::NodeInserter;
use crate::system::web::StyleSetter;
use crate::system::web::dom::DomContainer;
use crate::system::web::dom::ResizeCallback;
use super::eps;

use nalgebra::Vector2;
use nalgebra::Matrix4;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::HtmlElement;
use basegl_system_web::get_element_by_id;


// ===================
// === Js Bindings ===
// ===================

mod js {
    use super::*;
    #[wasm_bindgen(module = "/src/system/web/dom/html/snippets.js")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn setup_perspective(dom: &JsValue, znear: &JsValue);

        #[allow(unsafe_code)]
        pub fn setup_camera_orthographic(dom:&JsValue, matrix_array:&JsValue);

        #[allow(unsafe_code)]
        pub fn setup_camera_perspective
        ( dom          : &JsValue
        , near         : &JsValue
        , matrix_array : &JsValue
        );
    }
}

/// Inverts Matrix Y coordinates.
/// It's equivalent to scaling by (1.0, -1.0, 1.0).
pub fn invert_y(mut m: Matrix4<f32>) -> Matrix4<f32> {
    // Negating the second column to invert Y.
    m.row_part_mut(1, 4).iter_mut().for_each(|a| *a = -*a);
    m
}

#[allow(unsafe_code)]
fn setup_camera_perspective
(dom:&JsValue, near:f32, matrix:&Matrix4<f32>) { // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::setup_camera_perspective(
            &dom,
            &near.into(),
            &matrix_array
        )
    }
}

#[allow(unsafe_code)]
fn setup_camera_orthographic(dom:&JsValue, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::setup_camera_orthographic(&dom, &matrix_array)
    }
}



// =========================
// === Css3dRendererData ===
// =========================

#[derive(Debug)]
pub struct Css3dRendererData {
    pub front_dom    : HtmlElement,
    pub back_dom     : HtmlElement,
    pub front_camera : HtmlElement,
    pub back_camera  : HtmlElement
}

impl Css3dRendererData {
    pub fn new
    ( front_dom:HtmlElement
    , back_dom:HtmlElement
    , front_camera:HtmlElement
    , back_camera:HtmlElement) -> Self {
        Self {front_dom,back_dom,front_camera,back_camera}
    }
}

impl Css3dRendererData {
    fn set_dimensions(&self, dimensions:Vector2<f32>) {
        let width  = format!("{}px", dimensions.x);
        let height = format!("{}px", dimensions.y);
        let doms   = vec![&self.front_dom,&self.back_dom,&self.front_camera,&self.back_camera];
        for dom in doms {
            dom.set_property_or_panic("width" , &width);
            dom.set_property_or_panic("height", &height);
        }
    }
}

// ====================
// === Css3dRenderer ===
// ====================

/// A renderer for `Css3dObject`s.
#[derive(Clone,Debug)]
pub struct Css3dRenderer {
    container      : DomContainer,
    data           : Rc<Css3dRendererData>,
    logger         : Logger
}

impl Css3dRenderer {
    /// Creates a Css3dRenderer inside an element.
    pub fn from_element<L:Into<Logger>>(logger:L, element:HtmlElement) -> Result<Self> {
        let logger                     = logger.into();
        let container                  = DomContainer::from_element(element);
        let front_dom    : HtmlElement = dyn_into(create_element("div")?)?;
        let back_dom     : HtmlElement = dyn_into(create_element("div")?)?;
        let front_camera : HtmlElement = dyn_into(create_element("div")?)?;
        let back_camera  : HtmlElement = dyn_into(create_element("div")?)?;

        front_dom.set_property_or_panic("position", "absolute");
        front_dom.set_property_or_panic("top", "0px");
        front_dom.set_property_or_panic("overflow", "hidden");
        front_dom.set_property_or_panic("overflow", "hidden");
        front_dom.set_property_or_panic("width", "100%");
        front_dom.set_property_or_panic("height", "100%");
        front_dom.set_property_or_panic("pointer-events", "none");
        back_dom.set_property_or_panic("position", "absolute");
        back_dom.set_property_or_panic("top", "0px");
        back_dom.set_property_or_panic("overflow", "hidden");
        back_dom.set_property_or_panic("overflow", "hidden");
        back_dom.set_property_or_panic("width", "100%");
        back_dom.set_property_or_panic("height", "100%");
        back_dom.set_property_or_panic("pointer-events", "none");
        back_dom.set_property_or_panic("z-index", "-1");
        front_camera.set_property_or_panic("width", "100%");
        front_camera.set_property_or_panic("height", "100%");
        front_camera.set_property_or_panic("transform-style", "preserve-3d");
        back_camera.set_property_or_panic("width", "100%");
        back_camera.set_property_or_panic("height", "100%");
        back_camera.set_property_or_panic("transform-style", "preserve-3d");

        container.dom.append_or_panic(&front_dom);
        container.dom.append_or_panic(&back_dom);
        front_dom.append_or_panic(&front_camera);
        back_dom.append_or_panic(&back_camera);

        let data             = Css3dRendererData::new(front_dom, back_dom, front_camera, back_camera);
        let data             = Rc::new(data);
        let mut renderer     = Self {container,data,logger};

        renderer.init_listeners();
        Ok(renderer)
    }

    /// Creates a Css3dRenderer.
    pub fn new<L:Into<Logger>>(logger:L, dom_id: &str) -> Result<Self> {
        Self::from_element(logger,dyn_into(get_element_by_id(dom_id)?)?)
    }

    pub(crate) fn logger(&self) -> Logger {
        self.logger.clone()
    }

    fn init_listeners(&mut self) {
        let dimensions = self.dimensions();
        self.set_dimensions(dimensions);
        let data = self.data.clone();
        self.add_resize_callback(move |dimensions:&Vector2<f32>| {
            data.set_dimensions(*dimensions);
        });
    }

    /// Creates a new instance of Css3dObject and adds it to parent.
    pub(super) fn new_instance
    <S:AsRef<str>>(&self, dom_name:S, parent:DisplayObjectData) -> Result<Css3dObject> {
        let front_camera = self.data.front_camera.clone();
        let back_camera  = self.data.back_camera.clone();
        let object = Css3dObject::new(self.logger.sub("object"),dom_name,front_camera,back_camera);
        object.as_ref().map(|object| {
            parent.add_child(object);
        }).ok();
        object
    }

    /// Renders `Camera`'s point of view.
    pub fn render(&self, camera: &Camera2d) {
        let trans_cam  = camera.transform().matrix().try_inverse();
        let trans_cam  = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam  = trans_cam.map(eps);
        let trans_cam  = invert_y(trans_cam);
        let half_dim   = self.container.dimensions() / 2.0;
        let fovy_slope = camera.half_fovy_slope();
        let near       = half_dim.y / fovy_slope;

        match camera.projection() {
            Projection::Perspective{..} => {
                js::setup_perspective(&self.data.front_dom, &near.into());
                js::setup_perspective(&self.data.back_dom, &near.into());
                setup_camera_perspective(
                    &self.data.front_camera,
                    near,
                    &trans_cam
                );
                setup_camera_perspective(
                    &self.data.back_camera,
                    near,
                    &trans_cam
                );
            },
            Projection::Orthographic => {
                setup_camera_orthographic(&self.data.front_camera, &trans_cam);
                setup_camera_orthographic(&self.data.back_camera, &trans_cam);
            }
        }
    }

    /// Adds a ResizeCallback.
    pub fn add_resize_callback<T:ResizeCallback>(&mut self, callback : T) {
        self.container.add_resize_callback(callback);
    }

    /// Sets Css3dRenderer's container dimensions.
    pub fn set_dimensions(&mut self, dimensions : Vector2<f32>) {
        self.data.set_dimensions(dimensions);
        self.container.set_dimensions(dimensions);
    }
}


// === Getters ===

impl Css3dRenderer {
    /// Gets Css3dRenderer's container.
    pub fn container(&self) -> &DomContainer {
        &self.container
    }

    /// Gets Css3dRenderer's DOM.
    pub fn dom(&self) -> &HtmlElement {
        &self.data.front_dom
    }

    /// Gets the Css3dRenderer's dimensions.
    pub fn dimensions(&self) -> Vector2<f32> {
        self.container.dimensions()
    }
}
