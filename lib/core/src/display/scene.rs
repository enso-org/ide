use crate::prelude::*;

use crate::display::symbol::display_object::Camera;
use basegl_system_web::Logger;


// =============
// === Scene ===
// =============

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scene {
    pub camera: Camera
}

// === Implementation ===


impl Scene {
    pub fn new(logger:Logger) -> Self {
        let camera = Camera::new(logger.sub("camera"));
        Self {camera}
    }
}

