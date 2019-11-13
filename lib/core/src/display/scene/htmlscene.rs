use crate::system::web::{Result, Error};
use web_sys::{Element};
use super::Scene;
use std::ops::Deref;
use crate::display::scene::HTMLObject;

impl Deref for HTMLScene {
    type Target = Scene;
    fn deref(&self) -> &Self::Target {
        &self.scene
    }
}

pub struct HTMLScene {
    scene : Scene,
    div : HTMLObject,
    objects : Vec<HTMLObject>
}

impl HTMLScene {
    pub fn new(id : &str) -> Result<Self> {
        let scene = Scene::new(id)?;

        let div = HTMLObject::new("div")?;
        let objects = Vec::new();

        match scene.container.append_child(&div) {
            Ok(_) => Ok(Self {scene, div, objects}),
            Err(_) => Err(Error::missing("div"))
        }
    }

    pub fn add(&mut self, object: HTMLObject) {
        object.style().set_property("position", "absolute");
        self.div.append_child(&object).unwrap();
        self.objects.push(object);
    }

    pub fn update(&mut self) {
        for object in &mut self.objects {
            object.update();
        }
    }
}
