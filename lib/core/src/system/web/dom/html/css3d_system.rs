use crate::prelude::*;

use super::Css3dObject;
use crate::display::scene::Scene;
use crate::display::world;
use crate::display::object::DisplayObjectData;
use crate::display::object::DisplayObjectOps;
use crate::system::web::Result;

use logger::Logger;

pub struct Css3dSystem {
    display_object : DisplayObjectData,
    scene          : Scene,
    logger         : Logger
}

impl Css3dSystem {
    pub fn new() -> Self {
        let scene          = world::get_scene();
        let css3d_renderer = scene.css3d_renderer();
        let logger         = scene.css3d_renderer().logger().sub("Css3dSystem");
        let display_object = DisplayObjectData::new(&logger);
        Self{display_object,scene,logger}
    }

    pub fn new_instance<S:AsRef<str>>(&self, dom_name:S) -> Result<Css3dObject> {
        self.scene.css3d_renderer().new_instance(dom_name, self.into())
    }
}

impl From<&Css3dSystem> for DisplayObjectData {
    fn from(t:&Css3dSystem) -> Self {
        t.display_object.clone_ref()
    }
}