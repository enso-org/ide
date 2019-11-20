use super::Camera;
use super::HTMLScene;
use super::IntoCSSMatrix;
use crate::system::web::StyleSetter;
use crate::prelude::*;

fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}

/// A renderer for `HTMLObject`s.
#[derive(Default)]
pub struct HTMLRenderer {}

impl HTMLRenderer {
    /// Creates a HTMLRenderer.
    pub fn new() -> Self { default() }

    /// Renders the `Scene` from `Camera`'s point of view.
    pub fn render(&self, camera: &mut Camera, scene: &HTMLScene) {
        let half_d = scene.get_dimensions() / 2.0;
        // Note [znear from projection matrix]
        let expr = camera.projection[(1, 1)]; // expr = 2.0 * near / (height)
        let near = expr * half_d.y;

        let near = format!("{}px", near);
        scene.div.element.set_property_or_panic("perspective", &near);

        let mut transform = camera
                      .transform.to_homogeneous().try_inverse()
                      .expect("Render couldn't get camera's matrix inverse")
                      .map(|a| eps(a));

        // Negating the second column to invert Y.
        // Equivalent to scaling by (1.0, -1.0, 1.0).
        transform.row_part_mut(1, 4).iter_mut().for_each(|a| *a = -*a);

        let translatez = format!("translateZ({})", near);
        let matrix3d = transform.into_css_matrix();
        let translate = format!("translate({}px, {}px)", half_d.x, half_d.y);
        let css = format!("{} {} {}", translatez, matrix3d, translate);
        scene.camera.element.set_property_or_panic("transform", css);

        scene
            .objects
            .items
            .iter()
            .filter(|object| object.is_some())
            .map(|a| a.as_ref().unwrap())
            .for_each(|object| {
                let mut transform = object.transform.to_homogeneous();
                transform.iter_mut().for_each(|a| *a = eps(*a));

                let matrix3d = transform.into_css_matrix();
                let css = format!("translate(-50%, -50%) {}", matrix3d);
                object.element.set_property_or_panic("transform", css);
            });
    }
}

// Note [znear from projection matrix]
// =================================
// https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L275
