
use crate::prelude::*;
use crate::list_view;
use crate::file_browser::{ModelWithFrp, icons};
use crate::file_browser::model::*;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::{display, Animation};
use ensogl_core::display::shape::*;
use std::path::PathBuf;
use crate::list_view::ListView;
use ensogl_core::display::object::Instance;
use ensogl_text as text;
use ensogl_core::data::color;
use crate::file_browser::icons::DynamicIcon;
use crate::shadow;
use ensogl_theme as theme;



mod column_shadow {
    use super::*;

    pub const SHADOW_PX:f32 = 10.0;

    ensogl_core::define_shape_system! {
        (style:Style,opacity:f32) {
            let background = HalfPlane().rotate(90.0_f32.to_radians().radians());
            let color         = style.get_color(theme::application::file_browser::background);
            let background = background.fill(color);
            let shadow = shadow::from_shape_with_alpha((&background).into(),&opacity,style);

            (shadow + background).into()
        }
    }
}


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
        label.add_to_scene_layer(&app.display.scene().layers.panel_text);

        let icon: Box<dyn DynamicIcon>;
        let arrow: Option<super::icons::Arrow>;
        match entry.type_ {
            EntryType::File => {
                icon = Box::new(icons::File::new());
                app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::file::View>();
                app.display.scene().layers.add_shapes_order_dependency::<icons::file::View,list_view::io_rect::View>();
                icon.set_color(color::Rgba(0.475,0.678,0.216,1.0));
                arrow = None;
            }
            EntryType::Folder {type_,..} => {
                match type_ {
                    FolderType::Standard => {
                        icon = Box::new(icons::Folder::new());
                        app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::folder::View>();
                        app.display.scene().layers.add_shapes_order_dependency::<icons::folder::View,list_view::io_rect::View>();
                    }
                    FolderType::Root => {
                        icon = Box::new(icons::Root::new());
                        app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::root::View>();
                        app.display.scene().layers.add_shapes_order_dependency::<icons::root::View,list_view::io_rect::View>();
                    }
                    FolderType::Home => {
                        icon = Box::new(icons::Home::new());
                        app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::home::View>();
                        app.display.scene().layers.add_shapes_order_dependency::<icons::home::View,list_view::io_rect::View>();
                    }
                    FolderType::Project => {
                        icon = Box::new(icons::Project::new());
                        app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::project::View>();
                        app.display.scene().layers.add_shapes_order_dependency::<icons::project::View,list_view::io_rect::View>();
                    }
                    _ => {
                        icon = Box::new(icons::Root::new());
                        app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::root::View>();
                        app.display.scene().layers.add_shapes_order_dependency::<icons::root::View,list_view::io_rect::View>();
                    }
                };
                icon.set_color(color::Rgba(0.5,0.5,0.5,1.0));
                let arrow_icon = super::icons::Arrow::new();
                app.display.scene().layers.add_shapes_order_dependency::<list_view::selection::View,icons::arrow::View>();
                app.display.scene().layers.add_shapes_order_dependency::<icons::arrow::View,list_view::io_rect::View>();
                display_object.add_child(&arrow_icon);
                app.display.scene().layers.panel.add_exclusive(&arrow_icon);
                arrow_icon.set_color(color::Rgba(0.5,0.5,0.5,1.0));
                arrow = Some(arrow_icon);
            }
        }
        display_object.add_child(&*icon);
        app.display.scene().layers.panel.add_exclusive(icon.as_ref());
        icon.deref().set_position_x(10.0 + 8.0);

        Self { display_object,label,icon,arrow }
    }

    fn width(&self) -> f32 {
        self.label.width.value() + 62.0
    }
}

impl display::Object for ListEntry {
    fn display_object(&self) -> &Instance {
        &self.display_object
    }
}

impl list_view::entry::Entry for ListEntry {
    fn set_selected(&self, selected: bool) {
        let text_color = if selected {
            color::Rgba::white()
        } else {
            color::Rgba(0.341,0.341,0.341,1.0)
        };
        self.label.set_color_all(text_color);
        let stroke_width = if selected {
            1.5
        } else {
            1.0
        };
        self.icon.set_stroke_width(stroke_width);
        self.icon.set_color(text_color);
        if let Some(arrow) = &self.arrow {
            arrow.set_stroke_width(stroke_width);
            arrow.set_color(text_color);
        }
    }

    fn set_width(&self, width: f32) {
        if let Some(arrow) = &self.arrow {
            arrow.set_position_x(width - 13.0);
        }
    }
}

#[derive(Debug)]
struct ListEntryProvider(Rc<Vec<Rc<ListEntry>>>);

impl ListEntryProvider {
    fn new<'a>(app:&Application, entries:impl IntoIterator<Item=&'a Entry>) -> Self {
        ListEntryProvider(Rc::new(entries.into_iter().map(|entry| Rc::new(ListEntry::new(app,entry.clone()))).collect()))
    }

    fn width(&self) -> f32 {
        self.0.iter().map(|entry| entry.width()).max_by(|x,y| x.partial_cmp(y).unwrap()).unwrap_or(0.0)
    }
}

impl list_view::entry::EntryProvider for ListEntryProvider {
    fn entry_count(&self) -> usize {
        self.0.len()
    }

