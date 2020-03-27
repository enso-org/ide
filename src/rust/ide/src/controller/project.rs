//! Project controller.
//!
//! Responsible for owning any remote connection clients, and providing controllers for specific
//! files and modules. Expected to live as long as the project remains open in the IDE.

use crate::prelude::*;
use crate::controller::notification::Publisher;

use file_manager_client as fmc;
use json_rpc::Transport;
use parser::Parser;
use weak_table::weak_value_hash_map::Entry::Occupied;
use weak_table::weak_value_hash_map::Entry::Vacant;



// ===============================
// === Module Controller Cache ===
// ===============================

#[derive(Debug,Clone)]
enum StateEntry<Handle,Strong> {
    Loaded(Handle),
    Loading(Publisher<Strong>),
}

type ModuleState          = controller::module::State;
type ModuleStateEntry     = StateEntry<Rc<ModuleState>  , Rc<ModuleState>>;
type WeakModuleStateEntry = StateEntry<Weak<ModuleState>, Rc<ModuleState>>;

impl WeakElement for WeakModuleStateEntry {
    type Strong = ModuleStateEntry;

    fn new(view: &Self::Strong) -> Self {
        match view {
            StateEntry::Loaded(handle)     => StateEntry::Loaded(Rc::downgrade(handle)),
            StateEntry::Loading(publisher) => StateEntry::Loading(publisher.republish()),
        }
    }

    fn view(&self) -> Option<Self::Strong> {
        match self {
            StateEntry::Loaded(handle)     => handle.upgrade().map(|h| StateEntry::Loaded(h)),
            StateEntry::Loading(publisher) => Some(StateEntry::Loading(publisher.republish()))
        }
    }
}

impl<Handle,Strong> Default for StateEntry<Handle,Strong> {
    fn default() -> Self {
        Self::Loading(default())
    }
}

impl CloneRef<Handle,Strong> for StateEntry<Handle,Strong> {}

#[derive(Debug,Default)]
struct ModuleRegistry {
    cache : RefCell<WeakValueHashMap<ModuleLocation,WeakModuleStateEntry>>
}

impl ModuleRegistry {

    pub fn get(&self, loc:&ModuleLocation) -> Option<ModuleStateEntry> {
        self.cache.borrow_mut().get(loc).map(|rc| rc.clone_ref())
    }

    /// Returns a module controller which have module opened from file.
    pub fn get_or_mark_as_loading(&self, loc:ModuleLocation) -> ModuleStateEntry {
        match self.cache.borrow_mut().entry(loc) {
            Occupied(entry) => entry.get().clone(),
            Vacant(entry) => {
                let loading_entry = default();
                entry.insert(loading_entry.clone_ref());
                loading_entry
            }
        }
    }
}


// ==========================
// === Project Controller ===
// ==========================

type ModuleLocation = controller::module::Location;


/// Project controller's state.
#[derive(Debug)]
pub struct Handle {
    /// File Manager Client.
    pub file_manager: fmc::Handle,
    /// Cache of module controllers.
    pub module_cache: Rc<ModuleRegistry>,
    /// Parser handle.
    pub parser: Parser,
}

impl Handle {
    /// Create a new project controller.
    ///
    /// The remote connections should be already established.
    pub fn new(file_manager_transport:impl Transport + 'static) -> Self {
        Handle {
            file_manager : fmc::Handle::new(file_manager_transport),
            module_cache : default(),
            parser       : Parser::new_or_panic(),
        }
    }

    /// Creates a new project controller. Schedules all necessary execution with
    /// the global executor.
    pub fn new_running(file_manager_transport:impl Transport + 'static) -> Self {
        let ret = Self::new(file_manager_transport);
        crate::executor::global::spawn(ret.file_manager.runner());
        ret
    }

    /// Returns a text controller for given file path.
    ///
    /// It may be a controller for both modules and plain text files.
    pub async fn text_controller(&self, path:fmc::Path)
                                 -> FallibleResult<controller::text::Handle> {
        match ModuleLocation::from_path(&path) {
            Some(location) => {
                let module = self.module_controller(location).await?;
                Ok(controller::text::Handle::new_for_module(module))
            },
            None => {
                let fm = self.file_manager.clone_ref();
                Ok(controller::text::Handle::new_for_plain_text(path, fm))
            }
        }
    }

    /// Returns a module controller which have module opened from file.
    pub async fn module_controller(&self, location:ModuleLocation)
                               -> FallibleResult<controller::module::Handle> {
        let (cached,module) = self.module_cache.get_or_default(location.clone());
        let fm              = self.file_manager.clone_ref();
        let parser          = self.parser.clone_ref();
        let controller      = controller::module::Handle::new(location,module,fm,parser);
        if !cached {
            controller.load_file().await?
        }
        Ok(controller)
    }
}



#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::transport::test_utils::TestWithMockedTransport;

    use file_manager_client::Path;
    use json_rpc::test_util::transport::mock::MockTransport;
    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;


    wasm_bindgen_test_configure!(run_in_browser);



    #[wasm_bindgen_test]
    fn obtain_module_controller() {
        let transport = MockTransport::new();
        let mut test  = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::project::Handle::new_running(transport);
            let location     = controller::module::Location("TestLocation".to_string());
            let another_loc  = controller::module::Location("TestLocation2".to_string());

            let module_ctrl         = project_ctrl.module_controller(location.clone()).await.unwrap();
            let same_module_ctrl    = project_ctrl.module_controller(location.clone()).await.unwrap();
            let another_module_ctrl = project_ctrl.module_controller(another_loc.clone()).await.unwrap();

            assert_eq!(location   , module_ctrl        .location);
            assert_eq!(another_loc, another_module_ctrl.location);
            assert!(Rc::ptr_eq(&module_ctrl.module, &same_module_ctrl.module));
        });

        test.when_stalled_send_response("2 + 2");
        test.when_stalled_send_response("3+3");
    }

    #[wasm_bindgen_test]
    fn obtain_plain_text_controller() {
        let transport       = MockTransport::new();
        TestWithLocalPoolExecutor::set_up().run_test(async move {
            let project_ctrl        = controller::project::Handle::new_running(transport);
            let path                = Path("TestPath".to_string());
            let another_path        = Path("TestPath2".to_string());

            let text_ctrl        = project_ctrl.text_controller(path.clone()).await.unwrap();
            let same_text_ctrl   = project_ctrl.text_controller(path.clone()).await.unwrap();
            let another_txt_ctrl = project_ctrl.text_controller(another_path.clone()).await.unwrap();

            assert!(project_ctrl.file_manager.identity_equals(&text_ctrl       .file_manager()));
            assert!(project_ctrl.file_manager.identity_equals(&another_txt_ctrl.file_manager()));
            assert_eq!(path        , text_ctrl       .file_path()  );
            assert_eq!(another_path, another_txt_ctrl.file_path()  );
        });
    }

    #[wasm_bindgen_test]
    fn obtain_text_controller_for_module() {
        let transport       = MockTransport::new();
        let mut test        = TestWithMockedTransport::set_up(&transport);
        test.run_test(async move {
            let project_ctrl = controller::project::Handle::new_running(transport);
            let path         = controller::module::Location("test".to_string()).to_path();
            let text_ctrl    = project_ctrl.text_controller(path.clone()).await.unwrap();
            let content      = text_ctrl.read_content().await.unwrap();
            assert_eq!("2 + 2", content.as_str());
        });
        test.when_stalled_send_response("2 + 2");
    }
}
