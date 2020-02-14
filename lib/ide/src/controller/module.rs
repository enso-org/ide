use crate::prelude::*;

use shapely::shared;


/// Structure uniquely identifying module location in the project.
/// Mappable to filesystem path.
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Location(pub String);

impl Location {
    /// Obtains path (within a project context) to the file with this module.
    pub fn to_path(&self) -> file_manager_client::Path {
        // TODO [mwu] Extremely provisional. When multiple files support is
        //            added, needs to be fixed, if not earlier.
        let Location(string) = self;
        let result = format!("./{}.luna", self.0);
        file_manager_client::Path::new(result)
    }
}

shared! { ControllerHandle
    /// State data of the module controller.
    #[derive(Debug)]
    pub struct State {
        /// This module's location.
        location : Location,
    }

    impl {
        pub fn new(location:Location) -> Self {
            State {location}
        }

        pub fn location_clone(&self) -> Location {
            self.location.clone()
        }

        pub fn location_as_path(&self) -> file_manager_client::Path {
            self.location.to_path()
        }
    }
}

impl ControllerHandle {
    /// Receives a notification call when file with this module has been
    /// modified by a third-party tool (like non-IDE text editor).
    pub async fn file_externally_modified(&self) {
        // TODO: notify underlying text/graph controllers about the changes
        todo!()
    }
}