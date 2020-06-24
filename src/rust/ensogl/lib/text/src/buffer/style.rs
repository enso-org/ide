use crate::prelude::*;

use crate::data::color;
use super::*;



// TODO: refactor


macro_rules! newtype {
    ($(#$meta:tt)* $name:ident($field_type:ty)) => {
        $(#$meta)*
        #[derive(Clone,Copy,Debug,Default,From,PartialEq,PartialOrd)]
        #[allow(missing_docs)]
        pub struct $name {
            /// The raw, weakly typed value.
            pub raw : $field_type
        }

        /// Smart constructor.
        $(#$meta)*
        pub fn $name(raw:$field_type) -> $name { $name {raw} }
    };
}





// ================
// === Property ===
// ================

/// Style property, like `color` or `bold`. Contains information about text spans the style is
/// applied to and a default value which can be changed at runtime.
#[derive(Clone,Debug,Default)]
#[allow(missing_docs)]
pub struct Property<T:Clone> {
    pub spans : data::Spans<Option<T>>,
    default   : T,
}

impl<T:Clone> Property<T> {
    /// Return new property narrowed to the given range.
    pub fn focus(&self, range: data::Range<Bytes>) -> Self {
        let spans   = self.spans.focus(range);
        let default = self.default.clone();
        Self {spans,default}
    }

    /// Convert the property to a vector of spans.
    pub fn to_vector(&self) -> Vec<(data::Range<Bytes>, T)> {
        let spans_iter = self.spans.to_vector().into_iter();
        spans_iter.map(|t|(t.0,t.1.unwrap_or_else(||self.default.clone()))).collect_vec()
    }

    /// The default value of this property.
    pub fn default(&self) -> &T {
        &self.default
    }
}


// === Deref ===

impl<T:Clone> Deref for Property<T> {
    type Target = data::Spans<Option<T>>;
    fn deref(&self) -> &Self::Target {
        &self.spans
    }
}

impl<T:Clone> DerefMut for Property<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.spans
    }
}



// ================
// === Iterator ===
// ================

/// Byte-based iterator for the `Style`.
pub struct StyleIterator {
    offset    : Bytes,
    value     : StyleIteratorValue,
    component : StyleIteratorComponents,
}

impl StyleIterator {
    fn new(component:StyleIteratorComponents) -> Self {
        let offset = default();
        let value  = default();
        Self {offset,value,component}
    }

    /// Drop the given amount of bytes.
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

        // ==================
        // === StyleValue ===
        // ==================

        /// The value of a style at some point in the buffer.
        #[derive(Debug,Default)]
        pub struct StyleValue {
            $(pub $field : $field_type),*
        }

        struct StyleIteratorComponents {
            $($field : std::vec::IntoIter<(data::Range<Bytes>,$field_type)>),*
        }


        // ================
        // === Iterator ===
        // ================

        #[derive(Default)]
        struct StyleIteratorValue {
            $($field : Option<(data::Range<Bytes>,$field_type)>),*
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


        // =============
        // === Style ===
        // =============

        /// Definition of possible text styles, like `color`, or `bold`. Each style is encoded as
        /// `Property` for some spans in the buffer.
        #[derive(Clone,Debug,Default)]
        #[allow(missing_docs)]
        pub struct Style {
            $(pub $field : Property<$field_type>),*
        }

        impl Style {
            /// Constructor.
            pub fn new() -> Self {
                Self::default()
            }

            /// Return new style narrowed to the given range.
            pub fn focus(&self, bounds:data::Range<Bytes>) -> Self {
                $(let $field = self.$field.focus(bounds);)*
                Self {$($field),*}
            }

            /// Iterate over style values for subsequent bytes of the buffer.
            pub fn iter(&self) -> StyleIterator {
                $(let $field = self.$field.to_vector().into_iter();)*
                StyleIterator::new(StyleIteratorComponents {$($field),*})
            }
        }

        $(
            impl Setter<$field_type> for Buffer {
                fn set(&self, range:impl data::RangeBounds, data:$field_type) {
                    let range = self.data.borrow().crop_range(range); // FIXME
                    self.data.borrow_mut().style.$field.set(range,Some(data))
                }
            }

            impl DefaultSetter<$field_type> for Buffer {
                fn set_default(&self, data:$field_type) {
                    self.data.borrow_mut().style.$field.default = data;
                }
            }
        )*
    };
}


// ========================
// === Style Definition ===
// ========================

// newtype!(Size(f32));
newtype!(Bold(bool));
newtype!(Italics(bool));
newtype!(Underline(bool));

#[derive(Clone,Copy,Debug,From,PartialEq,PartialOrd)]
#[allow(missing_docs)]
pub struct Size {
    pub raw: f32
}
pub fn Size(raw:f32) -> Size { Size { raw } }

impl Default for Size {
    fn default() -> Self {
        let raw = 12.0;
        Self {raw}
    }
}

define_styles! {
    size      : Size,
    color     : color::Rgba,
    bold      : Bold,
    italics   : Italics,
    underline : Underline,
}
