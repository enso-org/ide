
use crate::prelude::*;

use ensogl::control::callback::CallbackMut1;
use ensogl::data::color::Srgba;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::display::{Sprite, Attribute};
use ensogl::math::Vector2;
use ensogl::math::Vector3;
use logger::Logger;
use std::any::TypeId;
use enso_prelude::std_reexports::fmt::{Formatter, Error};
use ensogl::animation::physics::inertia::DynInertiaSimulator;
use enso_frp;
use enso_frp as frp;
use enso_frp::frp;
use enso_frp::core::node::class::EventEmitterPoly;
use ensogl::display::{AnyBuffer,Buffer};
use ensogl::data::color::*;
use ensogl::display::shape::*;
use ensogl::display::shape::primitive::system::ShapeSystemDefinition;
use ensogl::display::world::World;
use ensogl::display::scene::{Scene,Component,MouseTarget,ShapeSystem,ShapeWrapper,ComponentSystem,ShapeDefinition,Shape};






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
            pub shape_system : ShapeSystemDefinition,
            $(pub $gpu_param : Buffer<$gpu_param_type>),*
        }

        impl ShapeSystem for System {
            type ShapeDefinition = ThisShapeDefinition;

            fn new(scene:&Scene) -> Self {
                let shape_system = ShapeSystemDefinition::new(scene,&Self::shape_def());
                $(let $gpu_param = shape_system.add_input(stringify!($gpu_param),default::<$gpu_param_type>());)*
                Self {shape_system,$($gpu_param),*}
            }

            fn new_instance(&self) -> Shape<Self> {
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

macro_rules! component {
    (
        $name:ident {
            $($field:ident : $field_type:ty),* $(,)?
        }

        Shape ($($shape_field:ident : $shape_field_type:ty),* $(,)?) {
            $($shape_body:tt)*
        }
    ) => {
        pub type $name = ComponentWrapperX<Definition>;

        #[derive(Debug,Clone,CloneRef)]
        pub struct Definition {
            $(pub $field : $field_type),*
        }

        impl Component for Definition {
            type ComponentSystem = System;
        }

        shape! { ($($shape_field : $shape_field_type),*) { $($shape_body)* } }

    };
}



pub mod icons {
    use super::*;

    pub fn history() -> AnyShape {
        let radius_diff    = 0.5.px();
        let corners_radius = 2.0.px();
        let width_diff     = &corners_radius * 3.0;
        let offset         = 2.px();
        let width          = 32.px();
        let height         = 16.px();
        let persp_diff1    = 6.px();

        let width2          = &width  - &width_diff;
        let width3          = &width2 - &width_diff;
        let corners_radius2 = &corners_radius  - &radius_diff;
        let corners_radius3 = &corners_radius2 - &radius_diff;
        let persp_diff2     = &persp_diff1 * 2.0;

        let rect1 = Rect((&width ,&height)).corners_radius(&corners_radius);
        let rect2 = Rect((&width2,&height)).corners_radius(&corners_radius2).translate_y(&persp_diff1);
        let rect3 = Rect((&width3,&height)).corners_radius(&corners_radius3).translate_y(&persp_diff2);

        let rect3 = rect3 - rect2.translate_y(&offset);
        let rect2 = rect2 - rect1.translate_y(&offset);

        let rect1 = rect1.fill(Srgba::new(0.26, 0.69, 0.99, 1.00));
        let rect2 = rect2.fill(Srgba::new(0.26, 0.69, 0.99, 0.6));
        let rect3 = rect3.fill(Srgba::new(0.26, 0.69, 0.99, 0.4));

        let icon = (rect3 + rect2 + rect1).translate_y(-persp_diff2/2.0);
        icon.into()
    }
}

pub fn ring_angle<R,W,A>(inner_radius:R, width:W, angle:A) -> AnyShape
    where R : Into<Var<Distance<Pixels>>>,
          W : Into<Var<Distance<Pixels>>>,
          A : Into<Var<Angle<Radians>>> {
    let inner_radius = inner_radius.into();
    let width        = width.into();
    let angle        = angle.into();

    let angle2  = &angle / 2.0;
    let radius  = &width / 2.0;
    let inner   = Circle(&inner_radius);
    let outer   = Circle(&inner_radius + &width);
    let section = Plane().cut_angle(&angle);
    let corner1 = Circle(&radius).translate_y(inner_radius + radius);
    let corner2 = corner1.rotate(&angle2);
    let corner1 = corner1.rotate(-&angle2);
    let ring    = &outer - &inner;
    let pie     = &ring * &section;
    let out     = &pie + &corner1 + &corner2;
    let out     = out.fill(Srgba::new(0.9,0.9,0.9,1.0));
    out.into()
}

pub fn node_shape() -> AnyShape {
    let node_radius = 32.0;
    let border_size = 16.0;

    let node = Circle(node_radius.px());
    let node = node.fill(Srgb::new(0.97,0.96,0.95));
    let bg   = Circle((node_radius*2.0).px());
    let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));

    let shadow2 = Circle((node_radius + border_size).px());
    let shadow2_color = LinearGradient::new()
        .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
        .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
    let shadow2_color = SdfSampler::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
    let shadow2       = shadow2.fill(shadow2_color);

    let selection = Circle((node_radius - 1.0).px() + border_size.px() * "input_selection");
    let selection = selection.fill(Srgba::new(0.22,0.83,0.54,1.0));

    let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
    let loader_angle2 = &loader_angle / 2.0;
    let loader        = ring_angle((node_radius).px(), (border_size).px(), loader_angle);
    let loader        = loader.rotate(loader_angle2);
    let loader        = loader.rotate("Radians(input_time/200.0)");
    let icon          = icons::history();
    let out           = loader + selection + shadow2 + node + icon;
    out.into()
}




