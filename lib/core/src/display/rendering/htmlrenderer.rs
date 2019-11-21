use crate::prelude::*;

use super::Camera;
use super::HTMLScene;
use crate::math::utils::IntoCSSMatrix;

use crate::system::web::StyleSetter;
use nalgebra::Matrix4;

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
        // Note [znear from projection matrix]
        let half_dim  = scene.get_dimensions() / 2.0;
        let expr      = camera.projection[(1, 1)];
        let near      = format!("{}px", expr * half_dim.y);
        let trans_cam = camera.transform.to_homogeneous().try_inverse();
        let trans_cam = trans_cam.expect("Camera's matrix is not invertible.");
        let trans_cam = trans_cam.map(|a| eps(a));
        let trans_cam = invert_y(trans_cam);
        let trans_z   = format!("translateZ({})", near);
        let matrix3d  = trans_cam.into_css_matrix();
        let trans     = format!("translate({}px,{}px)", half_dim.x, half_dim.y);
        let css       = format!("{} {} {}", trans_z, matrix3d, trans);

        scene.div   .element.set_property_or_panic("perspective", near);
        scene.camera.element.set_property_or_panic("transform"  , css);

        for object in &scene.objects {
            let mut transform = object.transform.to_homogeneous();
            transform.iter_mut().for_each(|a| *a = eps(*a));
            let matrix3d = transform.into_css_matrix();
            let css      = format!("translate(-50%, -50%) {}", matrix3d);
            object.element.set_property_or_panic("transform", css);
        }
    }
}

// =============
// === Utils ===
// =============

// eps is used to round very small values to 0.0 for numerical stability
fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}

// Inverts Matrix Y coordinates.
// It's equivalent to scaling by (1.0, -1.0, 1.0).
fn invert_y(mut m: Matrix4<f32>) -> Matrix4<f32> {
    // Negating the second column to invert Y.
    m.row_part_mut(1, 4).iter_mut().for_each(|a| *a = -*a);
    m
}

// Note [znear from projection matrix]
// =================================
// https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L275
