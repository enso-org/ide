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
            Error => Result::Err(Error::missing(id))
        }
    }
}
