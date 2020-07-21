//! This module provides a view for breadcrumbs, enabling us to know which node the graph being
//! edited belongs to.

use crate::prelude::*;

pub mod breadcrumb;
pub use breadcrumb::Breadcrumb;

use enso_frp as frp;
use ensogl::display;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use logger::enabled::Logger;
use logger::AnyLogger;



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct FrpInputs {
    /// Push breadcrumb.
    pub push_breadcrumb : frp::Source<String>,
    /// Pop breadcrumb.
    pub pop_breadcrumb : frp::Source,
}

impl FrpInputs {
    /// Create new FrpInputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            def push_breadcrumb = source();
            def pop_breadcrumb  = source();
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
    pub breadcrumb_pop : frp::Source
}

impl FrpOutputs {
    /// Create new FrpOutputs.
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! {network
            def breadcrumb_pop = source();
        }
        Self{breadcrumb_pop}
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
    /// Create new Frp.
    pub fn new() -> Self {
        let network = frp::Network::new();
        let inputs  = FrpInputs::new(&network);
        let outputs = FrpOutputs::new(&network);
        Self{network,inputs,outputs}
    }
}



// ==================
// === Animations ===
// ==================

/// ProjectName's animations handlers.
#[derive(Debug,Clone,CloneRef,Copy)]
pub struct Animations {}

impl Animations {
    /// Create new animations handlers.
    pub fn new(_network:&frp::Network) -> Self {
        Self{}
    }
}



// ========================
// === BreadcrumbsModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct BreadcrumbsModel {
    logger         : Logger,
    animations     : Animations,
    display_object : display::object::Instance,
    scene          : Scene,
    breadcrumbs    : Rc<RefCell<Vec<Breadcrumb>>>,
    breadcrumb_pop : frp::Source
}

impl BreadcrumbsModel {
    /// Create new ProjectNameModel.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S,frp:&Frp) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("Breadcrumbs");
        let display_object = display::object::Instance::new(&logger);
        let animations     = Animations::new(&frp.network);
        let scene          = scene.clone_ref();
        let breadcrumbs    = Rc::new(RefCell::new(default()));
        let breadcrumb_pop = frp.outputs.breadcrumb_pop.clone_ref();
        Self{logger,display_object,animations,scene,breadcrumbs,breadcrumb_pop}
    }

    fn width(&self) -> f32 {
        let mut width = 0.0;
        for breadcrumb in self.breadcrumbs.borrow().iter() {
            width += breadcrumb.width();
        }
        width
    }

    fn select_breadcrumb(&self, index:usize) {
        // If we have more crumbs after `index`, we will pop them.
        let breadcrumb_length = self.breadcrumbs.borrow().len();
        let last_index        = breadcrumb_length - 1;
        let pop_amount        = last_index - index;
        for _ in 0..pop_amount {
            self.breadcrumb_pop.emit(());
        }
        info!(self.logger,"Selecting breadcrumb #{index}");
    }

    fn push_breadcrumb(&self, name:impl Str) {
        let breadcrumb       = Breadcrumb::new(&self.scene,name);
        let network          = &breadcrumb.frp.network;
        let breadcrumb_index = self.breadcrumbs.borrow().len();
        let model            = self.clone_ref();

        frp::extend! { network
            eval_ breadcrumb.frp.outputs.selected(model.select_breadcrumb(breadcrumb_index));
        }

        info!(self.logger,"Pushing {breadcrumb.name} breadcrumb");
        breadcrumb.set_position(Vector3::new(self.width(),0.0,0.0));
        self.add_child(&breadcrumb);
        self.breadcrumbs.borrow_mut().push(breadcrumb);
    }

    fn pop_breadcrumb(&self) {
        info!(self.logger, "Popping breadcrumb view.");
        self.breadcrumbs.borrow_mut().pop().map(|breadcrumb| breadcrumb.unset_parent());
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

/// The project name's view used for visualizing the project name and renaming it.
#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
#[allow(missing_docs)]
pub struct Breadcrumbs {
    #[shrinkwrap(main_field)]
    model   : Rc<BreadcrumbsModel>,
    pub frp : Frp
}

impl Breadcrumbs {
    /// Create a new ProjectName view.
    pub fn new<'t,S:Into<&'t Scene>>(scene:S) -> Self {
        let frp     = Frp::new();
        let model   = Rc::new(BreadcrumbsModel::new(scene,&frp));
        let network = &frp.network;
        frp::extend! { network
            eval frp.push_breadcrumb((name) {model.push_breadcrumb(name)});
            eval_ frp.pop_breadcrumb(model.pop_breadcrumb());
        }
        Self{frp,model}
    }
}

impl display::Object for Breadcrumbs {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
