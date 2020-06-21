

pub mod location;
pub mod movement;
pub mod selection;
pub mod view;
pub mod text;

pub use movement::Movement;
pub use selection::Selection;
pub use view::View;
pub use location::Column;
pub use location::Bytes;
pub use location::Line;
pub use text::Lines;


use crate::prelude::*;

use crate::prelude::*;
use crate::data::color;
use crate::text::Text;


#[derive(Debug,Default)]
pub struct StyleValue {
    pub color     : color::Rgba,
    pub bold      : bool,
    pub italics   : bool,
    pub underline : bool,
}


pub struct StyleIteratorComponents {
    color     : std::vec::IntoIter<(text::rope::Interval,color::Rgba)>,
    bold      : std::vec::IntoIter<(text::rope::Interval,bool)>,
    italics   : std::vec::IntoIter<(text::rope::Interval,bool)>,
    underline : std::vec::IntoIter<(text::rope::Interval,bool)>,
}

#[derive(Default)]
pub struct StyleIteratorValue {
    color     : Option<(text::rope::Interval,color::Rgba)>,
    bold      : Option<(text::rope::Interval,bool)>,
    italics   : Option<(text::rope::Interval,bool)>,
    underline : Option<(text::rope::Interval,bool)>,
}

pub struct StyleIterator {
    byte_index : usize,
    value      : StyleIteratorValue,
    component  : StyleIteratorComponents,
}

impl StyleIterator {
    pub fn new(component:StyleIteratorComponents) -> Self {
        let byte_index = default();
        let value      = default();
        Self {byte_index,value,component}
    }

    pub fn drop(&mut self, bytes:usize) {
        for _ in 0 .. bytes {
            self.next();
        }
    }
}

impl Iterator for StyleIterator {
    type Item = StyleValue;
    fn next(&mut self) -> Option<Self::Item> {
        if self.value.color.map(|t| self.byte_index < t.0.end) != Some(true) {self.value.color = self.component.color.next()}
        if self.value.bold.map(|t| self.byte_index < t.0.end) != Some(true) {self.value.bold = self.component.bold.next()}
        if self.value.italics.map(|t| self.byte_index < t.0.end) != Some(true) {self.value.italics = self.component.italics.next()}
        if self.value.underline.map(|t| self.byte_index < t.0.end) != Some(true) {self.value.underline = self.component.underline.next()}

        let color = self.value.color?.1;
        let bold = self.value.bold?.1;
        let italics = self.value.italics?.1;
        let underline = self.value.underline?.1;

        self.byte_index += 1;

        Some(StyleValue {color,bold,italics,underline})
    }
}



// =============
// === Style ===
// =============

#[derive(Clone,CloneRef,Debug,Default)]
pub struct Style {
    pub color     : text::Spans<color::Rgba>,
    pub bold      : text::Spans<bool>,
    pub italics   : text::Spans<bool>,
    pub underline : text::Spans<bool>,
}

impl Style {

    pub fn focus(&self, bounds:impl text::rope::IntervalBounds+Clone) -> Self {
        let color = self.color.focus(bounds.clone());
        let bold = self.bold.focus(bounds.clone());
        let italics = self.italics.focus(bounds.clone());
        let underline = self.underline.focus(bounds);
        Self {color,bold,italics,underline}
    }

    pub fn iter(&self) -> StyleIterator {
        let color = self.color.to_vector().into_iter();
        let bold = self.bold.to_vector().into_iter();
        let italics = self.italics.to_vector().into_iter();
        let underline = self.underline.to_vector().into_iter();
        let components = StyleIteratorComponents {color,bold,italics,underline};
        StyleIterator::new(components)
    }
}


#[derive(Clone,Copy,PartialEq,Eq)]
pub struct Interval {
    pub start : Bytes,
    pub end   : Bytes,
}

pub trait IntervalBounds {
    fn with_upper_bound(self, upper_bound:Bytes) -> Interval;
}

impl IntervalBounds for std::ops::RangeFull {
    fn with_upper_bound(self, end:Bytes) -> Interval {
        let start = Bytes(0);
        Interval {start,end}
    }
}

impl From<Interval> for text::rope::Interval {
    fn from(t:Interval) -> Self {
        let start = t.start.raw;
        let end   = t.end.raw;
        Self {start,end}
    }
}



// ==============
// === Buffer ===
// ==============

/// Text container with applied styles.
#[derive(Clone,CloneRef,Debug,Default)]
#[allow(missing_docs)]
pub struct Buffer {
    pub text  : Text,
    pub style : Style,
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

    pub fn from_text(text:Text) -> Self {
        let range = text::rope::Interval::from(0..text.len().raw);
        let style = Style::default();
        style.color.set_default(range);
        style.bold.set_default(range);
        style.italics.set_default(range);
        style.underline.set_default(range);
        Self {text,style}
    }

    pub fn set_color(&self, interval:impl IntervalBounds, color:color::Rgba) {
        let interval = interval.with_upper_bound(self.len());
        self.style.color.set(interval,color);
    }
}

impl Deref for Buffer {
    type Target = Text;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

// === Conversions ===

impl From<Text>     for Buffer { fn from(text:Text)  -> Self { Self::from_text(text) } }
impl From<&Text>    for Buffer { fn from(text:&Text) -> Self { text.clone().into() } }
impl From<&str>     for Buffer { fn from(s:&str)     -> Self { Text::from(s).into() } }
impl From<String>   for Buffer { fn from(s:String)   -> Self { Text::from(s).into() } }
impl From<&String>  for Buffer { fn from(s:&String)  -> Self { Text::from(s).into() } }
impl From<&&String> for Buffer { fn from(s:&&String) -> Self { (*s).into() } }
impl From<&&str>    for Buffer { fn from(s:&&str)    -> Self { (*s).into() } }
