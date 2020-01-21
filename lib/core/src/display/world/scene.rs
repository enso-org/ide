#![allow(missing_docs)]

use crate::prelude::*;

use crate::display::camera::Camera2d;
use crate::display::object::DisplayObjectData;
use crate::system::gpu::data::uniform::UniformScope;



// =============
// === Scene ===
// =============

#[derive(Derivative)]
#[derivative(Debug(bound=""))]
pub struct Scene {
    pub root   : DisplayObjectData,
    pub camera : Camera2d
}


// === Implementation ===

impl Scene {
    pub fn new(logger:Logger, width:f32, height:f32, globals:&UniformScope) -> Self {
        let root    = DisplayObjectData::new(logger.sub("root"));
        let camera  = Camera2d::new(logger.sub("camera"),width,height);
        let uniform = globals.add_or_panic("zoom", 1.0);
        camera.add_zoom_update_callback(move |zoom| uniform.set(zoom));
        Self {root,camera}
    }
}
