//! Ensogl text rendering implementation.
//!
//! To properly understand the implementation and its assumptions, you have to know a lot about
//! text encoding in different formats and text rendering. Especially, these links are very useful:
//! - https://gankra.github.io/blah/text-hates-you
//! - https://lord.io/blog/2019/text-editing-hates-you-too
//! - https://utf8everywhere.org
//! - https://docs.google.com/document/d/1wuzzMOvKOJw93SWZAqoim1VUl9mloUxE0W6Ki_G23tw/edit
//!   (copy) https://docs.google.com/document/d/1D7iWPWQHrWY276WPVFZTi8JJqUnTcIVJs4dlG0IdCp8
//!
//! As a very short introduction, there are several common names used in this implementation:
//!
//! - **Code point**
//!   Any numerical value in the Unicode codespace. For instance: U+3243F.
//!
//! - **Code unit**
//!   The minimal bit combination that can represent a unit of encoded text. For example, UTF-8,
//!   UTF-16 and UTF-32 use 8-bit, 16-bit and 32-bit code units respectively. The above code point
//!   will be encoded as four code units ‘f0 b2 90 bf’ in UTF-8, two code units ‘d889 dc3f’ in
//!   UTF-16 and as a single code unit ‘0003243f’ in UTF-32. Note that these are just sequences of
//!   groups of bits; how they are stored on an octet-oriented media depends on the endianness of
//!   the particular encoding. When storing the above UTF-16 code units, they will be converted to
//!   ‘d8 89 dc 3f’ in UTF-16BE and to ‘89 d8 3f dc’ in UTF-16LE.
//!
//! - **Abstract character**
//!   A unit of information used for the organization, control, or representation of textual data.
//!   The standard says:
//!
//!   > For the Unicode Standard, [...] the repertoire is inherently open. Because Unicode is a
//!   > universal encoding, any abstract character that could ever be encoded is a potential
//!   > candidate to be encoded, regardless of whether the character is currently known.
//!
//!   The definition is indeed abstract. Whatever one can think of as a character—is an abstract
//!   character. For example, "tengwar letter ungwe" is an abstract character, although it is not
//!   yet representable in Unicode.
//!
//! - **Encoded character, Coded character**
//!   A mapping between a code point and an abstract character. For example, U+1F428 is a coded
//!   character which represents the abstract character <koala image>.
//!
//!   This mapping is neither total, nor injective, nor surjective:
//!   - Surragates, noncharacters and unassigned code points do not correspond to abstract
//!     characters at all.
//!   - Some abstract characters can be encoded by different code points; U+03A9 greek capital
//!     letter omega and U+2126 ohm sign both correspond to the same abstract character ‘Ω’, and
//!     must be treated identically.
//!   - Some abstract characters cannot be encoded by a single code point. These are represented by
//!     sequences of coded characters. For example, the only way to represent the abstract character
//!     <cyrillic small letter yu with acute> is by the sequence U+044E cyrillic small letter yu
//!     followed by U+0301 combining acute accent.
//!
//!   Moreover, for some abstract characters, there exist representations using multiple code
//!   points, in addition to the single coded character form. The abstract character ǵ can be coded
//!   by the single code point U+01F5 latin small letter g with acute, or by the sequence
//!   <U+0067 latin small letter g, U+0301 combining acute accent>.
//!
//! - **User-perceived character**
//!   Whatever the end user thinks of as a character. This notion is language dependent. For
//!   instance, ‘ch’ is two letters in English and Latin, but considered to be one letter in Czech
//!   and Slovak.
//!
//! - **Grapheme cluster**
//!   A sequence of coded characters that ‘should be kept together’. Grapheme clusters approximate
//!   the notion of user-perceived characters in a language independent way. They are used for,
//!   e.g., cursor movement and selection.
//!
//! - **Glyph**
//!   A particular shape within a font. Fonts are collections of glyphs designed by a type designer.
//!   It’s the text shaping and rendering engine responsibility to convert a sequence of code points
//!   into a sequence of glyphs within the specified font. The rules for this conversion might be
//!   complicated, locale dependent, and are beyond the scope of the Unicode standard.

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
pub use crate::buffer::location::*;


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
//    let foo = buffer.style.color.subseq(0..15);
//    let foo = foo.iter().collect_vec();
//    println!("{:#?}",foo);

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

    let mut spans : text::rope::Spans<usize> = default();

    let interval : text::rope::Interval = (0..100).into();
    let mut builder = text::rope::SpansBuilder::new(100);
    builder.add_span((..),0);
    spans.edit((0..100),builder.build());

    let interval : text::rope::Interval = (0..3).into();
    let mut builder = text::rope::SpansBuilder::new(3);
    builder.add_span((..),1);
    spans.edit((0..3),builder.build());

    let interval : text::rope::Interval = (0..2).into();
    let mut builder = text::rope::SpansBuilder::new(2);
    builder.add_span((..),2);
    spans.edit((6..8),builder.build());

    println!("{:?}",spans.subseq(0..100).iter().collect_vec());
}



//#[derive(Debug,Clone,Copy,PartialEq,Eq,PartialOrd,Ord,Hash)]
//pub struct BufferId(pub usize);
//
//pub struct BufferMap {
//    map : BTreeMap<BufferId,Buffer>
//}
//




















