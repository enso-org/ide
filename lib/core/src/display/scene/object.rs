use super::Transform;

pub struct Object {
    pub transform : Transform
}

impl Object {
    pub fn new() -> Object {
        Self { transform : Transform::identity() }
    }

    pub fn set_position(&mut self, x : f32, y : f32, z : f32) {
        self.transform.set_position(x, y, z)
    }

    pub fn set_rotation(&mut self, roll : f32, pitch : f32, yaw : f32) {
        self.transform.set_rotation(roll, pitch, yaw)
    }
}
