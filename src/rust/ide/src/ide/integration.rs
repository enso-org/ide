pub mod project;

use crate::prelude::*;

use crate::controller::FilePath;

use enso_protocol::language_server::MethodPointer;
use parser::Parser;
use ide_view::graph_editor::SharedHashMap;
use crate::controller::ide::StatusNotification;


// =================
// === Constants ===
// =================



// =======================
// === IDE Integration ===
// =======================

#[derive(Clone,CloneRef,Debug)]
pub struct Integration {
    pub logger              : Logger,
    pub controller          : controller::ide::Handle,
    pub view                : ide_view::project::View,
    pub project_integration : Rc<RefCell<Option<project::Integration>>>,
}

impl Integration {
    pub fn new(controller:controller::ide::Handle, view:ide_view::project::View) -> Self {
        let logger              = Logger::new("ide::Integration");
        let project_integration = default();
        Self {logger,controller,view,project_integration} .init()
    }

    pub fn init(self) -> Self {
        self.initialize_status_bar_integration();
        self.setup_and_display_new_project();
        self
    }

    fn initialize_status_bar_integration(&self) {
        use controller::ide::ProcessHandle    as ControllerHandle;
        use ide_view::status_bar::process::Id as ViewHandle;

        let logger      = self.logger.clone_ref();
        let process_map = SharedHashMap::<controller::ide::ProcessHandle,ide_view::status_bar::process::Id>::new();
        let status_bar  = self.view.status_bar().clone_ref();
        let status_notif_sub = self.controller.status_notifications().subscribe();
        let status_notif_updates = status_notif_sub.for_each(move |notification| {
            match notification {
                StatusNotification::Event {label} => {
                    status_bar.add_event(ide_view::status_bar::event::Label::new(label));
                },
                StatusNotification::ProcessStarted {label,handle} => {
                    status_bar.add_process(ide_view::status_bar::process::Label::new(label));
                    let view_handle = status_bar.last_process.value();
                    process_map.insert(handle,view_handle);
                },
                StatusNotification::ProcessFinished {handle} => {
                    if let Some(view_handle) = process_map.remove(&handle) {
                        status_bar.finish_process(view_handle);
                    } else {
                        warning!(logger, "Controllers finished process not displayed in view");
                    }
                }
            }
            futures::future::ready(())
        });

        executor::global::spawn(status_notif_updates)
    }

    fn setup_and_display_new_project(&self) {
        // Remove the old integration first. We want to be sure the old and new integrations will
        // not race for the view.
        *self.project_integration.borrow_mut() = None;

        let project_model        = self.controller.current_project();
        let status_notifications = self.controller.status_notifications().clone_ref();
        let project              = controller::Project::new(project_model,status_notifications.clone_ref());
        let view                 = self.view.clone_ref();
        let integration          = self.project_integration.clone_ref();

        executor::global::spawn(async move {
            match project.initial_setup().await {
                Ok(result) => {
                    let text    = result.main_module_text;
                    let graph   = result.main_graph;
                    let project = project.model;
                    *integration.borrow_mut() = Some(project::Integration::new(view,graph,text,project));
                }
                Err(err) => {
                    let err_msg = format!("Failed to initialize project: {}", err);
                    status_notifications.publish_event(err_msg)
                }
            }
        });
    }
}
