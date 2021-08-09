//! A debug scene which shows the Select Component. The chosen entries are logged in console.

use crate::prelude::*;

use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::display::object::ObjectOps;
use ensogl_text_msdf_sys::run_once_initialized;
use ensogl_gui_components::list_view2 as list_view;
use logger::TraceLogger as Logger;
use wasm_bindgen::prelude::*;
use ensogl_core::display::Scene;
use ensogl_text::buffer::data::unit::Bytes;
use ensogl_theme as theme;



// ===================
// === Entry Point ===
// ===================

/// An entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_list_view2() {
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
//
// #[derive(Clone,Debug)]
// struct MockEntries {
//     logger        : Logger,
//     scene         : Scene,
//     entries_count : usize,
// }
//
// impl MockEntries {
//     fn new(app:&Application, entries_count:usize) -> Self {
//         let logger = Logger::new("MockEntries");
//         let scene  = app.display.scene().clone_ref();
//         Self {logger,scene,entries_count}
//     }
// }
//
// impl list_view::entry::Provider<list_view::entry::GlyphHighlightedLabel> for MockEntries {
//     fn len(&self) -> usize { self.entries_count }
//
//     fn get(&self, id:usize) -> Option<list_view::entry::GlyphHighlightedLabelModel> {
//         if id >= self.entries_count {
//             None
//         } else {
//             let label = iformat!("Entry {id}");
//             let highlighted = if id == 10 { vec![(Bytes(1)..Bytes(3)).into()] } else { vec![] };
//             Some(list_view::entry::GlyphHighlightedLabelModel {label,highlighted})
//         }
//     }
// }



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {
    theme::builtin::dark::register(&app);
    theme::builtin::light::register(&app);
    theme::builtin::light::enable(&app);

    let list_view = app.new_view::<list_view::ListView<list_view::entry::GlyphHighlightedLabel>>();
    // let provider  = list_view::entry::provider::Any::new(MockEntries::new(app,1000));
    // list_view.frp.resize(Vector2(100.0,140.0));
    // list_view.frp.set_entries(provider);
    app.display.add_child(&list_view);

    // FIXME[WD]: This should not be needed after text gets proper depth-handling.
    app.display.scene().layers.below_main.add_exclusive(&list_view);

    let logger : Logger = Logger::new("SelectDebugScene");
    let network = enso_frp::Network::new("test");
    enso_frp::extend! {network
        eval list_view.chosen_entry([logger](entry) {
            info!(logger, "Chosen entry {entry:?}")
        });
    }

    let world = &app.display;
    let mut frame = 0;
    world.on_frame(move |_time| {
        let _keep_alive = &list_view;
        let _keep_alive = &network;

        if frame == 50 {
            DEBUG!("Resize.");
            list_view.set_size(list_view::Size::default());
        }
        if frame == 150 {
            DEBUG!("--- Setting entry #1 ---");
            let label = iformat!("Entry 1");
            let highlighted = vec![];
            let entry = list_view::entry::GlyphHighlightedLabelModel {label,highlighted};
            list_view.set_entry((1,Rc::new(Some(entry))));
        }
        frame += 1;
    }).forget();
}