    fn get(&self, _app: &Application, id: usize) -> Option<list_view::entry::AnyEntry> {
        Some(list_view::entry::AnyEntry::from(self.0.get(id)?.clone()))
    }
}


#[derive(Clone,Debug)]
pub struct Model {
    app : Application,
    pub entries : RefCell<Option<Rc<Vec<Entry>>>>,
    pub list_view : ListView,
    state_label: text::Area,
    shadow: column_shadow::View,
}

impl Model {
}


ensogl_core::define_endpoints! {
    Input {
        set_entries    (Rc<Vec<Entry>>),
        set_error      (ImString),
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
        app.display.scene().layers.panel.add_exclusive(&list_view);
        // list_view.frp.resize(Vector2(WIDTH,super::CONTENT_HEIGHT));
        // let x = WIDTH/2.0+WIDTH * index as f32;
        // let y = -super::CONTENT_HEIGHT / 2.0;
        // list_view.set_position_xy(Vector2(x,y));
        list_view.set_selection_method(list_view::SelectionMethod::Click);
        let frp = Frp::new();
        let network = &frp.network;

        let entries = RefCell::new(None);

        let state_label = text::Area::new(app);
        list_view.add_child(&state_label);
        state_label.add_to_scene_layer(&app.display.scene().layers.panel_text);
        state_label.set_default_color(color::Rgba(0.5, 0.5, 0.5, 1.0));
        state_label.set_position_y(text::component::area::LINE_HEIGHT/2.0 + 5.0);
        state_label.set_content("Loading");
        state_label.set_position_x(-state_label.width.value()/2.0);

        let shadow = column_shadow::View::new(&Logger::new("file browser column"));
        list_view.add_child(&shadow);
        shadow.size.set(Vector2(column_shadow::SHADOW_PX*2.0,super::CONTENT_HEIGHT));
        app.display.scene().layers.add_shapes_order_dependency::<super::background::View,column_shadow::View>();
        app.display.scene().layers.add_shapes_order_dependency::<column_shadow::View,list_view::selection::View>();

        let model = Rc::new(Model {app:app.clone(), entries, list_view:list_view.clone(),
            state_label: state_label.clone(), shadow:shadow.clone()});

        let shadow_opacity = Animation::new(&network);

        frp::extend!{ network
            updating_entries <- source::<bool>();
            eval frp.set_entries([weak_browser,shadow,model,list_view,updating_entries,state_label](entries) {
                updating_entries.emit(true);
                if entries.is_empty() {
                    state_label.set_content("This folder is empty");
                    state_label.set_position_x(-state_label.width.value()/2.0);
                } else {
                    state_label.set_content("");
                }
                model.entries.set(entries.clone());
                let list_entries = ListEntryProvider::new(&model.app,entries.as_ref());
                let width = if entries.is_empty() {
                    state_label.width.value() + 40.0
                } else {
                    list_entries.width()+2.0*list_view::PADDING_HORIZONTAL
                };
                list_view.resize(Vector2(width,super::CONTENT_HEIGHT));
                let left = if index > 0 {
                    if let Some(predecessor) = weak_browser.upgrade().unwrap().model.columns.borrow().get(index-1).cloned() {
                        predecessor.position().x + predecessor.model.list_view.size.value().x / 2.0
                    } else {
                        0.0
                    }
                } else {
                    0.0
                };
                let x = left + width / 2.0;
                let y = -super::CONTENT_HEIGHT / 2.0;
                list_view.set_position_xy(Vector2(x,y));
                shadow.set_position_x(width/2.0);
                list_view.set_entries(list_view::entry::AnyEntryProvider::from(list_entries));
                list_view.move_selection_to_first();
                updating_entries.emit(false);
                list_view.skip_animations();
            });

            open_folder <- all(&model.list_view.selected_entry,&list_view.focus)._0().gate_not(&updating_entries);

            focus_this_column <- frp.entry_selected.gate_not(&updating_entries).constant(());
            eval_ focus_this_column(weak_browser.upgrade().unwrap().model.focus_column(index));
            open_folder_changed <- open_folder.on_change();
            eval open_folder_changed([weak_browser,model](id) {
                weak_browser.upgrade().unwrap().close_columns_from(index + 1);
                if let &Some(id) = id {
                    let selected_entry = model.entries.borrow().as_ref().unwrap()[id].clone();
                    if let EntryType::Folder{content,..} = selected_entry.type_ {
                        weak_browser.upgrade().unwrap().push_column(content);
                    }
                }
            });
            shadow_opacity.target <+ open_folder.map(|id| if id.is_some() {1.0} else {0.0});
            eval shadow_opacity.value((&opacity) shadow.opacity.set(opacity));
            browser.frp.source.entry_chosen <+ model.list_view.chosen_entry.filter_map(
                f!([model](&id) {
                    let selected_entry = model.entries.borrow().as_ref()?[id?].clone();
                    Some(selected_entry.path)
                }));
            frp.source.entry_selected <+ model.list_view.selected_entry.filter_map(
                f!([model](&id) {
                    let selected_entry = model.entries.borrow().as_ref()?[id?].clone();
                    Some(selected_entry.path)
                }));
            browser.frp.source.entry_selected <+ frp.entry_selected;
            eval frp.set_error((error) {
                state_label.set_content(error.to_string());
                state_label.set_position_x(10.0);
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
