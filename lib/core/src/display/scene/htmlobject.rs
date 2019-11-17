use crate::system::web::{Result, create_element_as};
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
        let element = create_element_as::<HtmlElement>(name)?;
        Ok(Self::from_element(element))
    }

    pub fn from_element(element : HtmlElement) -> Self {
        let style = element.style();
        style.set_property("transform-style", "preserve-3d").expect("transform-style: preserve-3d");
        style.set_property("position", "absolute").expect("position: absolute");
        style.set_property("width", "0px").expect("width: 0px");
        style.set_property("height", "0px").expect("height: 0px");
        Self { object: Object::new(), element, dimension : Vector3::new(0.0, 0.0, 0.0) }
    }

    // We need to validate html. We can use Result<Node, Error> from element.children[0]. 
    pub fn from_html_string(html : &str) -> Result<Self> {
        let element = create_element_as::<HtmlElement>("div")?;
        element.set_inner_html(html);

        Ok(Self::from_element(element))
    }

    pub fn set_dimension(&mut self, width: f32, height: f32) {
        self.dimension = Vector3::new(width, height, 0.0);
        let style = self.element.style();
        style.set_property("width", &format!("{}px", width)).expect("set width");
        style.set_property("height", &format!("{}px", height)).expect("set height");
    }

    pub fn get_dimension(&mut self) -> &Vector3<f32> {
        &self.dimension
    }
}
