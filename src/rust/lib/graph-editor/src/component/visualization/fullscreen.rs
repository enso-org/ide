//! Provides generic operations that can be applied to UI components.
//!
//! Rationale: this is a step towards a higher level abstraction for arbitrary UI elements. That is,
//! instead of doing a lot of low level functionality within our UI components, we should instead be
//! able to access some data about them and manipulate them in a consistent manner. For example:
//! instead of every component implementing a "fullscreen mode", they just provide resizing
//! functionality and access to their su-components and instead of determining the layer of shapes
//! within the UI component, we instead make the shapes accessible via an API that indicates the
//! expected behaviour of the shapes, and the actual layer management happens outside of the
//! component.
//!
//! This is also a step to avoid relying too much on the presence of the `Scene` within the UI
//! component.
use crate::prelude::*;

use ensogl::display::Scene;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::component::animation;
use crate::component::visualization::traits::Resizable;
use crate::component::visualization::traits::HasSymbols;
use crate::component::visualization::traits::HasNetwork;


pub trait Fullscreenable = display::Object+Resizable+HasSymbols+HasNetwork+CloneRef+'static;

// ==================================
// === Fullscreen Operator Handle ===
// ==================================

/// FullscreenOperatorCellHandle is a helper type that wraps a `FullscreenOperator` and applies an
/// undos the operator on the inner component. This can be used to ensure that only a single
/// component is made fullscreen at any time.
#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct FullscreenState<T> {
    operator: Rc<RefCell<Option<FullscreenOperation<T>>>>
}

impl<T:Fullscreenable> FullscreenState<T> {
    /// returns whether there is a component that is in fullscreen mode.
    pub fn is_active(&self) -> bool {
        self.operator.borrow().is_some()
    }

    /// Enables fullscreen mode for the given component. If there is another component already in
    /// fullscreen mode, it disables fullscreen for that component.
    pub fn set_fullscreen(&self, target:T, scene:Scene) {
        self.disable_fullscreen();
        let operator = FullscreenOperation::enable_fullscreen(target, scene);
        self.operator.set(operator);
    }

    /// Disables fullscreen mode for the given component.
    pub fn disable_fullscreen(&self) {
        if let Some(old) = self.operator.borrow_mut().take() {
            old.disable_fullscreen();
        }
    }

    /// Return a ref clone of the fullscreen element.
    pub fn get_element(&self) -> Option<T>{
        self.operator.borrow().as_ref().map(|op| op.target.clone_ref())
    }
}



// ============================
// === Fullscreen Operation ===
// ============================

/// A `FullscreenOperator` can be used to apply fullscreen mode to a UI element as well as undo the
/// the fullscreen operation and restore the previous state. The  `FullscreenOperator` can be
/// applied to any target that implements `display::Object`, `Resizable` and `NativeUiElement`.
// TODO consider incorporating these traits into display::Object or another common "SceneElement"
// type. But it is important that complex UI components can provide information about their
// sub-components (for example, multiple sub-shapes or HTML components).
#[derive(Debug)]
pub struct FullscreenOperation<T> {
    target            : T,
    scene             : Scene,
    size_original     : Vector3<f32>,
    position_original : Vector3<f32>,
    parent_original   : Option<display::object::Instance>,
}

impl<T:Fullscreenable> FullscreenOperation<T> {
    /// Make the provided target fullscreen within the given scene and return the
    /// `FullscreenOperator`.
    pub fn enable_fullscreen(target:T, scene:Scene) -> Self {
        let size_original     = target.size();
        let position_original = target.position();
        let parent_original   = target.display_object().rc.parent();
        FullscreenOperation {target,scene,size_original,position_original,parent_original}.init()
    }

    fn init(self) -> Self {
        let original_pos = self.target.display_object().global_position();

        // Change parent
        self.target.display_object().set_parent(self.scene.display_object());
        self.target.set_layers_fullscreen(&self.scene);

        let margin      = 0.0;
        let scene_shape = self.scene.shape();
        let size_new    = Vector3::new(scene_shape.width(), scene_shape.height(),0.0) * (1.0 - margin);

        // TODO Currently we assume objects are center aligned, but this needs to be properly
        // accounted for here.

        let frp_network      = &self.target.network().clone_ref();
        let target_pos       = Vector3::zero();
        let original_size    = self.size_original;
        let target_size      = size_new;
        let target           = self.target.clone_ref();
        let resize_animation = animation(frp_network, move |value| {
            let pos  = original_pos  * (1.0 - value) + target_pos  * value;
            let size = original_size * (1.0 - value) + target_size * value;
            target.set_position(pos);
            target.set_size(size);
        });
        resize_animation.set_target_position(1.0);

        self.scene.views.toggle_overlay_cursor();

        self
    }

    /// Undo the fullscreen operation and restore the previous state exactly as it was.
    pub fn disable_fullscreen(self) {
        let global_pos_start = self.target.global_position();

        self.target.set_layers_normal(&self.scene);

        if let Some(parent) = self.parent_original.as_ref() {
            self.target.display_object().set_parent(&parent);
        }

        println!("{:?}", self.target.display_object().has_parent());
        let parent_pos     = self.parent_original.map(|p| p.global_position());
        let parent_pos     = parent_pos.unwrap_or_else(Vector3::zero);
        let mut source_pos = self.target.position();
        source_pos        += global_pos_start ;
        source_pos        -= parent_pos ;
        let source_pos     = source_pos;

        self.target.set_position(source_pos);

        let original_pos     = self.target.position();
        let target_pos       = self.position_original;
        let original_size    = self.target.size();
        let target_size      = self.size_original;
        let target           = self.target.clone_ref();
        let frp_network      = &self.target.network().clone_ref();
        let resize_animation = animation(frp_network, move |value| {
            let pos  = original_pos  * (1.0 - value) + target_pos * value;
            let size = original_size * (1.0 - value) + target_size * value;
            target.set_position(pos);
            target.set_size(size);
        });
        resize_animation.set_target_position(1.0);

        self.scene.views.toggle_overlay_cursor();
    }
}
