use nalgebra::{Vector3, UnitQuaternion};

pub struct Object {
    pub position : Vector3<f32>,
    pub rotation : UnitQuaternion<f32>
}

impl Object {
    pub fn new() -> Object {
        Self { position : Vector3::new(0.0, 0.0, 0.0), rotation : UnitQuaternion::identity() }
    }

    pub fn set_position(&mut self, x : f32, y : f32, z : f32) {
        self.position = Vector3::new(x, y, z);
    }
    pub fn set_rotation(&mut self, roll : f32, pitch : f32, yaw : f32) {
        self.rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
    }
}
