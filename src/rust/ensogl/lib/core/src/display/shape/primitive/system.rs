//! This module defines a "shape system". It is a wrapper over a "sprite system" and it defines
//! the required default material parameters.

use crate::prelude::*;

use super::def;

use crate::display::scene::Scene;
use crate::display::shape::primitive::shader;
use crate::display::symbol::geometry::compound::sprite;
use crate::display::symbol::geometry::Sprite;
use crate::display::symbol::geometry::SpriteSystem;
use crate::display::symbol::material::Material;
use crate::display::symbol::material;
use crate::display;
use crate::system::gpu::data::attribute;
use crate::system::gpu::data::buffer::item::Storable;
use crate::system::gpu::types::*;



// =====================
// === ShapeSystemId ===
// =====================

newtype_prim_no_default_no_display! {
    /// The ID of a user generated shape system.
    ShapeSystemId(std::any::TypeId);
}



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

/// Trait for user defined shape systems. The easiest way to define custom shape system is by using
/// the `define_shape_system` macro.
pub trait ShapeSystemInstance : 'static + CloneRef {
    /// The ID of the shape system.
    fn id() -> ShapeSystemId;
    /// Constructor.
    fn new(scene:&Scene) -> Self;
    /// The [`ShapeSystem`] instance of the user defined shape system.
    fn shape_system(&self) -> &ShapeSystem;
    /// List of shape systems this shape system should always be drawn on above of. See the
    /// [`crate::display::scene::Layers`] documentation to learn more about compile time shapes
    /// ordering relations.
    fn always_above() -> Vec<ShapeSystemId>;
    /// List of shape system this shape system should always be drawn on below of. See the
    /// [`crate::display::scene::Layers`] documentation to learn more about compile time shapes
    /// ordering relations.
    fn always_below() -> Vec<ShapeSystemId>;
}

/// Trait for user defined shape systems. The easiest way to define custom shape system is by using
/// the `define_shape_system` macro.
pub trait StaticShapeSystemInstance : ShapeSystemInstance {
    /// The shape type of this shape system definition.
    type Shape : Shape<System=Self>;
    /// New shape constructor.
    fn new_instance(&self) -> Self::Shape;
}

/// Trait for user defined shape systems. The easiest way to define custom shape system is by using
/// the `define_shape_system` macro.
pub trait DynShapeSystemInstance : ShapeSystemInstance {
    /// The dynamic shape type of this shape system definition.
    type DynamicShape : DynamicShape<System=Self>;
    /// New shape instantiation. Used to bind a shape to a specific scene implementation.
    fn instantiate(&self, shape:&Self::DynamicShape) -> attribute::InstanceIndex;
}

/// Abstraction for every entity which is associated with a shape system (user generated one). For
/// example, all defined shapes are associated with a shape system, and thus they implement this
/// trait.
pub trait KnownShapeSystemId {
    /// The ID of a user defined shape system.
    fn shape_system_id() -> ShapeSystemId;
}



// =============
// === Shape ===
// =============

/// Type for every shape bound to a specific scene and GPU buffers. The easiest way to define such a
/// shape is by using the `define_shape_system` macro.
pub trait Shape : display::Object + CloneRef + Debug + Sized {
    /// The shape system instance this shape belongs to.
    type System : StaticShapeSystemInstance<Shape=Self>;
    /// Accessor for the underlying sprite.
    fn sprite(&self) -> &Sprite;
}


/// Type for every shape which can, but does not have to be bound to a specific scene and GPU
/// buffers. Dynamic shapes can be created freely and will be bound to a scene after being attached
/// as scene children and an update frame event will be emitted.
///
/// Dynamic shapes contain copy of all shape parameters and use them to set up the GPU parameters
/// on bound.
///
/// The easiest way to define such a shape is by using the `define_shape_system` macro.
pub trait DynamicShape : display::Object + CloneRef + Debug + Sized {
    /// The shape system instance this shape belongs to.
    type System : DynShapeSystemInstance<DynamicShape=Self>;
    /// Constructor.
    fn new(logger:impl AnyLogger) -> Self;
    /// Accessor for the underlying sprite, if the shape is initialized.
    fn sprites(&self) -> Vec<Sprite>;

    fn drop_instances(&self);

    fn size(&self) -> &DynamicParam<sprite::Size>;
}


// === Type Families ===

/// Accessor for the `Shape::System` associated type.
pub type ShapeSystemOf<T> = <T as Shape>::System;

/// Accessor for the `Shape::System` associated type.
pub type DynShapeSystemOf<T> = <T as DynamicShape>::System;



// ================
// === ShapeOps ===
// ================

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



// ====================
// === DynamicParam ===
// ====================

