//! Defines a `Stage` struct that helps to manage the visualisations in the `Scene`. It keeps track
//! of visualisations coming onto the stage, changing their state, leaving the stage. It also ensure
//! that the behaviour of visualisations stays consistent, e.g., for selection and during
//! fullscreen state changes.

use crate::prelude::*;

use ensogl::display::Scene;
use ensogl::display::Object;
use ensogl::display::object::Id;
use ensogl::display::object::ObjectOps;
use ensogl::frp;

use crate::SharedHashMap;
use crate::SharedHashSet;
use crate::component::visualization::fullscreen::FullscreenState;
use crate::component::visualization::*;
use crate::component::visualization::traits::HasSymbols;



// ===========
// === FRP ===
// ===========

#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct StageFrp {
    pub clicked : frp::Source<Id>,
}

impl StageFrp {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def clicked = source();
        }
        Self {clicked}
    }
}



// =============
// === Stage ===
// =============

/// A stage provides functionality to control a group of UI components and ensure consistent state
/// between state changes, e.g., for selection or switching from/to fullscreen mode.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Stage {
    network             : frp::Network,
    pub frp             : StageFrp,
    logger              : Logger,
    all                 : SharedHashMap<Id,Container>,
    selected            : SharedHashSet<Id>,
    scene               : Scene,
    fullscreen_state    : FullscreenState::<Container>,
}

impl Stage {

    /// Create a new stage for the givens scene.
    pub fn new(scene:Scene,logger:Logger) -> Self{
        let network             = frp::Network::new();
        let frp                 = StageFrp::new(&network);
        let all                 = default();
        let selected            = default();
        let fullscreen_state    = default();
        Self {network,frp,logger,scene,all,selected,fullscreen_state}
    }

    /// Add a new visualisation containers to the stage.
    pub fn push(&self, container:Container) {
        let network = &self.network;
        let frp     = self.frp.clone_ref();

        let id = container.display_object().id();
        frp::extend! { network
            def _clicked = container.frp.clicked.map(move |_| frp.clicked.emit(id));
        }
        container.set_layers_normal(&self.scene);
        self.all.insert(id,container);
    }

    /// Set the given visualisation on the selected container. If no container is selected, nothing
    /// happens.
    pub fn set_vis_for_selected(&self, vis:Visualization) {
        if let Some(container) = self.get_selected() {
            container.set_visualization(vis);
            if self.fullscreen_state.is_fullscreen() {
                container.set_layers_fullscreen(&self.scene)
            } else {
                container.set_layers_normal(&self.scene)
            }
        }
    }

    /// Return whether the given container is in fullscreen mode.
    pub fn is_fullscreen(&self, container:Container) -> bool {
        self.fullscreen_state.get_element() == Some(container)
    }

    /// Return whether the given container is selected.
    pub fn is_selected(&self, id:impl Into<Id>) -> bool {
        self.selected.contains(&id.into())
    }

    /// Return container for the given id, if it exists.
    pub fn get_container(&self, id:impl Into<Id>) -> Option<Container> {
        self.all.get_cloned_ref(&id.into())
    }

    /// Set the given container as selected. Will also propagate the event to the container
    /// and trigger the appropriate animation.
    pub fn set_selected(&self, id:impl Into<Id>) {
        let id:Id = id.into();

        if self.is_selected(&id){
            return
        }

        let container = self.get_container(&id)
            .expect("Invalid selection. Id was selected but not in stage register.");

        self.clear_selection();
        self.selected.insert(id);
        container.frp.select.emit(());

        self.selected.insert(container.display_object().id());
    }

    /// Get the selected visualisation containers.
    pub fn get_selected(&self) -> Option<Container> {
        let selected = self.selected.raw.borrow();
        let selected = selected.iter().take(1).collect_vec();
        let id       = selected.get(0)?;
        self.all.get_cloned(id)
    }

    /// Execute the given closure for the selected container, if there is one.
    pub fn for_selected<T:Fn(Container)> (&self, f:T) {
        self.selected.for_each(|id| f(self.get_container(id).unwrap()))
    }

    /// Remove the selection status from the selected container. Will also end fullscreen status
    /// if the selected container was in fullscreen mode.
    pub fn clear_selection(&self) {
        self.selected.for_each(|id| {
            self.all
                .get_cloned_ref(id)
                .for_each_ref(|container| { container.frp.deselect.emit(()) });
        });
        self.selected.clear();
        self.fullscreen_state.disable_fullscreen();
    }

    /// Change the fullscreen status of the selected container.
    pub fn toggle_fullscreen_for_selected_visualization(&self) {
        if self.fullscreen_state.is_fullscreen() {
            self.fullscreen_state.disable_fullscreen()
        } else if let Some(container) = self.get_selected() {
            container.data.set_visibility(true);
            self.fullscreen_state.enable_fullscreen(container, self.scene.clone_ref());
        }
    }
}
