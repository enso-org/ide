use super::HTMLObject;
use super::Scene;
use crate::data::opt_vec::OptVec;
use crate::system::web::Result;

/// A collection for holding 3D `HTMLObject`s
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLScene {
    #[shrinkwrap(main_field)]
    pub scene: Scene,
    pub div: HTMLObject,
    pub camera: HTMLObject,
    pub objects: OptVec<HTMLObject>,
}

impl HTMLScene {
    /// Searches for a HtmlElement identified by id and appends to it
    ///
    /// # Arguments
    /// * id - the HtmlElement container's id
    pub fn new(id: &str) -> Result<Self> {
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

        Ok(Self { scene, div, camera, objects })
    }

    /// Moves a HTMLObject to the Scene and returns an index to it
    pub fn add(&mut self, object: HTMLObject) -> usize {
        self.camera.element.append_child(&object.element).expect("append child");
        self.objects.insert(|_| object)
    }

    /// Removes and retrieves a HTMLObject based on the index provided by
    /// HTMLScene::add # Example
    /// ```rust,no_run
    /// use basegl::display::rendering::{HTMLScene, HTMLObject};
    /// let mut scene = HTMLScene::new("an_existing_html_element_id").expect("scene");
    /// let object = HTMLObject::new("code").expect("html <code> tag");
    /// let object_id = scene.add(object);
    /// match scene.remove(object_id) {
    ///     Some(object) => println!("We got the code back! :)"),
    ///     None => println!("Omg! Where is my code? :(")
    /// }
    /// ```
    pub fn remove(&mut self, index: usize) -> Option<HTMLObject> {
        if let Some(object) = self.objects.remove(index) {
            self.camera.element.remove_child(&object.element).expect("remove child");
            Some(object)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.len() == 0
    }
}
