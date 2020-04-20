//! A module containing all actions provided by SpanTree.
//!
//! The actions are in WIP state - they will be implemented along connection operations.
use crate::prelude::*;

use crate::node;

use ast::Ast;
use ast::crumbs::*;
use crate::node::Kind;



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
        action(root,to)
    }

    fn insert_before(&self, root:&Ast, new:Ast) -> FallibleResult<Ast> {
        let operation = Action::InsertBefore;
        let action    = self.insert_before_impl().ok_or(ActionNotAvailable{operation})?;
        action(root, new)
    }

    fn erase(&self, root:&Ast) -> FallibleResult<Ast> {
        let operation = Action::Erase;
        let action    = self.erase_impl().ok_or(ActionNotAvailable{operation})?;
        action(root)
    }
}



/// ==============================
/// === Actions Implementation ===
/// ==============================


#[allow(missing_docs)]
trait Implementation {
    fn set_impl<'a>(&'a self)           -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
    fn insert_before_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
    fn erase_impl<'a>(&'a self)         -> Option<Box<dyn FnOnce(&Ast)     -> FallibleResult<Ast> + 'a>>;
}

impl<'x> Implementation for node::Ref<'x> {
    fn set_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast> + 'a>> {
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

                _ => Some(Box::new(move |root, new| {
                    root.set_traversing(self.ast_crumbs.iter().cloned(),new)
                }))
            }
        }
    }

    fn insert_before_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast> + 'a>> {
        match (&self.node.kind,self.ast_crumbs.last()) {
            (Kind::Empty,_) => Some(Box::new(move |root,new| {
                let ast = root.get_traversing(self.ast_crumbs.iter().cloned())?.clone_ref();
                let new_ast = match (ast.shape().clone(),self.crumbs.last()) {
                    (ast::Shape::SectionSides(ast::SectionSides{opr}),Some(0)) => Ast::new(ast::SectionRight {opr,off:1,arg:new},None),
                    (ast::Shape::SectionSides(ast::SectionSides{opr}        ),Some(2)) => Ast::new(ast::SectionLeft  {opr,off:1,arg:new},None),
                    (ast::Shape::SectionLeft (ast::SectionLeft {arg,off,opr}),Some(0)) => Ast::new(ast::Infix {larg:new,loff:1  ,opr,roff:1  ,rarg:ast},None),
                    (ast::Shape::SectionLeft (ast::SectionLeft {arg,off,opr}),Some(3)) => Ast::new(ast::Infix {larg:arg,loff:off,opr,roff:1  ,rarg:new},None),
                    (ast::Shape::SectionRight(ast::SectionRight {opr,off,arg}),Some(0)) => Ast::new(ast::Infix {larg:new,loff:1  ,opr,roff:off,rarg:ast},None),
                    (ast::Shape::SectionRight(ast::SectionRight {opr,off,arg}),Some(3)) => Ast::new(ast::Infix {larg:ast,loff:1  ,opr,roff:1  ,rarg:arg},None),
                    (ast::Shape::Infix       (ast::Infix{opr,..}     ),Some(0)) => Ast::new(ast::Infix {larg:new,loff:1  ,opr,roff:1  ,rarg:ast},None),
                    (ast::Shape::Infix       (ast::Infix{opr,..}     ),Some(3)) => Ast::new(ast::Infix {larg:ast,loff:1  ,opr,roff:1  ,rarg:new},None),
                    (ast::Shape::Prefix      (ast::Prefix{..}         ),Some(2)) => Ast::new(ast::Prefix {func:ast,off:1,arg:new},None),
                    _ => panic!("Inconsistent SpanTree structure"),
                };
                root.set_traversing(&self.ast_crumbs, new_ast)
            })),
            (Kind::Target   ,Some(Crumb::Prefix(PrefixCrumb::Arg))) |
            (Kind::Parameter,Some(Crumb::Prefix(PrefixCrumb::Arg))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Prefix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.func = Ast::new(ast::Prefix {func:s.func.clone_ref(), off:1, arg:new}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            (Kind::Target,Some(Crumb::Infix(InfixCrumb::LeftOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.larg = Ast::new(ast::Infix {larg:new,loff:1,opr:s.opr.clone_ref(),roff:1,rarg:s.larg.clone_ref()}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            (Kind::Target,Some(Crumb::Infix(InfixCrumb::RightOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.rarg = Ast::new(ast::Infix {larg:s.rarg.clone_ref(),loff:1,opr:s.opr.clone_ref(),roff:1,rarg:new}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            (Kind::Parameter,Some(Crumb::Infix(InfixCrumb::LeftOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.rarg = Ast::new(ast::Infix {larg:new,loff:1,opr:s.opr.clone_ref(),roff:1,rarg:s.rarg.clone_ref()}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            (Kind::Parameter,Some(Crumb::Infix(InfixCrumb::RightOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.larg = Ast::new(ast::Infix {larg:s.larg.clone_ref(),loff:1,opr:s.opr.clone_ref(),roff:1,rarg:new}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            _ => None,
        }
    }

    fn erase_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast) -> FallibleResult<Ast> + 'a>> {
        None
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use wasm_bindgen_test::wasm_bindgen_test;
    use parser::Parser;
    use ast::HasRepr;

    struct Case {
        expr    : &'static str,
        crumbs  : &'static [usize],
        action  : Action,
        expected: &'static str,
    }

    impl Case {
        fn run(&self, parser:&Parser) {
            let ast    = parser.parse_line(self.expr).unwrap();
            let tree   = ast.generate_tree().unwrap();
            let node   = tree.root_ref().traverse_subnode(self.crumbs.iter().cloned()).unwrap();
            let arg    = Ast::new(ast::Var {name:"foo".to_string()},None);
            let result = match &self.action {
                Action::Set          => node.set(&ast,arg),
                Action::InsertBefore => node.insert_before(&ast,arg),
                Action::Erase        => node.erase(&ast),
            }.unwrap();
            let result_repr = result.repr();
            assert_eq!(result_repr,self.expected);
        }
    }

    #[wasm_bindgen_test]
    fn setting_ast_in_span_tree() {
        use Action::*;
        let cases = &
            // Setting
            [ Case{expr:"a + b"    , crumbs:&[]   , action:Set         , expected:"foo"            }
            , Case{expr:"a + b"    , crumbs:&[0]  , action:Set         , expected:"foo + b"        }
            , Case{expr:"a + b"    , crumbs:&[2]  , action:Set         , expected:"a + foo"        }
            , Case{expr:"a + b + c", crumbs:&[0,0], action:Set         , expected:"foo + b + c"    }
            , Case{expr:"a + b + c", crumbs:&[0,2], action:Set         , expected:"a + foo + c"    }
            , Case{expr:"a , b , c", crumbs:&[1]  , action:Set         , expected:"foo , b , c"    }
            , Case{expr:"a , b , c", crumbs:&[3,0], action:Set         , expected:"a , foo , c"    }
            , Case{expr:"a , b , c", crumbs:&[3,2], action:Set         , expected:"a , b , foo"    }
            , Case{expr:"f a b"    , crumbs:&[0,0], action:Set         , expected:"foo a b"        }
            , Case{expr:"f a b"    , crumbs:&[0,1], action:Set         , expected:"f foo b"        }
            , Case{expr:"f a b"    , crumbs:&[1]  , action:Set         , expected:"f a foo"        }
            // Inserting Before
            , Case{expr:"a + b"    , crumbs:&[0]  , action:InsertBefore, expected:"foo + a + b"    }
            , Case{expr:"a + b"    , crumbs:&[2]  , action:InsertBefore, expected:"a + foo + b"    }
            , Case{expr:"a + b"    , crumbs:&[3]  , action:InsertBefore, expected:"a + b + foo"    }
            , Case{expr:"a + b + c", crumbs:&[0,0], action:InsertBefore, expected:"foo + a + b + c"}
            , Case{expr:"a + b + c", crumbs:&[2]  , action:InsertBefore, expected:"a + b + foo + c"}
            , Case{expr:"a , b , c", crumbs:&[0]  , action:InsertBefore, expected:"foo , a , b , c"}
            , Case{expr:"a , b , c", crumbs:&[1]  , action:InsertBefore, expected:"a , foo , b , c"}
            , Case{expr:"a , b , c", crumbs:&[3,0], action:InsertBefore, expected:"a , b , foo , c"}
            , Case{expr:"a , b , c", crumbs:&[3,2], action:InsertBefore, expected:"a , b , c , foo" }
            , Case{expr:"f a b"    , crumbs:&[0,1], action:InsertBefore, expected:"f foo a b"      }
            , Case{expr:"f a b"    , crumbs:&[1]  , action:InsertBefore, expected:"f a foo b"      }
            , Case{expr:"f a b"    , crumbs:&[2]  , action:InsertBefore, expected:"f a b foo"      }
            ];
        let parser = Parser::new_or_panic();
        for case in cases { case.run(&parser); }
    }
}