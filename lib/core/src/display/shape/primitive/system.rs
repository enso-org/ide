use crate::prelude::*;

use crate::display::symbol::geometry::sprite::SpriteSystem;
use crate::display::world::World;
use crate::display::symbol::material::Material;
use crate::display::shape::primitive::shader;
use crate::display::shape::primitive::def::*;


#[derive(Shrinkwrap)]
#[shrinkwrap(mutable)]
pub struct ShapeSystem {
    pub sprite_system: SpriteSystem
}

impl ShapeSystem {
    pub fn new(world:&World) -> Self {
        let mut sprite_system = SpriteSystem::new(world);
        sprite_system.set_material(Self::material());
        Self {sprite_system}
    }

    fn material() -> Material {
        let mut material = Material::new();

        let s1 = Circle(10.0);
        let s2 = s1.translate(7.0,0.0);
        let s3 = &s2 + &s2;

        let code = shader::builder::Builder::run(&s3);
        material.set_code(code);
        material
    }
}



