//! This module defines a "shape system". It is a wrapper over a "sprite system" and it defines
//! the required default material parameters.

use crate::prelude::*;

use super::def::*;

use crate::display;
use crate::display::shape::primitive::shader;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::material;
use crate::display::symbol::material::Material;
use crate::display::world::World;
use crate::display::scene::Scene;
use crate::system::gpu::types::*;
use crate::display::object::traits::*;
use crate::system::gpu::data::buffer::item::Storable;


// ===================
// === ShapeSystem ===
// ===================

/// Defines a system containing shapes. It is a specialized `SpriteSystem` version.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct ShapeSystem {
    /// The underlying `SpriteSystem`.
    #[shrinkwrap(main_field)]
    pub sprite_system : SpriteSystem,
    material          : Rc<RefCell<Material>>,
}

impl CloneRef for ShapeSystem {
    fn clone_ref(&self) -> Self {
        let sprite_system = self.sprite_system.clone_ref();
        let material      = self.material.clone_ref();
        Self {sprite_system,material}
    }
}

impl ShapeSystem {
    /// Constructor.
    pub fn new<'t,S,Sh:Shape>(scene:S, shape:&Sh) -> Self
    where S : Into<&'t Scene> {
        let sprite_system = SpriteSystem::new(scene);
        let material      = Rc::new(RefCell::new(Self::surface_material()));
        let this          = Self {sprite_system,material};
        this.set_shape(shape);
        this
    }

    /// Defines a default material of this system.
    fn surface_material() -> Material {
        let mut material = Material::new();
        material.add_input  ("pixel_ratio"  , 1.0);
        material.add_input  ("zoom"         , 1.0);
        material.add_input  ("time"         , 0.0);
        material.add_input  ("symbol_id"    , 0);
        material.add_input  ("display_mode" , 0);
        material.add_output ("id"           , Vector4::<u32>::new(0,0,0,0));
        material
    }

    /// Replaces the shape definition.
    pub fn set_shape<S:Shape>(&self, shape:&S) {
        let code = shader::builder::Builder::run(shape);
        self.material.borrow_mut().set_code(code);
        self.reload_material();
    }

    pub fn add_input<T:material::Input + Storable>(&self, name:&str, t:T) -> Buffer<T>
    where AnyBuffer: From<Buffer<T>> {
        self.material.borrow_mut().add_input(name,t);
        let buffer = self.sprite_system.symbol().surface().instance_scope().add_buffer(name);
        self.reload_material();
        buffer
    }

    fn reload_material(&self) {
        self.sprite_system.set_material(&*self.material.borrow());
    }
}

impl<'t> From<&'t ShapeSystem> for &'t display::object::Node {
    fn from(shape_system:&'t ShapeSystem) -> Self {
        shape_system.sprite_system.display_object()
    }
}
