use super::{Camera, HTMLScene};
use nalgebra::{Translation, Vector3};

pub struct HTMLRenderer {}

impl HTMLRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, camera : &mut Camera, scene : &HTMLScene) {
        let (view_width, view_height) = scene.get_dimension();
        let transform = Translation::from(Vector3::new(view_width / 2.0, view_height / 2.0, 0.0)).to_homogeneous();
        let camera_transform = camera.rotation.conjugate().to_homogeneous() * Translation::from(-camera.position).to_homogeneous();
        let fov = camera.projection.as_matrix()[5] * scene.get_dimension().1 / 2.0;
        scene.camera.element.style().set_property("perspective", &format!("{}px", fov)).expect("set perspective");
        scene.camera.element.style().set_property("transform", &format!(
            "matrix3d({}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {})",
                      transform[ 0], transform[ 1], transform[ 2], transform[ 3],
                      transform[ 4], transform[ 5], transform[ 6], transform[ 7],
                      transform[ 8], transform[ 9], transform[10], transform[11],
                      transform[12], transform[13], transform[14], transform[15])).expect("set camera transform");

        for object in &scene.objects.items {
            match object {
                Some(object) => {
                    let transform = camera_transform * Translation::from(object.position - object.dimension / 2.0).to_homogeneous() * object.rotation.to_homogeneous();
                    object.element.style().set_property("transform", &format!(
                        "matrix3d({}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {})",
                                  transform[ 0], transform[ 1], transform[ 2], transform[ 3],
                                  transform[ 4], transform[ 5], transform[ 6], transform[ 7],
                                  transform[ 8], transform[ 9], transform[10], transform[11],
                                  transform[12], transform[13], transform[14], transform[15])).expect("set object transform");
                 },
                 None => ()
             }
         }
    }
}
