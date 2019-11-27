use crate::system::web::{get_element_by_id, dyn_into, Result};
use web_sys::HtmlElement;
use nalgebra::Vector2;

/// A collection for holding 3D `Object`s.
#[derive(Debug)]
pub struct Scene {
    pub container : HtmlElement,
    dimensions : Vector2<f32>
}

impl Scene {
    /// Searches for a HtmlElement identified by id and appends to it.
    pub fn new(dom_id: &str) -> Result<Self> {
        let container : HtmlElement = dyn_into(get_element_by_id(dom_id)?)?;
        let width  = container.client_width()  as f32;
        let height = container.client_height() as f32;
        let dimensions = Vector2::new(width, height);
        Ok(Self { container, dimensions })
    }

    /// Gets the HtmlElement container's dimensions.
    pub fn get_dimensions(&self) -> Vector2<f32> { self.dimensions }
}
