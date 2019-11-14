use crate::system::web::{Result, Error, create_element_as};
use web_sys::HtmlElement;
use super::Object;
use nalgebra::{Vector3};

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLObject {
    #[shrinkwrap(main_field)]
    pub object : Object,
    pub element : HtmlElement,
    pub dimension : Vector3<f32>
}

impl HTMLObject {
    pub fn new(name : &str) -> Result<Self> {
        let element = create_element_as::<HtmlElement>(name);
        match element {
            Ok(element) => {
                let style = element.style();
                style.set_property("transform-style", "preserve-3d").unwrap();
                style.set_property("position", "absolute").unwrap();
                style.set_property("width", "0px").unwrap();
                style.set_property("height", "0px").unwrap();
                Ok(Self { object: Object::new(), element, dimension : Vector3::new(0.0, 0.0, 0.0) })
            },
            Err(_) => Result::Err(Error::missing("element"))
        }
    }

    pub fn set_dimension(&mut self, width: f32, height: f32) {
        self.dimension = Vector3::new(width, height, 0.0);
        let style = self.element.style();
        style.set_property("width", &format!("{}px", width)).unwrap();
        style.set_property("height", &format!("{}px", height)).unwrap();
    }

    pub fn get_dimension(&mut self) -> &Vector3<f32> {
        &self.dimension
    }
}
