
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
use ensogl::display::shape::primitive::system::ShapeSystem;
use ensogl::display::world::World;
use ensogl::display::scene::{Scene,Component,MouseTarget,ComponentSystemTrait};



//#[derive(Clone,CloneRef,Debug)]
//#[clone_ref(bound="Params:CloneRef")]
//pub struct ComponentShapeWrapper<Params> {
//    pub params : Params,
//    pub sprite : Sprite,
//}


macro_rules! shape {
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


        // ==============
        // === System ===
        // ==============

        #[derive(Clone,CloneRef,Debug)]
        pub struct System {
            pub shape_system : ShapeSystem,
            $(pub $gpu_param : Buffer<$gpu_param_type>),*
        }

        impl ComponentSystemTrait for System {
            type ComponentShape = Shape;

            fn new(scene:&Scene) -> Self {
                let shape_system = ShapeSystem::new(scene,&Self::shape_def());
                $(let $gpu_param = shape_system.add_input(stringify!($gpu_param),default::<$gpu_param_type>());)*
                Self {shape_system,$($gpu_param),*}
            }

            fn new_instance(&self) -> Self::ComponentShape {
                let sprite = self.shape_system.new_instance();
                let id     = sprite.instance_id;
                $(let $gpu_param = self.$gpu_param.at(id);)*
                Shape {sprite,$($gpu_param),*}
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
        #[derive(Debug,Clone,CloneRef)]
        pub struct $name {
            pub logger         : Logger,
            pub display_object : display::object::Node,
            pub shape          : Rc<RefCell<Option<Shape>>>,
            $(pub $field : $field_type),*
        }

        impl Component for $name {
            type ComponentSystem = System;
        }

        shape! { ($($shape_field : $shape_field_type),*) { $($shape_body)* } }

        impl<'t> From<&'t $name> for &'t display::object::Node {
            fn from(t:&'t $name) -> Self {
                &t.display_object
            }
        }
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


pub struct ComponentWrapper<T> {
    pub definition     : T,
    pub logger         : Logger,
    pub display_object : display::object::Node,
}

impl<T:Component> ComponentWrapper<T> {
    pub fn create(definition:T) -> Self {
        let logger = Logger::new("xxx");
        let display_object      = display::object::Node::new(&logger);
        let display_object_weak = display_object.downgrade();

//        display_object.set_on_show_with(enclose!((definition) move |scene| {
//            let node_system = scene.shapes.get(PhantomData::<T>).unwrap();
//            let instance   = node_system.new_instance();
//            display_object_weak.upgrade().for_each(|t| t.add_child(&instance.sprite));
//            instance.sprite.size().set(Vector2::new(200.0,200.0));
//            scene.shapes.insert_mouse_target(*instance.sprite.instance_id,definition);
////            *shape.borrow_mut() = Some(instance);
//        }));

//
//        display_object.set_on_hide_with(enclose!((shape) move |_| {
//            shape.borrow().as_ref().for_each(|shape| {
//                // TODO scene.shapes.remove_mouse_target(...)
//            });
//            *shape.borrow_mut() = None;
//        }));

        Self {definition,logger,display_object}
    }
}

impl Node {
    pub fn new() -> Self {
        let logger = Logger::new("node");
        let shape : Rc<RefCell<Option<Shape>>> = default();
        let display_object      = display::object::Node::new(&logger);
        let display_object_weak = display_object.downgrade();


        frp! {
            label               = source::<String>        ();
            mouse_down          = source::<()>            ();
            selected            = mouse_down.toggle        ();
            selection_animation = source::<f32>           ();
//            debug = selection.map(|t| {println!("SS: {:?}",t);})

        }

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

        let events = Events {mouse_down};


        let display_object2 = display_object.clone_ref();
        let shape2 = shape.clone_ref();

        let this = Self {logger,shape,display_object,label,events};

        let shape = shape2;

        display_object2.set_on_show_with(enclose!((this,shape) move |scene| {
            let node_system = scene.shapes.get(PhantomData::<Node>).unwrap();
            let instance   = node_system.new_instance();
            display_object_weak.upgrade().for_each(|t| t.add_child(&instance.sprite));
            instance.sprite.size().set(Vector2::new(200.0,200.0));
            scene.shapes.insert_mouse_target(*instance.sprite.instance_id,this.clone());
            *shape.borrow_mut() = Some(instance);
        }));


        display_object2.set_on_hide_with(enclose!((shape) move |_| {
            shape.borrow().as_ref().for_each(|shape| {
                // TODO scene.shapes.remove_mouse_target(...)
            });
            *shape.borrow_mut() = None;
        }));


        this
    }

}

impl MouseTarget for Node {
    fn mouse_down(&self) -> &frp::Dynamic<()> {
        &self.events.mouse_down
    }
}

