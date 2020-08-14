//! A single entry in Select
use crate::prelude::*;

use ensogl_core::display;
use enso_frp as frp;
use ensogl_text as text;
use ensogl_core::application::Application;
use ensogl_core::gui::component;
use ensogl_core::display::shape::*;

use logger::enabled::Logger;
use ensogl_core::data::color;
use enso_prelude::fmt::Formatter;
use std::borrow::Borrow;
use ensogl_core::gui::component::ShapeViewEvents;


// =================
// === Constants ===
// =================

pub const PADDING        : f32 = 2.0;
pub const HEIGHT         : f32 = 20.0;
pub const LABEL_SIZE     : f32 = 12.0;
pub const ICON_SIZE      : f32 = 16.0;
pub const ICON_LABEL_GAP : f32 = 2.0;



// ===================
// === Entry Model ===
// ===================

pub type Id = usize;

#[derive(Derivative)]
#[derivative(Debug)]
pub struct Model {
    pub label : String,
    #[derivative(Debug="ignore")]
    pub icon  : display::object::Any,
}


// === Entry Model Provider ===

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


// === Empty Model Provider ===

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
            icon  : display::object::Instance::new(&self.logger).into_any(),
        }
    }
}



// =============
// === Entry ===
// =============

mod hover_area {
    use super::*;

    ensogl_core::define_shape_system! {
        () {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let color  = color::Rgba::new(0.0,0.0,0.0,0.0);
            Rect((&width,&height)).fill(color).into()
        }
    }
}

#[derive(Clone,CloneRef,Derivative)]
#[derivative(Debug)]
pub struct Entry {
    id             : Rc<Cell<Option<Id>>>,
    label          : text::Area,
    #[derivative(Debug="ignore")]
    icon           : Rc<CloneRefCell<display::object::Any>>,
    hover_area     : component::ShapeView<hover_area::Shape>,
    display_object : display::object::Instance,
}

impl Entry {
    fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let id             = default();
        let label          = app.new_view::<text::Area>();
        let icon           = display::object::Instance::new(Logger::new("DummyIcon"));
        let hover_area     = component::ShapeView::<hover_area::Shape>::new(&logger,app.display.scene());
        let display_object = display::object::Instance::new(logger);
        display_object.add_child(&label);
        display_object.add_child(&icon);
        display_object.add_child(&hover_area);
        icon.set_position_xy(Vector2(PADDING + ICON_SIZE/2.0, 0.0));
        label.set_position_xy(Vector2(PADDING + ICON_SIZE + ICON_LABEL_GAP, LABEL_SIZE/2.0));
        label.set_default_color(color::Rgba::new(1.0,1.0,1.0,0.7));
        label.set_default_text_size(text::Size(LABEL_SIZE));
        let icon = Rc::new(CloneRefCell::new(icon.into_any()));
        Entry{id,label,icon,hover_area,display_object}
    }

    fn invalidate_model(&self) {
        self.id.set(None)
    }

    fn set_model(&self, id:Id, model:&Model) {
        self.remove_child(&self.icon.get());
        self.add_child(&model.icon);
        model.icon.set_position_xy(Vector2(PADDING + ICON_SIZE/2.0, 0.0));
        self.id.set(Some(id));
        self.icon.set(model.icon.clone_ref());
        self.label.set_content(&model.label);
    }

    pub fn set_width(&self, width:f32) {
        let new_size = Vector2(width,HEIGHT);
        self.hover_area.shape.sprite.size.set(new_size);
        self.hover_area.set_position_y(width/2.0);
    }

    pub fn events(&self) -> &ShapeViewEvents {
        &self.hover_area.events
    }
}

impl display::Object for Entry {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}



// =================
// === EntryList ===
// =================

#[derive(Clone,CloneRef,Debug)]
pub struct List {
    logger         : Logger,
    app            : Application,
    display_object : display::object::Instance,
    entries        : Rc<RefCell<Vec<Entry>>>,
    entries_range  : Rc<CloneCell<Range<Id>>>,
    provider       : Rc<CloneRefCell<AnyModelProvider>>,
}

impl List {
    pub fn new(parent:impl AnyLogger, app:&Application) -> Self {
        let app           = app.clone_ref();
        let logger        = Logger::sub(parent,"EntryContainer");
        let entries       = default();
        let entries_range = Rc::new(CloneCell::new(default()..default()));
        let display_object = display::object::Instance::new(&logger);
        let provider = default();
        List {logger,app,display_object,entries,entries_range,provider}
    }

    pub fn position_y_of_entry(id:Id) -> f32 { id as f32 * -HEIGHT }

    pub fn position_y_range_of_entry(id:Id) -> Range<f32> {
        let position = Self::position_y_of_entry(id);
        (position - HEIGHT / 2.0)..(position + HEIGHT / 2.0)
    }

    pub fn update_entries(&self, mut range:Range<Id>) {
        if range != self.entries_range.get() {
            debug!(self.logger, "Update entries for {range:?}");
            let new_entry   = || {
                let entry = Entry::new(&self.logger,&self.app);
                self.add_child(&entry);
                entry
            };
            let provider        = self.provider.get();
            range.end           = range.end.min(provider.entry_count());
            let current_entries:HashSet<Id> = self.entries.deref().borrow().iter().take(range.len()).filter_map(|entry| entry.id.get()).collect();
            let missing         = range.clone().filter(|id| !current_entries.contains(id));
            let models          = missing.map(|id| (id,provider.get(id)));
            let mut entries     = self.entries.borrow_mut();
            entries.resize_with(range.len(),new_entry);
            let outdated = entries.iter_mut().filter(|e| e.id.get().map_or(true, |i| !range.contains(&i)));
            for (entry,(id,model)) in outdated.zip(models) {
                debug!(self.logger, "Setting new model {model:?} for entry {id}; old entry: {entry.id.get():?}");
                entry.set_model(id,&model);
                entry.set_position_xy(Vector2(0.0, Self::position_y_of_entry(id)));
            }
            self.entries_range.set(range);
        }
    }

    pub fn update_entries_new_provider
    (&self, provider:impl Into<AnyModelProvider> + 'static, range:Range<Id>) {
        self.provider.set(provider.into());
        self.update_entries(range);
    }
}

impl display::Object for List {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}