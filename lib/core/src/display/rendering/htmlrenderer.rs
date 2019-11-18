use super::{Camera, HTMLScene};
use crate::system::web::StyleSetter;

fn eps(value: f32) -> f32 {
    if value.abs() < 1e-10 { 0.0 } else { value }
}

/// A renderer for `HTMLObject`s
#[derive(Default)]
pub struct HTMLRenderer {}

impl HTMLRenderer {
    /// Creates a HTMLRenderer
    pub fn new() -> Self { Default::default() }

    /// Renders the `Scene` from `Camera`'s point of view
    pub fn render(&self, camera: &mut Camera, scene: &HTMLScene) {
        let (view_width, view_height) = scene.get_dimension();
        // Note [fov from projection matrix]
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L275
        let fov = camera.projection[5] * view_height / 2.0;

        let transform = camera.transform.to_homogeneous().try_inverse().expect("inverse");
        scene.div
            .element
            .set_property_or_panic("perspective", &format!("{}px", fov));

        // Note [CSS Matrix3D from Camera]
        // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
        // https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L100
        let matrix3d = format!(
            "matrix3d({}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {})",
                      eps(transform[ 0]), eps(-transform[ 1]), eps(transform[ 2]), eps(transform[ 3]),
                      eps(transform[ 4]), eps(-transform[ 5]), eps(transform[ 6]), eps(transform[ 7]),
                      eps(transform[ 8]), eps(-transform[ 9]), eps(transform[10]), eps(transform[11]),
                      eps(transform[12]), eps(-transform[13]), eps(transform[14]), eps(transform[15]));
        scene.camera.element.set_property_or_panic("transform",
                &format!("translateZ({}px) {} translate({}px, {}px)",
                    fov,
                    matrix3d,
                    view_width / 2.0,
                    view_height / 2.0
                ),
            );

        for object in &scene.objects.items {
            match object {
                Some(object) => {
                    let transform = object.transform.to_homogeneous();
                    // Note [CSS Matrix3D from Object]
                    // ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
                    // https://github.com/mrdoob/three.js/blob/22ed6755399fa180ede84bf18ff6cea0ad66f6c0/examples/js/renderers/CSS3DRenderer.js#L125
                    let matrix3d = format!(
                        "matrix3d({}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {})",
                                  eps( transform[ 0]), eps( transform[ 1]), eps( transform[ 2]), eps( transform[ 3]),
                                  eps(-transform[ 4]), eps(-transform[ 5]), eps(-transform[ 6]), eps(-transform[ 7]),
                                  eps( transform[ 8]), eps( transform[ 9]), eps( transform[10]), eps( transform[11]),
                                  eps( transform[12]), eps( transform[13]), eps( transform[14]), eps( transform[15]));
                    object.element
                        .set_property_or_panic("transform",
                            &format!("translate(-50%, -50%) {}", matrix3d)
                        );
                }
                None => (),
            }
        }
    }
}
