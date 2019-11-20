use super::Object;
use nalgebra::base::Matrix4;
use nalgebra::geometry::Perspective3;
use std::f32::consts::PI;
use crate::prelude::*;

/// A 3D camera representation with its own 3D `Transform` and
/// projection matrix.
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Camera {
    #[shrinkwrap(main_field)]
    pub object     : Object,
    pub projection : Matrix4<f32>,
}

impl Camera {
    /// Creates a Camera with perspective projection.
    ///
    /// # Example
    /// ```
    /// use basegl::display::rendering::Camera;
    /// let dimensions = (1920.0, 1080.0);
    /// let aspect_ratio = dimensions.0 / dimensions.1;
    /// let camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
    /// ```
    pub fn perspective(field_of_view_degrees : f32,
                       aspect_ratio          : f32,
                       near_clipping         : f32,
                       far_clipping          : f32) -> Self {
        let projection = *Perspective3::new(
                aspect_ratio,
                field_of_view_degrees / 180.0 * PI,
                near_clipping,
                far_clipping
            ).as_matrix();
        Self { object: default(), projection }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn perspective() {
        use super::Camera;
        use nalgebra::Matrix4;
        let camera = Camera::perspective(45.0, 1920.0 / 1080.0, 1.0, 1000.0);

        let expected = Matrix4::new(1.357995,       0.0,       0.0,       0.0,
                                         0.0, 2.4142134,       0.0,       0.0,
                                         0.0,       0.0, -1.002002, -2.002002,
                                         0.0,       0.0,      -1.0,       0.0);

       assert_eq!(camera.projection, expected);
    }
}
