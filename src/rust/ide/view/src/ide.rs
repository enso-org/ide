use crate::prelude::*;

use crate::project;



ensogl::define_endpoints! {
    Input {
    }

    Output {
        new_project_creation_requested ()
    }
}



pub struct Model {
    project: project::View
}

pub struct View {

}
