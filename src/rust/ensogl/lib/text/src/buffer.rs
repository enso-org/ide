

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


pub mod traits {
    pub use super::location::traits::*;
    pub use super::Setter as TRAIT_Setter;
    pub use super::DefaultSetter as TRAIT_DefaultSetter;
}




#[derive(Clone,Debug,Default)]
pub struct StyleProperty<T:Clone> {
    spans   : text::Spans<T>,
    default : T,
}

impl<T:Clone> StyleProperty<T> {
    pub fn focus(&self, range:text::Range<Bytes>) -> Self {
        let spans   = self.spans.focus(range);
        let default = self.default.clone();
        Self {spans,default}
    }

    pub fn to_vector(&self) -> Vec<(text::Range<Bytes>,T)> {
        let spans_iter = self.spans.to_vector().into_iter();
        spans_iter.map(|t|(t.0,t.1.unwrap_or_else(||self.default.clone()))).collect_vec()
    }

    pub fn default(&self) -> &T {
        &self.default
    }
}

impl<T:Clone> Deref for StyleProperty<T> {
    type Target = text::Spans<T>;
    fn deref(&self) -> &Self::Target {
        &self.spans
    }
}

impl<T:Clone> DerefMut for StyleProperty<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spans
    }
}


pub struct StyleIterator {
    offset     : Bytes,
    value      : StyleIteratorValue,
    component  : StyleIteratorComponents,
}

impl StyleIterator {
    pub fn new(component:StyleIteratorComponents) -> Self {
        let offset = default();
        let value  = default();
        Self {offset,value,component}
    }

    pub fn drop(&mut self, bytes:Bytes) {
        for _ in 0 .. bytes.raw {
            self.next();
        }
    }
}


// =============
// === Style ===
// =============

macro_rules! define_styles {
    ($($field:ident : $field_type:ty),* $(,)?) => {
        #[derive(Debug,Default)]
        pub struct StyleValue {
            $(pub $field : $field_type),*
        }

        pub struct StyleIteratorComponents {
            $($field : std::vec::IntoIter<(text::Range<Bytes>,$field_type)>),*
        }

        #[derive(Default)]
        pub struct StyleIteratorValue {
            $($field : Option<(text::Range<Bytes>,$field_type)>),*
        }

        impl Iterator for StyleIterator {
            type Item = StyleValue;
            fn next(&mut self) -> Option<Self::Item> {
                $(
                    if self.value.$field.map(|t| self.offset < t.0.end) != Some(true) {
                        self.value.$field = self.component.$field.next()
                    }
                    let $field = self.value.$field?.1;
                )*
                self.offset += 1;
                Some(StyleValue {$($field),*})
            }
        }

        #[derive(Clone,Debug,Default)]
        pub struct Style {
            $(pub $field : StyleProperty<$field_type>),*
        }

        impl Style {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn focus(&self, bounds:text::Range<Bytes>) -> Self {
                $(let $field = self.$field.focus(bounds);)*
                Self {$($field),*}
            }

            pub fn iter(&self) -> StyleIterator {
                $(let $field = self.$field.to_vector().into_iter();)*
                StyleIterator::new(StyleIteratorComponents {$($field),*})
            }
        }

        $(
            impl Setter<$field_type> for Buffer {
                fn set(&self, range:impl text::RangeBounds, data:$field_type) {
                    let range = self.crop_range(range);
                    self.style.borrow_mut().$field.set(range,Some(data))
                }
            }

            impl DefaultSetter<$field_type> for Buffer {
                fn set_default(&self, data:$field_type) {
                    self.style.borrow_mut().$field.default = data;
                }
            }

        )*
    };
}

macro_rules! newtype {
    ($(#$meta:tt)* $name:ident($field_type:ty)) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,Eq,From,Hash,Ord,PartialEq,PartialOrd)]
        pub struct $name { raw : $field_type }

        /// Smart constructor.
        $(#$meta)*
        pub fn $name(raw:$field_type) -> $name { $name {raw} }
    };
}

newtype!(Bold(bool));
newtype!(Italics(bool));
newtype!(Underline(bool));



define_styles! {
    color     : color::Rgba,
    bold      : Bold,
    italics   : Italics,
    underline : Underline,
}


// ==============
// === Buffer ===
// ==============

/// Text container with applied styles.
#[derive(Clone,CloneRef,Debug,Default)]
#[allow(missing_docs)]
pub struct Buffer {
    pub text : Text,
    style    : Rc<RefCell<Style>>,
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
        let range = text.range();
        let mut style = Style::default();
        // FIXME: Remove the following after adding text edits, and always create text as empty first.
        style.color.spans.TMP_set_default(range);
        style.bold.spans.TMP_set_default(range);
        style.italics.spans.TMP_set_default(range);
        style.underline.spans.TMP_set_default(range);
        let style = Rc::new(RefCell::new(style));
        Self {text,style}
    }

    pub fn focus_style(&self, range:impl text::RangeBounds) -> Style {
        let range = range.with_upper_bound(self.len());
        self.style.borrow().focus(range)
    }

    pub fn style(&self) -> Style {
        self.style.borrow().clone()
    }
}

impl Deref for Buffer {
    type Target = Text;
    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

pub trait Setter<T> {
    fn set(&self, range:impl text::RangeBounds, data:T);
}

pub trait DefaultSetter<T> {
    fn set_default(&self, data:T);
}


// === Conversions ===

impl From<Text>     for Buffer { fn from(text:Text)  -> Self { Self::from_text(text) } }
impl From<&Text>    for Buffer { fn from(text:&Text) -> Self { text.clone().into() } }
impl From<&str>     for Buffer { fn from(s:&str)     -> Self { Text::from(s).into() } }
impl From<String>   for Buffer { fn from(s:String)   -> Self { Text::from(s).into() } }
impl From<&String>  for Buffer { fn from(s:&String)  -> Self { Text::from(s).into() } }
impl From<&&String> for Buffer { fn from(s:&&String) -> Self { (*s).into() } }
impl From<&&str>    for Buffer { fn from(s:&&str)    -> Self { (*s).into() } }
