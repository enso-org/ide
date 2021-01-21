//! Root module for GUI related components.

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use crate::prelude::*;

use crate::display::object::traits::*;

use crate::display::scene::MouseTarget;
use crate::display::scene::Scene;
use crate::display::scene::ShapeRegistry;
use crate::display::scene::layer::ShapeRegistry2;
use crate::display::scene::layer::LayerId;
use crate::display::scene;
use crate::display::shape::primitive::system::DynamicShape;
use crate::display::shape::primitive::system::Shape;
use crate::display::shape::primitive::system::ShapeSystemId;
use crate::display::symbol::SymbolId;
use crate::display;
use crate::system::gpu::data::attribute;

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



// ============================
// === ShapeView_DEPRECATED ===
// ============================

/// # Depreciation
/// This model is deprecated. Please use [`ShapeView`] instead. There are few major differences
/// worth considering when upgrading:
/// 1. The new [`ShapeView`] does not initialize the shape when created. Instead, it initializes the
///    shape lazily.
/// 2. The new [`ShapeView`] gets its scene layer information from `display::object::Instance`
///    hierarchy.
/// 3. When using the `define_shape_system!` macro, you do not need to use [`ShapeView`] explicitly
///    anymore. Just use `my_def::View` instead, which is a type alias for
///    `ShapeView<my_def::DynamicShape>`.
#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="S:CloneRef")]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
pub struct ShapeView_DEPRECATED<S:Shape> {
    model : Rc<ShapeViewModel_DEPRECATED<S>>
}

impl<S:Shape> Deref for ShapeView_DEPRECATED<S> {
    type Target = Rc<ShapeViewModel_DEPRECATED<S>>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

/// Model of [`ShapeView_DEPRECATED`].
#[derive(Debug)]
#[allow(missing_docs)]
#[allow(non_camel_case_types)]
pub struct ShapeViewModel_DEPRECATED<S:Shape> {
    pub registry       : ShapeRegistry,
    pub shape          : S,
    pub display_object : display::object::Instance,
    pub events         : ShapeViewEvents,
}

impl<S:Shape> Drop for ShapeViewModel_DEPRECATED<S> {
    fn drop(&mut self) {
        let sprite      = self.shape.sprite();
        let symbol_id   = sprite.symbol_id();
        let instance_id = sprite.instance_id;
        self.registry.remove_mouse_target(symbol_id,instance_id);
        self.events.on_drop.emit(());
    }
}

impl<S:Shape> ShapeView_DEPRECATED<S> {
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
        let instance_id = sprite.instance_id;
        registry.insert_mouse_target(symbol_id,instance_id,events2);

        let model = Rc::new(ShapeViewModel_DEPRECATED {registry,display_object,events,shape});
        Self {model}
    }
}

impl<T:Shape> display::Object for ShapeView_DEPRECATED<T> {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// =================
// === ShapeView ===
// =================

/// A view for a shape definition. The view manages the lifetime and scene-registration of a shape
/// instance. In particular, it registers / unregisters callbacks for shape initialization and mouse
/// events handling.
#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="S:CloneRef")]
#[allow(missing_docs)]
pub struct ShapeView<S:DynamicShape> {
    model : Rc<ShapeViewModel<S>>
}

impl<S:DynamicShape> Deref for ShapeView<S> {
    type Target = Rc<ShapeViewModel<S>>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<S:DynamicShape+'static> ShapeView<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let model = Rc::new(ShapeViewModel::new(logger));
        Self {model} . init()
    }

    fn init(self) -> Self {
        self.init_on_show();
        self.init_on_scene_layer_changed();
        self
    }

    fn init_on_show(&self) {
        let weak_model = Rc::downgrade(&self.model);
        self.display_object().set_on_show(move |scene,layers| {
            if let Some(model) = weak_model.upgrade() {
                if model.before_first_show.get() {
                    model.on_scene_layers_changed(scene,layers)
                }
            }
        });
    }

    fn init_on_scene_layer_changed(&self) {
        let weak_model = Rc::downgrade(&self.model);
        self.display_object().set_on_scene_layer_changed(move |scene,layers| {
            if let Some(model) = weak_model.upgrade() {
                model.on_scene_layers_changed(scene,layers)
            }
        });
    }
}

impl<S:DynamicShape> HasContent for ShapeView<S> {
    type Content = S;
}



// ======================
// === ShapeViewModel ===
// ======================

/// Model of [`ShapeView`].
#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct ShapeViewModel<S:DynamicShape> {
    pub shape         : S,
    pub events        : ShapeViewEvents,
    pub registry      : Rc<RefCell<Option<ShapeRegistry>>>,
    before_first_show : Rc<Cell<bool>>,
}

impl<S:DynamicShape> Deref for ShapeViewModel<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl<S:DynamicShape> Drop for ShapeViewModel<S> {
    fn drop(&mut self) {
        self.unregister_existing_mouse_target();
    }
}

impl<S:DynamicShape> ShapeViewModel<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let shape    = S::new(logger);
        let events   = ShapeViewEvents::new();
        let registry = default();
        let before_first_show = Rc::new(Cell::new(true));
        ShapeViewModel {shape,events,registry,before_first_show}
    }

    fn on_scene_layers_changed(&self, scene:&Scene, scene_layers:Option<&Vec<LayerId>>) {
        match scene_layers {
            None => {
                self.set_scene_layer(&scene,&scene.layers.main);
            },
            Some(scene_layers) => {
                if scene_layers.len() != 1 {
                    panic!("Adding a shape to multiple scene layers is not supported currently.")
                }
                if let Some(scene_layer) = scene.layers.get(scene_layers[0]) {
                    self.set_scene_layer(&scene,&scene_layer);
                } else {
                    self.set_scene_layer(&scene,&scene.layers.main);
                }
            }
        }
    }

    fn set_scene_layer(&self, scene:&Scene, layer:&scene::Layer) -> (ShapeSystemId,SymbolId,attribute::InstanceIndex) {
        self.before_first_show.set(false);
        let (shape_system_id,symbol_id,instance_id) = self.set_scene_registry(&scene.shapes,&layer.shape_registry);
        layer.add_shape(shape_system_id,symbol_id);
        (shape_system_id,symbol_id,instance_id)
    }

    fn set_scene_registry(&self, registry1:&ShapeRegistry, registry:&ShapeRegistry2) -> (ShapeSystemId,SymbolId,attribute::InstanceIndex) {
        self.unregister_existing_mouse_target();
        let (shape_system_id,symbol_id,instance_id) = registry.instantiate(&self.shape);
        registry1.insert_mouse_target(symbol_id,instance_id,self.events.clone_ref());
        *self.registry.borrow_mut() = Some(registry1.clone_ref());
        (shape_system_id,symbol_id,instance_id)
    }

    fn unregister_existing_mouse_target(&self) {
        if let (Some(registry),Some(sprite)) = (&*self.registry.borrow(),&self.shape.sprite()) {
            let symbol_id   = sprite.symbol_id();
            let instance_id = sprite.instance_id;
            registry.remove_mouse_target(symbol_id,instance_id);
            self.events.on_drop.emit(());
        }
    }
}

impl<T:DynamicShape> display::Object for ShapeViewModel<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}

impl<T:DynamicShape> display::Object for ShapeView<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}
