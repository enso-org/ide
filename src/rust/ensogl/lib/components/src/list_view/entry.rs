#![allow(missing_docs)]

//! A single entry in Select
use crate::prelude::*;

use ensogl_core::application::Application;
use ensogl_core::display;
use ensogl_core::display::shape::StyleWatch;
use ensogl_text as text;
use ensogl_theme;


// =================
// === Constants ===
// =================

/// Padding inside entry in pixels.
pub const PADDING:f32 = 14.0;
/// The overall entry's height (including padding).
pub const HEIGHT:f32 = 30.0;
/// The text size of entry's labe.
pub const LABEL_SIZE:f32 = 12.0;
/// The size in pixels of icons inside entries.
pub const ICON_SIZE:f32 = 0.0; // TODO[ao] restore when we create icons for the searcher.
/// The gap between icon and label.
pub const ICON_LABEL_GAP:f32 = 0.0; // TODO[ao] restore when we create icons for the searcher.



// ===================
// === Entry Model ===
// ===================

/// Entry id. 0 is the first entry in component.
pub type Id = usize;


/// Should be anchored at the center left
// TODO: Mention additional constraints
pub trait Entry: display::Object + Debug {
    fn set_selected(&self, selected:bool);
    fn set_width(&self, width:f32);
}

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct AnyEntry(Rc<dyn Entry>);

impl<T: Entry + 'static> From<T> for AnyEntry {
    fn from(entry:T) -> Self { Self(Rc::new(entry)) }
}

impl<T: Entry + 'static> From<Rc<T>> for AnyEntry {
    fn from(entry:Rc<T>) -> Self { Self(entry) }
}

impl display::Object for AnyEntry {
    fn display_object(&self) -> &display::object::Instance {
        self.0.display_object()
    }
}



// === Entry Provider ===

/// The Entry Provider for select component.
///
/// The select does not display all entries at once, instead it lazily ask for models of entries
/// when they're about to be displayed. So setting the select content is essentially providing
/// implementor of this trait.
pub trait EntryProvider: Debug {
    /// Number of all entries.
    fn entry_count(&self) -> usize;

    /// Get the model of entry with given id. The implementors should return `None` only when the
    /// requested id is greater or equal to the entry count.
    fn get(&self, app:&Application, id:Id) -> Option<AnyEntry>;
}

/// A wrapper for shared instance of some ModelProvider.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct AnyEntryProvider(Rc<dyn EntryProvider>);

impl<T: EntryProvider + 'static> From<T> for AnyEntryProvider {
    fn from(provider:T) -> Self { Self(Rc::new(provider)) }
}

impl<T: EntryProvider + 'static> From<Rc<T>> for AnyEntryProvider {
    fn from(provider:Rc<T>) -> Self { Self(provider) }
}

impl Default for AnyEntryProvider {
    fn default() -> Self {EmptyProvider.into()}
}


// === Empty Model Provider ===

/// An Entry Model Provider giving no entries.
///
/// This is the default provider for new select components.
#[derive(Clone,CloneRef,Copy,Debug)]
pub struct EmptyProvider;

#[derive(Debug,Copy,Clone)]
pub enum EmptyProviderEntry {}

impl display::Object for EmptyProviderEntry {
    fn display_object(&self) -> &display::object::Instance {
        match *self {}
    }
}

impl Entry for EmptyProviderEntry {
    fn set_selected(&self, _selected: bool) {}

    fn set_width(&self, _width: f32) {}
}

impl EntryProvider for EmptyProvider {
    fn entry_count(&self) -> usize {
        0
    }

    fn get(&self, _:&Application, _:usize) -> Option<AnyEntry> {
        None
    }
}


// === Model Provider for Vectors ===

#[derive(Debug)]
struct StringEntry {
    display_object : display::object::Instance,
    label          : text::Area,
}

