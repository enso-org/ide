//! A single entry in Select
use crate::prelude::*;

use enso_frp as frp;
use ensogl_core::application::Application;
use ensogl_core::data::color;
use ensogl_core::display;
use ensogl_core::display::shape::*;
use ensogl_core::gui::component;
use ensogl_core::gui::component::ShapeViewEvents;
use ensogl_text as text;
use ensogl_text::buffer::data::unit::Bytes;
use ensogl_text::buffer::data::range::Range as TextRange;
use logger::enabled::Logger;
use std::borrow::Borrow;



// =================
// === Constants ===
// =================

/// Padding inside entry in pixels.
pub const PADDING:f32 = 2.0;
/// The overall entry's height (including padding).
pub const HEIGHT:f32 = 20.0;
/// The text size of entry's labe.
pub const LABEL_SIZE:f32 = 12.0;
/// The size in pixels of icons inside entries.
pub const ICON_SIZE:f32 = 16.0;
/// The gap between icon and label.
pub const ICON_LABEL_GAP:f32 = 2.0;



// ===================
// === Entry Model ===
// ===================

/// Entry id. 0 is the first entry in component.
pub type Id = usize;

/// A model on which the view bases.
#[allow(missing_docs)]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct Model {
    pub label       : String,
    pub highlighted : Vec<TextRange<Bytes>>,
    #[derivative(Debug="ignore")]
    pub icon        : display::object::Any,
}


// === Entry Model Provider ===

/// The Entry Model Provider for select component.
///
/// The select does not display all entries at once, instead it lazily ask for models of entries
/// when they're about to be displayed. So setting the select content is essentially providing
/// implementor of this trait.
pub trait ModelProvider : Debug {
    /// Number of all entries.
    fn entry_count(&self) -> usize;

    /// Get the model of entry with given id.
    fn get(&self, id:Id) -> Model;
}

/// A wrapper for shared instance of some ModelProvider.
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

/// An Entry Model Provider giving no entries.
///
/// This is the default provider for new select components.
#[derive(Clone,CloneRef,Debug)]
pub struct EmptyProvider {
    logger : Logger,
}

impl ModelProvider for EmptyProvider {
    fn entry_count(&self) -> usize { 0 }
    fn get(&self, id:usize) -> Model {
        error!(self.logger, "Getting {id} from empty provider!");
        Model {
            label       : "Invalid".to_string(),
            highlighted : default(),
            icon        : display::object::Instance::new(&self.logger).into_any(),
        }
    }
}



// =============
// === Entry ===
// =============

/// A displayed entry in select component.
///
/// The Display Object position of this component is docked to the middle of left entry's boundary.
/// It differs from usual behaviour of EnsoGl components, but makes the entries alignment much
/// simpler.
#[derive(Clone,CloneRef,Derivative)]
#[derivative(Debug)]
pub struct Entry {
    id             : Rc<Cell<Option<Id>>>,
    label          : text::Area,
    #[derivative(Debug="ignore")]
    icon           : Rc<CloneRefCell<display::object::Any>>,
    display_object : display::object::Instance,
}

impl Entry {
    /// Create new entry view.
    pub fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let id             = default();
        let label          = app.new_view::<text::Area>();
        let icon           = display::object::Instance::new(Logger::new("DummyIcon"));
        let display_object = display::object::Instance::new(logger);
        display_object.add_child(&label);
        display_object.add_child(&icon);
        label.set_default_color(color::Rgba::new(1.0,1.0,1.0,0.7));
        label.set_default_text_size(text::Size(LABEL_SIZE));
        let icon = Rc::new(CloneRefCell::new(icon.into_any()));
        Entry{id,label,icon,display_object}
    }

    /// Set the new model for this view.
    ///
    /// This function updates icon and label.
    pub fn set_model(&self, id:Id, model:&Model) {
        self.remove_child(&self.icon.get());
        self.add_child(&model.icon);
        model.icon.set_position_xy(Vector2(PADDING + ICON_SIZE/2.0, 0.0));
        self.id.set(Some(id));
        self.icon.set(model.icon.clone_ref());
        self.label.set_content(&model.label);
        for highlighted in &model.highlighted {
            self.label.set_color_bytes(highlighted,color::Rgba::new(0.0,0.0,1.0,1.0));
        }
    }
}

impl display::Object for Entry {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}



// =================
// === EntryList ===
// =================

/// A view containing an entry list, arranged in column.
///
/// Not all entries are displayed at once, only those visible.
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
    /// Entry List View constructor.
    pub fn new(parent:impl AnyLogger, app:&Application) -> Self {
        let app           = app.clone_ref();
        let logger        = Logger::sub(parent,"EntryContainer");
        let entries       = default();
        let entries_range = Rc::new(CloneCell::new(default()..default()));
        let display_object = display::object::Instance::new(&logger);
        let provider = default();
        List {logger,app,display_object,entries,entries_range,provider}
    }

    /// Y position of entry with given id, relative to Entry List position.
    pub fn position_y_of_entry(id:Id) -> f32 { id as f32 * -HEIGHT }

    /// Y position range of entry with given id, relative to Entry List position.
    pub fn position_y_range_of_entry(id:Id) -> Range<f32> {
        let position = Self::position_y_of_entry(id);
        (position - HEIGHT / 2.0)..(position + HEIGHT / 2.0)
    }

    /// Update displayed entries to show the given range.
    pub fn update_entries(&self, mut range:Range<Id>) {
        if range != self.entries_range.get() {
            debug!(self.logger, "Update entries for {range:?}");
            let create_new_entry = || {
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
            entries.resize_with(range.len(),create_new_entry);
            let outdated = entries.iter_mut().filter(|e| e.id.get().map_or(true, |i| !range.contains(&i)));
            for (entry,(id,model)) in outdated.zip(models) {
                debug!(self.logger, "Setting new model {model:?} for entry {id}; old entry: {entry.id.get():?}");
                entry.set_model(id,&model);
                entry.set_position_xy(Vector2(0.0, Self::position_y_of_entry(id)));
            }
            self.entries_range.set(range);
        }
    }

    /// Update displayed entries, giving new provider.
    pub fn update_entries_new_provider
    (&self, provider:impl Into<AnyModelProvider> + 'static, range:Range<Id>) {
        self.provider.set(provider.into());
        self.update_entries(range)
    }
}

impl display::Object for List {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}