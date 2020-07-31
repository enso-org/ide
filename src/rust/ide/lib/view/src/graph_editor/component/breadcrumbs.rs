//! This module provides a view for breadcrumbs, enabling us to know which node the graph being
//! edited belongs to and navigating through them.

use crate::prelude::*;

pub mod breadcrumb;
pub mod project_name;

pub use breadcrumb::Breadcrumb;
pub use project_name::ProjectName;

use crate::graph_editor::LocalCall;

use enso_frp as frp;
use ensogl::display;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use ensogl::display::shape::text::text_field::FocusManager;
use logger::enabled::Logger;
use logger::AnyLogger;



// =================
// === Constants ===
// =================

// FIXME[dg] hardcoded literal for glyph of height 12.0. Copied from port.rs
const GLYPH_WIDTH       : f32 = 7.224_609_4;
const VERTICAL_MARGIN   : f32 = GLYPH_WIDTH;
const HORIZONTAL_MARGIN : f32 = GLYPH_WIDTH;
const TEXT_SIZE         : f32 = 12.0;



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpInputs {
    pub push_breadcrumb             : frp::Source<Option<LocalCall>>,
    pub pop_breadcrumb              : frp::Source,
    pub outside_press               : frp::Source,
    pub cancel_project_name_editing : frp::Source,
    pub project_name                : frp::Source<String>,
}

impl FrpInputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            push_breadcrumb             <- source();
            pop_breadcrumb              <- source();
            outside_press               <- source();
            cancel_project_name_editing <- source();
            project_name                <- source();
        }
        Self{push_breadcrumb,pop_breadcrumb,outside_press,cancel_project_name_editing,project_name}
    }
}



// ==================
// === FrpOutputs ===
// ==================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpOutputs {
    pub breadcrumb_push : frp::Source<Option<LocalCall>>,
    pub breadcrumb_pop  : frp::Source,
    pub project_name    : frp::Any<String>
}

impl FrpOutputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            breadcrumb_push <- source();
            breadcrumb_pop  <- source();
            project_name    <- any_mut();
        }
        Self{breadcrumb_push,breadcrumb_pop,project_name}
    }
}



// ===========
// === Frp ===
// ===========

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Frp {
    pub inputs  : FrpInputs,
    pub outputs : FrpOutputs,
    pub network : frp::Network,
}

impl Deref for Frp {
    type Target = FrpInputs;
    fn deref(&self) -> &Self::Target {
        &self.inputs
    }
}

impl Default for Frp {
    fn default() -> Self {
        Self::new()
    }
}

impl Frp {
    /// Constructor.
    pub fn new() -> Self {
        let network = frp::Network::new();
        let inputs  = FrpInputs::new(&network);
        let outputs = FrpOutputs::new(&network);
        Self{network,inputs,outputs}
    }
}



// ========================
// === BreadcrumbsModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct BreadcrumbsModel {
    logger                : Logger,
    display_object        : display::object::Instance,
    project_name          : ProjectName,
    breadcrumbs_container : display::object::Instance,
    scene                 : Scene,
    breadcrumbs           : Rc<RefCell<Vec<Breadcrumb>>>,
    frp_outputs           : FrpOutputs,
    current_index         : Rc<Cell<usize>>
}

