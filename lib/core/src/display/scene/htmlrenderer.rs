use super::{Camera, HTMLScene};

fn eps(value : f32) -> f32 { if value.abs() < 1e-10 { 0.0 } else { value } }

pub struct HTMLRenderer {}

impl HTMLRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, camera : &mut Camera, scene : &HTMLScene) {
        let (view_width, view_height) = scene.get_dimension();
        let fov = camera.projection[5] * view_height / 2.0;

        let transform = camera.transform.to_homogeneous().try_inverse().expect("inverse");
        scene.div.element.style().set_property("perspective", &format!("{}px", fov)).expect("set perspective");
        scene.camera.element.style().set_property("transform", &format!(
            "translateZ({}px)
             matrix3d({}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {},
                      {}, {}, {}, {})
             translate({}px, {}px)",
                      fov,
                      eps( transform[ 0]), eps(-transform[ 1]), eps( transform[ 2]), eps( transform[ 3]),
                      eps( transform[ 4]), eps(-transform[ 5]), eps( transform[ 6]), eps( transform[ 7]),
                      eps( transform[ 8]), eps(-transform[ 9]), eps( transform[10]), eps( transform[11]),
                      eps( transform[12]), eps(-transform[13]), eps( transform[14]), eps( transform[15]),
                      view_width / 2.0, view_height / 2.0)).expect("set camera transform");

        for object in &scene.objects.items {
            match object {
                Some(object) => {
                    let transform = object.transform.to_homogeneous();
                    object.element.style().set_property("transform", &format!(
                        "translate(-50%, -50%)
                         matrix3d({}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {},
                                  {}, {}, {}, {})",
                                  eps( transform[ 0]), eps(transform[ 1]), eps( transform[ 2]), eps( transform[ 3]),
                                  eps(-transform[ 4]), eps(-transform[ 5]), eps(-transform[ 6]), eps(-transform[ 7]),
                                  eps( transform[ 8]), eps( transform[ 9]), eps( transform[10]), eps( transform[11]),
                                  eps( transform[12]), eps( transform[13]), eps( transform[14]), eps( transform[15]))).expect("set object transform");
                 },
                 None => ()
             }
         }
    }
}
