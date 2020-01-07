use crate::prelude::*;

use crate::display::symbol::geometry::sprite::SpriteSystem;
use crate::display::world::World;
use crate::display::symbol::material::Material;
use crate::display::shape::primitive::shader;
use crate::display::shape::primitive::def::*;
use crate::display::shape::primitive::def::class::Shape;


#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct ShapeSystem {
    pub sprite_system: SpriteSystem
}

impl ShapeSystem {
    pub fn new<S:Shape>(world:&World, shape:&S) -> Self {
        let mut sprite_system = SpriteSystem::new(world);
        sprite_system.set_material(Self::material(shape));
        Self {sprite_system}
    }

    fn material<S:Shape>(shape:&S) -> Material {
        let mut material = Material::new();
        material.add_input("pixel_ratio", 1.0);
        material.add_input("zoom"       , 1.0);
        material.add_input("time"       , 0.0);
        let code = shader::builder::Builder::run(shape);
        material.set_code(code);
        material
    }
}



