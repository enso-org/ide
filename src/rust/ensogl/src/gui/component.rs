//! Root module for GUI related components.

use crate::prelude::*;

use crate::display;
use crate::display::scene::MouseTarget;
use crate::display::scene::ShapeRegistry;
use crate::display::object::traits::*;
use crate::display::shape::primitive::system::Shape;
use crate::display::scene::Scene;
use enso_frp as frp;
use enso_frp::frp;
use crate::animation::physics::inertia::DynSimulator;
use enso_frp::core::node::class::EventEmitterPoly;



// =======================
// === ShapeViewEvents ===
// =======================

/// FRP event endpoints exposed by each shape view. In particular these are all mouse events
/// which are triggered by mouse interactions after the shape view is placed on the stage.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ShapeViewEvents {
    pub mouse_down : frp::Dynamic<()>,
}

impl Default for ShapeViewEvents {
    fn default() -> Self {
        frp! { mouse_down = source::<()> (); }
        Self {mouse_down}
    }
}

impl MouseTarget for ShapeViewEvents {
    fn mouse_down(&self) -> Option<frp::Dynamic<()>> {
        Some(self.mouse_down.clone_ref())
    }
}


// =================
// === ShapeView ===
// =================

/// Automatically managed view of a shape. The view is initially empty and is filled with a
/// reference to an existing `Shape` as soon as it is placed on the stage and the stage is updated.
/// As soon as it is removed from the stage, the shape is freed.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct ShapeView<T:ShapeViewDefinition> {
    pub display_object : display::object::Node,
    pub events         : ShapeViewEvents,
    pub data           : Rc<RefCell<Option<ShapeViewData<T>>>>,
}

/// A structure containing data which is constructed or dropped when the `ShapeView` is added or
/// removed from the stage.
#[derive(Debug)]
pub struct ShapeViewData<T:ShapeViewDefinition> {
    /// A data associated with the shape. In simple cases, this data could be just a marker struct.
    /// In more complex examples, it could contain callback handles. For example, for a cursor
    /// implementation, its `data` contains a callback listening to scene size change in order to
    /// update the `shape` dimensions.
    pub data : T,
    /// A shape instance. Refer to `Shape` docs to learn more.
    pub shape : T::Shape,
}

impl<T:ShapeViewDefinition> ShapeView<T> {
    /// Constructor.
    pub fn new(logger:&Logger) -> Self {
        let display_object = display::object::Node::new(logger);
        let events         = default();
        let data           = default();
        Self {display_object,events,data} . init()
    }

    fn init(self) -> Self {
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
                    let data = ShapeViewData {data,shape};
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

/// Definition of a new shape view. In simple cases this could be a marker struct. To learn more
/// refer to documentation of `ShapeViewData` and example usages in components.
pub trait ShapeViewDefinition : 'static {
    /// Associated shape instance type.
    type Shape : Shape;
    /// Constructor.
    fn new(shape:&Self::Shape, scene:&Scene, shape_registry:&ShapeRegistry) -> Self;
}



// =================
// === Animation ===
// =================

// TODO: This should grow and then should be refactored somewhere else.
/// Define a new animation FRP network.
pub fn animation<F>(f:F) -> DynSimulator<f32>
    where F : Fn(f32) + 'static {
    frp! { target = source::<f32> (); }
    target.map("animation", move |value| f(*value));
    DynSimulator::<f32>::new(Box::new(move |t| {
        target.event.emit(t);
    }))
}
