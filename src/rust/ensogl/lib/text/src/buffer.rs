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



// ==============
// === Buffer ===
// ==============

/// Text container with associated styles.
#[derive(Clone,CloneRef,Debug,Default)]
pub struct Buffer {
    pub(crate) data  : Data,
    pub(crate) style : Rc<RefCell<Style>>,
}

impl Buffer {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Creates a new `View` for the buffer.
    pub fn new_view(&self) -> View {
        View::new(self)
    }

    pub fn from_text(data:Data) -> Self {
        let range = data.range();
        let mut style = Style::default();
        // FIXME: Remove the following after adding data edits, and always create data as empty first.
        style.size.spans.set(range,None);
        style.color.spans.set(range,None);
        style.bold.spans.set(range,None);
        style.italics.spans.set(range,None);
        style.underline.spans.set(range,None);
        let style = Rc::new(RefCell::new(style));
        Self {data,style}
    }

    pub fn focus_style(&self, range:impl data::RangeBounds) -> Style {
        let range = range.with_upper_bound(self.len());
        self.style.borrow().focus(range)
    }

    pub fn style(&self) -> Style {
        self.style.borrow().clone()
    }
}

impl Deref for Buffer {
    type Target = Data;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

pub trait Setter<T> {
    fn set(&self, range:impl data::RangeBounds, data:T);
}

pub trait DefaultSetter<T> {
    fn set_default(&self, data:T);
}


// === Conversions ===

impl From<Data>     for Buffer { fn from(data:Data)  -> Self { Self::from_text(data) } }
impl From<&Data>    for Buffer { fn from(data:&Data) -> Self { data.clone().into() } }
impl From<&str>     for Buffer { fn from(s:&str)     -> Self { Data::from(s).into() } }
impl From<String>   for Buffer { fn from(s:String)   -> Self { Data::from(s).into() } }
impl From<&String>  for Buffer { fn from(s:&String)  -> Self { Data::from(s).into() } }
impl From<&&String> for Buffer { fn from(s:&&String) -> Self { (*s).into() } }
impl From<&&str>    for Buffer { fn from(s:&&str)    -> Self { (*s).into() } }
