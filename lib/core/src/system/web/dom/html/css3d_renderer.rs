//! This module contains the Css3dRenderer, a struct used to render CSS3D elements.

use crate::prelude::*;

use crate::display::camera::Camera2d;
use crate::display::camera::camera2d::Projection;
use crate::system::web::dom::html::Css3dObject;
use crate::system::gpu::data::JsBufferView;
use crate::system::web;
use crate::system::web::NodeInserter;
use crate::system::web::StyleSetter;

use nalgebra::Matrix4;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::HtmlDivElement;
use js_sys::Object;



// ===================
// === Js Bindings ===
// ===================

mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        function arr_to_css_matrix3d(a) {
            return `matrix3d(${a.join(',')})`
        }

        export function set_object_transform(dom, matrix_array) {
            let css = arr_to_css_matrix3d(matrix_array);
            dom.style.transform = 'translate(-50%, -50%)' + css;
        }

        export function setup_perspective(dom, perspective) {
            dom.style.perspective = perspective + 'px';
        }

        export function setup_camera_orthographic(dom, matrix_array) {
            dom.style.transform = arr_to_css_matrix3d(matrix_array);
        }

        export function setup_camera_perspective
        (dom, near, matrix_array) {
            let translateZ  = 'translateZ(' + near + 'px)';
            let matrix3d    = arr_to_css_matrix3d(matrix_array);
            let transform   = translateZ + matrix3d;
            dom.style.transform = transform;
        }
    ")]
    extern "C" {
        /// Setup perspective CSS 3D projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_perspective(dom: &JsValue, znear: &JsValue);

        /// Setup Camera orthographic projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_camera_orthographic(dom:&JsValue, matrix_array:&JsValue);

        /// Setup Camera perspective projection on DOM.
        #[allow(unsafe_code)]
        pub fn setup_camera_perspective(dom:&JsValue, near:&JsValue, matrix_array:&JsValue);

        /// Sets object's CSS 3D transform.
        #[allow(unsafe_code)]
        pub fn set_object_transform(dom:&JsValue, matrix_array:&Object);
    }
}

#[allow(unsafe_code)]
pub fn set_object_transform(dom:&JsValue, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
    // resized. Check documentation of IntoFloat32ArrayView trait for more
    // details.
    unsafe {
        let matrix_array = matrix.js_buffer_view();
        js::set_object_transform(&dom,&matrix_array);
    }
}


#[allow(unsafe_code)]
fn setup_camera_perspective(dom:&JsValue, near:f32, matrix:&Matrix4<f32>) {
    // Views to WASM memory are only valid as long the backing buffer isn't
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



// =============
// === Utils ===
// =============

/// Inverts Matrix Y coordinates. It's equivalent to scaling by (1.0, -1.0, 1.0).
pub fn invert_y(mut m: Matrix4<f32>) -> Matrix4<f32> {
    // Negating the second column to invert Y.
    m.row_part_mut(1, 4).iter_mut().for_each(|a| *a = -*a);
    m
}



// =========================
// === Css3dRendererData ===
// =========================

#[derive(Clone,Debug)]
pub struct Css3dRendererData {
    pub dom                 : HtmlDivElement,
    pub view_projection_dom : HtmlDivElement,
    logger                  : Logger
}

impl Css3dRendererData {
    pub fn new(dom:HtmlDivElement, view_projection_dom:HtmlDivElement, logger:Logger) -> Self {
        Self {logger,dom, view_projection_dom }
    }

//    fn set_dimensions(&self, dimensions:Vector2<f32>) {
//        let width  = format!("{}px", dimensions.x);
//        let height = format!("{}px", dimensions.y);
//        let doms   = vec![&self.dom, &self.view_projection_dom];
//        for dom in doms {
//            dom.set_style_or_warn("width"  , &width  , &self.logger);
//            dom.set_style_or_warn("height" , &height , &self.logger);
//        }
//    }
}



// =====================
// === Css3dRenderer ===
// =====================

/// `Css3dRenderer` is a renderer for `Css3dObject`s. It integrates with other rendering contexts,
/// such as WebGL, by placing two HtmlElements in front and behind of the Canvas element,
/// allowing the move `Css3dObject`s between these two layers, mimicking z-index ordering.
///
/// To make use of its functionalities, the API user can create a `Css3dSystem` by using
/// the `Css3dRenderer::new_system` method which creates and manages instances of
/// `Css3dObject`s.




#[derive(Clone,Debug,Shrinkwrap)]
pub struct Css3dRenderer {
    data : Rc<Css3dRendererData>,
}

impl Css3dRenderer {
    /// Constructor.
    pub fn new(logger:&Logger) -> Self {
        let logger              = logger.sub("Css3dRenderer");
        let dom                 = web::create_div();
        let view_projection_dom = web::create_div();

        dom.set_style_or_warn("position"       , "absolute" , &logger);
        dom.set_style_or_warn("top"            , "0px"      , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("overflow"       , "hidden"   , &logger);
        dom.set_style_or_warn("width"          , "100%"     , &logger);
        dom.set_style_or_warn("height"         , "100%"     , &logger);
        dom.set_style_or_warn("pointer-events" , "none"     , &logger);

        view_projection_dom.set_style_or_warn("width"           , "100%"        , &logger);
        view_projection_dom.set_style_or_warn("height"          , "100%"        , &logger);
        view_projection_dom.set_style_or_warn("transform-style" , "preserve-3d" , &logger);

        dom.append_or_warn(&view_projection_dom,&logger);

        let data = Css3dRendererData::new (dom,view_projection_dom,logger);
        let data = Rc::new(data);
        Self {data}
    }

    /// Sets the z-index of this DOM element.
    pub fn set_z_index(&self, z:i32) {
        self.data.dom.set_style_or_warn("z-index", z.to_string(), &self.logger);
    }

    /// Creates a new instance of Css3dObject and adds it to parent.
    pub fn manage(&self, object:&Css3dObject) {
        let front_layer = self.data.view_projection_dom.clone();
        front_layer.append_or_warn(&object.dom(),&self.data.logger);
    }

    /// Update the objects to match the new camera's point of view. This function should be called
    /// only after camera position change.
    pub fn update_view_projection(&self, camera:&Camera2d) {
        let trans_cam  = camera.transform().matrix().try_inverse();
        let trans_cam  = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam  = trans_cam.map(eps);
        let trans_cam  = invert_y(trans_cam);
        let half_dim   = camera.screen().height / 2.0;
        let fovy_slope = camera.half_fovy_slope();
        let near       = half_dim / fovy_slope;

        match camera.projection() {
            Projection::Perspective{..} => {
                js::setup_perspective(&self.data.dom , &near.into());
                setup_camera_perspective(&self.data.view_projection_dom , near, &trans_cam);
            },
            Projection::Orthographic => {
                setup_camera_orthographic(&self.data.view_projection_dom , &trans_cam);
            }
        }
    }
}

impl CloneRef for Css3dRenderer {}



// =============
// === Utils ===
// =============

/// eps is used to round very small values to 0.0 for numerical stability
pub fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}
