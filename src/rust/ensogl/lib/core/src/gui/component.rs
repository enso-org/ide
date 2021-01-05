//! Root module for GUI related components.

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use crate::prelude::*;

use crate::display::object::traits::*;
use crate::display::scene::MouseTarget;
use crate::display::scene::Scene;
use crate::display::scene::ShapeRegistry;
use crate::display::scene;
use crate::display::shape::primitive::system::DynShape;
use crate::display::shape::primitive::system::Shape;
use crate::display;
use crate::system::gpu::data::attribute::AttributeInstanceIndex;

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
    pub mouse_up   : frp::Source,
    pub mouse_down : frp::Source,
    pub mouse_over : frp::Source,
    pub mouse_out  : frp::Source,
    pub on_drop    : frp::Source,
}

impl ShapeViewEvents {
    fn new() -> Self {
        frp::new_network! { network
            on_drop    <- source_();
            mouse_down <- source_();
            mouse_up   <- source_();
            mouse_over <- source_();
            mouse_out  <- source_();

            is_mouse_over <- bool(&mouse_out,&mouse_over);
            out_on_drop   <- on_drop.gate(&is_mouse_over);
            eval_ out_on_drop (mouse_out.emit(()));
        }
        Self {network,mouse_down,mouse_up,mouse_over,mouse_out,on_drop}
    }
}

impl MouseTarget for ShapeViewEvents {
    fn mouse_down (&self) -> &frp::Source { &self.mouse_down }
    fn mouse_up   (&self) -> &frp::Source { &self.mouse_up   }
    fn mouse_over (&self) -> &frp::Source { &self.mouse_over }
    fn mouse_out  (&self) -> &frp::Source { &self.mouse_out  }
}

impl Default for ShapeViewEvents {
    fn default() -> Self {
        Self::new()
    }
}



// =================
// === ShapeView ===
// =================

#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="S:CloneRef")]
#[allow(missing_docs)]
pub struct ShapeView<S:Shape> {
    model : Rc<ShapeViewModel<S>>
}

impl<S:Shape> Deref for ShapeView<S> {
    type Target = Rc<ShapeViewModel<S>>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

#[derive(Debug)]
#[allow(missing_docs)]
pub struct ShapeViewModel<S:Shape> {
    pub registry       : ShapeRegistry,
    pub shape          : S,
    pub display_object : display::object::Instance,
    pub events         : ShapeViewEvents,
}

impl<S:Shape> Drop for ShapeViewModel<S> {
    fn drop(&mut self) {
        let sprite      = self.shape.sprite();
        let symbol_id   = sprite.symbol_id();
        let instance_id = *sprite.instance_id;
        self.registry.remove_mouse_target(symbol_id,instance_id);
        self.events.on_drop.emit(());
    }
}
///// A structure containing data which is constructed or dropped when the `ShapeView` is added or
///// removed from the scene.
//#[derive(Clone,CloneRef,Debug)]
//pub struct ShapeViewData<T:ShapeViewDefinition> {
//    /// A data associated with the shape. In simple cases, this data could be just a marker struct.
//    /// In more complex examples, it could contain callback handles. For example, for a cursor
//    /// implementation, its `data` contains a callback listening to scene size change in order to
//    /// update the `shape` dimensions.
//    pub phantom : PhantomData<T>,
//    /// A shape instance. Refer to `Shape` docs to learn more.
//    pub shape : T::Shape,
//}

impl<S:Shape> ShapeView<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, scene:&Scene) -> Self {
        let logger         = Logger::sub(logger,"shape_view");
        let display_object = display::object::Instance::new(logger);
        let registry       = scene.shapes.clone_ref();
        let shape          = registry.new_instance::<S>();
        let events         = ShapeViewEvents::new();
        display_object.add_child(&shape);

        let sprite      = shape.sprite();
        let events2     = events.clone_ref();
        let symbol_id   = sprite.symbol_id();
        let instance_id = *sprite.instance_id;
        registry.insert_mouse_target(symbol_id,instance_id,events2);

