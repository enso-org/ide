//! This module defines a "shape system". It is a wrapper over a "sprite system" and it defines
//! the required default material parameters.

use crate::prelude::*;

use super::def::*;
use super::def;

use crate::display;
use crate::display::shape::primitive::shader;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::material;
use crate::display::symbol::material::Material;
use crate::display::world::World;
use crate::display::scene::Scene;
use crate::system::gpu::types::*;
use crate::display::object::traits::*;
use crate::system::gpu::data::buffer::item::Storable;


// ===================
// === ShapeSystemDefinition ===
// ===================

/// Defines a system containing shapes. It is a specialized `SpriteSystem` version.
#[derive(Clone,Debug,Shrinkwrap)]
pub struct ShapeSystemDefinition {
    /// The underlying `SpriteSystem`.
    #[shrinkwrap(main_field)]
    pub sprite_system : SpriteSystem,
    material          : Rc<RefCell<Material>>,
}

impl CloneRef for ShapeSystemDefinition {
    fn clone_ref(&self) -> Self {
        let sprite_system = self.sprite_system.clone_ref();
        let material      = self.material.clone_ref();
        Self {sprite_system,material}
    }
}

impl ShapeSystemDefinition {
    /// Constructor.
    pub fn new<'t,S,Sh:def::Shape>(scene:S, shape:&Sh) -> Self
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
    pub fn set_shape<S:def::Shape>(&self, shape:&S) {
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

impl<'t> From<&'t ShapeSystemDefinition> for &'t display::object::Node {
    fn from(shape_system:&'t ShapeSystemDefinition) -> Self {
        shape_system.sprite_system.display_object()
    }
}



#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[clone_ref(bound="Params:CloneRef")]
pub struct ShapeWrapper<Params> {
    #[shrinkwrap(main_field)]
    pub params : Params,
    pub sprite : Sprite,
}


pub trait ShapeSystem {
    type ShapeDefinition;
    fn new(scene:&Scene) -> Self;
    fn new_instance(&self) -> Shape<Self>;
}

pub type ShapeDefinition<T> = <T as ShapeSystem>::ShapeDefinition;

pub type Shape<T> = ShapeWrapper<ShapeDefinition<T>>;



#[macro_export]
macro_rules! shape {
    (
        ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?)
        {$($body:tt)*}
    ) => {

        // =============
        // === Shape ===
        // =============

        #[derive(Clone,Debug)]
        pub struct ThisShapeDefinition {
            $(pub $gpu_param : Attribute<$gpu_param_type>),*
        }


        // ==============
        // === System ===
        // ==============

        #[derive(Clone,CloneRef,Debug)]
        pub struct System {
            pub shape_system : $crate::display::shape::ShapeSystemDefinition,
            $(pub $gpu_param : Buffer<$gpu_param_type>),*
        }

        impl ShapeSystem for System {
            type ShapeDefinition = ThisShapeDefinition;

            fn new(scene:&Scene) -> Self {
                let shape_system = $crate::display::shape::ShapeSystemDefinition::new(scene,&Self::shape_def());
                $(let $gpu_param = shape_system.add_input(stringify!($gpu_param),default::<$gpu_param_type>());)*
                Self {shape_system,$($gpu_param),*}
            }

            fn new_instance(&self) -> $crate::display::shape::primitive::system::Shape<Self> {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                let params = ThisShapeDefinition {$($gpu_param),*};
                ShapeWrapper {sprite,params}

            }
        }

        impl System {
            pub fn shape_def() -> AnyShape {
                $($body)*
            }


        }
    };
}