//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;

use crate::controller::*;

use json_rpc::Transport;
use weak_table::weak_value_hash_map::Entry::Occupied;
use weak_table::weak_value_hash_map::Entry::Vacant;
use file_manager_client as fmc;
use shapely::shared;



shared! { ControllerHandle

    /// Project controller's state.
    #[derive(Debug)]
    pub struct State {
        /// File Manager Client.
        file_manager: fmc::ClientHandle,
        /// Cache of module controllers.
        module_cache: WeakValueHashMap<module::Location, module::WeakControllerHandle>,
        /// Cache of text controllers.
        text_cache: WeakValueHashMap<file_manager_client::Path,text::WeakControllerHandle>,
    }

    impl {
        /// Create a new project controller.
        ///
        /// The remote connections should be already established.
        pub fn new(file_manager_transport:impl Transport + 'static) -> Self {
            State {
                file_manager           : fmc::ClientHandle::new(file_manager_transport),
                module_cache           : default(),
                text_cache             : default(),
            }
        }

        /// Returns a module controller for given module location.
        pub fn open_module(&mut self, loc:module::Location) -> module::ControllerHandle {
            match self.module_cache.entry(loc.clone()) {
                Occupied(entry) => entry.get().clone(),
                Vacant(entry)   => entry.insert(module::ControllerHandle::new(loc)),
            }
        }

        /// Returns a text controller for given file path.
        pub fn open_text_file(&mut self, path:file_manager_client::Path) -> text::ControllerHandle {
            let fm = self.file_manager.clone();
            match self.text_cache.entry(path.clone()) {
                Occupied(entry) => entry.get().clone(),
                Vacant(entry)   => entry.insert(text::ControllerHandle::new(path,fm)),
            }
        }
    }
}



#[cfg(test)]
mod test {
    use super::*;

    use file_manager_client::Path;
    use json_rpc::test_util::transport::mock::MockTransport;

    #[test]
    fn obtain_module_controller() {
        let transport        = MockTransport::new();
        let project_ctrl     = ControllerHandle::new(transport);
        let location         = module::Location("TestLocation".to_string());
        let another_location = module::Location("TestLocation2".to_string());

        let module_ctrl         = project_ctrl.open_module(location.clone());
        let same_module_ctrl    = project_ctrl.open_module(location.clone());
        let another_module_ctrl = project_ctrl.open_module(another_location.clone());

        assert_eq!(location        , module_ctrl        .location_clone());
        assert_eq!(another_location, another_module_ctrl.location_clone());
        assert!(module_ctrl.identity_equals(&same_module_ctrl));
    }

    #[test]
    fn obtain_text_controller() {
        let transport           = MockTransport::new();
        let project_ctrl        = ControllerHandle::new(transport);
        let file_manager_handle = project_ctrl.with_borrowed(|s| s.file_manager.clone());
        let path                = Path("TestPath".to_string());
        let another_path        = Path("TestPath2".to_string());

        let text_ctrl         = project_ctrl.open_text_file(path.clone());
        let same_text_ctrl    = project_ctrl.open_text_file(path.clone());
        let another_text_ctrl = project_ctrl.open_text_file(another_path.clone());

        assert!(file_manager_handle.identity_equals(&text_ctrl        .file_manager()));
        assert!(file_manager_handle.identity_equals(&another_text_ctrl.file_manager()));
        assert_eq!(path        , text_ctrl        .file_path_clone()  );
        assert_eq!(another_path, another_text_ctrl.file_path_clone()  );
        assert!(text_ctrl.identity_equals(&same_text_ctrl));
    }
}