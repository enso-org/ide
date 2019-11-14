use super::Object;

#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct Camera {
    pub object : Object
}

impl Camera {
    pub fn new() -> Self {
        Self { object : Object::new() }
    }
}
