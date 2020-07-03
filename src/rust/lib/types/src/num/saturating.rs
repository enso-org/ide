//! Definition of numeric types whose operations are saturating by default.

use crate::algebra::*;

#[derive(Clone,Copy,Debug,Default,Eq,Hash,Ord,PartialEq,PartialOrd)]
pub struct SaturatingUsize {
    pub raw : usize,
}

//crate::impl_T_x_T_to_T!{Pow::pow as saturating_pow for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{Mul::mul as saturating_mul for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{Add::add as saturating_add for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{Sub::sub as saturating_sub for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{Div::div for SaturatingUsize {raw}}

crate::impl_T_x_T_to_T!{SaturatingMul::saturating_mul for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{SaturatingAdd::saturating_add for SaturatingUsize {raw}}
crate::impl_T_x_T_to_T!{SaturatingSub::saturating_sub for SaturatingUsize {raw}}