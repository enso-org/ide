//! A debug scene which shows the file browser. The selected and chosen entries are logged on the
//! console.

use crate::prelude::*;

use ensogl_gui_components::file_browser::*;
use ensogl_gui_components::file_browser::model::*;
use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::display::object::ObjectOps;
use ensogl_text_msdf_sys::run_once_initialized;
use wasm_bindgen::prelude::*;
use ensogl_theme as theme;
use enso_frp as frp;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_file_browser() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}



// ====================
// === Mock Content ===
// ====================

#[derive(Debug,Clone)]
struct MockFolderContent {
    entries: Rc<Vec<Entry>>
}

impl MockFolderContent {
    fn new(entries: Vec<Entry>) -> Self {
        Self { entries:Rc::new(entries) }
    }
}

impl FolderContent for MockFolderContent {
    fn request_entries
    (&self, entries_loaded: frp::Any<Rc<Vec<Entry>>>, _error_occurred: frp::Any<ImString>) {
        entries_loaded.emit(self.entries.clone());
    }
}


#[derive(Debug)]
struct GeneratedFolderContent;

impl FolderContent for GeneratedFolderContent {
    fn request_entries
    (&self, entries_loaded: frp::Any<Rc<Vec<Entry>>>, _error_occurred: frp::Any<ImString>) {
        entries_loaded.emit(
            Rc::new((0..20).map(|i|
                Entry {
                    name: format!("Folder {}", i),
                    path: format!("Folder {}", i).into(),
                    type_: EntryType::Folder {
                        type_: FolderType::Standard,
                        content: GeneratedFolderContent.into()
                    }
                }
            ).collect_vec())
        );
    }
}


#[derive(Debug)]
struct ErrorContent;

impl FolderContent for ErrorContent {
    fn request_entries
    (&self, _entries_loaded: frp::Any<Rc<Vec<Entry>>>, error_occurred: frp::Any<ImString>) {
        error_occurred.emit(ImString::new("Could not open folder"));
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {
    theme::builtin::dark::register(&app);
    theme::builtin::light::register(&app);
    theme::builtin::light::enable(&app);

    let file_browser = app.new_view::<FileBrowser>();
    let fs = MockFolderContent::new(vec![
        Entry {
            name: "Project's Data".to_string(),
            path: "Project's Data".into(),
            type_: EntryType::Folder {
                type_: FolderType::Project,
                content: MockFolderContent::new(vec![]).into()
            },
        },
        Entry {
            name: "Home".to_string(),
            path: "Home".into(),
            type_: EntryType::Folder {
                type_: FolderType::Home,
                content: MockFolderContent::new(vec![
                    Entry {
                        name: "Applications".to_string(),
                        path: "Applications".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Desktop".to_string(),
                        path: "Desktop".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Documents".to_string(),
                        path: "Documents".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Downloads".to_string(),
                        path: "Downloads".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Enso".to_string(),
                        path: "Enso".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: GeneratedFolderContent.into(),
                        },
                    },
                    Entry {
                        name: "Movies".to_string(),
                        path: "Movies".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Music".to_string(),
                        path: "Music".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Pictures".to_string(),
                        path: "Pictures".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Public".to_string(),
                        path: "Public".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: Default::default(),
                        },
                    },
                    Entry {
                        name: "Error".to_string(),
                        path: "Error".into(),
                        type_: EntryType::Folder {
                            type_: FolderType::Standard,
                            content: ErrorContent.into(),
                        },
                    },
                    Entry {
                        name: "File 1".to_string(),
                        path: "File 1".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 2".to_string(),
                        path: "File 2".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 3".to_string(),
                        path: "File 3".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 4".to_string(),
                        path: "File 4".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 5".to_string(),
                        path: "File 5".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 6".to_string(),
                        path: "File 6".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 7".to_string(),
                        path: "File 7".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 8".to_string(),
                        path: "File 8".into(),
                        type_: EntryType::File
                    },
                    Entry {
                        name: "File 9".to_string(),
                        path: "File 9".into(),
                        type_: EntryType::File
                    },
                ]).into()
            },
        },
        Entry {
            name: "Root".to_string(),
            path: "Root".into(),
            type_: EntryType::Folder {
                type_: FolderType::Root,
                content: MockFolderContent::new(vec![]).into()
            },
        },
    ]);
    file_browser.set_content(AnyFolderContent::from(fs.clone()));
    app.display.add_child(&file_browser);

    let network = enso_frp::Network::new("test");
    enso_frp::extend! {network
        trace file_browser.entry_chosen;
        trace file_browser.entry_selected;
        trace file_browser.copy;
        trace file_browser.cut;
        trace file_browser.paste_into;
    }

    std::mem::forget(file_browser);
    std::mem::forget(network);
}
