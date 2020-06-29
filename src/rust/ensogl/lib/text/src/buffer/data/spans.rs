#![allow(missing_docs)]

//! Text spans used to store metadata information.

use crate::prelude::*;

use super::range::Range;
use super::rope;
use super::unit::*;

pub use rope::SpansBuilder;



// =============
// === Spans ===
// =============

#[derive(Clone,Debug,Default)]
pub struct Spans<T:Clone> {
    raw : rope::Spans<T>
}

impl<T:Clone> Spans<T> {
    pub fn len(&self) -> Bytes {
        Bytes(self.raw.len())
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    pub fn set(&mut self, range:Range<Bytes>, length:Bytes, data:T) {
        let mut builder = SpansBuilder::new(length.value);
        builder.add_span(..,data);
        self.edit(range,builder.build());
    }

    pub fn focus(&self, range:Range<Bytes>) -> Self {
        let raw = self.raw.subseq(range.into_rope_interval());
        Self {raw}
    }

    pub fn to_vector(&self) -> Vec<(Range<Bytes>,T)> {
        self.raw.iter().map(|t| (Range::new(Bytes(t.0.start),Bytes(t.0.end)),t.1.clone())).collect_vec()
    }

    pub fn edit
    (&mut self, range:Range<Bytes>, new:impl Into<rope::tree::Node<rope::SpansInfo<T>>>) {
        self.raw.edit(range.into_rope_interval(),new)
    }

    pub fn raw(&self) -> &rope::Spans<T> {
        &self.raw
    }

    pub fn raw_mut(&mut self) -> &mut rope::Spans<T> {
        &mut self.raw
    }
}