impl StringEntry {
    fn new(app:&Application, string:&str) -> Self {
        let logger = Logger::new("StringEntry");
        let display_object = display::object::Instance::new(logger);
        let label = text::Area::new(app);
        label.add_to_scene_layer(&app.display.scene().layers.label);
        display_object.add_child(&label);
        let styles = StyleWatch::new(&app.display.scene().style_sheet);
        let text_color = styles.get_color(ensogl_theme::widget::list_view::text);
        label.set_default_color(text_color);
        label.set_default_text_size(text::Size(LABEL_SIZE));
        label.set_position_xy(Vector2(PADDING + ICON_SIZE + ICON_LABEL_GAP, LABEL_SIZE/2.0));
        label.set_content(string);
        Self {display_object,label}
    }
}

impl display::Object for StringEntry {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}

impl Entry for StringEntry {
    fn set_selected(&self, _selected: bool) {}

    fn set_width(&self, _width: f32) {}
}

#[derive(Debug,Shrinkwrap)]
struct VecEntryProvider(Rc<Vec<String>>);

impl EntryProvider for VecEntryProvider {
    fn entry_count(&self) -> usize {
        self.len()
    }

    fn get(&self, app:&Application, id:usize) -> Option<AnyEntry> {
        let string = self.0.get(id)?;
        Some(StringEntry::new(app,string).into())
    }
}

impl Into<AnyEntryProvider> for Rc<Vec<String>> {
    fn into(self) -> AnyEntryProvider {
        VecEntryProvider(self).into()
    }
}


// === Masked Model Provider ===

/// An Entry Model Provider that wraps a `AnyModelProvider` and allows the masking of a single item.
#[derive(Clone,Debug)]
pub struct SingleMaskedProvider {
    content : AnyEntryProvider,
    mask    : Cell<Option<Id>>,
}

impl EntryProvider for SingleMaskedProvider {
    fn entry_count(&self) -> usize {
        match self.mask.get() {
            None    => self.content.entry_count(),
            Some(_) => self.content.entry_count().saturating_sub(1),
        }
    }

    fn get(&self, app:&Application, ix:usize) -> Option<AnyEntry> {
        let internal_ix = self.unmasked_index(ix);
        self.content.get(app, internal_ix)
    }
}

impl SingleMaskedProvider {

    /// Return the index to the unmasked underlying data. Will only be valid to use after
    /// calling `clear_mask`.
    ///
    /// Transform index of an element visible in the menu, to the index of the all the objects,
    /// accounting for the removal of the selected item.
    ///
    /// Example:
    /// ```text
    /// Mask              `Some(1)`
    /// Masked indices    [0,     1, 2]
    /// Unmasked Index    [0, 1,  2, 3]
    /// -------------------------------
    /// Mask              `None`
    /// Masked indices    [0, 1, 2, 3]
    /// Unmasked Index    [0, 1, 2, 3]
    /// ```
    pub fn unmasked_index(&self, ix:Id) -> Id {
        match self.mask.get() {
            None                 => ix,
            Some(id) if ix < id  => ix,
            Some(_)              => ix+1,
        }
    }

    /// Mask out the given index. All methods will now skip this item and the `SingleMaskedProvider`
    /// will behave as if it was not there.
    ///
    /// *Important:* The index is interpreted according to the _masked_ position of elements.
    pub fn set_mask(&self, ix:Id) {
        let internal_ix = self.unmasked_index(ix);
        self.mask.set(Some(internal_ix));
    }

    /// Mask out the given index. All methods will now skip this item and the `SingleMaskedProvider`
    /// will behave as if it was not there.
    ///
    /// *Important:* The index is interpreted according to the _unmasked_ position of elements.
    pub fn set_mask_raw(&self, ix:Id) {
        self.mask.set(Some(ix));
    }

    /// Clear the masked item.
    pub fn clear_mask(&self) {
        self.mask.set(None)
    }
}

impl From<AnyEntryProvider> for SingleMaskedProvider {
    fn from(content: AnyEntryProvider) -> Self {
        let mask = default();
        SingleMaskedProvider{content,mask}
    }
}