/// A dynamic version of shape parameter. In case the shape was initialized and bound to the
/// GPU, the `attribute` will be initialized as well and will point to the right buffer
/// section. Otherwise, changing the parameter will not have any visual effect, however,
/// all the changes will be recorded and applied as soon as the shape will get initialized.
#[derive(Clone,CloneRef,Derivative)]
#[derivative(Default(bound="T::Item:Default"))]
#[derivative(Debug(bound="T::Item:Copy+Debug, T:Debug"))]
#[allow(missing_docs)]
pub struct DynamicParam<T:HasItem> {
    cache      : Rc<Cell<T::Item>>,
    attributes : Rc<RefCell<Vec<T>>>
}

impl<T> DynamicParam<T>
where T:CellProperty, T::Item:Copy {
    // FIXME: move to separate trait in order to not use such names
    pub fn __remove_attributes_bindings(&self) {
        *self.attributes.borrow_mut() = default();
    }

    // FIXME: move to separate trait in order to not use such names
    pub fn __add_attribute_binding
    (&self, attribute:T) {
        attribute.set(self.cache.get());
        self.attributes.borrow_mut().push(attribute);
    }

    /// Set the parameter value.
    pub fn set(&self, value:T::Item) {
        self.cache.set(value);
        for attribute in &*self.attributes.borrow() {
            attribute.set(value)
        }
    }

    /// Get the parameter value.
    pub fn get(&self) -> T::Item {
        self.cache.get()
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
    (
        $(always_above = [$($always_above_1:tt $(::$always_above_2:tt)*),*];)?
        $(always_below = [$($always_below_1:tt $(::$always_below_2:tt)*),*];)?
        ($style:ident : Style $(,$gpu_param : ident : $gpu_param_type : ty)* $(,)?) {$($body:tt)*}
    ) => {
        $crate::_define_shape_system! {
            $(always_above = [$($always_above_1 $(::$always_above_2)*),*];)?
            $(always_below = [$($always_below_1 $(::$always_below_2)*),*];)?
            [$style] ($($gpu_param : $gpu_param_type),*){$($body)*}
        }
    };

    (
        $(always_above = [$($always_above_1:tt $(::$always_above_2:tt)*),*];)?
        $(always_below = [$($always_below_1:tt $(::$always_below_2:tt)*),*];)?
        ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?) {$($body:tt)*}
    ) => {
        $crate::_define_shape_system! {
            $(always_above = [$($always_above_1 $(::$always_above_2)*),*];)?
            $(always_below = [$($always_below_1 $(::$always_below_2)*),*];)?
            [style] ($($gpu_param : $gpu_param_type),*){$($body)*}
        }
    }
}

