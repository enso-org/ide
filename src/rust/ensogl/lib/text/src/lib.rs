//! Ensogl text rendering implementation.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

pub mod glyph;

/// Commonly used types and functions.
pub mod prelude {
    pub use ensogl::prelude::*;
}

pub use ensogl::display;

//use prelude::*;
//
//
//use xi_rope::Rope;
//use xi_rope::LinesMetric;
//use xi_rope::rope::BaseMetric;
//use xi_rope::tree::*;
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


/// Test.
pub fn main() {
//    let mut text = Rope::from("hello\nworld\n!!!\nyo");
//    let mut cursor = Cursor::new(&text, 0);
//
//    while cursor.pos() < text.len() - 2 {
//        cursor.next::<BaseMetric>();
//
//        println!("{:?}",cursor.pos());
//    }
//    a.edit(5..6, "!");
//    for i in 0..1000000 {
//        let l = a.len();
//        a.edit(l..l, &(i.to_string() + "\n"));
//    }
//    let l = a.len();
//    for s in a.clone().iter_chunks(1000..3000) {
//        println!("chunk {:?}", s);
//    }
//    a.edit(1000..l, "");
//    //a = a.subrange(0, 1000);
//    println!("{:?}", String::from(a));
}