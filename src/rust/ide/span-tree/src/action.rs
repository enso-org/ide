use crate::prelude::*;

use crate::Node;

use ast::Ast;
use enumset::EnumSetType;
use enumset::EnumSet;

#[derive(EnumSetType,Debug)]
pub enum Action { Insert, Erase, Set }

pub type Actions = EnumSet<Action>;

struct SetChild<'a> {
    node        : &'a Node,
    child_index : usize,
}

impl SetChild<'_> {

}

struct InsertChild<'a> {
    node     : &'a Node,
    at_index : usize,
}

struct EraseChild<'a> {
    node        : &'a Node,
    child_index : usize,
}

trait SpanTreeActions {
    fn set_child(&self, index:usize) -> Option<SetChild>;
    fn insert_child(&self, at_index:usize) -> Option<InsertChild>;
    fn erase_child(&self, index:usize) -> Option<EraseChild>;
}