// =================
// === EntryList ===
// =================

/// The output of `entry_at_y_position`
#[allow(missing_docs)]
#[derive(Copy,Clone,Debug,Eq,Hash,PartialEq)]
pub enum IdAtYPosition {
    AboveFirst, UnderLast, Entry(Id)
}

impl IdAtYPosition {
    /// Returns id of entry if present.
    pub fn entry(&self) -> Option<Id> {
        if let Self::Entry(id) = self { Some(*id) }
        else                          { None      }
    }
}

/// A view containing an entry list, arranged in column.
///
/// Not all entries are displayed at once, only those visible.
#[derive(Clone,CloneRef,Debug)]
pub struct List {
    logger         : Logger,
    app            : Application,
    display_object : display::object::Instance,
    visible_entries: Rc<RefCell<HashMap<Id,AnyEntry>>>,
    visible_range  : Rc<CloneCell<Range<f32>>>,
    provider       : Rc<CloneRefCell<AnyEntryProvider>>,
    selected_id    : Rc<Cell<Option<Id>>>,
    entry_width: Rc<Cell<f32>>,
}

impl List {
    /// Entry List View constructor.
    pub fn new(parent:impl AnyLogger, app:&Application) -> Self {
        let app            = app.clone_ref();
        let logger         = Logger::sub(parent,"entry::List");
        let visible_entries = default();
        let visible_range  = Rc::new(CloneCell::new(default()..default()));
        let display_object = display::object::Instance::new(&logger);
        let provider       = default();
        let selected_id    = default();
        let width          = default();
        List {logger,app,display_object,visible_entries,visible_range,provider,selected_id, entry_width: width }
    }

    /// The number of all entries in List, including not displayed.
    pub fn entry_count(&self) -> usize {
        self.provider.get().entry_count()
    }

    /// The number of all displayed entries in List.
    pub fn visible_entry_count(&self) -> usize {
        ((self.visible_range.get().end - self.visible_range.get().start) / HEIGHT) as usize
    }

    /// Y position of entry with given id, relative to Entry List position.
    pub fn position_y_of_entry(id:Id) -> f32 { id as f32 * -HEIGHT - 0.5 * HEIGHT }

    /// Y range of entry with given id, relative to Entry List position.
    pub fn y_range_of_entry(id:Id) -> Range<f32> {
        let position = Self::position_y_of_entry(id);
        (position - HEIGHT / 2.0)..(position + HEIGHT / 2.0)
    }

    /// Y range of all entries in this list, including not displayed.
    pub fn total_height(entry_count:usize) -> f32 {
        entry_count as f32 * HEIGHT
    }

    /// Get the entry id which lays on given y coordinate.
    pub fn entry_at_y_position(y:f32, entry_count:usize) -> IdAtYPosition {
        use IdAtYPosition::*;
        let height = Self::total_height(entry_count);
        if y > 0.0          { AboveFirst               }
        else if y < -height { UnderLast                }
        else                { Entry((-y/HEIGHT) as Id) }
    }

    pub fn set_visible_range(&self, range:Range<f32>) {
        self.visible_range.set(range);
        self.update_visible_entries();
    }

    /// Update displayed entries to show the given range.
    fn update_visible_entries(&self) {
        let entry_at_y_saturating = |y:f32| {
            match Self::entry_at_y_position(y,self.entry_count()) {
                IdAtYPosition::AboveFirst => 0,
                IdAtYPosition::UnderLast  => self.entry_count().saturating_sub(1),
                IdAtYPosition::Entry(id)  => id,
            }
        };
        let first = entry_at_y_saturating(self.visible_range.get().end);
        let last  = entry_at_y_saturating(self.visible_range.get().start);
        let visible_ids: Range<Id> = first..(last+1);


        // It can be extremely slow to create or destroy objects, in particular text areas.
        // Therefor, we only destroy or create those that enter or leave the visible area.
        let mut visible_entries = self.visible_entries.borrow_mut();

        // Remove entries that went out of sight
        visible_entries.retain(|id, _| visible_ids.contains(&id));

        // Add entries that came into sight
        for id in visible_ids {
            if !visible_entries.contains_key(&id) {
                if let Some(new_entry) = self.provider.get().get(&self.app,id) {
                    self.display_object.add_child(&new_entry);
                    new_entry.set_position_y(Self::position_y_of_entry(id));
                    if self.selected_id.get() == Some(id) {
                        new_entry.set_selected(true);
                    }
                    new_entry.set_width(self.entry_width.get());
                    visible_entries.insert(id, new_entry);
                }
            }
        }
    }

