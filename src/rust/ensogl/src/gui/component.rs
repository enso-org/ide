//! Root module for GUI related components.

use crate::prelude::*;

use crate::animation::physics::inertia::DynSimulator;
use crate::display::object::traits::*;
use crate::display::scene::MouseTarget;
use crate::display::scene::Scene;
use crate::display::scene::ShapeRegistry;
use crate::display::shape::primitive::system::Shape;
use crate::display;

use enso_frp as frp;



// =======================
// === ShapeViewEvents ===
// =======================

/// FRP event endpoints exposed by each shape view. In particular these are all mouse events
/// which are triggered by mouse interactions after the shape view is placed on the scene.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ShapeViewEvents {
    pub network    : frp::Network,
    pub mouse_down : frp::Source,
}

impl Default for ShapeViewEvents {
    fn default() -> Self {
        frp::new_network! { shape_view
            def mouse_down = source_();
        }
        let network = shape_view;
        Self {network,mouse_down}
    }
}

impl MouseTarget for ShapeViewEvents {
    fn mouse_down(&self) -> Option<frp::Source> {
        Some(self.mouse_down.clone_ref())
    }
}


// =================
// === ShapeView ===
// =================

/// Automatically managed view of a `Shape`. The view is initially empty and is filled with a
/// reference to an existing `Shape` as soon as it is placed on the scene and the scene is updated.
/// As soon as it is removed from the scene, the shape is freed.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ShapeView<T:ShapeViewDefinition> {
    pub display_object : display::object::Instance,
    pub events         : ShapeViewEvents,
    pub data           : ShapeViewData<T>,
}

/// A structure containing data which is constructed or dropped when the `ShapeView` is added or
/// removed from the scene.
#[derive(Clone,CloneRef,Debug)]
pub struct ShapeViewData<T:ShapeViewDefinition> {
    /// A data associated with the shape. In simple cases, this data could be just a marker struct.
    /// In more complex examples, it could contain callback handles. For example, for a cursor
    /// implementation, its `data` contains a callback listening to scene size change in order to
    /// update the `shape` dimensions.
    pub phantom : PhantomData<T>,
    /// A shape instance. Refer to `Shape` docs to learn more.
    pub shape : T::Shape,
}

impl<T:ShapeViewDefinition> ShapeView<T> {
    /// Constructor.
    pub fn new(logger:&Logger, scene:&Scene) -> Self {
        let display_object = display::object::Instance::new(logger);
        let events         = ShapeViewEvents::default();
//        let data           = default();
        let shape_registry: &ShapeRegistry = &scene.shapes;
        let shape = shape_registry.new_instance::<T::Shape>();
        display_object.add_child(&shape);
        for sprite in shape.sprites() {
            let events      = events.clone_ref();
            let symbol_id   = sprite.symbol_id();
            let instance_id = *sprite.instance_id;
            shape_registry.insert_mouse_target(symbol_id,instance_id,events);
        }
//        let data = T::new(&shape,scene,shape_registry);
        let phantom = PhantomData;
        let data = ShapeViewData {phantom,shape};

        Self {display_object,events,data} // . init()
    }

//    fn init(self) -> Self {
//        self.init_on_show();
//        self.init_on_hide();
//        self
//    }
//
//    fn init_on_show(&self) {
//        let weak_data   = Rc::downgrade(&self.data);
//        let weak_parent = self.display_object.downgrade();
//        let events      = self.events.clone_ref();
//        self.display_object.set_on_show_with(move |scene| {
//            let shape_registry: &ShapeRegistry = &scene.shapes;
//            weak_data.upgrade().for_each(|self_data| {
//                weak_parent.upgrade().for_each(|parent| {
//                    let shape = shape_registry.new_instance::<T::Shape>();
//                    parent.add_child(&shape);
//                    for sprite in shape.sprites() {
//                        let events      = events.clone_ref();
//                        let symbol_id   = sprite.symbol_id();
//                        let instance_id = *sprite.instance_id;
//                        shape_registry.insert_mouse_target(symbol_id,instance_id,events);
//                    }
//                    let data = T::new(&shape,scene,shape_registry);
//                    let data = ShapeViewData {data,shape};
//                    *self_data.borrow_mut() = Some(data);
//                })
//            });
//        });
//    }
//
//    fn init_on_hide(&self) {
//        let weak_data = Rc::downgrade(&self.data);
//        self.display_object.set_on_hide_with(move |scene| {
//            let shape_registry: &ShapeRegistry = &scene.shapes;
//            weak_data.upgrade().for_each(|data| {
//                data.borrow().for_each_ref(|data| {
//                    for sprite in data.shape.sprites() {
//                        let symbol_id   = sprite.symbol_id();
//                        let instance_id = *sprite.instance_id;
//                        shape_registry.remove_mouse_target(symbol_id,instance_id);
//                    }
//                });
//                *data.borrow_mut() = None;
//            });
//        });
//    }
}

impl<T:ShapeViewDefinition> display::Object for ShapeView<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

/// Definition of a new shape view. In simple cases this could be a marker struct. To learn more
/// refer to documentation of `ShapeViewData` and example usages in components.
pub trait ShapeViewDefinition : CloneRef + 'static {
    /// Associated shape instance type.
    type Shape : Shape;
//    fn new(shape:&Self::Shape, scene:&Scene, shape_registry:&ShapeRegistry) -> Self;
}



// =================
// === Animation ===
// =================

// TODO: This should grow and then should be refactored somewhere else.
/// Define a new animation FRP network.
pub fn animation<F>(network:&frp::Network, f:F) -> DynSimulator<f32>
where F : Fn(f32) + 'static {
    frp::extend! { network
        def target = source::<f32> ();
        def _eval  = target.map(move |value| f(*value));
    }
    DynSimulator::<f32>::new(Box::new(move |t| target.emit(t)))
}
