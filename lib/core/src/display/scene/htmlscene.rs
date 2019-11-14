use crate::system::web::{Result, Error};
use super::Scene;
use super::HTMLObject;
use crate::data::opt_vec::OptVec;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLScene {
    #[shrinkwrap(main_field)]
    pub scene : Scene,
    pub div : HTMLObject,
    pub objects : OptVec<HTMLObject>
}

impl HTMLScene {
    pub fn new(id : &str) -> Result<Self> {
        let scene = Scene::new(id)?;
        let div = HTMLObject::new("div")?;
        let objects = OptVec::new();

        match scene.container.append_child(&div.element) {
            Ok(_) => Ok(Self {scene, div, objects}),
            Err(_) => Err(Error::missing("div"))
        }
    }

    pub fn add(&mut self, object: HTMLObject) -> usize {
        self.div.element.append_child(&object.element).unwrap();
        self.objects.insert(|_| object)
    }

    pub fn remove(&mut self, index: usize) {
        self.objects.remove(index);
    }
}
