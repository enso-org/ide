use super::Object;
use crate::system::web::create_element;
use crate::system::web::dyn_into;
use crate::system::web::Result;
use crate::system::web::Error;
use nalgebra::Vector3;
use web_sys::HtmlElement;


/// A structure for representing a 3D HTMLElement in a `HTMLScene`
#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct HTMLObject {
    #[shrinkwrap(main_field)]
    pub object: Object,
    pub element: HtmlElement,
    pub dimensions: Vector3<f32>,
}

impl HTMLObject {
    /// Creates a HTMLObject from element name
    /// # Example
    /// ```rust,no_run
    /// use basegl::display::rendering::HTMLObject;
    /// let object = HTMLObject::new("div");
    /// ```
    pub fn new(name: &str) -> Result<Self> {
        let element = dyn_into(create_element(name)?)?;
        Ok(Self::from_element(element))
    }

    /// Creates a HTMLObject from a web_sys::HtmlElement
    pub fn from_element(element: HtmlElement) -> Self {
        let style = element.style();
        style.set_property("transform-style", "preserve-3d").expect("transform-style: preserve-3d");
        style.set_property("position", "absolute").expect("position: absolute");
        style.set_property("width", "0px").expect("width: 0px");
        style.set_property("height", "0px").expect("height: 0px");
        Self { object: Default::default(), element, dimensions: Vector3::new(0.0, 0.0, 0.0) }
    }

    /// Creates a HTMLObject from a HTML string
    /// # Example
    /// ```rust,no_run
    /// use basegl::display::rendering::HTMLObject;
    /// let html_string = "<b>hello</b>";
    /// let object = HTMLObject::from_html_string(html_string).expect("valid object");
    /// assert_eq!(object.element.inner_html(), html_string);
    /// ```
    pub fn from_html_string(html: &str) -> Result<Self> {
        let element = create_element("div")?;
        element.set_inner_html(html);
        match element.children().item(0) {
            Some(element) => Ok(Self::from_element(dyn_into(element)?)),
            None => Err(Error::missing("valid HTML")),
        }
    }

    /// Sets the underlying HtmlElement dimension
    pub fn set_dimensions(&mut self, width: f32, height: f32) {
        self.dimensions = Vector3::new(width, height, 0.0);
        let style = self.element.style();
        style.set_property("width", &format!("{}px", width)).expect("set width");
        style.set_property("height", &format!("{}px", height)).expect("set height");
    }

    /// Gets the underlying HtmlElement dimension
    pub fn get_dimensions(&mut self) -> &Vector3<f32> {
        &self.dimensions
    }
}
