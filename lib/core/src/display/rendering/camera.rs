use super::Object;
use nalgebra::base::Matrix4;
use nalgebra::geometry::Perspective3;
use std::f32::consts::PI;

/// A 3D camera representation with its own 3D `Transform` and projection matrix
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Camera {
    #[shrinkwrap(main_field)]
    pub object: Object,
    pub projection: Matrix4<f32>,
}

impl Camera {
    /// Creates a Camera with perspective projection
    ///
    /// # Arguments
    /// fov - Field of View in degrees
    /// aspect - Aspect ratio of the screen
    /// znear - Near distance clipping
    /// zfar - Far distance clipping
    ///
    /// # Example
    /// ```
    /// use basegl::display::rendering::Camera;
    /// let dimension = (1920.0, 1080.0);
    /// let camera = Camera::perspective(45.0, dimension.0 / dimension.1, 1.0, 1000.0);
    /// ```
    pub fn perspective(fov: f32, aspect: f32, znear: f32, zfar: f32) -> Self {
        let projection = *Perspective3::new(aspect, fov / 180.0 * PI, znear, zfar).as_matrix();
        Self { object: Default::default(), projection }
    }
}