    /// Update displayed entries, giving new provider.
    pub fn set_provider(&self, provider:AnyEntryProvider) {
        const MAX_SAFE_ENTRIES_COUNT:usize = 1000;
        let provider = provider;
        if provider.entry_count() > MAX_SAFE_ENTRIES_COUNT {
            error!(self.logger, "ListView entry count exceed {MAX_SAFE_ENTRIES_COUNT} - so big \
            number of entries can cause visual glitches, e.g. https://github.com/enso-org/ide/\
            issues/757 or https://github.com/enso-org/ide/issues/758");
        }
        self.visible_entries.borrow_mut().clear();
        self.provider.set(provider);
        self.update_visible_entries()
    }

    pub fn set_selection(&self, new:Option<Id>) {
        let old = self.selected_id.replace(new);
        if new != old {
            if let Some(previous) = old {
                if let Some(previous_entry) = self.visible_entries.deref().borrow().get(&previous) {
                    previous_entry.set_selected(false);
                }
            }
            if let Some(new) = new {
                if let Some(new_entry) = self.visible_entries.deref().borrow().get(&new) {
                    new_entry.set_selected(true);
                }
            }
        }
    }

    pub fn set_entry_width(&self, width:f32) {
        self.entry_width.set(width);
        for entry in self.visible_entries.deref().borrow().values() {
            entry.set_width(width);
        }
    }
}

impl display::Object for List {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}


// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_masked_provider() {
//         let test_data   = vec!["A", "B", "C", "D"];
//         let test_models = test_data.into_iter().map(|label| Model::new(label)).collect_vec();
//         let provider:AnyModelProvider     = test_models.into();
//         let provider:SingleMaskedProvider = provider.into();
//
//         assert_eq!(provider.entry_count(), 4);
//         assert_eq!(provider.get(0).unwrap().label, "A");
//         assert_eq!(provider.get(1).unwrap().label, "B");
//         assert_eq!(provider.get(2).unwrap().label, "C");
//         assert_eq!(provider.get(3).unwrap().label, "D");
//
//         provider.set_mask_raw(0);
//         assert_eq!(provider.entry_count(), 3);
//         assert_eq!(provider.get(0).unwrap().label, "B");
//         assert_eq!(provider.get(1).unwrap().label, "C");
//         assert_eq!(provider.get(2).unwrap().label, "D");
//
//         provider.set_mask_raw(1);
//         assert_eq!(provider.entry_count(), 3);
//         assert_eq!(provider.get(0).unwrap().label, "A");
//         assert_eq!(provider.get(1).unwrap().label, "C");
//         assert_eq!(provider.get(2).unwrap().label, "D");
//
//         provider.set_mask_raw(2);
//         assert_eq!(provider.entry_count(), 3);
//         assert_eq!(provider.get(0).unwrap().label, "A");
//         assert_eq!(provider.get(1).unwrap().label, "B");
//         assert_eq!(provider.get(2).unwrap().label, "D");
//
//         provider.set_mask_raw(3);
//         assert_eq!(provider.entry_count(), 3);
//         assert_eq!(provider.get(0).unwrap().label, "A");
//         assert_eq!(provider.get(1).unwrap().label, "B");
//         assert_eq!(provider.get(2).unwrap().label, "C");
//     }
// }
