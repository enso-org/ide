use crate::system::web::{get_element_by_id, dyn_into, Result};
use web_sys::HtmlElement;

/// A collection for holding 3D `Object`s
pub struct Scene {
    pub container : HtmlElement,
}

impl Scene {
    /// Searches for a HtmlElement identified by id and appends to it
    ///
    /// # Arguments
    /// * id - the HtmlElement container's id
    pub fn new(id: &str) -> Result<Self> {
        let container = dyn_into(get_element_by_id(id)?)?;
        Ok(Self { container })
    }

    /// Gets the HtmlElement container's dimensions
    pub fn get_dimension(&self) -> (f32, f32) {
        (self.container.client_width() as f32, self.container.client_height() as f32)
    }
}
