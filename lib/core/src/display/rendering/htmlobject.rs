use super::Object;
use crate::system::web::{create_element_as, Result};
use nalgebra::Vector3;
use web_sys::HtmlElement;

/// A structure for representing a 3D HTMLElement in a `HTMLScene`
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLObject {
    #[shrinkwrap(main_field)]
    pub object: Object,
    pub element: HtmlElement,
    pub dimension: Vector3<f32>,
}

impl HTMLObject {
    /// Creates a HTMLObject from element name
    /// # Example
    /// ```rust,no_run
    /// use basegl::display::rendering::HTMLObject;
    /// let object = HTMLObject::new("div");
    /// ```
    pub fn new(name: &str) -> Result<Self> {
        let element = create_element_as::<HtmlElement>(name)?;
        Ok(Self::from_element(element))
    }

    /// Creates a HTMLObject from a web_sys::HtmlElement
    pub fn from_element(element: HtmlElement) -> Self {
        let style = element.style();
        style.set_property("transform-style", "preserve-3d").expect("transform-style: preserve-3d");
        style.set_property("position", "absolute").expect("position: absolute");
        style.set_property("width", "0px").expect("width: 0px");
        style.set_property("height", "0px").expect("height: 0px");
        Self { object: Object::new(), element, dimension: Vector3::new(0.0, 0.0, 0.0) }
    }

    /// Creates a HTMLObject from a HTML string
    /// # Example
    /// ```rust,no_run
    /// use basegl::display::rendering::HTMLObject;
    /// let html_string = "<b>hello</b>";
    /// let object = HTMLObject::from_html_string(html_string).expect("valid object");
    /// assert_eq!(object.element.inner_html(), html_string);
    /// ```
    // We need to validate html. We can use Result<Node, Error> from
    // element.children[0].
    pub fn from_html_string(html: &str) -> Result<Self> {
        let element = create_element_as::<HtmlElement>("div")?;
        element.set_inner_html(html);

        Ok(Self::from_element(element))
    }

    /// Sets the underlying HtmlElement dimension
    pub fn set_dimension(&mut self, width: f32, height: f32) {
        self.dimension = Vector3::new(width, height, 0.0);
        let style = self.element.style();
        style.set_property("width", &format!("{}px", width)).expect("set width");
        style.set_property("height", &format!("{}px", height)).expect("set height");
    }

    /// Gets the underlying HtmlElement dimension
    pub fn get_dimension(&mut self) -> &Vector3<f32> {
        &self.dimension
    }
}
