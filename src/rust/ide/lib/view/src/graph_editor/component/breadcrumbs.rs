//! This module provides a view for breadcrumbs, enabling us to know which node the graph being
//! edited belongs to and navigating through them.

use crate::prelude::*;

pub mod breadcrumb;
pub mod project_name;

pub use breadcrumb::Breadcrumb;
pub use project_name::ProjectName;

use crate::graph_editor::MethodPointer;

use enso_frp as frp;
use enso_protocol::language_server::ExpressionId;
use ensogl::display;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use ensogl::display::shape::text::text_field::FocusManager;
use logger::enabled::Logger;
use logger::AnyLogger;
use std::cmp;



// =================
// === LocalCall ===
// =================

/// A specific function call occurring within another function's definition body.
/// It's closely related to the `LocalCall` type defined in `Language Server` types, but uses the
/// new type `MethodPointer` defined in `GraphEditor`.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct LocalCall {
    /// An expression being a call to a method.
    pub call:ExpressionId,
    /// A pointer to the called method.
    pub definition:MethodPointer,
}



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
    /// Push breadcrumb.
    pub push_breadcrumb:frp::Source<Option<LocalCall>>,
    /// Pop breadcrumb.
    pub pop_breadcrumb:frp::Source,
}

impl FrpInputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            push_breadcrumb <- source();
            pop_breadcrumb  <- source();
        }
        Self{push_breadcrumb,pop_breadcrumb}
    }
}



// ==================
// === FrpOutputs ===
// ==================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpOutputs {
    pub breadcrumb_push : frp::Source<Option<LocalCall>>,
    pub breadcrumb_pop  : frp::Source
}

impl FrpOutputs {
    /// Constructor.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            breadcrumb_push <- source();
            breadcrumb_pop  <- source();
        }
        Self{breadcrumb_push,breadcrumb_pop}
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
    pub project_name      : ProjectName,
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
        self.breadcrumbs_container.set_position(Vector3(HORIZONTAL_MARGIN + width,0.0,0.0));
    }

    fn select_breadcrumb(&self, index:usize) -> Vec<frp::Source> {
        let mut sources = Vec::new();
        let current_index = self.current_index.get();
        match index.cmp(&current_index) {
            cmp::Ordering::Less => {
                // If we have more crumbs after `index`, we will pop them.
                let popped_count = current_index - index;
                for _ in 0..popped_count {
                    sources.push(self.frp_outputs.breadcrumb_pop.clone_ref());
                }
            },
            cmp::Ordering::Greater => {
                for index in current_index..index {
                    let info = self.breadcrumbs.borrow().get(index).map(|breadcrumb| {
                        let definition = breadcrumb.info.method_pointer.clone();
                        let call       = breadcrumb.info.expression_id;
                        LocalCall{call,definition}
                    }).as_ref().cloned();
                    if info.is_some() {
                        self.frp_outputs.breadcrumb_push.emit(info);
                    } else {
                        error!(self.logger, "LocalCall info is invalid.");
                        self.remove_breadcrumbs_history(index);
                        break;
                    }
                }
            },
            cmp::Ordering::Equal => ()
        }
        info!(self.logger,"Selecting breadcrumb #{index}");
        sources
    }

    fn push_breadcrumb(&self, local_call:&Option<LocalCall>) -> Vec<frp::Source> {
        let mut sources = Vec::new();
        if let Some(local_call) = local_call {
            let method_pointer = &local_call.definition;
            let expression_id  = &local_call.call;
            let current_index = self.current_index.get();
            let next_index = current_index + 1;

            let breadcrumb_exists =
                self.breadcrumbs.borrow_mut().get(current_index).contains_if(|breadcrumb| {
                    breadcrumb.info.expression_id == *expression_id
                });

            if breadcrumb_exists {
                debug!(self.logger, "Entering an existing {method_pointer.name} breadcrumb.");
                //TODO[dg]: Highlight breadcrumb.
            } else {
                debug!(self.logger, "Creating a new {method_pointer.name} breadcrumb.");
                self.remove_breadcrumbs_history(self.current_index.get());
                let breadcrumb = Breadcrumb::new(&self.scene, method_pointer, expression_id);
                let network = &breadcrumb.frp.network;
                let breadcrumb_index = next_index;
                let model = self.clone_ref();


                // === User Interaction ===

                frp::extend! { network
                    eval_ breadcrumb.frp.outputs.selected(
                        model.select_breadcrumb(breadcrumb_index).iter().for_each(|source| {
                            source.emit(())
                        })
                    );
                }

                info!(self.logger, "Pushing {breadcrumb.info.method_pointer.name} breadcrumb.");
                breadcrumb.set_position(Vector3(self.width(),0.0,0.0));
                sources.push(breadcrumb.frp.fade_in.clone_ref());
                self.breadcrumbs_container.add_child(&breadcrumb);
                self.breadcrumbs.borrow_mut().push(breadcrumb);
            }
            self.current_index.set(next_index);
            sources.append(&mut self.update_selection());
        }
        sources
    }

    fn pop_breadcrumb(&self) -> Vec<frp::Source> {
        debug!(self.logger, "Popping {self.current_index.get()}");
        if self.current_index.get() > 0 {
            info!(self.logger, "Popping breadcrumb view.");
            self.current_index.set(self.current_index.get() - 1);
            self.update_selection()
        } else {
            default()
        }
    }

    fn update_selection(&self) -> Vec<frp::Source> {
        let mut sources = Vec::new();
        let current_index = self.current_index.get();
        for (index,breadcrumb) in self.breadcrumbs.borrow_mut().iter().enumerate() {
            if index + 1 == current_index {
                sources.push(breadcrumb.frp.select.clone_ref());
            } else {
                sources.push(breadcrumb.frp.deselect.clone_ref());
            }
        }
        sources
    }

    fn remove_breadcrumbs_history(&self, index:usize) {
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
        frp::extend! { network
            eval frp.push_breadcrumb((local_call) {
                model.push_breadcrumb(local_call).iter().for_each(|source| source.emit(()));
            });
            eval_ frp.pop_breadcrumb(
                model.pop_breadcrumb().iter().for_each(|source| source.emit(()))
            );
        }


        // === GUI Update ===

        frp::extend! { network
            eval model.project_name.frp.outputs.width((width) {
                model.relayout_for_project_name_width(*width)
            });
        }


        // === User Interaction ===

        frp::extend! {network
            eval_ model.project_name.frp.outputs.mouse_down(
                model.select_breadcrumb(0).iter().for_each(|source| source.emit(()))
            );
        }


        Self{frp,model}
    }
}

impl display::Object for Breadcrumbs {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
