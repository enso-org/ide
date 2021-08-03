//! A debug scene which shows the Select Component. The chosen entries are logged in console.

use crate::prelude::*;

use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::display;
use ensogl_core::display::object::ObjectOps;
use ensogl_text_msdf_sys::run_once_initialized;
use ensogl_gui_components::list_view;
use ensogl_gui_components::drop_down_menu;
use logger::TraceLogger as Logger;
use wasm_bindgen::prelude::*;
use ensogl_core::display::Scene;
use ensogl_theme as theme;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_drop_down_menu() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();
    run_once_initialized(|| {
        let app = Application::new(&web::get_html_element_by_id("root").unwrap());
        init(&app);
        mem::forget(app);
    });
}



// ====================
// === Mock Entries ===
// ====================

#[derive(Clone,Debug)]
struct MockEntries {
    logger        : Logger,
    scene         : Scene,
    entries_count : usize,
}

impl MockEntries {
    fn new(app:&Application, entries_count:usize) -> Self {
        let logger = Logger::new("MockEntries");
        let scene  = app.display.scene().clone_ref();
        Self {logger,scene,entries_count}
    }
}

impl list_view::entry::Provider<list_view::entry::Label> for MockEntries {
    fn len(&self) -> usize { self.entries_count }

    fn get(&self, id:usize) -> Option<String> {
        if id >= self.entries_count {
            None
        } else {
            Some(iformat!("Entry {id}"))
        }
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {
    theme::builtin::dark::register(&app);
    theme::builtin::light::register(&app);
    theme::builtin::light::enable(&app);

    let menu     = drop_down_menu::DropDownMenu::new(app);
    let provider = list_view::entry::provider::Any::new(MockEntries::new(app,13));
    menu.frp.set_icon_size(Vector2(20.0,20.0));
    menu.frp.set_entries(provider);
    app.display.add_child(&menu);
    // FIXME[WD]: This should not be needed after text gets proper depth-handling.
    // app.display.scene().layers.below_main.add_exclusive(&menu);
    menu.set_position_xy(Vector2(100.0,100.0));

    let logger : Logger = Logger::new("SelectDebugScene");
    let network = enso_frp::Network::new("test");
    enso_frp::extend! {network
        eval menu.chosen_entry([logger](entry) {
            info!(logger, "Chosen entry {entry:?}")
        });
    }

    // === Selection Target Redirection ===

    // TODO This was copied from GraphEditor. It should not be there, but somewhere in Scene
    // instead.
    let scene = app.display.scene();
    enso_frp::extend! { network
        mouse_down_target <- scene.mouse.frp.down_primary.map(f_!(scene.mouse.target.get()));
        mouse_up_target   <- scene.mouse.frp.up_primary.map(f_!(scene.mouse.target.get()));

        eval mouse_down_target([scene](target) {
            if let display::scene::PointerTarget::Symbol {..} = target {
                if let Some(target) = scene.shapes.get_mouse_target(*target) {
                    target.mouse_down().emit(());
                }
            }
        });

        eval mouse_up_target([scene](target) {
            if let display::scene::PointerTarget::Symbol {..} = target {
                if let Some(target) = scene.shapes.get_mouse_target(*target) {
                    target.mouse_up().emit(());
                }
            }
        });
    }

    std::mem::forget(menu);
    std::mem::forget(network);
}
