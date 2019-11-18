use crate::system::web::{get_element_by_id_as, Result};
use web_sys::HtmlElement;

pub struct Scene {
    pub container: HtmlElement,
}

impl Scene {
    pub fn new(id: &str) -> Result<Self> {
        let container = get_element_by_id_as::<HtmlElement>(id)?;
        Ok(Self { container })
    }

    pub fn get_dimension(&self) -> (f32, f32) {
        (self.container.client_width() as f32, self.container.client_height() as f32)
    }
}
