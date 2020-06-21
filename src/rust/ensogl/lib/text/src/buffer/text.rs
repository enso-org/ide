
pub mod rope {
    pub use xi_rope::*;
    pub use xi_rope::rope::Lines;
    pub use xi_rope::spans::Spans;
    pub use xi_rope::spans::SpansBuilder;
    pub use xi_rope::spans::SpansInfo;
    pub use xi_rope::interval::IntervalBounds;
}

pub use rope::SpansBuilder;
pub use rope::Cursor;
pub use rope::LinesMetric;
pub use rope::Lines;

use crate::prelude::*;
use crate::buffer::location::*;

use rope::Rope;



// ============
// === Text ===
// ============

impl_clone_ref_as_clone!(Text);
#[derive(Debug,Clone,Default,Deref)]
#[allow(missing_docs)]
pub struct Text {
    pub rope : Rope,
}

impl Text {
    /// Return the len of the text in bytes.
    pub fn len(&self) -> Bytes {
        Bytes(self.rope.len())
    }

    /// Return the offset to the previous grapheme if any.
    pub fn prev_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.prev_grapheme_offset(offset.raw).map(Bytes)
    }

    /// Return the offset to the next grapheme if any.
    pub fn next_grapheme_offset(&self, offset:Bytes) -> Option<Bytes> {
        self.rope.next_grapheme_offset(offset.raw).map(Bytes)
    }
}

impl From<&str>     for Text { fn from(t:&str)     -> Self { Self {rope:t.into()} } }
impl From<String>   for Text { fn from(t:String)   -> Self { Self {rope:t.into()} } }
impl From<&String>  for Text { fn from(t:&String)  -> Self { Self {rope:t.into()} } }
impl From<&&String> for Text { fn from(t:&&String) -> Self { (*t).into() } }
impl From<&&str>    for Text { fn from(t:&&str)    -> Self { (*t).into() } }



// =============
// === Spans ===
// =============

#[derive(CloneRef,Clone,Debug,Default)]
pub struct Spans<T:Clone> {
    rc : Rc<RefCell<rope::Spans<T>>>
}

impl<T:Clone> Spans<T> {
    pub fn set(&self, interval:impl Into<rope::Interval>, data:impl Into<T>) {
        let interval    = interval.into();
        let data        = data.into();
        let mut builder = SpansBuilder::new(interval.end - interval.start);
        builder.add_span((..),data);
        self.edit(interval,builder.build());
    }

    pub fn set_default(&self, interval:impl Into<rope::Interval>) where T:Default {
        let data : T = default();
        self.set(interval,data);
    }

    pub fn subseq(&self, bounds:impl rope::IntervalBounds) -> rope::tree::Node<rope::SpansInfo<T>> {
        self.rc.borrow().subseq(bounds)
    }

    pub fn focus(&self, bounds:impl rope::IntervalBounds) -> Self {
        let rc = Rc::new(RefCell::new(self.subseq(bounds)));
        Self {rc}
    }

    pub fn to_vector(&self) -> Vec<(rope::Interval,T)> {
        self.rc.borrow().iter().map(|t| (t.0,t.1.clone())).collect_vec()
    }

    pub fn edit
    (&self, bounds:impl rope::IntervalBounds, new:impl Into<rope::tree::Node<rope::SpansInfo<T>>>) {
        self.rc.borrow_mut().edit(bounds,new)
    }

    pub fn raw(&self) -> rope::Spans<T> {
        self.rc.borrow().clone()
    }
}
