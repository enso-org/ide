//! Root module for GUI related components.

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

use crate::prelude::*;

use crate::display::object::traits::*;

use crate::display::scene::MouseTarget;
use crate::display::scene::Scene;
use crate::display::scene::ShapeRegistry;
use crate::display::scene::layer::LayerId;
use crate::display::scene;
use crate::display::shape::primitive::system::DynamicShape;
use crate::display::shape::primitive::system::DynamicShapeInternals;
use crate::display;
use crate::display::symbol::SymbolId;
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
        Self {network,mouse_up,mouse_down,mouse_over,mouse_out,on_drop}
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

/// A view for a shape definition. The view manages the lifetime and scene-registration of a shape
/// instance. In particular, it registers / unregisters callbacks for shape initialization and mouse
/// events handling.
#[derive(Clone,CloneRef,Debug)]
#[clone_ref(bound="S:CloneRef")]
#[allow(missing_docs)]
pub struct ShapeView<S> {
    model : Rc<ShapeViewModel<S>>
}

impl<S> Deref for ShapeView<S> {
    type Target = Rc<ShapeViewModel<S>>;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl<S:DynamicShapeInternals+'static> ShapeView<S> {
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
                    model.before_first_show.set(false);
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

impl<S> HasContent for ShapeView<S> {
    type Content = S;
}



// ======================
// === ShapeViewModel ===
// ======================

/// Model of [`ShapeView`].
#[derive(Debug,Default)]
#[allow(missing_docs)]
pub struct ShapeViewModel<S> {
    shape               : S,
    pub events          : ShapeViewEvents,
    pub registry        : RefCell<Option<ShapeRegistry>>,
    pub pointer_targets : RefCell<Vec<(SymbolId,attribute::InstanceIndex)>>,
    before_first_show   : Cell<bool>,
}

impl<S> Deref for ShapeViewModel<S> {
    type Target = S;
    fn deref(&self) -> &Self::Target {
        &self.shape
    }
}

impl<S> Drop for ShapeViewModel<S> {
    fn drop(&mut self) {
        self.unregister_existing_mouse_targets();
        self.events.on_drop.emit(());
    }
}

impl<S:DynamicShapeInternals> ShapeViewModel<S> {
    fn on_scene_layers_changed(&self, scene:&Scene, layers:&[LayerId]) {
        self.drop_from_all_scene_layers();
        let default_layers = &[scene.layers.main.id()];
        let target_layers  = if layers.is_empty() { default_layers } else { layers };
        for &layer_id in target_layers {
            if let Some(layer) = scene.layers.get_child(layer_id) {
                self.add_to_scene_layer(scene,&layer)
            }
        }
    }

    fn drop_from_all_scene_layers(&self) {
        self.shape.drop_instances();
        self.unregister_existing_mouse_targets();
    }
}


impl<S:DynamicShape> ShapeViewModel<S> {
    /// Constructor.
    pub fn new(logger:impl AnyLogger) -> Self {
        let shape             = S::new(logger);
        let events            = ShapeViewEvents::new();
        let registry          = default();
        let pointer_targets   = default();
        let before_first_show = Cell::new(true);
        ShapeViewModel {shape,events,registry,pointer_targets,before_first_show}
    }

    fn add_to_scene_layer(&self, scene:&Scene, layer:&scene::Layer) {
        let (shape_system_info,symbol_id,instance_id) = layer.shape_system_registry.instantiate(scene,&self.shape);
        // FIXME: This is implemented incorrectly, as it does not remove the shape from other layers if the symbol_id is different:
        layer.add_shape_exclusive(shape_system_info,symbol_id);
        scene.shapes.insert_mouse_target(symbol_id,instance_id,self.events.clone_ref());
        self.pointer_targets.borrow_mut().push((symbol_id,instance_id));
        *self.registry.borrow_mut() = Some(scene.shapes.clone_ref());
    }
}

impl<S> ShapeViewModel<S> {
    fn unregister_existing_mouse_targets(&self) {
        if let Some(registry) = &*self.registry.borrow() {
            for (symbol_id,instance_id) in mem::take(&mut *self.pointer_targets.borrow_mut()) {
                registry.remove_mouse_target(symbol_id,instance_id);
            }
        }
    }
}

impl<T:display::Object> display::Object for ShapeViewModel<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}

impl<T:display::Object> display::Object for ShapeView<T> {
    fn display_object(&self) -> &display::object::Instance {
        self.shape.display_object()
    }
}
