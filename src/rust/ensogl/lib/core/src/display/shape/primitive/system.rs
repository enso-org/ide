//! This module defines a "shape system". It is a wrapper over a "sprite system" and it defines
//! the required default material parameters.

use crate::prelude::*;

use super::def;

use crate::display::scene::Scene;
use crate::display::shape::primitive::shader;
use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::material::Material;
use crate::display::symbol::material;
use crate::display;
use crate::system::gpu::data::buffer::item::Storable;
use crate::system::gpu::types::*;



// ===================
// === ShapeSystem ===
// ===================

/// Definition of a shape management system.
///
/// Please note that you would rather not need to use it directly, as it would require manual
/// management of buffer handlers. In order to automate the management, there is
/// `ShapeSystemInstance` and the `define_shape_system` macro.
///
/// Under the hood, it is a specialized version of `SpriteSystem`.
#[allow(missing_docs)]
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct ShapeSystem {
    #[shrinkwrap(main_field)]
    pub sprite_system  : SpriteSystem,
    pub shape          : Rc<RefCell<def::AnyShape>>,
    pub material       : Rc<RefCell<Material>>,
    pub pointer_events : Rc<Cell<bool>>,
}

impl ShapeSystem {
    /// Constructor.
    pub fn new<'t,S,Sh>(scene:S, shape:Sh) -> Self
    where S:Into<&'t Scene>, Sh:Into<def::AnyShape> {
        let shape          = shape.into();
        let sprite_system  = SpriteSystem::new(scene);
        let material       = Rc::new(RefCell::new(Self::surface_material()));
        let pointer_events = Rc::new(Cell::new(true));
        let shape          = Rc::new(RefCell::new(shape));
        let this           = Self {sprite_system,material,pointer_events,shape};
        this.reload_shape();
        this
    }

    // TODO
    // We should handle these attributes in a nicer way. Currently, they are hardcoded here and we
    // use magic to access them in shader builders.
    /// Defines a default material of this system.
    fn surface_material() -> Material {
        let mut material = Material::new();
        material.add_input  ("pixel_ratio"  , 1.0);
        material.add_input  ("z_zoom_1"     , 1.0);
        material.add_input  ("time"         , 0.0);
        material.add_input  ("symbol_id"    , 0);
        material.add_input  ("display_mode" , 0);
        material.add_output ("id"           , Vector4::<f32>::zero());
        material
    }

    /// Enables or disables pointer events on this shape system. All shapes of a shape system which
    /// has pointer events disabled would be completely transparent for the mouse (they would pass
    /// through all mouse events).
    pub fn set_pointer_events(&self, val:bool) {
        self.pointer_events.set(val);
        self.reload_shape();
    }

    /// Replaces the shape definition.
    pub fn set_shape<S:Into<def::AnyShape>>(&self, shape:S) {
        let shape = shape.into();
        *self.shape.borrow_mut() = shape;
        self.reload_shape();
    }

    /// Generates the shape again. It is used after some parameters are changed, like setting new
    /// `pointer_events` value.
    fn reload_shape(&self) {
        let code = shader::builder::Builder::run(&*self.shape.borrow(),self.pointer_events.get());
        self.material.borrow_mut().set_code(code);
        self.reload_material();
    }

    /// Define a new shader input.
    pub fn add_input<T:material::Input + Storable>(&self, name:&str, t:T) -> Buffer<T>
    where AnyBuffer: From<Buffer<T>> {
        self.material.borrow_mut().add_input(name,t);
        let buffer = self.sprite_system.symbol().surface().instance_scope().add_buffer(name);
        self.reload_material();
        buffer
    }

    /// Regenerate the shader with the current material.
    fn reload_material(&self) {
        self.sprite_system.set_material(&*self.material.borrow());
    }
}

impl display::Object for ShapeSystem {
    fn display_object(&self) -> &display::object::Instance {
        self.sprite_system.display_object()
    }
}



// ===========================
// === ShapeSystemInstance ===
// ===========================

/// Type for every `ShapeSystem` with automatic buffer management. The easiest way to define such a
/// shape system instance is by using the `define_shape_system` macro.
pub trait ShapeSystemInstance : 'static + CloneRef {
    /// The shape type of this shape system definition.
    type Shape : Shape<System=Self>;
    /// Constructor.
    fn new(scene:&Scene) -> Self;
    /// New shape constructor.
    fn new_instance(&self) -> Self::Shape;
}

pub trait DynShapeSystemInstance : 'static + CloneRef {
    /// The shape type of this shape system definition.
    type DynShape : DynShape<System=Self>;
    /// Constructor.
    fn new(scene:&Scene) -> Self;
    /// New shape constructor.
    fn instantiate(&self, shape:&Self::DynShape);
}



// =============
// === Shape ===
// =============

