use crate::system::web::{Result, Error, document, create_element_as};
use web_sys::HtmlElement;
use std::ops::Deref;
use nalgebra::{Vector3, UnitQuaternion, Translation};

impl Deref for HTMLObject {
    type Target = HtmlElement;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

pub struct HTMLObject {
    element : HtmlElement,
    position : Vector3<f32>,
    rotation : UnitQuaternion<f32>
}

impl HTMLObject {
    pub fn new(name : &str) -> Result<Self> {
        let element = create_element_as(name);
        match element {
            Ok(element) => Ok(Self { element, position : Vector3::new(0.0, 0.0, 0.0), rotation : UnitQuaternion::identity() }),
            Err(_) => Result::Err(Error::missing("element"))
        }
    }
    pub fn set_position(&mut self, x : f32, y : f32, z : f32) {
        self.position = Vector3::new(x, y, z);
    }
    pub fn set_rotation(&mut self, roll : f32, pitch : f32, yaw : f32) {
        self.rotation = UnitQuaternion::from_euler_angles(roll, pitch, yaw);
    }
    pub fn update(&mut self) {
        let transform = self.rotation.to_homogeneous() * Translation::from_vector(self.position).to_homogeneous();
        self.style().set_property("transform", &format!("perspective(400px) matrix3d({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})", transform[0], transform[1], transform[2], transform[3], transform[4], transform[5], transform[6], transform[7], transform[8], transform[9], transform[10], transform[11], transform[12], transform[13], transform[14], transform[15]));
    }
}
