//! A module containing all actions provided by SpanTree.
//!
//! The actions are in WIP state - they will be implemented along connection operations.
use crate::prelude::*;

use crate::node;

use ast::Ast;
use ast::crumbs::*;
use crate::node::Kind;
use ast::Shape::SectionLeft;


/// ==============
/// === Errors ===
/// ==============

#[derive(Clone,Debug,Fail)]
#[fail(display="Action {:?} not available for this SpanTree node.",operation)]
struct ActionNotAvailable {
    operation : Action
}


/// =====================
/// === Actions Trait ===
/// =====================

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum Action{Set,InsertBefore,Erase}

pub trait Actions {
    fn is_action_available(&self, action:Action) -> bool;

    fn set          (&self, root:&Ast, to:Ast) -> FallibleResult<Ast>;
    fn insert_before(&self, root:&Ast, new:Ast) -> FallibleResult<Ast>;
    fn erase        (&self, root:&Ast) -> FallibleResult<Ast>;
}

impl<T:Implementation> Actions for T {
    fn is_action_available(&self, action:Action) -> bool {
        match action {
            Action::Set          => self.set_impl().is_some(),
            Action::InsertBefore => self.insert_before_impl().is_some(),
            Action::Erase        => self.erase_impl().is_some(),
        }
    }

    fn set(&self, root:&Ast, to:Ast) -> FallibleResult<Ast> {
        let operation = Action::Set;
        let action    = self.set_impl().ok_or(ActionNotAvailable{operation})?;
        Ok(action(root,to))
    }

    fn insert_before(&self, root:&Ast, new:Ast) -> FallibleResult<Ast> {
        let operation = Action::InsertBefore;
        let action    = self.insert_before_impl().ok_or(ActionNotAvailable{operation})?;
        Ok(action(root, new))
    }

    fn erase(&self, root:&Ast) -> FallibleResult<Ast> {
        let operation = Action::Erase;
        let action    = self.insert_before_impl().ok_or(ActionNotAvailable{operation})?;
        Ok(action(root))
    }
}



/// ==============================
/// === Actions Implementation ===
/// ==============================


#[allow(missing_docs)]
trait Implementation {
    fn set_impl(&self)           -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast>>>;
    fn insert_before_impl(&self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast>>>;
    fn erase_impl(&self)         -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast>>>;
}

impl<'a> Implementation for node::Ref<'a> {
    fn set_impl(&self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast>>> {
        match &self.node.kind {
            Kind::Empty => None,
            _ => match &self.ast_crumbs.last() {
                // Operators should be treated in a special way - setting functions in place in
                // a operator should replace Infix with Prefix with two applications.
                // TODO[ao] Maybe some day...
                Some(Crumb::Infix(InfixCrumb::Operator))          |
                Some(Crumb::SectionLeft(SectionLeftCrumb::Opr))   |
                Some(Crumb::SectionRight(SectionRightCrumb::Opr)) |
                Some(Crumb::SectionSides(SectionSidesCrumb))      => None,

                _ => Some(Box::new(|root, new| {
                    root.set_traversing(&self.ast_crumbs,new)
                }))
            }
        }
    }

    fn insert_before_impl(&self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> Ast>> {
        match &self.node.kind {
            Kind::Empty => Some(Box::new(|root,new| {
                let ast = root.get_traversing(&self.ast_crumbs)?;
                let new_ast = match (ast.shape().clone(),self.crumbs.last()) {
                    (ast::Shape::SectionSides {opr}        ,Some(0)) => Ast::new(ast::Shape::SectionRight {opr,off:1,arg:new},None),
                    (ast::Shape::SectionSides {opr}        ,Some(2)) => Ast::new(ast::Shape::SectionLeft  {opr,off:1,arg:new},None),
                    (ast::Shape::SectionLeft  {arg,off,opr},Some(0)) => Ast::new(ast::Shape::Infix {larg:new,loff:1  ,opr,roff:1  ,rarg:ast},None),
                    (ast::Shape::SectionLeft  {arg,off,opr},Some(3)) => Ast::new(ast::Shape::Infix {larg:arg,loff:off,opr,roff:1  ,rarg:new},None),
                    (ast::Shape::SectionRight {opr,off,arg},Some(0)) => Ast::new(ast::Shape::Infix {larg:new,loff:1  ,opr,roff:off,rarg:ast},None),
                    (ast::Shape::SectionRight {opr,off,arg},Some(3)) => Ast::new(ast::Shape::Infix {larg:ast,loff:1  ,opr,roff:1  ,rarg:arg},None),
                    (ast::Shape::Infix        {opr,..}     ,Some(0)) => Ast::new(ast::Shape::Infix {larg:new,loff:1  ,opr,roff:1  ,rarg:ast},None),
                    (ast::Shape::Infix        {opr,..}     ,Some(3)) => Ast::new(ast::Shape::Infix {larg:new,loff:1  ,opr,roff:1  ,rarg:ast},None),
                    (ast::Shape::Prefix       {..}         ,Some(1)) => Ast::new(ast::Shape::Prefix {func:ast,off:1,arg:new},None),
                    _ => panic!("Inconsistent SpanTree structure"),
                };
                root.set_traversing(&self.ast_crumbs, new_ast)
            })),
            Kind::Parameter => match &self.ast_crumbs.last() {
                Some(Crumb::Prefix(PrefixCrumb::Arg)) => Some(Box::new(|root,new| {
                    let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                    let mut parent = ast::known::Prefix::try_new(root.get_traversing(parent)?.clone_ref())?;
                    parent.update_shape(|s| s.)
                    root.set_traversing(parent_crumb, parent.with_shape)
                })),
            }
            _ => None,
        }
    }

    fn erase_impl(&self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> Ast>> {
        None
    }
}

fn insert_infix()