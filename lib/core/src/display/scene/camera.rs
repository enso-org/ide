use super::Object;
use nalgebra::geometry::{Perspective3};
use nalgebra::base::{Matrix4};
use std::f32::consts::PI;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Camera {
    #[shrinkwrap(main_field)]
    pub object : Object,
    pub projection : Matrix4<f32>
}

impl Camera {
    pub fn perspective(fov : f32, aspect : f32, znear : f32, zfar : f32) -> Self {
        let projection = Perspective3::new(aspect, fov / 180.0 * PI, znear, zfar).as_matrix().clone();
        Self { object : Object::new(), projection }
    }
}
