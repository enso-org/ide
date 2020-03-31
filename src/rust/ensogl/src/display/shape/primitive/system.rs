//! This module defines a "shape system". It is a wrapper over a "sprite system" and it defines
//! the required default material parameters.

use crate::prelude::*;

use super::def;

use crate::display;
use crate::display::shape::primitive::shader;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::material;
use crate::display::symbol::material::Material;
use crate::display::scene::Scene;
use crate::system::gpu::types::*;
use crate::display::object::traits::*;
use crate::system::gpu::data::buffer::item::Storable;



// ===================
// === ShapeSystem ===
// ===================

/// Definition of a shape management system.
///
/// Please note that you would rather not need to use it
/// directly, as it would require manual management of buffer handlers. In order to automate the
/// management, there is `ShapeSystemInstance` and the `define_shape_system` macro.
///
/// Under the hood, it is a specialized version of `SpriteSystem`.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct ShapeSystem {
    #[shrinkwrap(main_field)]
    pub sprite_system : SpriteSystem,
    pub material      : Rc<RefCell<Material>>,
}

impl ShapeSystem {
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

impl<'t> From<&'t ShapeSystem> for &'t display::object::Node {
    fn from(shape_system:&'t ShapeSystem) -> Self {
        shape_system.sprite_system.display_object()
    }
}






pub trait ShapeSystemInstance : 'static + CloneRef {
    type Shape : Shape<System=Self>;
    fn new(scene:&Scene) -> Self;
    fn new_instance(&self) -> Self::Shape;
}

pub trait Shape : Debug + Sized {
    type System : ShapeSystemInstance<Shape=Self>;
    fn sprite(&self) -> &Sprite;
}

pub type ShapeSystemOf<T> = <T as Shape>::System;


#[macro_export]
macro_rules! define_shape_system {
    (
        ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?)
        {$($body:tt)*}
    ) => {

        // =============
        // === Shape ===
        // =============

        #[derive(Clone,Debug)]
        pub struct Shape {
            pub sprite : Sprite,
            $(pub $gpu_param : Attribute<$gpu_param_type>),*
        }

        impl $crate::display::shape::system::Shape for Shape {
            type System = ShapeSystem;
            fn sprite(&self) -> &Sprite {
                &self.sprite
            }
        }

        impl<'t> From<&'t Shape> for &'t display::object::Node {
            fn from(t:&'t Shape) -> Self {
                &t.sprite.display_object()
            }
        }


        // ==============
        // === System ===
        // ==============

        #[derive(Clone,CloneRef,Debug)]
        pub struct ShapeSystem {
            pub shape_system : $crate::display::shape::ShapeSystem,
            $(pub $gpu_param : Buffer<$gpu_param_type>),*
        }

        impl $crate::display::shape::ShapeSystemInstance for ShapeSystem {
            type Shape = Shape;

            fn new(scene:&Scene) -> Self {
                let shape_system = $crate::display::shape::ShapeSystem::new(scene,&Self::shape_def());
                $(let $gpu_param = shape_system.add_input(stringify!($gpu_param),$crate::system::gpu::data::default::gpu_default::<$gpu_param_type>());)*
                Self {shape_system,$($gpu_param),*}
            }

            fn new_instance(&self) -> Self::Shape {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                Shape {sprite, $($gpu_param),*}
            }
        }

        impl ShapeSystem {
            pub fn shape_def() -> AnyShape {
                $($body)*
            }
        }
    };
}
