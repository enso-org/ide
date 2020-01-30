use crate::prelude::*;

use super::Css3dObject;
use crate::display::world;
use crate::display::object::DisplayObjectData;
use crate::system::web::Result;
use crate::system::web::dom::html::Css3dRenderer;

use logger::Logger;

/// Css3dSystem enables us to create instances of HtmlElement objects in the 3d world.
#[derive(Debug)]
pub struct Css3dSystem {
    display_object : DisplayObjectData,
    css3d_renderer : Css3dRenderer,
    logger         : Logger
}

impl Css3dSystem {
    /// Creates a new instance of Css3dSystem.
    pub fn new() -> Self {
        let scene          = world::get_scene();
        let css3d_renderer = scene.css3d_renderer();
        let logger         = css3d_renderer.logger().sub("Css3dSystem");
        let display_object = DisplayObjectData::new(&logger);
        Self{display_object,css3d_renderer,logger}
    }

    /// Creates a new instance of Css3dObject.
    pub fn new_instance<S:AsRef<str>>(&self, dom_name:S) -> Result<Css3dObject> {
        self.css3d_renderer.new_instance(dom_name, self.into())
    }
}

impl From<&Css3dSystem> for DisplayObjectData {
    fn from(t:&Css3dSystem) -> Self {
        t.display_object.clone_ref()
    }
}