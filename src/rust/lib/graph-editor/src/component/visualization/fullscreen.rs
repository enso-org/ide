//! Provides the fullscreen operation that can be applied to UI components.
use crate::prelude::*;

//use crate::component::visualization::traits::HasFullscreenDecoration;
//use crate::component::visualization::traits::Resizable;
//
//use ensogl::control::callback;
//use ensogl::display::Scene;
//use ensogl::display::traits::*;
//use ensogl::display;
//use ensogl::frp;
//use ensogl::gui::component::Animation;
//use ensogl::system::web;
//
//
//
//// ====================
//// === Helper Types ===
//// ====================
//
//pub trait FrpEntity = Debug + CloneRef + 'static;
//
//pub trait Fullscreenable = FrpEntity + display::Object + Resizable + HasFullscreenDecoration;
//
//
//
//// =========================
//// === Animation Helpers ===
//// =========================
//
///// Weak ref to the inner `State` of the `StateModel`.
//type WeakState<T> = Weak<RefCell<StateModel<T>>>;
//
///// Initial and target state of the animated component. This is used in the animation to interpolate between
///// the initial and the target state.
//#[derive(Debug,Clone)]
//struct AnimationTargetState<T> {
//    /// Animated UI component.
//    target      : T,
//    /// Position at the start of the animation.
//    source_pos  : Vector3<f32>,
//    /// Size at the start of the animation.
//    source_size : Vector3<f32>,
//    /// Position at the end of the animation.
//    target_pos  : Vector3<f32>,
//    /// Size at the end of the animation.
//    target_size : Vector3<f32>,
//}
//
//impl<T:Fullscreenable> AnimationTargetState<T> {
//    /// Set the interpolated values on the target.
//    fn apply_interpolation(&self, value:f32) {
//        let target      = &self.target;
//        let source_pos  = self.source_pos;
//        let source_size = self.source_size;
//        let target_pos  = self.target_pos;
//        let target_size = self.target_size;
//        let pos         = source_pos  * (1.0 - value) + target_pos * value;
//        let size        = source_size * (1.0 - value) + target_size * value;
//        target.set_position(pos);
//        target.set_size(size);
//    }
//}
//
///// Helper function that contains the logic of the animation. Ensure the animation is only executed
///// in the animation state, and executes the state transfer when the animation has finished.
//fn transition_animation_fn<T:Fullscreenable>(state: WeakState<T>, value:f32) {
//    if let Some(state) = state.upgrade() {
//        // Regular animation step
//        // We animate only if the correct state is given.
//        match state.borrow().deref(){
//            StateModel::TransitioningFromFullscreen {animation_data, .. }
//            | StateModel::TransitioningToFullscreen { animation_data,.. } => {
//                animation_data.apply_interpolation(value);
//            },
//            _  => ()
//        }
//        // Check for end of animation and update the state.
//        if value >= 1.0 {
//           state.borrow_mut().animation_end_transition();
//        }
//    }
//}
//
//
//
//// ============================
//// === FullscreenStateData  ===
//// ============================
//
///// The `FullscreenStateData` preserves the initial state of a UI element, so it can be restored
///// after it leaves the fullscreen mode again. It also provides functionality
///// to transition the UI component from/to fullscreen state. It handles the direct interactions with
///// the UI component (setting of size/layers) outside of the animation and and provides the target
///// state for the animation.
//#[derive(Debug,Clone)]
//pub struct FullscreenStateData<T> {
//    target            : T,
//    scene             : Scene,
//    size_original     : Vector3<f32>,
//    position_original : Vector3<f32>,
//    parent_original   : Option<display::object::Instance>,
//}
//
//impl<T:Fullscreenable> FullscreenStateData<T> {
//    /// Make the provided target fullscreen within the given scene and return the
//    /// `FullscreenOperator`.
//    pub fn new(target:T, scene:Scene) -> Self {
//        let size_original     = target.size();
//        let position_original = target.position();
//        let parent_original   = target.display_object().rc.parent();
//        FullscreenStateData {target,scene,size_original,position_original,parent_original}
//    }
//
//    /// Prepare the target component for the fullscreen animation and return the animation target
//    /// state.
//    fn prepare_fullscreen_animation(&self) -> AnimationTargetState<T> {
//        let scene_shape = self.scene.shape();
//        let source_pos  = self.target.display_object().global_position();
//        let source_pos  = self.scene.views.main.camera.apply_transform(source_pos);
//        let scene_delta = Vector3::new(scene_shape.width(),  scene_shape.height(), 0.0);
//        let source_pos  = source_pos + scene_delta;
//
//        // Change parent
//        self.target.display_object().set_parent(self.scene.display_object());
//        self.target.set_layers_fullscreen(&self.scene);
//        self.target.set_dom_layers_overlay(&self.scene);
//        self.target.enable_fullscreen_decoration();
//
//
//        // FIXME Currently we assume `Symbols` are center aligned, but they might not be.
//        // We should check the alignment here and change the computations accordingly.
//        let size_new    = Vector3::new(scene_shape.width(), scene_shape.height(),0.0);
//        let target_pos  = size_new / 2.0;
//        let source_size = self.size_original * self.scene.views.main.camera.zoom();
//        let target_size = size_new;
//        self.scene.views.toggle_overlay_cursor();
//
//        AnimationTargetState {
//            target: self.target.clone_ref(),
//            source_pos,
//            source_size,
//            target_pos,
//            target_size,
//        }
//    }
//    /// Prepare the target component for the non-fullscreen animation and return the animation target
//    /// state.
//    fn prepare_non_fullscreen_animation(&self) -> AnimationTargetState<T> {
//        let global_pos_start = self.target.global_position();
//
//        self.target.set_layers_normal(&self.scene);
//        self.target.set_dom_layers_normal(&self.scene);
//        self.target.disable_fullscreen_decoration();
//
//        if let Some(parent) = self.parent_original.as_ref() {
//            self.target.display_object().set_parent(&parent);
//        }
//
//        let parent_pos     = self.parent_original.as_ref().map(|p| p.global_position());
//        let parent_pos     = parent_pos.unwrap_or_else(Vector3::zero);
//        let mut source_pos = self.target.position();
//        source_pos        += global_pos_start ;
//        source_pos        -= parent_pos ;
//        let source_pos     = source_pos;
//
//        self.target.set_position(source_pos);
//
//        let source_pos  = self.target.position();
//        let target_pos  = self.position_original;
//        let source_size = self.target.size();
//        let target_size = self.size_original;
//
//        self.scene.views.toggle_overlay_cursor();
//        AnimationTargetState {
//            target: self.target.clone_ref(),
//            source_pos,
//            source_size,
//            target_pos,
//            target_size,
//        }
//    }
//
//    fn make_resize_handle(&self) -> callback::Handle {
//        let target = self.target.clone_ref();
//        self.scene.on_resize(enclose!((target) move |scene_shape:&web::dom::ShapeData| {
//            let size_new  = Vector3::new(scene_shape.width(), scene_shape.height(),0.0);
//            target.set_size(size_new);
//        }))
//    }
//}
//
//
//
//// ==================
//// === StateModel ===
//// ==================
//
///// Represents the internal state of the FullscreenState. This is used to ensure animations always
///// finish without interruption.
/////
///// There sre two main states `Fullscreen` and `NotFullscreen` and two transition states,
///// `TransitioningFromFullscreen` and `TransitioningToFullscreen`. These help to ensure, that
///// transitions from one state to the other can be aborted, and the correct state for the animated
///// component can be restored, before the next animation is started.
/////
///// Consider the following diagram
/////
///// ```text
/////                      o---->   `Fullscreen`   -----o
/////                      |                            |
/////                      |                            V
/////    `TransitioningToFullscreen`              `TransitioningFromFullscreen`
/////                       ^                            |
/////                       |                            |
/////                       o----  `NotFullscreen`  <----o
///// ```
/////
///// If we are in one of the intermediary stages and receive an event that tells us to go back to
///// either of the other stages, we need to finish/abort to leave the component in transition in a
///// sane state. Only then can we start the new transition. That is why we need to keep track of the
///// intermediary states.
//#[derive(Debug,Clone)]
//enum StateModel<T> {
//    /// There is a UI component and it is in fullscreen mode.
//    Fullscreen {
//        /// The state of the component that will be in fullscreen mode.
//        data           : FullscreenStateData<T>,
//        /// The animation handle that keeps the size of the fullscreen component in sync with the
//        /// scene.
//        resize_handle  : callback::Handle,
//    },
//    /// There is an animation running from fullscreen mode to non-fullscreen mode.
//    TransitioningFromFullscreen {
//        /// The data required to run an animation.
//        animation_data : AnimationTargetState<T>
//    },
//    /// There is an animation running from non-fullscreen mode to fullscreen mode.
//    TransitioningToFullscreen {
//        /// The data required to run an animation.
//        animation_data : AnimationTargetState<T>,
//        /// The state of the component that will be in fullscreen mode.
//        target_state   : FullscreenStateData<T>
//    },
//    /// There is no UI component in fullscreen mode.
//    NotFullscreen
//}
//
//impl<T:Fullscreenable> StateModel<T> {
//    /// Called to indicate the the running animation has ended. Changes the state to the correct
//    /// follow up state. Does nothing if no animation was running.
//    fn animation_end_transition(&mut self) {
//        let new_state = match self.clone() {
//            StateModel::TransitioningFromFullscreen { .. } => {
//                StateModel::NotFullscreen
//            }
//            StateModel::TransitioningToFullscreen { target_state, ..} => {
//                // If we enter fullscreen mode, we also need to ensure the the component stays the
//                // same size as the scene. This handle is dropped if the state changes.
//                let resize_handle = target_state.make_resize_handle();
//                StateModel::Fullscreen { data: target_state, resize_handle }
//            }
//            other    => other,
//        };
//        *self = new_state;
//    }
//
//    /// Returns whether there is a component that is in fullscreen mode. Animation phases count as
//    /// still in fullscreen mode.
//    pub fn is_fullscreen(&self) -> bool {
//        match self {
//            StateModel::NotFullscreen{..} => false,
//            _                             => true,
//        }
//    }
//
//    /// Indicates whether this state is an animation state.
//    pub fn is_animation_state(&self) -> bool {
//        match self {
//            StateModel::TransitioningFromFullscreen{..} => true,
//            StateModel::TransitioningToFullscreen{..}   => true,
//            _                                           => false,
//        }
//    }
//
//    /// Return the `FullscreenStateData` if we are in fullscreen mode.
//    pub fn get_fullscreen_data(&self) -> Option<FullscreenStateData<T>> {
//        if let StateModel::Fullscreen {data, ..} = self {
//            Some(data.clone())
//        } else {
//            None
//        }
//    }
//
//    /// Return a ref clone of the fullscreen element.
//    pub fn get_target_component(&self) -> Option<T> {
//        match self {
//            StateModel::Fullscreen{ data, .. } => Some(data.target.clone_ref()),
//            _                              => None,
//        }
//    }
//}
//
//impl<T> Default for StateModel<T> {
//    fn default() -> Self {
//        StateModel::NotFullscreen
//    }
//}
//
//
//
//// =======================
//// === FullscreenState ===
//// =======================
//
///// The `FullscreenState` manages the state changes between fullscreen mode and non-fullscreen mode
///// for a UI component. It creates animations for the state changes and ensure that the component
///// cannot come into an illegal state during the transition.
/////
///// This is achieved by having an internal state enum `StateModel` that tracks the state we are in.
///// This wa we can handle state changes correctly, e.g. restore the correct state if we need to
///// abort/finish an animation,
//#[derive(Debug,CloneRef,Derivative)]
//#[derivative(Clone(bound=""))]
//pub struct FullscreenState<T:FrpEntity> {
//        state     : Rc<RefCell<StateModel<T>>>,
//        animation : Animation<f32>,
//}
//
//impl<T:Fullscreenable> FullscreenState<T> {
//
//    fn new(network:&frp::Network) -> Self {
//        let state      = Rc::new(RefCell::new(StateModel::<T>::default()));
//        let weak_state = Rc::downgrade(&state);
//        let animation  = Animation::new(&network);
//
//        frp::extend! { network
//            eval animation.value ([](value) transition_animation_fn(weak_state.clone_ref(),*value));
//        }
//
//        FullscreenState{state,animation}
//    }
//
//    /// Returns whether there is a component that is in fullscreen mode. Animation phases count as
//    /// still in fullscreen mode.
//    pub fn is_fullscreen(&self) -> bool {
//        self.state.borrow().is_fullscreen()
//    }
//
//    /// Enables fullscreen mode for the given component.
//    pub fn enable_fullscreen(&self, target:T, scene:Scene) {
//        if !self.is_fullscreen() {
//            self.transition_to_fullscreen(target, scene)
//        }
//    }
//
//    /// Disables fullscreen mode for the given component.
//    pub fn disable_fullscreen(&self) {
//        let data =  self.state.borrow().get_fullscreen_data();
//        if let Some(data) = data {
//            self.transition_to_non_fullscreen(&data)
//        }
//    }
//
//    /// Start the transition to non-fullscreen mode. Triggers the UI component state change and
//    /// starts the animation.
//    fn transition_to_non_fullscreen(&self, source_state:&FullscreenStateData<T>) {
//        self.abort_animation();
//        let animation_data = source_state.prepare_non_fullscreen_animation();
//        let new_state      = StateModel::TransitioningFromFullscreen { animation_data };
//        self.state.replace(new_state);
//        self.start_animation();
//    }
//
//    /// Start the transition to fullscreen mode. Triggers the UI component state change and
//    /// starts the animation.
//    fn transition_to_fullscreen(&self,target:T, scene:Scene) {
//        self.abort_animation();
//        let target_state   = FullscreenStateData::new(target, scene);
//        let animation_data = target_state.prepare_fullscreen_animation();
//        let new_state      = StateModel::TransitioningToFullscreen { animation_data,target_state };
//        self.state.replace(new_state);
//        self.start_animation();
//    }
//
//    /// Start the animation from 0.0;
//    fn start_animation(&self) {
//        self.animation.set_value(0.0);
//        self.animation.set_target_value(1.0);
//    }
//
//    /// Abort and immediately finish the animation, if it is running.
//    fn abort_animation(&self) {
//        if self.state.borrow().is_animation_state() {
//            self.animation.set_value(1.0);
//        }
//    }
//
//    /// Return a ref clone of the fullscreen element.
//    pub fn get_fullscreen_component(&self) -> Option<T> {
//       self.state.borrow().get_target_component()
//    }
//}
//
//
//
//// ==========================
//// === FullscreenStateFrp ===
//// ==========================
//
///// FullscreenStateFrp events.
//#[derive(CloneRef,Debug,Derivative)]
//#[derivative(Clone(bound=""))]
//#[allow(missing_docs)]
//pub struct FullscreenStateFrp<T:FrpEntity> {
//    pub set_fullscreen     : frp::Source<Option<(T,Scene)>>,
//    pub disable_fullscreen : frp::Source,
//}
//
//impl<T:FrpEntity> FullscreenStateFrp<T> {
//    fn new(network:&frp::Network) -> Self {
//        frp::extend! { network
//            def set_fullscreen     = source();
//            def disable_fullscreen = source();
//        }
//        Self {set_fullscreen,disable_fullscreen}
//    }
//}
//
//
//
//// ==============================
//// === FullscreenStateHandle  ===
//// ==============================
//
///// FullscreenStateHandle that ensures separation of FRP, network and internal data. Is
///// `clone_ref`able for use in FRPs.
//#[derive(Clone,CloneRef,Debug,Derivative,Shrinkwrap)]
//#[derivative(PartialEq)]
//#[allow(missing_docs)]
//pub struct FullscreenStateHandle<T:FrpEntity> {
//    #[derivative(PartialEq="ignore")]
//    network   : frp::Network,
//    #[derivative(PartialEq="ignore")]
//    pub frp   : FullscreenStateFrp<T>,
//    #[derivative(PartialEq(compare_with="Rc::ptr_eq"))]
//    #[shrinkwrap(main_field)]
//    pub state : Rc<FullscreenState<T>>
//}
//
//impl<T:Fullscreenable> FullscreenStateHandle<T>{
//
//    fn new() -> Self {
//        let network = frp::Network::new();
//        let frp     = FullscreenStateFrp::new(&network);
//        let state   = FullscreenState::new(&network);
//        let state   = Rc::new(state);
//        FullscreenStateHandle {network,frp,state }.init_frp()
//    }
//
//    fn init_frp(self) -> Self {
//        let frp     = &self.frp;
//        let network = &self.network;
//        let data    = self.state.clone_ref();
//
//        frp::extend! { network
//            eval frp.disable_fullscreen ((_) data.disable_fullscreen());
//
//            def _set_fullscreen =  frp.set_fullscreen.map(f!([data](target_data) {
//                 if let Some((target,scene)) = target_data {
//                    data.enable_fullscreen(target.clone_ref(),scene.clone_ref())
//                } else {
//                    data.disable_fullscreen()
//                }
//            }));
//        };
//        self
//    }
//}
//
//impl<T:Fullscreenable> Default for FullscreenStateHandle<T>{
//    fn default() -> Self {
//        Self::new()
//    }
//}
