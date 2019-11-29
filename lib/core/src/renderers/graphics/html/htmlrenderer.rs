use crate::prelude::*;
use crate::renderers;
use renderers::*;
use renderers::graphics::*;
use crate::renderers::graphics::GraphicsRenderer;

use super::HTMLObject;
use crate::math::utils::IntoFloat32Array;
use crate::math::utils::eps;
use crate::math::utils::invert_y;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use crate::system::web::Result;
use crate::math::Vector2;
use crate::system::web::create_element;
use crate::system::web::dyn_into;
use crate::system::web::NodeInserter;
use crate::system::web::StyleSetter;
use web_sys::HtmlElement;

#[wasm_bindgen(module = "/js/htmlrenderer.js")]
extern "C" {
    fn set_object_transform(dom: &JsValue, matrix_array: &JsValue);
    fn setup_perspective(dom: &JsValue, znear : &JsValue);
    fn setup_camera_transform(
        dom          : &JsValue,
        znear        : &JsValue,
        half_width   : &JsValue,
        half_height  : &JsValue,
        matrix_array : &JsValue);
}

// ========================
// === HTMLRendererData ===
// ========================

#[derive(Debug)]
pub struct HTMLRendererData {
    pub div    : HtmlElement,
    pub camera : HtmlElement
}

impl HTMLRendererData {
    pub fn new(div : HtmlElement, camera : HtmlElement) -> Self {
        Self { div, camera }
    }

    pub fn set_dimensions(&self, dimensions : Vector2<f32>) {
        let width  = format!("{}px", dimensions.x);
        let height = format!("{}px", dimensions.y);
        self.div   .set_property_or_panic("width" , &width);
        self.div   .set_property_or_panic("height", &height);
        self.camera.set_property_or_panic("width" , &width);
        self.camera.set_property_or_panic("height", &height);
    }
}

// ====================
// === HTMLRenderer ===
// ====================

/// A renderer for `HTMLObject`s.
#[derive(Shrinkwrap, Debug)]
pub struct HTMLRenderer {
    #[shrinkwrap(main_field)]
    pub renderer : GraphicsRenderer,
    pub data     : Rc<HTMLRendererData>
}

impl HTMLRenderer {
    /// Creates a HTMLRenderer.
    pub fn new(dom_id: &str) -> Result<Self> {
        let mut renderer = GraphicsRenderer::new(dom_id)?;

        let div    : HtmlElement = dyn_into(create_element("div")?)?;
        let camera : HtmlElement = dyn_into(create_element("div")?)?;
        div   .set_property_or_panic("width", "100%");
        div   .set_property_or_panic("height", "100%");
        camera.set_property_or_panic("width", "100%");
        camera.set_property_or_panic("height", "100%");
        camera.set_property_or_panic("transform-style", "preserve-3d");

        renderer.container.dom.append_child_or_panic(&div);
        div                   .append_child_or_panic(&camera);

        let data = Rc::new(HTMLRendererData::new(div, camera));

        let data_clone = data.clone();
        renderer.add_resize_callback(Box::new(move |dimensions| {
            data_clone.set_dimensions(*dimensions);
        }));

        let dimensions = renderer.get_dimensions();
        let mut htmlrenderer = Self { renderer, data };
        htmlrenderer.set_dimensions(dimensions);
        Ok(htmlrenderer)
    }

    /// Renders the `Scene` from `Camera`'s point of view.
    pub fn render(&self, camera: &mut Camera, scene: &Scene<HTMLObject>) {
        let trans_cam = camera.transform.to_homogeneous().try_inverse();
        let trans_cam = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam = trans_cam.map(eps);
        let trans_cam = invert_y(trans_cam);

        // Note [znear from projection matrix]
        let half_dim     = self.renderer.container.get_dimensions() / 2.0;
        let expr         = camera.projection[(1, 1)];
        let near         = (expr * half_dim.y).into();

        let half_width   = half_dim.x.into();
        let half_height  = half_dim.y.into();

        let matrix_array = trans_cam.into_float32array();

        setup_perspective(&self.data.div, &near);
        setup_camera_transform(
            &self.data.camera,
            &near,
            &half_width,
            &half_height,
            &matrix_array
        );

        let scene : &Scene<HTMLObject> = &scene;
        for object in &mut scene.into_iter() {
            if !self.data.camera.is_same_node(object.dom.parent_node().as_ref()) {
                self.data.camera.append_child_or_panic(&object.dom)
            }
            let mut transform = object.transform.to_homogeneous();
            transform.iter_mut().for_each(|a| *a = eps(*a));

            let matrix_array = transform.into_float32array();
            set_object_transform(&object.dom, &matrix_array);
        }
    }

    pub fn set_dimensions(&mut self, dimensions : Vector2<f32>) {
        self.renderer.set_dimensions(dimensions);
        self.data.set_dimensions(dimensions);
    }
}

// Note [znear from projection matrix]
// ===================================
// https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L275