/// Internal helper for `define_shape_system`.
#[macro_export]
macro_rules! _define_shape_system {
    (
        $(always_above = [$($always_above_1:tt $(::$always_above_2:tt)*),*];)?
        $(always_below = [$($always_below_1:tt $(::$always_below_2:tt)*),*];)?
        [$style:ident]
        ($($gpu_param : ident : $gpu_param_type : ty),* $(,)?)
        {$($body:tt)*}
    ) => {

        // =============
        // === Shape ===
        // =============

        /// An initialized, GPU-bound shape definition. All changed parameters are immediately
        /// reflected in the [`Buffer`] and will be synchronised with GPU before next frame is
        /// drawn.
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



        // ==========================
        // === DynamicShapeParams ===
        // ==========================

        /// Parameters of the [`DynamicShape`].
        #[derive(Clone,CloneRef,Debug,Default)]
        #[allow(missing_docs)]
        pub struct DynamicShapeParams {
            pub size:$crate::display::shape::system::DynamicParam<$crate::display::symbol::geometry::compound::sprite::Size>,
            $(pub $gpu_param:$crate::display::shape::system::DynamicParam<$crate::system::gpu::data::Attribute<$gpu_param_type>>),*
        }



        // ====================
        // === DynamicShape ===
        // ====================

        /// A dynamic version of the [`Shape`]. In case the shape was initialized and bound to the
        /// GPU, the parameters will be initialized as well and will point to the right buffers
        /// sections. Otherwise, changing a parameter will not have any visual effect, however,
        /// all the changes will be recorded and applied as soon as the shape will get initialized.
        #[derive(Clone,CloneRef,Debug)]
        #[allow(missing_docs)]
        pub struct DynamicShape {
            display_object : $crate::display::object::Instance,
            shapes         : Rc<RefCell<Vec<Shape>>>,
            params         : DynamicShapeParams,
        }

        impl Deref for DynamicShape {
            type Target = DynamicShapeParams;
            fn deref(&self) -> &Self::Target {
                &self.params
            }
        }

        impl DynamicShape {
            /// Set the dynamic shape binding.
            pub fn add_shape_binding(&self, shape:Shape) {
                self.display_object.add_child(&shape);
                self.params.size.__add_attribute_binding(shape.sprite.size.clone_ref());
                $(
                    let gpu_param = shape.$gpu_param.clone_ref();
                    self.params.$gpu_param.__add_attribute_binding(gpu_param);
                )*
                self.shapes.borrow_mut().push(shape);
            }
        }

        impl $crate::display::shape::system::DynamicShape for DynamicShape {
            type System = ShapeSystem;

            fn new(logger:impl AnyLogger) -> Self {
                let logger : Logger = Logger::sub(&logger,"dyn_shape");
                let display_object  = $crate::display::object::Instance::new(&logger);
                let shapes          = default();
                let params          = default();
                Self {display_object,shapes,params}
            }

            fn sprites(&self) -> Vec<$crate::display::symbol::geometry::Sprite> {
                self.shapes.borrow().iter().map(|t|t.sprite.clone_ref()).collect()
            }

            fn drop_instances(&self) {
                for shape in mem::take(&mut *self.shapes.borrow_mut()) {
                    self.display_object.remove_child(&shape);
                }
                self.params.size.__remove_attributes_bindings();
                $(self.params.$gpu_param.__remove_attributes_bindings();)*
            }

            fn size(&self) -> &$crate::display::shape::system::DynamicParam<$crate::display::symbol::geometry::compound::sprite::Size> {
                &self.size
            }
        }

        impl $crate::display::Object for DynamicShape {
            fn display_object(&self) -> &$crate::display::object::Instance {
                &self.display_object
            }
        }



        // ============
        // === View ===
        // ============

        /// A view of the defined shape. You can place the view in your objects and it will
        /// automatically initialize on-demand.
        pub type View = $crate::gui::component::ShapeView<DynamicShape>;

        impl $crate::display::shape::KnownShapeSystemId for DynamicShape {
            fn shape_system_id() -> $crate::display::shape::ShapeSystemId {
                ShapeSystem::shape_system_id()
            }
        }



        // ===================
        // === ShapeSystem ===
        // ===================

        /// Shape system allowing the creation of new [`Shape`]s and instantiation of
        /// [`DynamicShape`]s.
        #[derive(Clone,CloneRef,Debug)]
        #[allow(missing_docs)]
        pub struct ShapeSystem {
            pub shape_system : $crate::display::shape::ShapeSystem,
            style_manager    : $crate::display::shape::StyleWatch,
            $(pub $gpu_param : $crate::system::gpu::data::Buffer<$gpu_param_type>),*
        }

        impl $crate::display::shape::ShapeSystemInstance for ShapeSystem {
            fn id() -> $crate::display::shape::ShapeSystemId {
                std::any::TypeId::of::<ShapeSystem>().into()
            }

            fn new(scene:&$crate::display::scene::Scene) -> Self {
                let style_manager = $crate::display::shape::StyleWatch::new(&scene.style_sheet);
                let shape_def     = Self::shape_def(&style_manager);
                let shape_system  = $crate::display::shape::ShapeSystem::new(scene,&shape_def);
                $(
                    let name = stringify!($gpu_param);
                    let val  = $crate::system::gpu::data::default::gpu_default::<$gpu_param_type>();
                    let $gpu_param = shape_system.add_input(name,val);
                )*
                Self {shape_system,style_manager,$($gpu_param),*} . init_refresh_on_style_change()
            }

            fn shape_system(&self) -> &$crate::display::shape::ShapeSystem {
                &self.shape_system
            }

            fn always_above() -> Vec<ShapeSystemId> {
                vec![ $($($always_above_1 $(::$always_above_2)* :: ShapeSystem :: id()),*)? ]
            }
            fn always_below() -> Vec<ShapeSystemId> {
                vec![ $($($always_below_1 $(::$always_below_2)* :: ShapeSystem :: id()),*)? ]
            }
        }

        impl $crate::display::shape::StaticShapeSystemInstance for ShapeSystem {
            type Shape = Shape;

            fn new_instance(&self) -> Self::Shape {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                Shape {sprite, $($gpu_param),*}
            }
        }

        impl $crate::display::shape::DynShapeSystemInstance for ShapeSystem {
            type DynamicShape = DynamicShape;

            fn instantiate(&self, dyn_shape:&Self::DynamicShape)
            -> $crate::system::gpu::data::attribute::InstanceIndex {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                let shape = Shape {sprite, $($gpu_param),*};
                dyn_shape.add_shape_binding(shape);
                id
            }
        }

        impl $crate::display::shape::KnownShapeSystemId for ShapeSystem {
            fn shape_system_id() -> $crate::display::shape::ShapeSystemId {
                std::any::TypeId::of::<ShapeSystem>().into()
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
