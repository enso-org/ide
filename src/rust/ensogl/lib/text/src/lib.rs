//! Ensogl text rendering implementation.

#![feature(clamp)]
#![feature(saturating_neg)]
#![feature(trait_alias)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

pub mod buffer;
pub mod component;
pub mod typeface;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl::prelude::*;
}

//pub use ensogl::display;
use ensogl::display;
pub use ensogl::data;
pub use buffer::Buffer;
pub use component::Area;

use buffer::*;

use selection::Selection;

//use prelude::*;
//
//
use crate::buffer::location::*;


use text::rope::spans::Spans;
use text::rope::spans::SpansBuilder;
use text::rope::breaks::{BreakBuilder, Breaks, BreaksInfo, BreaksMetric};
use text::rope::{Interval, LinesMetric, Rope, RopeDelta, RopeInfo};
//use rope::LinesMetric;
//use rope::rope::BaseMetric;
//use rope::tree::*;
//
//
//
//
//
//
//pub struct Line {
//    text  : Rope,
//    index : usize,
//}

use std::cmp::max;
use std::cmp::min;



use crate::prelude::*;
use ensogl::data::color;
use crate::typeface::font;
use crate::typeface::pen;
use typeface::glyph;
use typeface::glyph::Glyph;



pub fn main() {
    let buffer = Buffer::from("Test text!");
    buffer.style.color.set(1..2,color::Rgba::new(1.0,0.0,0.0,1.0));
    buffer.style.color.set(5..6,color::Rgba::new(1.0,0.0,0.0,1.0));
    let view = buffer.new_view();

    view.add_selection(Selection::new(Bytes(0),Bytes(0)));


//    let foo = buffer.color.iter().collect_vec();
    let foo = buffer.style.color.subseq(0..15);
    let foo = foo.iter().collect_vec();
    println!("{:#?}",foo);

    println!("{:#?}",view.selections());

    view.move_carets(Movement::Right);

    println!("{:#?}",view.selections());


    // ERROR: https://github.com/xi-editor/xi-editor/issues/1276
    //
    // let mut spans : text::rope::Spans<bool> = default();
    //
    // let interval : text::rope::Interval = (0..3).into();
    // let mut builder = text::rope::SpansBuilder::new(20);
    // builder.add_span(interval,true);
    // spans.edit(interval,builder.build());
    //
    // let interval : text::rope::Interval = (10..15).into();
    // let mut builder = text::rope::SpansBuilder::new(20);
    // builder.add_span(interval,true);
    // spans.edit(interval,builder.build());
    //
    // println!("{:?}",spans.subseq(0..100).iter().collect_vec());

     let mut spans : text::rope::Spans<bool> = default();

     let interval : text::rope::Interval = (0..3).into();
     let mut builder = text::rope::SpansBuilder::new(3);
     builder.add_span(interval,true);
     spans.edit((0..3),builder.build());

     let interval : text::rope::Interval = (0..2).into();
     let mut builder = text::rope::SpansBuilder::new(2);
     builder.add_span(interval,false);
     spans.edit((2..4),builder.build());

     println!("{:?}",spans.subseq(0..100).iter().collect_vec());
}



//#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
//pub struct BufferId(pub usize);
//
//pub struct BufferMap {
//    map : BTreeMap<BufferId,Buffer>
//}
//




















