use super::Transform;

/// Base structure for representing a 3D object in a `Scene`
pub struct Object {
    pub transform: Transform,
}

impl Default for Object {
    fn default() -> Self {
        Self { transform: Transform::identity() }
    }
}

impl Object {
    /// Creates a Default Object
    pub fn new() -> Object {
        Default::default()
    }

    /// Sets the object's position
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) {
        self.transform.set_translation(x, y, z)
    }

    /// Sets the object's rotation in YXZ (yaw -> roll -> pitch) order
    ///
    /// # Arguments
    ///
    /// * roll - rotates around x-axis (in radians)
    /// * pitch - rotates around y-axis (in radians)
    /// * yaw - rotates around z-axis (in radians)
    ///
    /// # Example
    ///
    /// ```
    /// // You can have rust code between fences inside the comments
    /// // If you pass --test to Rustdoc, it will even test it for you!
    /// use basegl::display::rendering::Object;
    /// use std::f32::consts::PI;
    ///
    /// let mut object = Object::new();
    /// object.set_rotation(PI, PI, PI);
    /// ```
    pub fn set_rotation(&mut self, roll: f32, pitch: f32, yaw: f32) {
        self.transform.set_rotation(roll, pitch, yaw)
    }
}