        let model = Rc::new(ShapeViewModel {registry,display_object,events,shape});
        Self {model}
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

impl<T:Shape> display::Object for ShapeView<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

///// Definition of a new shape view. In simple cases this could be a marker struct. To learn more
///// refer to documentation of `ShapeViewData` and example usages in components.
//pub trait ShapeViewDefinition : CloneRef + 'static {
//    /// Associated shape instance type.
//    type Shape : Shape;
////    fn new(shape:&Self::Shape, scene:&Scene, shape_registry:&ShapeRegistry) -> Self;
//}



// ==================
// === ShapeView2 ===
// ==================

#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="S:CloneRef")]
#[allow(missing_docs)]
pub struct ShapeView2<S:DynShape> {
    model : Rc<ShapeViewModel2<S>>
}

impl<S:DynShape> Deref for ShapeView2<S> {
    type Target = Rc<ShapeViewModel2<S>>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct ShapeViewModel2<S:DynShape> {
    pub shape    : S,
    pub events   : ShapeViewEvents,
    pub registry : Rc<RefCell<Option<ShapeRegistry>>>,
}

impl<S:DynShape> Drop for ShapeViewModel2<S> {
    fn drop(&mut self) {
        self.unregister_existing_mouse_target();
    }
}

impl<S:DynShape> ShapeView2<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, scene:&Scene) -> Self {
        let model = Rc::new(ShapeViewModel2::new(logger));
        Self {model} . init(scene)
    }

    fn init(self, scene:&Scene) -> Self {
        /// FIXME: In the future, this should not be needed, as the object should be added to the
        ///        default view after it is added as a child to the scene for the first time. With
        ///        SynShapeView we have now ability to do it (lazy initialization of symbols).
        self.switch_view(&scene.views.breadcrumbs);
        self
    }
}

impl<S:DynShape> ShapeViewModel2<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let shape    = S::new(logger);
        let events   = ShapeViewEvents::new();
        let registry = default();
        ShapeViewModel2 {shape,events,registry} . init()
    }

    pub fn switch_registry(&self, registry:&ShapeRegistry) -> (i32,AttributeInstanceIndex) {
        self.unregister_existing_mouse_target();
        let (symbol_id,instance_id) = registry.instantiate_dyn(&self.shape);
        registry.insert_mouse_target(symbol_id,*instance_id,self.events.clone_ref());
        *self.registry.borrow_mut() = Some(registry.clone_ref());
        (symbol_id,instance_id)
    }

    pub fn switch_view(&self, view:&scene::View) -> (i32,AttributeInstanceIndex) {
        let (symbol_id,instance_id) = self.switch_registry(&view.shape_registry);
        view.add_by_id(symbol_id);
        (symbol_id,instance_id)
    }

    fn unregister_existing_mouse_target(&self) {
        if let (Some(registry),Some(sprite)) = (&*self.registry.borrow(),&self.shape.sprite()) {
            let symbol_id   = sprite.symbol_id();
            let instance_id = *sprite.instance_id;
            registry.remove_mouse_target(symbol_id,instance_id);
            self.events.on_drop.emit(());
        }
    }

    fn init(self) -> Self {
        self.init_on_show();
        self
    }

    fn init_on_show(&self) {
        // let weak_data   = Rc::downgrade(&self.data);
        // let weak_parent = self.display_object.downgrade();
        // let events      = self.events.clone_ref();
        self.display_object().set_on_show(move |scene| {
            println!("INIT ON SHOW!!!");
            // let shape_registry: &ShapeRegistry = &scene.shapes;
            // weak_data.upgrade().for_each(|self_data| {
            //     weak_parent.upgrade().for_each(|parent| {
            //         let shape = shape_registry.new_instance::<T::Shape>();
            //         parent.add_child(&shape);
            //         for sprite in shape.sprites() {
            //             let events      = events.clone_ref();
            //             let symbol_id   = sprite.symbol_id();
            //             let instance_id = *sprite.instance_id;
            //             shape_registry.insert_mouse_target(symbol_id,instance_id,events);
            //         }
            //         let data = T::new(&shape,scene,shape_registry);
            //         let data = ShapeViewData {data,shape};
            //         *self_data.borrow_mut() = Some(data);
            //     })
            // });
        });
    }
}

impl<T:DynShape> display::Object for ShapeViewModel2<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}

impl<T:DynShape> display::Object for ShapeView2<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}
