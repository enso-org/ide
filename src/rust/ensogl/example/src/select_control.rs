use crate::prelude::*;

use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::system::web;
use ensogl_core::application::Application;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::theme;
use ensogl_core::data::color;
use ensogl_text_msdf_sys::run_once_initialized;
use ensogl_select as select;
use logger::enabled::Logger;
use wasm_bindgen::prelude::*;


#[wasm_bindgen]
#[allow(dead_code)]
pub fn run_example_select_control() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
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

mod icon {
    use super::*;
    ensogl_core::define_shape_system! {
        (style:Style,id:f32) {
            let width  = select::entry::ICON_SIZE.px();
            let height = select::entry::ICON_SIZE.px();
            let color  : Var<color::Rgba> = "rgba(input_id/16.0,0.0,0.0,1.0)".into();
            let radius = 4.0.px();
            Rect((&width,&height)).fill(color).into()
        }
    }
}

#[derive(Clone,Debug)]
struct MockEntry {
    id   : usize,
    icon : gui::component::ShapeView<icon::Shape>,
}

#[derive(Clone,Debug,Default)]
struct MockEntries {
    entries:Vec<MockEntry>,
}

impl MockEntries {
    fn new(app:&Application, entries_count:usize) -> Self {
        let logger  = Logger::new("MockEntries");
        let scene   = app.display.scene();
        let entries = (0..entries_count).map(|id| {
            let icon = gui::component::ShapeView::<icon::Shape>::new(&logger,scene);
            icon.shape.sprite.size.set(Vector2(select::entry::ICON_SIZE,select::entry::ICON_SIZE));
            icon.shape.id.set(id as f32);
            MockEntry {id,icon}
        });
        Self {
            entries : entries.collect()
        }
    }
}

impl select::entry::ModelProvider for MockEntries {
    fn entry_count(&self) -> usize { self.entries.len() }

    fn get(&self, id:usize) -> select::entry::Model {
        let entry = &self.entries[id];
        select::entry::Model {
            label : iformat!("Entry {entry.id}"),
            icon  : entry.icon.display_object().clone_ref()
        }
    }
}



// ========================
// === Init Application ===
// ========================

fn init(app:&Application) {

    let mut dark = theme::Theme::new();
    dark.insert("application.background.color", color::Lcha::new(0.13,0.013,0.18,1.0));
    // dark.insert("graph_editor.node.background.color", color::Lcha::new(0.2,0.013,0.18,1.0));
    // dark.insert("graph_editor.node.selection.color", color::Lcha::new(0.72,0.5,0.22,1.0));
    // dark.insert("graph_editor.node.selection.size", 7.0);
    dark.insert("animation.duration", 0.5);
    // dark.insert("graph.node.shadow.color", 5.0);
    // dark.insert("graph.node.shadow.size", 5.0);
    dark.insert("mouse.pointer.color", color::Rgba::new(0.3,0.3,0.3,1.0));

    app.themes.register("dark",dark);
    app.themes.set_enabled(&["dark"]);

    // let _bg = app.display.scene().style_sheet.var("application.background.color");

    // let world     = &app.display;
    // let scene     = world.scene();
    // let camera    = scene.camera();
    // let navigator = Navigator::new(&scene,&camera);

    // app.views.register::<GraphEditor>();
    // let graph_editor = app.new_view::<GraphEditor>();
    // let mut entry_container = select::entry::EntryList::new(logger, app);
    // entry_container.update_entries_new_provider(MockEntries::new(&app,12),0..7);
    // app.display.add_child(&entry_container);

    // std::mem::forget(entry_container);

    let provider:select::entry::AnyModelProvider = MockEntries::new(app,13).into();
    let select                                   = app.new_view::<select::component::Select>();
    select.frp.resize(Vector2(100.0,150.0));
    select.frp.set_entries(provider);
    app.display.add_child(&select);
    std::mem::forget(select);
}


