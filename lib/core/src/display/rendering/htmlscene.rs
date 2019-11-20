use crate::prelude::*;

use super::HTMLObject;
use super::Scene;
use crate::data::opt_vec::OptVec;
use crate::system::web::Result;
use crate::system::web::StyleSetter;

// =================
// === HTMLScene ===
// =================

/// A collection for holding 3D `HTMLObject`s.
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLScene {
    #[shrinkwrap(main_field)]
    pub scene   : Scene,
    pub div     : HTMLObject,
    pub camera  : HTMLObject,
    pub objects : OptVec<HTMLObject>,
}

impl HTMLScene {
    /// Searches for a HtmlElement identified by id and appends to it.
    pub fn new(dom_id: &str) -> Result<Self> {
        let scene    = Scene::new(dom_id)?;
        let view_dim = scene.get_dimensions();
        let width    = format!("{}px", view_dim.x);
        let height   = format!("{}px", view_dim.y);
        let div      = HTMLObject::new("div")?;
        let camera   = HTMLObject::new("div")?;
        let objects  = OptVec::new(); // FIXME: use default()

        scene.container.set_property_or_panic("overflow", "hidden");
        scene.container.append_child(&div.element).expect("Failed to append div"); // FIXME: change to append_child_or_panic
        div.element.append_child(&camera.element).expect("Failed to append camera to HTMLScene"); // FIXME: change to append_child_or_panic
        div   .element.set_property_or_panic("width"  , &width);
        div   .element.set_property_or_panic("height" , &height);
        camera.element.set_property_or_panic("width"  , &width);
        camera.element.set_property_or_panic("height" , &height);

        Ok(Self { scene, div, camera, objects })
    }

    /// Moves a HTMLObject to the Scene and returns an index to it.
    pub fn add(&mut self, object: HTMLObject) -> usize { // FIXME: change usize to a newtype Index
        self.camera.element.append_child(&object.element).expect("append child"); // FIXME: change to append_child_or_panic
        self.objects.insert(|_| object)
    }

    /// Removes and retrieves a HTMLObject based on the index provided by
    pub fn remove(&mut self, index: usize) -> Option<HTMLObject> {
        let result = self.objects.remove(index);
        result.iter().for_each(|object| {
            self.camera.element.remove_child(&object.element).expect("remove child"); // FIXME: change to remove_child_or_panic
        });
        result
    }
    
    // FIXME: docs
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    // FIXME: docs
    pub fn is_empty(&self) -> bool {
        self.objects.len() == 0 // FIXME: OptVec should have method is_empty
    }
}
