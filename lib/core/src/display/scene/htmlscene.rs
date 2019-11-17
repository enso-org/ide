use crate::system::web::{Result};
use super::Scene;
use super::HTMLObject;
use crate::data::opt_vec::OptVec;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLScene {
    #[shrinkwrap(main_field)]
    pub scene : Scene,
    pub div : HTMLObject,
    pub camera : HTMLObject,
    pub objects : OptVec<HTMLObject>
}

impl HTMLScene {
    pub fn new(id : &str) -> Result<Self> {
        let scene = Scene::new(id)?;
        scene.container.style().set_property("overflow", "hidden").expect("overflow: hidden");
        let (width, height) = scene.get_dimension();

        let div = HTMLObject::new("div")?;
        div.element.style().set_property("width", &format!("{}px", width)).expect("set width");
        div.element.style().set_property("height", &format!("{}px", height)).expect("set width");
        scene.container.append_child(&div.element).expect("append div");

        let camera = HTMLObject::new("div")?;
        camera.element.style().set_property("width", &format!("{}px", width)).expect("set width");
        camera.element.style().set_property("height", &format!("{}px", height)).expect("set width");
        div.element.append_child(&camera.element).expect("append camera");
        let objects = OptVec::new();

        Ok(Self {scene, div, camera, objects})
    }

    pub fn add(&mut self, object: HTMLObject) -> usize {
        self.camera.element.append_child(&object.element).expect("append child");
        self.objects.insert(|_| object)
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(object) = self.objects.remove(index) {
            self.camera.element.remove_child(&object.element).expect("remove child");
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }
}
