use crate::prelude::*;

use crate::NodeRef;

use ast::Ast;
use enumset::EnumSetType;
use enumset::EnumSet;

#[derive(EnumSetType,Debug)]
pub enum Action { Insert, Erase, Set }

pub type Actions = EnumSet<Action>;

trait SpanTreeActions {
    fn set(&self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> Ast>>;
    fn insert(&self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> Ast>>;
    fn erase(&self) -> Option<Box<dyn FnOnce(&Ast) -> Ast>>;

    fn allowed_actions(&self) -> Actions {
        let set    = self.set()   .map(|_| Action::Set   ).into_iter();
        let insert = self.insert().map(|_| Action::Insert).into_iter();
        let erase  = self.erase() .map(|_| Action::Erase ).into_iter();
        set.chain(insert).chain(erase).collect()
    }
}

impl<'a> SpanTreeActions for NodeRef<'a> {
    fn set(&self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> Ast>> {
        None
    }

    fn insert(&self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> Ast>> {
        None
    }

    fn erase(&self) -> Option<Box<dyn FnOnce(&Ast) -> Ast>> {
        None
    }
}