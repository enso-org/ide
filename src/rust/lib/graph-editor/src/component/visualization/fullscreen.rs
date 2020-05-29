//! Provides the fullscreen operation that can be applied to UI components.
use crate::prelude::*;

use crate::component::visualization::traits::{HasSymbols, HasFullscreenDecoration, HasDomSymbols};
use crate::component::visualization::traits::Resizable;

use ensogl::animation::physics::inertia::DynSimulator;
use ensogl::control::callback;
use ensogl::display::Scene;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::frp;
use ensogl::gui::component::animation;
use ensogl::system::web;



// ====================
// === Helper Types ===
// ====================

pub trait Fullscreenable = display::Object+Resizable+HasSymbols+HasDomSymbols+CloneRef
                           +HasFullscreenDecoration+'static;



// =========================
// === Animation Helpers ===
// =========================

/// Weak ref to the inner `State` of the `StateModel`.
type WeakState<T> = Weak<RefCell<StateModel<T>>>;

/// Initial and target state of the animated component. This is used in the animation to interpolate between
/// the initial and the target state.
#[derive(Debug,Clone)]
struct AnimationTargetState<T> {
    /// Animated UI component.
    target      : T,
    /// Position at the start of the animation.
    source_pos  : Vector3<f32>,
    /// Size at the start of the animation.
    source_size : Vector3<f32>,
    /// Position at the end of the animation.
    target_pos  : Vector3<f32>,
    /// Size at the end of the animation.
    target_size : Vector3<f32>,
}

impl<T:Fullscreenable> AnimationTargetState<T> {
    /// Set the interpolated values on the target.
    fn apply_interpolation(&self, value:f32) {
        let target      = &self.target;
        let source_pos  = self.source_pos;
        let source_size = self.source_size;
        let target_pos  = self.target_pos;
        let target_size = self.target_size;
        let pos         = source_pos  * (1.0 - value) + target_pos * value;
        let size        = source_size * (1.0 - value) + target_size * value;
        target.set_position(pos);
        target.set_size(size);
    }
}

/// Helper function that contains the logic of the animation.
fn transition_animation_fn<T:Fullscreenable>(state: WeakState<T>, value:f32) {
    if let Some(state) = state.upgrade() {
        // Regular animation step
        // We animate only if the correct state is given.
        match state.borrow().deref(){
            StateModel::TransitioningFromFullscreen {animation_data, .. }
            | StateModel::TransitioningToFullscreen { animation_data,.. } => {
                animation_data.apply_interpolation(value);
            },
            _  => ()
        }
        // Check for end of animation and update the state.
        if value >= 1.0 {
           state.borrow_mut().animation_end_transition();
        }
    }
}

// ==================
// === StateModel ===
// ==================

/// Represents the internal state of the FullscreenState. This is used to ensure animations always
/// finish without interruption.
#[derive(Debug,Clone)]
enum StateModel<T> {
    /// There is a UI component and it is in fullscreen mode.
    Fullscreen {
        data           : FullscreenStateData<T>,
        resize_handle  : callback::Handle,
    },
    /// There is an animation running from fullscreen mode to non-fullscreen mode.
    TransitioningFromFullscreen {
        animation_data : AnimationTargetState<T>
    },
    /// There is an animation running from non-fullscreen mode to fullscreen mode.
    TransitioningToFullscreen {
        animation_data : AnimationTargetState<T>,
        target_state   : FullscreenStateData<T>
    },
    /// There is no UI component in fullscreen mode.
    NotFullscreen
}

impl<T:Fullscreenable> StateModel<T> {
    /// Called to indicate the the running animation has ended. Changes the state to the correct
    /// follow up state. Does nothing if no animation was running.
    fn animation_end_transition(&mut self) {
        let new_state = match self.clone() {
            StateModel::TransitioningFromFullscreen { .. } => {
                StateModel::NotFullscreen
            }
            StateModel::TransitioningToFullscreen { target_state, ..} => {
                let resize_handle = target_state.make_resize_handle();
                StateModel::Fullscreen { data: target_state, resize_handle }
            }
            other    => other,
        };
        *self = new_state;
    }

    /// Returns whether there is a component that is in fullscreen mode. Animation phases count as
    /// still in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        match self {
            StateModel::NotFullscreen{..} => false,
            _                             => true,
        }
    }
}

impl<T> Default for StateModel<T> {
    fn default() -> Self {
        StateModel::NotFullscreen
    }
}



// =======================
// === FullscreenState ===
// =======================

/// The `FullscreenState` manages the state changes between fullscreen mode and non-fullscreen mode
/// for a UI component. It creates animations for the state changes and ensure that the component
/// cannot come into an illegal state during the transition.
#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct FullscreenState<T> {
    network   : frp::Network,
    state     : Rc<RefCell<StateModel<T>>>,
    animation : DynSimulator<f32>,
}

impl<T:Fullscreenable> Default for FullscreenState<T> {
    fn default() -> Self {
        let network    = frp::Network::new();
        let state      = Rc::new(RefCell::new(StateModel::<T>::default()));
        let weak_state = Rc::downgrade(&state);
        let animation  = animation(&network, move |value| {
            transition_animation_fn(weak_state.clone_ref(), value);
        });
        FullscreenState{network,state,animation}
    }
}

impl<T:Fullscreenable> FullscreenState<T> {
    /// Returns whether there is a component that is in fullscreen mode. Animation phases count as
    /// still in fullscreen mode.
    pub fn is_fullscreen(&self) -> bool {
        self.state.borrow().is_fullscreen()
    }