impl BreadcrumbsModel {
    /// Constructor.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S, frp:&Frp, focus_manager:&FocusManager) -> Self {
        let scene                 = scene.into();
        let project_name          = ProjectName::new(scene,focus_manager);
        let logger                = Logger::new("Breadcrumbs");
        let display_object        = display::object::Instance::new(&logger);
        let breadcrumbs_container = display::object::Instance::new(&logger);
        let scene                 = scene.clone_ref();
        let breadcrumbs           = default();
        let frp_outputs           = frp.outputs.clone_ref();
        let current_index         = default();
        Self{logger,display_object,scene,breadcrumbs,frp_outputs,
            project_name,breadcrumbs_container,current_index}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.project_name);
        self.add_child(&self.breadcrumbs_container);
        self.project_name.set_position(Vector3(HORIZONTAL_MARGIN,0.0,0.0));
        self.relayout_for_project_name_width(self.project_name.width());
        self
    }

    fn width(&self) -> f32 {
        self.breadcrumbs.borrow().iter().map(|breadcrumb| breadcrumb.width()).sum()
    }

    fn relayout_for_project_name_width(&self, width:f32) {
        self.breadcrumbs_container.set_position_x(HORIZONTAL_MARGIN + width);
    }

    fn get_breadcrumb(&self, index:usize) -> Option<Breadcrumb> {
        if index > 0 {
            self.breadcrumbs.borrow_mut().get(index - 1).map(|breadcrumb| breadcrumb.clone_ref())
        } else {
            None
        }
    }

    /// Selects the breadcrumb identified by its `index` and returns `(popped_count,local_calls)`,
    /// where `popped_count` is the number of breadcrumbs in the right side of `index` that needs to
    /// be popped or a list of `LocalCall`s identifying the breadcrumbs we need to push.
    fn select_breadcrumb(&self, index:usize) -> (usize,Vec<Option<LocalCall>>) {
        info!(self.logger,"Selecting breadcrumb #{index}.");
        let current_index = self.current_index.get();
        if index < current_index {
            (current_index - index, default())
        } else if index > current_index {
            let mut local_calls = Vec::new();
            for index in current_index..index {
                let info = self.breadcrumbs.borrow().get(index).map(|breadcrumb| {
                    let definition = breadcrumb.info.method_pointer.clone();
                    let call       = breadcrumb.info.expression_id;
                    LocalCall{call,definition}
                }).as_ref().cloned();
                if info.is_some() {
                    local_calls.push(info);
                } else {
                    error!(self.logger, "LocalCall info is not present.");
                    self.remove_breadcrumbs_history_beginning_from(index);
                    break;
                }
            }
            (default(),local_calls)
        } else {
            (default(),default())
        }
    }

    fn push_breadcrumb(&self, local_call:&Option<LocalCall>) -> Option<(usize,usize)> {
        if let Some(local_call) = local_call {
            let method_pointer = &local_call.definition;
            let expression_id  = &local_call.call;
            let old_index      = self.current_index.get();
            let new_index      = old_index + 1;

            let breadcrumb_exists =
                self.breadcrumbs.borrow_mut().get(old_index).contains_if(|breadcrumb| {
                    breadcrumb.info.expression_id == *expression_id
                });

            if breadcrumb_exists {
                debug!(self.logger, "Entering an existing {method_pointer.name} breadcrumb.");
            } else {
                debug!(self.logger, "Creating a new {method_pointer.name} breadcrumb.");
                self.remove_breadcrumbs_history_beginning_from(self.current_index.get());
                let breadcrumb       = Breadcrumb::new(&self.scene, method_pointer, expression_id);
                let network          = &breadcrumb.frp.network;
                let breadcrumb_index = new_index;
                let model            = self.clone_ref();


                // === User Interaction ===

                frp::extend! { network
                    selected <- breadcrumb.frp.outputs.selected.map(f!((_)
                        model.select_breadcrumb(breadcrumb_index)
                    ));
                    popped_count <= selected.map(f!([](selected) (0..selected.0).collect_vec()));
                    local_calls  <= selected.map(f!([](selected) selected.1.clone()));
                    eval popped_count((_) model.frp_outputs.breadcrumb_pop.emit(()));
                    eval local_calls((local_call)
                        model.frp_outputs.breadcrumb_push.emit(local_call)
                    );
                }

                info!(self.logger, "Pushing {breadcrumb.info.method_pointer.name} breadcrumb.");
                breadcrumb.set_position(Vector3(self.width(),0.0,0.0));
                self.breadcrumbs_container.add_child(&breadcrumb);
                self.breadcrumbs.borrow_mut().push(breadcrumb);
            }
            self.current_index.set(new_index);
            Some((old_index,new_index))
        } else {
            None
        }
    }

    fn pop_breadcrumb(&self) -> Option<(usize,usize)> {
        debug!(self.logger, "Popping {self.current_index.get()}");
        if self.current_index.get() > 0 {
            info!(self.logger, "Popping breadcrumb view.");
            let old_index = self.current_index.get();
            let new_index = old_index - 1;
            self.current_index.set(new_index);
            Some((old_index,new_index))
        } else {
            None
        }
    }

    fn remove_breadcrumbs_history_beginning_from(&self, index:usize) {
        for breadcrumb in self.breadcrumbs.borrow_mut().split_off(index) {
            info!(self.logger, "Removing {breadcrumb.info.method_pointer.name}.");
            breadcrumb.unset_parent();
        }
    }
}

