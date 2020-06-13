


pub mod glyph;

pub mod prelude {
    pub use ensogl::prelude::*;
}

pub use ensogl::display;

use prelude::*;


use xi_rope::Rope;
use xi_rope::LinesMetric;
use xi_rope::rope::BaseMetric;
use xi_rope::tree::*;






pub struct Line {
    text  : Rope,
    index : usize,
}



pub fn main() {
    let mut text = Rope::from("hello\nworld\n!!!\nyo");
    let mut cursor = Cursor::new(&text, 0);

    while cursor.pos() < text.len() - 2 {
        cursor.next::<BaseMetric>();

        println!("{:?}",cursor.pos());
    }
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