pub mod project;

use crate::prelude::*;

use enso_frp as frp;



#[derive(Clone,CloneRef,Debug)]
struct Model {
    view: ide_view::ide::View,
    project_integration: project::Integration,
    project_manager: controller::project_manager::Ide,
}

impl Model {
    fn new(view:ide_view::ide::View, project_manager: controller::project_manager::Ide) {

    }

    fn open_project
}

#[derive(Clone,CloneRef,Debug)]
pub struct Integration {
    frp: frp::Network,
    model: Model,
}

