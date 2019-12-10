use crate::prelude::*;

use crate::display::rendering::Object;

use nalgebra::base::Matrix4;
use nalgebra::Orthographic3;

// ==============
// === Camera ===
// ==============

/// A 3D camera representation with its own 3D `Transform` and
/// projection matrix.
#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct CameraOrthographic {
    #[shrinkwrap(main_field)]
    pub object     : Object,
    pub projection : Matrix4<f32>,
    pub left       : f32,
    pub right      : f32,
    pub bottom     : f32,
    pub top        : f32,
    pub near       : f32,
    pub far        : f32
}

impl CameraOrthographic {
    pub fn new(
        left   : f32,
        right  : f32,
        bottom : f32,
        top    : f32,
        near   : f32,
        far    : f32) -> Self {
        let projection = Orthographic3::new(left, right, bottom, top, near, far);
        let projection = *projection.as_matrix();
        let object     = default();
        Self { object, projection, left, right, bottom, top, near, far }
    }

    pub fn get_y_scale(&self) -> f32 { self.projection.m11 }
}

#[cfg(test)]
mod test {
//    #[test]
//    fn perspective() {
//        use super::CameraOrthographic;
//        use nalgebra::Matrix4;
//        let camera   = CameraPerspective::new(45.0, 1920.0 / 1080.0, 1.0, 1000.0);
//        let expected = Matrix4::new
//            ( 1.357995,       0.0,       0.0,       0.0
//              , 0.0     , 2.4142134,       0.0,       0.0
//              , 0.0     ,       0.0, -1.002002, -2.002002
//              , 0.0     ,       0.0,      -1.0,       0.0
//            );
//        assert_eq!(camera.projection, expected);
//    }
}
