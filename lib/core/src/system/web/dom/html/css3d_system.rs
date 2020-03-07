use crate::prelude::*;

use super::Css3dObject;
use crate::display;
use crate::display::world::World;
use crate::system::web::dom::html::Css3dRenderer;
use crate::display::object::DisplayObjectOps;

use logger::Logger;

/// Css3dSystem enables us to create instances of HtmlElement objects in the 3d world.
#[derive(Debug)]
pub struct Css3dSystem {
    pub(super) display_object : display::object::Node,
    pub(super) css3d_renderer : Css3dRenderer,
    pub(super) logger         : Logger
}

impl Css3dSystem {
    /// Creates a new instance of Css3dSystem.
    pub fn new(world:&World) -> Self {
        let scene          = world.scene();
        let css3d_renderer = scene.css3d_renderer();
        css3d_renderer.new_system()
    }

    /// Creates a new instance of Css3dObject.
    pub fn add_child2(&self,object:&Css3dObject) {
        self.add_child(object);
        self.css3d_renderer.new_instance(object);
    }
}

impl From<&Css3dSystem> for display::object::Node {
    fn from(t:&Css3dSystem) -> Self {
        t.display_object.clone_ref()
    }
}
