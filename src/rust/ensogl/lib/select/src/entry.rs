//! A single entry in Select
use crate::prelude::*;

use ensogl::display;
use ensogl_text as text;
use ensogl::application::Application;

use logger::enabled::Logger;
use ensogl::data::color;


pub const ENTRY_HEIGHT:f32 = 16.0;
pub const ICON_WIDTH:f32   = 16.0;

pub type Id = usize;

#[derive(Clone,Debug)]
pub struct Model {
    pub label : String,
    pub icon  : display::object::Instance,
}

pub trait ModelProvider : Debug {
    fn entry_count(&self) -> usize;

    fn get(&self, id:Id) -> Model;
}

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct AnyModelProvider(Rc<dyn ModelProvider>);

impl<T:ModelProvider + 'static> From<T> for AnyModelProvider {
    fn from(provider: T) -> Self { Self(Rc::new(provider)) }
}

impl Default for AnyModelProvider {
    fn default() -> Self {
        let logger = Logger::new("EmptyModelProvider");
        EmptyProvider{logger}.into()
    }
}

#[derive(Clone,CloneRef,Debug)]
struct EmptyProvider {
    logger : Logger,
}

impl ModelProvider for EmptyProvider {
    fn entry_count(&self) -> usize { 0 }
    fn get(&self, id:usize) -> Model {
        error!(self.logger, "Getting {id} from empty provider!");
        Model {
            label : "Invalid".to_string(),
            icon  : display::object::Instance::new(&self.logger)
        }
    }
}


#[derive(Clone,CloneRef,Debug)]
struct Entry {
    label          : text::Area,
    icon           : Rc<CloneRefCell<display::object::Instance>>,
    display_object : display::object::Instance,
}

impl Entry {
    fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let label          = app.new_view::<text::Area>();
        let icon           = display::object::Instance::new(Logger::new("DummyIcon"));
        let display_object = display::object::Instance::new(logger);
        display_object.add_child(&label);
        display_object.add_child(&icon);
        icon.set_position_xy(Vector2(ICON_WIDTH/2.0, 0.0));
        label.set_position_xy(Vector2(ICON_WIDTH, 0.0));
        label.set_default_color(color::Rgba::new(1.0,1.0,1.0,0.7));
        label.set_default_text_size(text::Size(12.0));
        let icon = Rc::new(CloneRefCell::new(icon));
        Entry{label,icon,display_object}
    }

    fn set_model(&self,model:&Model) {
        self.remove_child(&self.icon.get());
        self.add_child(&model.icon);
        model.icon.set_position_xy(Vector2(0.0,0.0));
        self.icon.set(model.icon.clone_ref());
        self.label.set_content(&model.label);
    }
}

impl display::Object for Entry {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}

#[derive(Clone,CloneRef,Debug)]
pub struct EntryList {
    logger                  : Logger,
    app                     : Application,
    display_object          : display::object::Instance,
    entries                 : Rc<RefCell<Vec<Entry>>>,
    provider                : Rc<CloneRefCell<AnyModelProvider>>,
}

impl EntryList {
    pub fn new(parent:impl AnyLogger, app:&Application) -> Self {
        let app    = app.clone_ref();
        let logger = Logger::sub(parent,"EntryContainer");
        let entries = default();
        let display_object = display::object::Instance::new(&logger);
        let provider = default();
        EntryList {logger,app,display_object,entries,provider}
    }

    pub fn update_entries(&self, range:Range<Id>) {
        debug!(self.logger, "Update entries for {range:?}");
        let new_entry   = || {
            let entry = Entry::new(&self.logger,&self.app);
            self.add_child(&entry);
            entry
        };
        let provider    = self.provider.get();
        let models      = range.clone().map(|id| provider.get(id)).collect_vec();
        let mut entries = self.entries.borrow_mut();
        entries.resize_with(range.len(),new_entry);
        for ((id,entry),model) in range.zip(entries.iter_mut()).zip(models.iter()) {
            debug!(self.logger, "Setting new model {model:?} for entry {id}");
            entry.set_model(model);
            entry.set_position_xy(Vector2(0.0, id as f32 * -ENTRY_HEIGHT));
        }
    }

    pub fn update_entries_new_provider
    (&self, provider:impl Into<AnyModelProvider> + 'static, range:Range<Id>) {
        self.provider.set(provider.into());
        self.update_entries(range);
    }
}

impl display::Object for EntryList {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}