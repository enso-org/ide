use crate::prelude::*;

use super::HTMLObject;
use super::Scene;
use crate::data::opt_vec::*;
use crate::system::web::Result;
use crate::system::web::StyleSetter;
use crate::system::web::NodeInserter;
use crate::system::web::NodeRemover;
use crate::data::types::Index;
use crate::math::Vector2;
use std::rc::Rc;

// =====================
// === HTMLSceneData ===
// =====================

#[derive(Debug)]
pub struct HTMLSceneData {
    pub div    : HTMLObject,
    pub camera : HTMLObject
}

impl HTMLSceneData {
    pub fn new(div : HTMLObject, camera : HTMLObject) -> Self {
        Self { div, camera }
    }

    pub fn set_dimensions(&self, dimensions : Vector2<f32>) {
        let width  = format!("{}px", dimensions.x);
        let height = format!("{}px", dimensions.y);
        self.div   .element.set_property_or_panic("width" , &width);
        self.div   .element.set_property_or_panic("height", &height);
        self.camera.element.set_property_or_panic("width" , &width);
        self.camera.element.set_property_or_panic("height", &height);
    }
}

// =================
// === HTMLScene ===
// =================

/// A collection for holding 3D `HTMLObject`s.
#[derive(Shrinkwrap, Debug)]
#[shrinkwrap(mutable)]
pub struct HTMLScene {
    #[shrinkwrap(main_field)]
    pub scene     : Scene,
    pub html_data : Rc<HTMLSceneData>,
    objects       : OptVec<HTMLObject>,
}

impl HTMLScene {
    /// Searches for a HtmlElement identified by id and appends to it.
    pub fn new(dom_id: &str) -> Result<Self> {
        let mut scene = Scene::new(dom_id)?;
        let div       = HTMLObject::new("div")?;
        let camera    = HTMLObject::new("div")?;
        let objects   = default();

        scene .dom    .append_child_or_panic(&div.element);
        div   .element.append_child_or_panic(&camera.element);
        camera.element.set_property_or_panic("transform-style", "preserve-3d");

        let html_data = Rc::new(HTMLSceneData::new(div, camera));

        let html_data_clone = html_data.clone();
        scene.add_resize_callback(Box::new(move |dimensions| {
            html_data_clone.set_dimensions(*dimensions);
        }));

        let dimensions = scene.get_dimensions();
        let mut htmlscene = Self { scene, html_data, objects };
        htmlscene.set_dimensions(dimensions);
        Ok(htmlscene)
    }

    /// Sets the HTMLScene DOM's dimensions.
    pub fn set_dimensions(&mut self, dimensions : Vector2<f32>) {
        self.html_data.set_dimensions(dimensions);
        self.scene.set_dimensions(dimensions);
    }

    /// Moves a HTMLObject to the Scene and returns an index to it.
    pub fn add(&mut self, object: HTMLObject) -> Index {
        self.html_data.camera.element.append_child_or_panic(&object.element);
        self.objects.insert(|_| object)
    }

    /// Removes and retrieves a HTMLObject based on the index provided by
    pub fn remove(&mut self, index: usize) -> Option<HTMLObject> {
        let result = self.objects.remove(index);
        result.iter().for_each(|object| {
           self.html_data.camera.element.remove_child_or_panic(&object.element);
        });
        result
    }

    /// Returns the number of `Object`s in the Scene,
    /// also referred to as its 'length'.
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// Returns true if the Scene contains no `Object`s.
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl<'a> IntoIterator for &'a HTMLScene {
    type Item = &'a HTMLObject;
    type IntoIter = Iter<'a, HTMLObject>;
    fn into_iter(self) -> Self::IntoIter {
        self.objects.into_iter()
    }
}

impl<'a> IntoIterator for &'a mut HTMLScene {
    type Item = &'a mut HTMLObject;
    type IntoIter = IterMut<'a, HTMLObject>;
    fn into_iter(self) -> Self::IntoIter {
        (&mut self.objects).into_iter()
    }
}
