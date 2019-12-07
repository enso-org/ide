pub mod builder;
pub mod glsl;

use crate::prelude::*;

pub fn main() {
    let mut cfg = builder::ShaderConfig::new();
    let mut sb = builder::ShaderBuilder::new();
    cfg.attributes.insert("foo".to_string(),builder::AttributeQualifier{
        storage: default(),
        prec: default(),
        typ: glsl::Type {
            prim: glsl::PrimType::Float,
            array: None
        }
    });
    sb.compute(&cfg,"--1--","--2--");
    let s  = sb.get();
    println!("{}",s.vertex);
    println!("{}",s.fragment);
}
