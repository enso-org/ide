use crate::system::web::{ Error, Result, document };
use web_sys::Element;

pub struct Scene {
    pub container : Element
}

impl Scene {
    pub fn new(id : &str) -> Result<Self> {
        let document = document()?;
        let container = document.get_element_by_id(id).ok_or(Error::missing(id));
        match container {
            Ok(container) => Ok(Self { container }),
            _error => Result::Err(Error::missing(id))
        }
    }

    pub fn get_dimension(&self) -> (f32, f32) {
        (self.container.client_width() as f32, self.container.client_height() as f32)
    }
}
