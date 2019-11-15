use super::Object;
use nalgebra::geometry::{Perspective3};

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Camera {
    #[shrinkwrap(main_field)]
    pub object : Object,
    pub projection : Perspective3<f32>
}

impl Camera {
    pub fn perspective(fov : f32, aspect : f32, znear : f32, zfar : f32) -> Self {
        let projection = Perspective3::new(fov, aspect, znear, zfar);
        Self { object : Object::new(), projection }
    }
}