impl display::Object for BreadcrumbsModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}



// ===================
// === Breadcrumbs ===
// ===================

/// The Breadcrumb's view used for visualizing the breadcrumbs and navigating them.
#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Breadcrumbs {
    #[shrinkwrap(main_field)]
    model   : Rc<BreadcrumbsModel>,
    pub frp : Frp
}

impl Breadcrumbs {
    /// Constructor.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S, focus_manager:&FocusManager) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(BreadcrumbsModel::new(scene,&frp,focus_manager));
        let network = &frp.network;


        // === Breadcrumb selection ===

        frp::extend! { network

            // === Pushing ===
            indices <= frp.push_breadcrumb.map(f!((local_call) model.push_breadcrumb(local_call)));
            old_breadcrumb <- indices.map(f!((indices) model.get_breadcrumb(indices.0)));
            new_breadcrumb <- indices.map(f!((indices) model.get_breadcrumb(indices.1)));
            eval old_breadcrumb([] (breadcrumb) breadcrumb.as_ref().map(|breadcrumb| {
                breadcrumb.frp.deselect.emit(());
            }));
            eval new_breadcrumb([] (breadcrumb) breadcrumb.as_ref().map(|breadcrumb| {
                breadcrumb.frp.select.emit(());
                breadcrumb.frp.fade_in.emit(());
            }));

            // === Popping ===
            indices <= frp.pop_breadcrumb.map(f!((_) model.pop_breadcrumb()));
            old_breadcrumb <- indices.map(f!((indices) model.get_breadcrumb(indices.0)));
            new_breadcrumb <- indices.map(f!((indices) model.get_breadcrumb(indices.1)));
            eval old_breadcrumb([] (breadcrumb) breadcrumb.as_ref().map(|breadcrumb| {
                breadcrumb.frp.deselect.emit(());
            }));
            eval new_breadcrumb([] (breadcrumb) breadcrumb.as_ref().map(|breadcrumb| {
                breadcrumb.frp.select.emit(());
            }));
        }


        // === Project Name ===

        frp::extend! { network
            eval frp.project_name((name) model.project_name.frp.name.emit(name));
            frp.outputs.project_name <+ model.project_name.frp.outputs.name;
        }


        // === GUI Update ===

        frp::extend! { network
            eval model.project_name.frp.outputs.width((width) {
                model.relayout_for_project_name_width(*width)
            });
        }


        // === User Interaction ===

        frp::extend! {network
            _mouse_down <- model.project_name.frp.outputs.mouse_down.map(f!([model] (_) {
                let (popped_count,local_calls) = model.select_breadcrumb(0);
                for _ in 0..popped_count {
                    model.frp_outputs.breadcrumb_pop.emit(());
                }
                for local_call in local_calls {
                    model.frp_outputs.breadcrumb_push.emit(local_call);
                }
            }));
            eval_ frp.cancel_project_name_editing(model.project_name.frp.cancel_editing.emit(()));
            eval_ frp.outside_press(model.project_name.frp.outside_press.emit(()));
        }

        Self{frp,model}
    }
}

impl display::Object for Breadcrumbs {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
