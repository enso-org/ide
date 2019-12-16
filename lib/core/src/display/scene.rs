use crate::prelude::*;

use crate::display::symbol::display_object::{Camera, DisplayObjectDescription};
use basegl_system_web::Logger;


// =============
// === Scene ===
// =============

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scene {
    pub root   : DisplayObjectDescription,
    pub camera : Camera
}

// === Implementation ===


impl Scene {
    pub fn new(logger:Logger) -> Self {
        let root   = DisplayObjectDescription::new(logger.sub("root"));
        let camera = Camera::new(logger.sub("camera"));
        Self {root,camera}
    }
}

