

pub mod location;
pub mod movement;
pub mod selection;
pub mod view;

pub mod rope {
    pub use xi_rope::*;
    pub use xi_rope::spans::Spans;
    pub use xi_rope::spans::SpansBuilder;
}

pub use movement::Movement;
pub use selection::Selection;
pub use view::View;


use crate::prelude::*;

use crate::prelude::*;
use crate::data::color;
use crate::rope::Rope;


// ==============
// === Buffer ===
// ==============

impl_clone_ref_as_clone!(Buffer);
#[derive(Clone,Debug,Default)]
pub struct Buffer {
    /// The contents of the buffer.
    pub rope: Rope,

    pub color : Rc<RefCell<rope::Spans<color::Rgba>>>,
}


impl Buffer {
    pub fn new() -> Self {
        default()
    }

    pub fn set_color(&self, interval:impl Into<rope::Interval>, color:impl Into<color::Rgba>) {
        let interval = interval.into();
        let color    = color.into();

        let mut sb = rope::SpansBuilder::new(interval.end());
        sb.add_span(interval,color);

        self.color.borrow_mut().edit(interval,sb.build());
    }

    pub fn view(&self) -> View {
        View::new(self)
    }
}


// === Conversions ===

impl From<Rope> for Buffer {
    fn from(rope:Rope) -> Self {
        Self {rope,..default()}
    }
}

impl From<&Rope> for Buffer {
    fn from(rope:&Rope) -> Self {
        let rope = rope.clone();
        Self {rope,..default()}
    }
}

impl From<&str> for Buffer {
    fn from(s:&str) -> Self {
        Rope::from(s).into()
    }
}

impl From<String> for Buffer {
    fn from(s:String) -> Self {
        Rope::from(s).into()
    }
}

impl From<&String> for Buffer {
    fn from(s:&String) -> Self {
        Rope::from(s).into()
    }
}

impl From<&&String> for Buffer {
    fn from(s:&&String) -> Self {
        (*s).into()
    }
}

impl From<&&str> for Buffer {
    fn from(s:&&str) -> Self {
        (*s).into()
    }
}