/// Type for every shape with automatic attribute management. The easiest way to define such a
/// shape is by using the `define_shape_system` macro.
pub trait Shape : display::Object + CloneRef + Debug + Sized {
    /// The shape system instance this shape belongs to.
    type System : ShapeSystemInstance<Shape=Self>;
    /// Accessor for the underlying sprite.
    fn sprite(&self) -> &Sprite;
}


pub trait DynShape : display::Object + CloneRef + Debug + Sized + Default {
    /// The shape system instance this shape belongs to.
    type System : DynShapeSystemInstance<DynShape=Self>;
}


/// Accessor for the `Shape::System` associated type.
pub type ShapeSystemOf<T> = <T as Shape>::System;

/// Accessor for the `Shape::System` associated type.
pub type DynShapeSystemOf<T> = <T as DynShape>::System;

/// Additional operations implemented for all structures implementing `Shape`.
pub trait ShapeOps {
    /// Check if given mouse-event-target means this shape.
    fn is_this_target(&self, target:display::scene::PointerTarget) -> bool;
}

impl<T:Shape> ShapeOps for T {
    fn is_this_target(&self, target:display::scene::PointerTarget) -> bool {
        self.sprite().is_this_target(target)
    }
}



// ==============
// === Macros ===
// ==============

/// Defines 'Shape' and 'ShapeSystem' structures. The generated Shape is a newtype for `Sprite`
/// and the shader attributes. The generated 'ShapeSystem' is a newtype for the `ShapeSystem` and
/// the required buffer handlers.
#[macro_export]
macro_rules! define_shape_system {
    ( ($style:ident : Style $(,)?) {$($body:tt)*} ) => {
        $crate::_define_shape_system! { [$style] (){$($body)*} }
    };

    ( ($style:ident : Style, $($gpu_param : ident : $gpu_param_type : ty),* $(,)?) {$($body:tt)*} ) => {
        $crate::_define_shape_system! { [$style] ($($gpu_param : $gpu_param_type),*){$($body)*} }
    };

    ( ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?) {$($body:tt)*} ) => {
        $crate::_define_shape_system! { [style] ($($gpu_param : $gpu_param_type),*){$($body)*} }
    }
}

