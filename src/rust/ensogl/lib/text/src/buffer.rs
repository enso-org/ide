

pub mod location;
pub mod movement;
pub mod selection;
pub mod view;
pub mod text;

pub use movement::Movement;
pub use selection::Selection;
pub use view::View;


use crate::prelude::*;

use crate::prelude::*;
use crate::data::color;
use crate::text::Text;



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
}


// === Conversions ===

impl From<Text>     for Buffer { fn from(text:Text)  -> Self { Self {text,..default()} } }
impl From<&Text>    for Buffer { fn from(text:&Text) -> Self { text.clone().into() } }
impl From<&str>     for Buffer { fn from(s:&str)     -> Self { Text::from(s).into() } }
impl From<String>   for Buffer { fn from(s:String)   -> Self { Text::from(s).into() } }
impl From<&String>  for Buffer { fn from(s:&String)  -> Self { Text::from(s).into() } }
impl From<&&String> for Buffer { fn from(s:&&String) -> Self { (*s).into() } }
impl From<&&str>    for Buffer { fn from(s:&&str)    -> Self { (*s).into() } }