#[derive(Debug,Derivative,Shrinkwrap)]
#[derivative(Clone(bound="Definition:Clone"))]
pub struct ComponentWrapper<Definition,Shape> {
    #[shrinkwrap(main_field)]
    pub definition     : Definition,
    pub logger         : Logger,
    pub display_object : display::object::Node,
    pub shape          : Rc<RefCell<Option<ShapeWrapper<Shape>>>>,
}


pub type ComponentWrapperX<Definition> = ComponentWrapper<Definition,ShapeDefinition<ComponentSystem<Definition>>>;

impl<Definition,Shape> CloneRef for ComponentWrapper<Definition,Shape>
where Definition : CloneRef {
    fn clone_ref(&self) -> Self {
        let definition     = self.definition.clone_ref();
        let logger         = self.logger.clone_ref();
        let display_object = self.display_object.clone_ref();
        let shape          = self.shape.clone_ref();
        Self {definition,logger,display_object,shape}
    }
}

impl<'t,Definition,Shape>
From<&'t ComponentWrapper<Definition,Shape>> for &'t display::object::Node {
    fn from(t:&'t ComponentWrapper<Definition,Shape>) -> Self {
        &t.display_object
    }
}

impl<Definition:Component> ComponentWrapper<Definition,ShapeDefinition<ComponentSystem<Definition>>> {
    pub fn create(label:&str,definition:Definition) -> Self {
        let logger         = Logger::new(label);
        let display_object = display::object::Node::new(&logger);
        let shape          = default();
        Self {shape,definition,logger,display_object} . create_init()
    }

    fn create_init(self) -> Self {
        let shape               = &self.shape;
        let definition          = &self.definition;
        let display_object_weak = self.display_object.downgrade();

        self.display_object.set_on_show_with(enclose!((shape,definition) move |scene| {
            let node_system = scene.shapes.get(PhantomData::<Definition>).unwrap();
            let instance   = node_system.new_instance();
            display_object_weak.upgrade().for_each(|t| t.add_child(&instance.sprite));
            instance.sprite.size().set(Vector2::new(200.0,200.0));
            scene.shapes.insert_mouse_target(*instance.sprite.instance_id,definition.clone_ref());
            *shape.borrow_mut() = Some(instance);
        }));

        self.display_object.set_on_hide_with(enclose!((shape) move |_| {
            shape.borrow().as_ref().for_each(|shape| {
                // TODO scene.shapes.remove_mouse_target(...)
            });
            *shape.borrow_mut() = None;
        }));

        self
    }
}



// ============
// === Node ===
// ============

#[derive(Debug,Clone,CloneRef)]
pub struct Events {
    pub mouse_down : frp::Dynamic<()>,
}

component! {
    Node {
        label  : frp::Dynamic<String>,
        events : Events,
    }

    Shape (selection:f32) {
        let node_radius = 32.0;
        let border_size = 16.0;

        let node = Circle(node_radius.px());
        let node = node.fill(Srgb::new(0.97,0.96,0.95));
        let bg   = Circle((node_radius*2.0).px());
        let bg   = bg.fill(Srgb::new(0.91,0.91,0.90));

        let shadow2 = Circle((node_radius + border_size).px());
        let shadow2_color = LinearGradient::new()
            .add(0.0,Srgba::new(0.0,0.0,0.0,0.0).into_linear())
            .add(1.0,Srgba::new(0.0,0.0,0.0,0.14).into_linear());
        let shadow2_color = SdfSampler::new(shadow2_color).max_distance(border_size).slope(Slope::Exponent(4.0));
        let shadow2       = shadow2.fill(shadow2_color);

        let selection = Circle((node_radius - 1.0).px() + border_size.px() * "input_selection");
        let selection = selection.fill(Srgba::new(0.22,0.83,0.54,1.0));

        let loader_angle : Var<Angle<Radians>> = "Radians(clamp(input_time/2000.0 - 1.0) * 1.99 * PI)".into();
        let loader_angle2 = &loader_angle / 2.0;
        let loader        = ring_angle((node_radius).px(), (border_size).px(), loader_angle);
        let loader        = loader.rotate(loader_angle2);
        let loader        = loader.rotate("Radians(input_time/200.0)");
        let icon          = icons::history();
        let out           = loader + selection + shadow2 + node + icon;
        out.into()
    }
}

impl Node {
    pub fn new() -> Self {
        frp! {
            label      = source::<String> ();
            mouse_down = source::<()>     ();
        }

        let events     = Events {mouse_down};
        let definition = Definition {label,events};
        Self::create("node",definition).init()
    }

    fn init(self) -> Self {
        let mouse_down = self.events.mouse_down.clone_ref();

        frp! {
            selected            = mouse_down.toggle ();
            selection_animation = source::<f32>     ();
            // debug = selection.map(|t| {println!("SS: {:?}",t);})
        }

        let shape = &self.shape;
        selection_animation.map("animation", enclose!((shape) move |value| {
            shape.borrow().as_ref().for_each(|t| t.selection.set(*value))
        }));

        let simulator = DynInertiaSimulator::<f32>::new(Box::new(move |t| {
            selection_animation.event.emit(t);
        }));

        selected.map("selection", enclose!((simulator) move |check| {
            let value = if *check { 1.0 } else { 0.0 };
            simulator.set_target_position(value);
        }));
        self
    }

}

impl MouseTarget for Definition {
    fn mouse_down(&self) -> Option<&frp::Dynamic<()>> {
        Some(&self.events.mouse_down)
    }
}
