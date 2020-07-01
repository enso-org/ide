#![allow(missing_docs)]

use crate::prelude::*;
use std::ops::AddAssign; // FIXME

use ensogl::math::unit;
use ensogl::math::newtype;

pub mod traits {
    pub use super::bytes::Into  as TRAIT_bytes_into;
    pub use super::line::Into   as TRAIT_line_into;
    pub use super::column::Into as TRAIT_column_into;
}
pub use traits::*;



// =============
// === Bytes ===
// =============

unit! {
/// An offset in the buffer in bytes.
Bytes::bytes(usize)
}

impl Bytes {
    // Interpret the byte offset as column.
    pub fn as_column(self) -> Column {
        Column(self.value)
    }
}

impl<T:Into<Bytes>> bytes::Into for Range<T> {
    type Output = Range<Bytes>;
    fn bytes(self) -> Self::Output {
        let start = self.start.bytes();
        let end   = self.end.bytes();
        Range {start,end}
    }
}



// ============
// === Line ===
// ============

unit! {
/// A type representing vertical measurements.
Line::line(usize)
}



// ==============
// === Column ===
// ==============

unit! {
/// A type representing horizontal measurements.
///
/// **WARNING**
/// This is currently in units that are not very well defined except that ASCII characters count as
/// 1 each. This should be fixed in the future.
Column::column(usize)
}

newtype! {
/// A type representing 2d measurements.
Location {
    line   : Line,
    column : Column,
}}
