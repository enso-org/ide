use crate::prelude::*;

use super::Camera;
use super::HTMLScene;
use crate::math::utils::IntoFloat32Array;
use crate::math::utils::eps;
use crate::math::utils::invert_y;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;

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

// ====================
// === HTMLRenderer ===
// ====================

/// A renderer for `HTMLObject`s.
#[derive(Default, Debug)]
pub struct HTMLRenderer {}

impl HTMLRenderer {
    /// Creates a HTMLRenderer.
    pub fn new() -> Self { default() }

    /// Renders the `Scene` from `Camera`'s point of view.
    pub fn render(&self, camera: &mut Camera, scene: &HTMLScene) {
        let trans_cam = camera.transform.to_homogeneous().try_inverse();
        let trans_cam = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam = trans_cam.map(eps);
        let trans_cam = invert_y(trans_cam);

        // Note [znear from projection matrix]
        let half_dim     = scene.get_dimensions() / 2.0;
        let expr         = camera.projection[(1, 1)];
        let near         = (expr * half_dim.y).into();

        let half_width   = half_dim.x.into();
        let half_height  = half_dim.y.into();

        let matrix_array = trans_cam.into_float32array();

        setup_perspective(&scene.html_data.div.element, &near);
        setup_camera_transform(
            &scene.html_data.camera.element,
            &near,
            &half_width,
            &half_height,
            &matrix_array
        );

        for object in scene {
            let mut transform = object.transform.to_homogeneous();
            transform.iter_mut().for_each(|a| *a = eps(*a));

            let matrix_array = transform.into_float32array();
            set_object_transform(&object.element, &matrix_array);
        }
    }
}

// Note [znear from projection matrix]
// ===================================
// https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L275
