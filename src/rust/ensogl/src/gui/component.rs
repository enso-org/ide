
use crate::prelude::*;

use crate::display;
use crate::display::scene::MouseTarget;
use crate::display::scene::ShapeRegistry;
use crate::display::object::traits::*;
use crate::display::shape::primitive::system::Shape;
use crate::display::scene::Scene;
use enso_frp as frp;
use enso_frp::frp;
use crate::animation::physics::inertia::DynInertiaSimulator;
use enso_frp::core::node::class::EventEmitterPoly;

//
//#[macro_export]
//macro_rules! component {
//    (
//        $name:ident
//
//        Definition {
//            $($field:ident : $field_type:ty),* $(,)?
//        }
//
//
//    ) => {
//        #[derive(Clone,CloneRef,Debug,Shrinkwrap)]
//        pub struct $name ($crate::gui::component::ComponentWrapper<Definition>);
//
//        impl<'t> From<&'t $name> for &'t display::object::Node {
//            fn from(t:&'t $name) -> Self {
//                t.0.display_object()
//            }
//        }
//
//        impl $name {
//            fn create(label:&str, definition:Definition) -> Self {
//                let data = $crate::gui::component::ComponentWrapper::create(label,definition);
//                Self(data)
//            }
//        }
//
//        #[derive(Debug,Clone,CloneRef)]
//        pub struct Definition {
//            $(pub $field : $field_type),*
//        }
//    };
//}


pub trait StrongRef : CloneRef {
    type WeakRef : WeakRef<StrongRef=Self>;
    fn downgrade(&self) -> Self::WeakRef;
}

pub trait WeakRef : CloneRef {
    type StrongRef : StrongRef<WeakRef=Self>;
    fn upgrade(&self) -> Option<Self::StrongRef>;
}





pub trait View : 'static {
    type Shape : display::Object + Shape; // FIXME: simplify bounds
    fn new(shape:&Self::Shape, scene:&Scene, shape_registry:&ShapeRegistry) -> Self;
}

#[derive(Clone,CloneRef,Debug)]
pub struct Events {
    pub mouse_down : frp::Dynamic<()>,
}

impl Default for Events {
    fn default() -> Self {
        frp! {
            mouse_down = source::<()> ();
        }
        Self {mouse_down}
    }
}

impl MouseTarget for Events {
    fn mouse_down(&self) -> Option<frp::Dynamic<()>> {
        Some(self.mouse_down.clone_ref())
    }
}

#[derive(Debug)]
pub struct ViewManagerData<T:View> {
    pub data  : T,
    pub shape : T::Shape,
}

#[derive(Debug)]
pub struct ViewManager<T:View> {
    pub display_object : display::object::Node,
    pub events         : Events,
    pub data           : Rc<RefCell<Option<ViewManagerData<T>>>>,
}

impl<T:View> ViewManager<T> {
    pub fn new(logger:&Logger) -> Self {
        let display_object = display::object::Node::new(logger);
        let events         = default();
        let data           = default();
        Self {display_object,events,data} . new_init()
    }

    fn new_init(self) -> Self {
        let weak_data   = Rc::downgrade(&self.data);
        let weak_parent = self.display_object.downgrade();
        let events      = self.events.clone_ref();
        self.display_object.set_on_show_with(move |scene| {
            let shape_registry: &ShapeRegistry = &scene.shapes;
            let events = events.clone_ref();
            weak_data.upgrade().for_each(|ddd| {
                weak_parent.upgrade().for_each(|parent| {
                    let shape = shape_registry.new_instance::<T::Shape>();
                    parent.add_child(&shape);
                    shape_registry.insert_mouse_target(*shape.sprite().instance_id,events);
                    let data = T::new(&shape,scene,shape_registry); // FIXME naming
                    let data = ViewManagerData {data,shape};
                    *ddd.borrow_mut() = Some(data); // FIXME naming
                })
            });
        });

        let weak_data = Rc::downgrade(&self.data);
        self.display_object.set_on_hide_with(move |scene| {
            let shape_registry: &ShapeRegistry = &scene.shapes;
            weak_data.upgrade().for_each(|data| {
                data.borrow().for_each_ref(|data| {
                    shape_registry.remove_mouse_target(&*data.shape.sprite().instance_id);
                });
                *data.borrow_mut() = None;
            });
        });

        self
    }
}


pub fn animation<F>(f:F) -> DynInertiaSimulator<f32>
    where F : Fn(f32) + 'static {
    frp! {
        target = source::<f32> ();
    }
    target.map("animation", move |value| f(*value));
    let simulator = DynInertiaSimulator::<f32>::new(Box::new(move |t| {
        target.event.emit(t);
    }));
    simulator
}