    /// Enables fullscreen mode for the given component. Does nothing if we are in an animation
    /// phase, or already in fullscreen mode.
    pub fn enable_fullscreen(&self, target:T, scene:Scene) {
        if !self.is_fullscreen() {
            self.transition_to_fullscreen(target, scene)
        }
    }

    /// Disables fullscreen mode for the given component. Does nothing if we are in an animation
    /// phase, or not in fullscreen mode.
    pub fn disable_fullscreen(&self) {
        let fullscreen_data = {
            let state = self.state.borrow();
            let state = state.deref();
            if let StateModel::Fullscreen {data, ..} = state {
                Some(data.clone())
            } else {
                None
            }
        };
        if let Some(data) = fullscreen_data {
            self.transition_to_non_fullscreen(&data)
        }
    }

    /// Start the transition to non-fullscreen mode. Triggers the UI component state change and
    /// starts the animation.
    fn transition_to_non_fullscreen(&self, source_state:&FullscreenStateData<T>) {
        let animation_data = source_state.prepare_non_fullscreen_animation();
        let new_state      = StateModel::TransitioningFromFullscreen { animation_data };
        self.state.replace(new_state);
        self.animation.set_position(0.0);
        self.animation.set_target_position(1.0);
    }

    /// Start the transition to fullscreen mode. Triggers the UI component state change and
    /// starts the animation.
    fn transition_to_fullscreen(&self,target:T, scene:Scene) {
        let target_state   = FullscreenStateData::new(target, scene);
        let animation_data = target_state.prepare_fullscreen_animation();
        let new_state      = StateModel::TransitioningToFullscreen { animation_data,target_state };
        self.state.replace(new_state);
        self.animation.set_position(0.0);
        self.animation.set_target_position(1.0);
    }

    /// Return a ref clone of the fullscreen element.
    pub fn get_element(&self) -> Option<T> {
        match self.state.borrow().deref() {
            StateModel::Fullscreen{ data, .. } => Some(data.target.clone_ref()),
            _                              => None,
        }
    }
}



// ============================
// === FullscreenStateData  ===
// ============================

/// The `FullscreenStateData` preserves the initial state of a UI element and provides functionality
/// to transition the UI component from/to fullscreen state. It handles the direct interactions with
/// the UI component (setting of size/layers) outside of the animation and and provides the target
/// state for the animation.
#[derive(Debug,Clone)]
pub struct FullscreenStateData<T> {
    target            : T,
    scene             : Scene,
    size_original     : Vector3<f32>,
    position_original : Vector3<f32>,
    parent_original   : Option<display::object::Instance>,
}


impl<T:Fullscreenable> FullscreenStateData<T> {
    /// Make the provided target fullscreen within the given scene and return the
    /// `FullscreenOperator`.
    pub fn new(target:T, scene:Scene) -> Self {
        let size_original     = target.size();
        let position_original = target.position();
        let parent_original   = target.display_object().rc.parent();
        FullscreenStateData {target,scene,size_original,position_original,parent_original}
    }

    /// Prepare the target component for the fullscreen animation and return the animation target
    /// state.
    fn prepare_fullscreen_animation(&self) -> AnimationTargetState<T> {
        let source_pos = self.target.display_object().global_position();

        // Change parent
        self.target.display_object().set_parent(self.scene.display_object());
        self.target.set_layers_fullscreen(&self.scene);
        self.target.set_dom_layers_overlay(&self.scene);
        self.target.enable_fullscreen_decoration();

        let margin      = 0.0;
        let scene_shape = self.scene.shape();
        let size_new    = Vector3::new(scene_shape.width(), scene_shape.height(),0.0) * (1.0 - margin);

        // FIXME Currently we assume `Symbols` are center aligned, but they might not be.
        // We should check the alignment here and change the computations accordingly.

        let target_pos  = size_new / 2.0;
        let source_size = self.size_original;
        let target_size = size_new;
        self.scene.views.toggle_overlay_cursor();

        AnimationTargetState {
            target: self.target.clone_ref(),
            source_pos,
            source_size,
            target_pos,
            target_size,
        }
    }
    /// Prepare the target component for the non-fullscreen animation and return the animation target
    /// state.
    fn prepare_non_fullscreen_animation(&self) -> AnimationTargetState<T> {
        let global_pos_start = self.target.global_position();

        self.target.set_layers_normal(&self.scene);
        self.target.set_dom_layers_normal(&self.scene);
        self.target.disable_fullscreen_decoration();

        if let Some(parent) = self.parent_original.as_ref() {
            self.target.display_object().set_parent(&parent);
        }

        let parent_pos     = self.parent_original.as_ref().map(|p| p.global_position());
        let parent_pos     = parent_pos.unwrap_or_else(Vector3::zero);
        let mut source_pos = self.target.position();
        source_pos        += global_pos_start ;
        source_pos        -= parent_pos ;
        let source_pos     = source_pos;

        self.target.set_position(source_pos);

        let source_pos  = self.target.position();
        let target_pos  = self.position_original;
        let source_size = self.target.size();
        let target_size = self.size_original;


        self.scene.views.toggle_overlay_cursor();
        AnimationTargetState {
            target: self.target.clone_ref(),
            source_pos,
            source_size,
            target_pos,
            target_size,
        }
    }

    fn make_resize_handle(&self) -> callback::Handle {
        let target = self.target.clone_ref();
        self.scene.on_resize(enclose!((target) move |scene_shape:&web::dom::ShapeData| {
            let size_new  = Vector3::new(scene_shape.width(), scene_shape.height(),0.0);
            target.set_size(size_new);
        }))
    }
}
