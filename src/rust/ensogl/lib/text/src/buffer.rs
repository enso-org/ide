//! Root of text buffer implementation. The text buffer is a sophisticated model for text styling
//! and editing operations.

use crate::prelude::*;



// ===============
// === Exports ===
// ===============

pub mod data;
pub mod style;
pub mod view;

pub mod traits {
    pub use super::data::traits::*;
    pub use super::Setter        as TRAIT_Setter;
    pub use super::DefaultSetter as TRAIT_DefaultSetter;
}

pub use data::Data;
pub use data::Lines;
pub use data::unit::*;
pub use view::*;
pub use style::*;

use data::crdt;
use std::collections::BTreeSet;





// ================
// === EditType ===
// ================

#[derive(PartialEq,Eq,Clone,Copy,Debug)]
pub enum EditType {
    /// A catchall for edits that don't fit elsewhere, and which should
    /// always have their own undo groups; used for things like cut/copy/paste.
    Other,
    /// An insert from the keyboard/IME (not a paste or a yank).
    Insert,
    Newline,
    /// An indentation adjustment.
    Indent,
    Delete,
    Undo,
    Redo,
    Transpose,
    Surround,
}

impl EditType {
    /// Checks whether a new undo group should be created between two edits.
    fn breaks_undo_group(self, previous:EditType) -> bool {
        self == EditType::Other || self == EditType::Transpose || self != previous
    }
}

impl Default for EditType {
    fn default() -> Self {
        Self::Other
    }
}





// TODO This could go much higher without issue but while developing it is
// better to keep it low to expose bugs in the GC during casual testing.
const MAX_UNDOS: usize = 20;


#[derive(Clone,CloneRef,Debug)]
pub struct Buffer {
    pub(crate) data : Rc<RefCell<BufferData>>
}

impl Buffer {
    /// Creates a new `View` for the buffer.
    pub fn new_view(&self) -> View {
        View::new(self)
    }

    pub fn focus_style(&self, range:impl data::RangeBounds) -> Style {
        self.data.borrow().focus_style(range)
    }
}


// ==============
// === BufferData ===
// ==============

/// Text container with associated styles.
#[derive(Debug)]
pub struct BufferData {
    pub(crate) data           : Data,
    pub(crate) engine         : crdt::Engine,
    pub(crate) style          : Style,
    /// Undo groups that may still be toggled
    pub(crate) live_undos     : Vec<usize>,
    pub(crate) this_edit_type : EditType,
    pub(crate) last_edit_type : EditType,
    force_undo_group: bool,
    undo_group_id: usize,
    /// undo groups that are no longer live and should be gc'ed
    gc_undos: BTreeSet<usize>,
    /// The index of the current undo; subsequent undos are currently 'undone'
    /// (but may be redone)
    cur_undo: usize,
}

impl Deref for BufferData {
    type Target = Data;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl BufferData {


    pub fn from_string(s:String) -> Self {
        let mut style = Style::default();

        let engine = crdt::Engine::new(data::Rope::from(s));
        let data   = Data::from(engine.get_head());

        // FIXME: Remove the following after adding data edits, and always create data as empty first.
        let range = data.range();
        style.size.spans.set(range,None);
        style.color.spans.set(range,None);
        style.bold.spans.set(range,None);
        style.italics.spans.set(range,None);
        style.underline.spans.set(range,None);
        let this_edit_type = default();
        let last_edit_type = default();

        // GC only works on undone edits or prefixes of the visible edits,
        // but initial file loading can create an edit with undo group 0,
        // so we want to collect that as part of the prefix.
        let live_undos = vec![0];
        let undo_group_id = 1;
        let force_undo_group = false;
        let gc_undos = BTreeSet::new();
        let cur_undo = 1;

        Self {data,engine,style,live_undos,force_undo_group,this_edit_type,last_edit_type,undo_group_id,gc_undos,cur_undo}
    }

    pub fn focus_style(&self, range:impl data::RangeBounds) -> Style {
        let range = self.crop_range(range);
        self.style.focus(range)
    }

    pub fn style(&self) -> Style {
        self.style.clone()
    }


    /// Applies a delta to the text, and updates undo state.
    ///
    /// Records the delta into the CRDT engine so that it can be undone. Also
    /// contains the logic for merging edits into the same undo group. At call
    /// time, self.this_edit_type should be set appropriately.
    ///
    /// This method can be called multiple times, accumulating deltas that will
    /// be committed at once with `commit_delta`. Note that it does not update
    /// the views. Thus, view-associated state such as the selection and line
    /// breaks are to be considered invalid after this method, until the
    /// `commit_delta` call.
    fn apply_change(&mut self, delta:data::Delta) {
        let head_rev_id = self.engine.get_head_rev_id();
        let undo_group  = self.calculate_undo_group();
        self.last_edit_type = self.this_edit_type;
        let priority = 0x10000;
        self.engine.edit_rev(priority, undo_group, head_rev_id.token(), delta);
        self.data = self.engine.get_head().into();
    }

    pub(crate) fn calculate_undo_group(&mut self) -> usize {
        let has_undos         = !self.live_undos.is_empty();
        let force_undo_group  = self.force_undo_group;
        let is_unbroken_group = !self.this_edit_type.breaks_undo_group(self.last_edit_type);

        if has_undos && (force_undo_group || is_unbroken_group) {
            *self.live_undos.last().unwrap()
        } else {
            let undo_group = self.undo_group_id;
            self.gc_undos.extend(&self.live_undos[self.cur_undo..]);
            self.live_undos.truncate(self.cur_undo);
            self.live_undos.push(undo_group);
            if self.live_undos.len() <= MAX_UNDOS {
                self.cur_undo += 1;
            } else {
                self.gc_undos.insert(self.live_undos.remove(0));
            }
            self.undo_group_id += 1;
            undo_group
        }
    }

    /// Replaces the selection with the text `T`.
    pub fn insert_change<T:Into<data::Rope>>(&self, regions:&selection::Group, text:T) -> data::rope::Delta {
        let rope = text.into();
        let mut builder = data::rope::DeltaBuilder::new(self.len().raw);
        for region in regions {
            let iv = data::rope::Interval::new(region.min().raw, region.max().raw);
            builder.replace(iv, rope.clone());
        }

        builder.build()
    }
}


// === Conversions ===

//impl From<Data>     for BufferData { fn from(data:Data)  -> Self { Self::from_text(data) } }
//impl From<&Data>    for BufferData { fn from(data:&Data) -> Self { data.clone().into() } }
impl From<String>   for BufferData { fn from(s:String)   -> Self { Self::from_string(s) } }
impl From<&str>     for BufferData { fn from(s:&str)     -> Self { s.to_string().into() } }
impl From<&String>  for BufferData { fn from(s:&String)  -> Self { s.clone().into() } }
impl From<&&String> for BufferData { fn from(s:&&String) -> Self { (*s).into() } }
impl From<&&str>    for BufferData { fn from(s:&&str)    -> Self { (*s).into() } }

impl<T:Into<BufferData>> From<T> for Buffer { fn from(t:T) -> Self {
    let data = Rc::new(RefCell::new(t.into()));
    Self {data}
} }


// ==============
// === Setter ===
// ==============

pub trait Setter<T> {
    fn set(&self, range:impl data::RangeBounds, data:T);
}

pub trait DefaultSetter<T> {
    fn set_default(&self, data:T);
}

//impl Setter<&str> for BufferData {
//    fn set(&self, range:impl data::RangeBounds, data:T) {
//        let range = self.crop_range(range);
//        self.rope
//    }
//}