/// Internal helper for `define_shape_system`.
#[macro_export]
macro_rules! _define_shape_system {
    (
        [$style:ident]
        ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?)
        {$($body:tt)*}
    ) => {

        // =============
        // === Shape ===
        // =============

        /// Shape definition.
        #[derive(Clone,CloneRef,Debug)]
        #[allow(missing_docs)]
        pub struct Shape {
            pub sprite : $crate::display::symbol::geometry::Sprite,
            $(pub $gpu_param : $crate::system::gpu::data::Attribute<$gpu_param_type>),*
        }

        impl Deref for Shape {
            type Target = $crate::display::symbol::geometry::Sprite;
            fn deref(&self) -> &Self::Target {
                &self.sprite
            }
        }

        impl $crate::display::shape::system::Shape for Shape {
            type System = ShapeSystem;
            fn sprite(&self) -> &$crate::display::symbol::geometry::Sprite {
                &self.sprite
            }
        }

        impl $crate::display::Object for Shape {
            fn display_object(&self) -> &$crate::display::object::Instance {
                self.sprite.display_object()
            }
        }



        // ================
        // === DynShape ===
        // ================

        #[derive(Clone,CloneRef,Derivative)]
        #[derivative(Default(bound="T::Item:Default"))]
        #[derivative(Debug(bound="T::Item:Copy+Debug, T:Debug"))]
        #[allow(missing_docs)]
        pub struct DynParam<T:HasItem> {
            cache     : Rc<Cell<T::Item>>,
            attribute : Rc<RefCell<Option<T>>>
        }

        impl<T> DynParam<T>
        where T:CellProperty, T::Item:Copy {
            fn set_attribute_binding
            (&self, attribute:Option<T>) {
                if let Some(attr) = &attribute { attr.set(self.cache.get()) }
                *self.attribute.borrow_mut() = attribute;
            }

            /// Set the parameter value;
            pub fn set(&self, value:T::Item) {
                self.cache.set(value);
                if let Some(attr) = &*self.attribute.borrow() { attr.set(value) }
            }

            /// Get the parameter value;
            pub fn get(&self) -> T::Item {
                self.cache.get()
            }
        }

        /// Dynamic shape parameters
        #[derive(Clone,CloneRef,Debug,Default)]
        #[allow(missing_docs)]
        pub struct DynShapeParams {
            pub size : DynParam<$crate::display::symbol::geometry::compound::sprite::Size>,
            $(pub $gpu_param : DynParam<$crate::system::gpu::data::Attribute<$gpu_param_type>>),*
        }

        /// Shape definition.
        #[derive(Clone,CloneRef,Debug)]
        #[allow(missing_docs)]
        pub struct DynShape {
            display_object : $crate::display::object::Instance,
            shape          : Rc<RefCell<Option<Shape>>>,
            params         : DynShapeParams,
        }

        impl Deref for DynShape {
            type Target = DynShapeParams;
            fn deref(&self) -> &Self::Target {
                &self.params
            }
        }

        impl DynShape {
            /// Constructor.
            pub fn new() -> Self {
                let logger : Logger = Logger::new("dyn_shape");
                let display_object  = $crate::display::object::Instance::new(&logger);
                let shape           = default();
                let params          = default();
                Self {display_object,shape,params}
            }

            /// Set the dynamic shape binding.
            pub fn set_shape_binding(&self, shape:Option<Shape>) {
                if let Some(current_shape) = &*self.shape.borrow() {
                    current_shape.unset_parent();
                }
                if let Some(shape) = &shape {
                    self.display_object.add_child(&shape);
                    self.params.size.set_attribute_binding(Some(shape.sprite.size.clone_ref()));
                    $(self.params.$gpu_param.set_attribute_binding(Some(shape.$gpu_param.clone_ref()));)*
                } else {
                    self.params.size.set_attribute_binding(None);
                    $(self.params.$gpu_param.set_attribute_binding(None);)*
                }
                *self.shape.borrow_mut() = shape;
            }
        }

        impl Default for DynShape {
            fn default() -> Self {
                Self::new()
            }
        }

        impl $crate::display::shape::system::DynShape for DynShape {
            type System = ShapeSystem;
        }

        impl $crate::display::Object for DynShape {
            fn display_object(&self) -> &$crate::display::object::Instance {
                &self.display_object
            }
        }



        // ==============
        // === System ===
        // ==============

        /// Shape system definition.
        #[derive(Clone,CloneRef,Debug)]
        #[allow(missing_docs)]
        pub struct ShapeSystem {
            pub shape_system : $crate::display::shape::ShapeSystem,
            style_manager    : $crate::display::shape::StyleWatch,
            $(pub $gpu_param : $crate::system::gpu::data::Buffer<$gpu_param_type>),*
        }

        impl $crate::display::shape::ShapeSystemInstance for ShapeSystem {
            type Shape = Shape;

            fn new(scene:&$crate::display::scene::Scene) -> Self {
                let style_manager = $crate::display::shape::StyleWatch::new(&scene.style_sheet);
                let shape_system  = $crate::display::shape::ShapeSystem::new(scene,&Self::shape_def(&style_manager));
                $(
                    let name       = stringify!($gpu_param);
                    let value      = $crate::system::gpu::data::default::gpu_default::<$gpu_param_type>();
                    let $gpu_param = shape_system.add_input(name,value);
                )*
                Self {shape_system,style_manager,$($gpu_param),*} . init_refresh_on_style_change()
            }

            fn new_instance(&self) -> Self::Shape {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                Shape {sprite, $($gpu_param),*}
            }
        }

        impl $crate::display::shape::DynShapeSystemInstance for ShapeSystem {
            type DynShape = DynShape;

            // FIXME: Duplicated (^^^)
            fn new(scene:&$crate::display::scene::Scene) -> Self {
                let style_manager = $crate::display::shape::StyleWatch::new(&scene.style_sheet);
                let shape_system  = $crate::display::shape::ShapeSystem::new(scene,&Self::shape_def(&style_manager));
                $(
                    let name       = stringify!($gpu_param);
                    let value      = $crate::system::gpu::data::default::gpu_default::<$gpu_param_type>();
                    let $gpu_param = shape_system.add_input(name,value);
                )*
                Self {shape_system,style_manager,$($gpu_param),*} . init_refresh_on_style_change()
            }

            fn instantiate(&self, dyn_shape:&Self::DynShape) {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                let shape = Shape {sprite, $($gpu_param),*};
                dyn_shape.set_shape_binding(Some(shape));
            }
        }

        impl ShapeSystem {
            fn init_refresh_on_style_change(self) -> Self {
                let shape_system  = self.shape_system.clone_ref();
                let style_manager = self.style_manager.clone_ref();
                self.style_manager.set_on_style_change(move || {
                    shape_system.set_shape(&Self::shape_def(&style_manager));
                });
                self
            }

            /// The canvas shape definition.
            pub fn shape_def
            (__style_watch__:&$crate::display::shape::StyleWatch)
            -> $crate::display::shape::primitive::def::AnyShape {
                #[allow(unused_imports)]
                use $crate::display::style::data::DataMatch; // Operations styles.

                __style_watch__.reset();
                let $style  = __style_watch__;
                // Silencing warnings about not used style.
                let _unused = &$style;
                $(
                    let $gpu_param : $crate::display::shape::primitive::def::Var<$gpu_param_type> =
                        concat!("input_",stringify!($gpu_param)).into();
                    // Silencing warnings about not used shader input variables.
                    let _unused = &$gpu_param;
                )*
                $($body)*
            }
        }
    };
}
