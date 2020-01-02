//! Root module for primitive shapes and their modifiers.

pub mod def;
pub mod shader;

pub use def::*;


/// Test.
pub fn main() {
    use shader::builder::Builder;

    let s1 = Circle(10.0);
    let s2 = s1.translate(7.0,0.0);
    let s3 = &s2 + &s2;

    println!("{}", Builder::run(&s3));
}


