
use crate::prelude::*;
use crate::list_view;
use crate::file_browser::{ModelWithFrp, icons};
use crate::file_browser::model::*;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::display;
use ensogl_core::display::shape::*;
use std::path::PathBuf;
use crate::list_view::ListView;
use ensogl_core::display::object::Instance;
use crate::list_view::entry::AnyEntry;
use ensogl_text as text;
use ensogl_core::data::color;
use crate::file_browser::icons::DynamicIcon;


pub const WIDTH: f32 = 170.0;


#[derive(Debug)]
struct ListEntry {
    display_object: display::object::Instance,
    label: text::Area,
    icon: Box<dyn DynamicIcon>,
    arrow: Option<super::icons::Arrow>,
}

impl ListEntry {
    fn new(app:&Application, entry:Entry) -> Self {
        let logger = Logger::new("ListEntry");
        let display_object = display::object::Instance::new(&logger);
        let label = text::Area::new(app);
        display_object.add_child(&label);
        label.set_position_xy(Vector2(10.0 + 16.0 + 5.0,6.0));
        label.set_default_color(color::Rgba(0.341,0.341,0.341,1.0));
        label.set_content(entry.name);
        label.add_to_scene_layer(&app.display.scene().layers.above_nodes_text);

        let icon: Box<dyn DynamicIcon>;
        let arrow: Option<super::icons::Arrow>;
        match entry.type_ {
            EntryType::File => {
                icon = Box::new(icons::File::new());
                icon.set_color(color::Rgba(0.475,0.678,0.216,1.0));
                arrow = None;
            }
            EntryType::Folder {type_,..} => {
                match type_ {
                    FolderType::Standard => {
                        icon = Box::new(icons::Folder::new());
                    }
                    FolderType::Root => {
                        icon = Box::new(icons::Root::new());
                    }
                    FolderType::Home => {
                        icon = Box::new(icons::Home::new());
                    }
                    _ => {
                        icon = Box::new(icons::Root::new());
                    }
                };
                icon.set_color(color::Rgba(0.5,0.5,0.5,1.0));
                let arrow_icon = super::icons::Arrow::new();
                display_object.add_child(&arrow_icon);
                app.display.scene().layers.above_nodes_text.add_exclusive(&arrow_icon);
                arrow_icon.set_position_x(WIDTH - 10.0 - 8.0);
                arrow_icon.set_color(color::Rgba(0.5,0.5,0.5,1.0));
                arrow = Some(arrow_icon);
            }
        }
        display_object.add_child(&*icon);
        app.display.scene().layers.above_nodes_text.add_exclusive(&*icon);
        icon.deref().set_position_x(10.0 + 8.0);

        Self { display_object,label,icon,arrow }
    }
}

impl display::Object for ListEntry {
    fn display_object(&self) -> &Instance {
        &self.display_object
    }
}

impl list_view::entry::Entry for ListEntry {
    fn set_selected(&self, _selected: bool) {}

    fn set_width(&self, _width: f32) {}
}

#[derive(Debug)]
struct ListEntryProvider(Rc<Vec<Entry>>);

impl list_view::entry::EntryProvider for ListEntryProvider {
    fn entry_count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, app: &Application, id: usize) -> Option<AnyEntry> {
        Some(ListEntry::new(app, self.0.get(id)?.clone()).into())
    }
}


#[derive(Clone,Debug)]
pub struct Model {
    app : Application,
    entries : RefCell<Option<Rc<Vec<Entry>>>>,
    pub list_view : ListView,
}

impl Model {
}


ensogl_core::define_endpoints! {
    Input {
        set_entries    (Rc<Vec<Entry>>)
    }
    Output {
        entry_selected (PathBuf),
        entry_chosen   (PathBuf),
        right          (f32),
    }
}

#[derive(Debug,Clone,CloneRef)]
pub struct Column {
    pub model : Rc<Model>,
    frp   : Frp,
}

impl Column {
    pub fn new(browser:Rc<ModelWithFrp>, index:usize) -> Column {
        let weak_browser = Rc::downgrade(&browser);
        let app = &browser.model.app;
        let list_view = app.new_view::<ListView>();
        list_view.frp.resize(Vector2(WIDTH,super::CONTENT_HEIGHT));
        let x = WIDTH/2.0+WIDTH * index as f32;
        let y = -super::CONTENT_HEIGHT / 2.0;
        list_view.set_position_xy(Vector2(x,y));
        // list_view.set_entries(list_view::entry::AnyEntryProvider::from(folder));
        // list_view.set_entries(entries.root_entries());
        // list_view.focus();
        list_view.set_selection_method(list_view::SelectionMethod::Click);
        let app = app.clone();
        let frp = Frp::new();
        let network = &frp.network;

        let entries = RefCell::new(None);
        let model = Rc::new(Model {app, entries, list_view:list_view.clone()});
        frp::extend!{ network
            eval frp.set_entries([model,list_view](entries) {
                model.entries.set(entries.clone());
                list_view.set_entries(list_view::entry::AnyEntryProvider::from(ListEntryProvider(entries.clone())));
            });

            eval model.list_view.selected_entry([weak_browser](id)
                if id.is_some() {
                    weak_browser.upgrade().unwrap().model.focus_column(index);
                });
            selection_changed <- model.list_view.selected_entry.on_change();
            eval selection_changed([weak_browser,model](id) {
                weak_browser.upgrade().unwrap().close_columns_from(index + 1);
                if let &Some(id) = id {
                    let selected_entry = model.entries.borrow().as_ref().unwrap()[id].clone();
                    if let EntryType::Folder{content,..} = selected_entry.type_ {
                        weak_browser.upgrade().unwrap().push_column(content);
                    }
                }
            });
        }

        Column {model,frp}
    }
}

impl Deref for Column {
    type Target = Frp;

    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl display::Object for Column {
    fn display_object(&self) -> &Instance {
        self.model.list_view.display_object()
    }
}
