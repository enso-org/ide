//! A module containing all actions provided by SpanTree.
//!
//! The actions are in WIP state - they will be implemented along connection operations.
use crate::prelude::*;

use crate::node;

use ast::Ast;
use ast::crumbs::*;
use crate::node::Kind;
use ast::assoc::Assoc;


/// ==============
/// === Errors ===
/// ==============

/// Error returned when tried to perform an action which is not available for specific SpanTree
/// node.
#[derive(Copy,Clone,Debug,Fail)]
#[fail(display="Action {:?} not available for this SpanTree node.",operation)]
pub struct ActionNotAvailable {
    operation : Action
}

/// Error returned when tried to do action but SpanTree does not seem to be generated from AST
/// passed as root.
#[derive(Copy,Clone,Debug,Fail)]
#[fail(display="Cannot apply action: ast structure does not match ApanTree.")]
pub struct AstSpanTreeMismatch;



/// =====================
/// === Actions Trait ===
/// =====================

/// Action enum used mainly for error messages.
#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
#[allow(missing_docs)]
pub enum Action{Set,InsertBefore,Erase}

/// A trait implementing SpanTree actions. Mean to be implemented on some SpanTree node
/// representation.
///
/// All actions take root AST which should be AST on which SpanTree was generated, and returns
/// processed AST.
pub trait Actions {
    /// Check if given action may be performed on this node.
    fn is_action_available(&self, action:Action) -> bool;

    /// Set the node's span to new AST.
    fn set(&self, root:&Ast, to:Ast) -> FallibleResult<Ast>;

    /// Insert a new element of operator or application chain before the element pointed by this
    /// node.
    fn insert_before(&self, root:&Ast, new:Ast) -> FallibleResult<Ast>;

    /// Erase element pointed by this node from operator or application chain.
    fn erase(&self, root:&Ast) -> FallibleResult<Ast>;
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
        action(root,new)
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

/// Implementation of actions - this is for keeping in one place checking of actions availability
/// and the performing the action.
#[allow(missing_docs)]
pub trait Implementation {
    fn set_impl<'a>(&'a self)           -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
    fn insert_before_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
    fn erase_impl<'a>(&'a self)         -> Option<Box<dyn FnOnce(&Ast)     -> FallibleResult<Ast> + 'a>>;
}

impl<'x> Implementation for node::Ref<'x> {
    fn set_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast> + 'a>> {
        match &self.node.kind {
            Kind::Append  => None,
            Kind::Missing => Some(Box::new(move |root,new| {
                let ast = root.get_traversing(self.ast_crumbs.iter().cloned())?.clone_ref();
                let new_ast = match (ast.shape().clone(),self.crumbs.last()) {
                    (ast::Shape::SectionSides(ast::SectionSides{opr}        ),Some(0)) => Ast::new(ast::SectionLeft {opr,off:1,arg:new},None),
                    (ast::Shape::SectionSides(ast::SectionSides{opr}        ),Some(2)) => Ast::new(ast::SectionRight{opr,off:1,arg:new},None),
                    (ast::Shape::SectionRight(ast::SectionRight{opr,off,arg}),Some(0)) => Ast::new(ast::Infix {larg:new,loff:1  ,opr,roff:off,rarg:arg},None),
                    (ast::Shape::SectionLeft (ast::SectionLeft {arg,off,opr}),Some(2)) => Ast::new(ast::Infix {larg:arg,loff:off,opr,roff:1  ,rarg:new},None),
                    _ => return Err(AstSpanTreeMismatch.into()),
                };
                root.set_traversing(&self.ast_crumbs, new_ast)
            })),
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
            (Kind::Append,_) => Some(Box::new(move |root,new| {
                let ast = root.get_traversing(&self.ast_crumbs)?.clone_ref();
                let new_ast = match ast.shape().clone() {
                    ast::Shape::Infix (ast::Infix{..}) => {
                        let mut infix = ast::known::Infix::try_new(ast.clone_ref())?;
                        let opr       = ast::known::Opr::try_new(infix.opr.clone_ref())?;

                        match Assoc::of(&opr.name) {
                            Assoc::Left  => Ast::new(ast::Infix {larg:ast,loff:1,opr:opr.ast().clone_ref(),roff:1,rarg:new},None),
                            Assoc::Right => {
                                infix.update_shape(|s| s.rarg = Ast::new(ast::Infix {larg:s.rarg.clone_ref(),loff:1,opr:opr.ast().clone_ref(),roff:1,rarg:new},None));
                                infix.ast().clone_ref()
                            }
                        }
                    },
                    ast::Shape::SectionRight (ast::SectionRight{..}) => {
                        let mut section = ast::known::SectionRight::try_new(ast.clone_ref())?;
                        let opr         = ast::known::Opr::try_new(section.opr.clone_ref())?;

                        match Assoc::of(&opr.name) {
                            Assoc::Left  => Ast::new(ast::Infix {larg:ast,loff:1,opr:opr.ast().clone_ref(),roff:1,rarg:new},None),
                            Assoc::Right => {
                                section.update_shape(|s| s.arg = Ast::new(ast::Infix {larg:s.arg.clone_ref(),loff:1,opr:opr.ast().clone_ref(),roff:1,rarg:new},None));
                                section.ast().clone_ref()
                            }
                        }
                    },
                    ast::Shape::Prefix(ast::Prefix{..}   ) => Ast::new(ast::Prefix {func:ast,off:1,arg:new},None),
                    _ => return Err(AstSpanTreeMismatch.into()),
                };
                root.set_traversing(&self.ast_crumbs, new_ast)
            })),
            (Kind::Target  ,Some(Crumb::Prefix(PrefixCrumb::Arg))) |
            (Kind::Argument,Some(Crumb::Prefix(PrefixCrumb::Arg))) => Some(Box::new(move |root,new| {
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
                parent.update_shape(|s| s.rarg = Ast::new(ast::Infix {larg:new,loff:1,opr:s.opr.clone_ref(),roff:1,rarg:s.rarg.clone_ref()}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            (Kind::Argument,Some(Crumb::Infix(InfixCrumb::LeftOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                let opr    = parent.opr.clone_ref();
                let new_parent = Ast::new(ast::Infix{larg:new,loff:1,opr,roff:1,rarg:parent.ast().clone_ref()},None);
                root.set_traversing(parent_crumb,new_parent)
            })),
            (Kind::Argument,Some(Crumb::Infix(InfixCrumb::RightOperand))) => Some(Box::new(move |root,new| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let mut parent = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                parent.update_shape(|s| s.larg = Ast::new(ast::Infix {larg:s.larg.clone_ref(),loff:1,opr:s.opr.clone_ref(),roff:1,rarg:new}, None));
                root.set_traversing(parent_crumb, parent.into())
            })),
            _ => None,
        }
    }

    fn erase_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast) -> FallibleResult<Ast> + 'a>> {
        let node_type_erasable = match self.node.kind {
            node::Kind::Argument | node::Kind::Target => true,
            _                                         => false
        };
        match self.ast_crumbs.last() {
            _ if !node_type_erasable => None,
            Some(Crumb::Infix(InfixCrumb::LeftOperand)) => Some(Box::new(move |root| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let parent       = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                root.set_traversing(parent_crumb,parent.rarg.clone_ref())
            })),
            Some(Crumb::Infix(InfixCrumb::RightOperand)) => Some(Box::new(move |root| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let parent       = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                root.set_traversing(parent_crumb,parent.larg.clone_ref())
            })),
            Some(Crumb::Prefix(PrefixCrumb::Arg)) => Some(Box::new(move |root| {
                let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
                let parent       = ast::known::Prefix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
                root.set_traversing(parent_crumb,parent.func.clone_ref())
            })),
            _ => None
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use Action::*;

    use wasm_bindgen_test::wasm_bindgen_test;
    use parser::Parser;
    use ast::HasRepr;

    #[wasm_bindgen_test]
    fn actions_in_span_tree() {
        #[derive(Debug)]
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
                    Set          => node.set(&ast,arg),
                    InsertBefore => node.insert_before(&ast,arg),
                    Erase        => node.erase(&ast),
                }.unwrap();
                let result_repr = result.repr();
                assert_eq!(result_repr,self.expected,"Wrong answer for case {:?}",self);
            }
        }

        let cases:&[Case] = &
            // Setting
            [ Case{expr:"a + b"    , crumbs:&[]   , action:Set         , expected:"foo"            }
            , Case{expr:"a + b"    , crumbs:&[0]  , action:Set         , expected:"foo + b"        }
            , Case{expr:"a + b"    , crumbs:&[2]  , action:Set         , expected:"a + foo"        }
            , Case{expr:"a + b + c", crumbs:&[0,0], action:Set         , expected:"foo + b + c"    }
            , Case{expr:"a + b + c", crumbs:&[0,2], action:Set         , expected:"a + foo + c"    }
            , Case{expr:"a , b , c", crumbs:&[0]  , action:Set         , expected:"foo , b , c"    }
            , Case{expr:"a , b , c", crumbs:&[2,0], action:Set         , expected:"a , foo , c"    }
            , Case{expr:"a , b , c", crumbs:&[2,2], action:Set         , expected:"a , b , foo"    }
            , Case{expr:"f a b"    , crumbs:&[0,0], action:Set         , expected:"foo a b"        }
            , Case{expr:"f a b"    , crumbs:&[0,1], action:Set         , expected:"f foo b"        }
            , Case{expr:"f a b"    , crumbs:&[1]  , action:Set         , expected:"f a foo"        }
            , Case{expr:"+ b"      , crumbs:&[0]  , action:Set         , expected:"foo + b"        }
            , Case{expr:"+ b"      , crumbs:&[2]  , action:Set         , expected:"+ foo"          }
            , Case{expr:"a +"      , crumbs:&[0]  , action:Set         , expected:"foo +"          }
            , Case{expr:"a +"      , crumbs:&[2]  , action:Set         , expected:"a + foo"        }
            , Case{expr:"+"        , crumbs:&[0]  , action:Set         , expected:"foo +"          }
            , Case{expr:"+"        , crumbs:&[2]  , action:Set         , expected:"+ foo"          }
            // Inserting Before
            , Case{expr:"a + b"    , crumbs:&[0]  , action:InsertBefore, expected:"foo + a + b"    }
            , Case{expr:"a + b"    , crumbs:&[2]  , action:InsertBefore, expected:"a + foo + b"    }
            , Case{expr:"a + b"    , crumbs:&[3]  , action:InsertBefore, expected:"a + b + foo"    }
            , Case{expr:"+ b"      , crumbs:&[3]  , action:InsertBefore, expected:"+ b + foo"      }
            , Case{expr:"a + b + c", crumbs:&[0,0], action:InsertBefore, expected:"foo + a + b + c"}
            , Case{expr:"a + b + c", crumbs:&[2]  , action:InsertBefore, expected:"a + b + foo + c"}
            , Case{expr:"a , b , c", crumbs:&[0]  , action:InsertBefore, expected:"foo , a , b , c"}
            , Case{expr:"a , b , c", crumbs:&[2,0], action:InsertBefore, expected:"a , foo , b , c"}
            , Case{expr:"a , b , c", crumbs:&[2,2], action:InsertBefore, expected:"a , b , foo , c"}
            , Case{expr:"a , b , c", crumbs:&[2,3], action:InsertBefore, expected:"a , b , c , foo"}
            , Case{expr:", b"      , crumbs:&[3]  , action:InsertBefore, expected:", b , foo"      }
            , Case{expr:"f a b"    , crumbs:&[0,1], action:InsertBefore, expected:"f foo a b"      }
            , Case{expr:"f a b"    , crumbs:&[1]  , action:InsertBefore, expected:"f a foo b"      }
            , Case{expr:"f a b"    , crumbs:&[2]  , action:InsertBefore, expected:"f a b foo"      }
            // Erasing
            , Case{expr:"a + b + c", crumbs:&[0,0], action:Erase       , expected:"b + c"          }
            , Case{expr:"a + b + c", crumbs:&[0,2], action:Erase       , expected:"a + c"          }
            , Case{expr:"a + b + c", crumbs:&[2]  , action:Erase       , expected:"a + b"          }
            , Case{expr:"a , b , c", crumbs:&[0]  , action:Erase       , expected:"b , c"          }
            , Case{expr:"a , b , c", crumbs:&[2,0], action:Erase       , expected:"a , c"          }
            , Case{expr:"a , b , c", crumbs:&[2,2], action:Erase       , expected:"a , b"          }
            , Case{expr:"f a b"    , crumbs:&[0,1], action:Erase       , expected:"f b"            }
            , Case{expr:"f a b"    , crumbs:&[1]  , action:Erase       , expected:"f a"            }
            ];
        let parser = Parser::new_or_panic();
        for case in cases { case.run(&parser); }
    }

    #[wasm_bindgen_test]
    fn possible_actions_in_span_tree() {
        #[derive(Debug)]
        struct Case {
            expr     : &'static str,
            crumbs   : &'static [usize],
            expected : &'static [Action],
        }

        impl Case {
            fn run(&self, parser:&Parser) {
                let ast    = parser.parse_line(self.expr).unwrap();
                let tree   = ast.generate_tree().unwrap();
                let node   = tree.root_ref().traverse_subnode(self.crumbs.iter().cloned()).unwrap();
                let expected:HashSet<Action> = self.expected.iter().cloned().collect();
                for action in &[Set,InsertBefore,Erase] {
                    assert_eq!(node.is_action_available(*action), expected.contains(action),
                    "Availability mismatch for action {:?} in case {:?}",action,self)
                }
            }
        }
        let cases:&[Case] = &
            [ Case{expr:"abc"      , crumbs:&[]   , expected: &[Set]                    }
            , Case{expr:"a + b"    , crumbs:&[]   , expected: &[Set]                    }
            , Case{expr:"a + b"    , crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a + b"    , crumbs:&[1]  , expected: &[]                       }
            , Case{expr:"a + b"    , crumbs:&[2]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a + b"    , crumbs:&[3]  , expected: &[InsertBefore]           }
            , Case{expr:"a + b + c", crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a + b + c", crumbs:&[0,0], expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a + b + c", crumbs:&[0,1], expected: &[]                       }
            , Case{expr:"a + b + c", crumbs:&[0,2], expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a , b , c", crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a , b , c", crumbs:&[1]  , expected: &[]                       }
            , Case{expr:"a , b , c", crumbs:&[2]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a , b , c", crumbs:&[2,0], expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a , b , c", crumbs:&[2,1], expected: &[]                       }
            , Case{expr:"a , b , c", crumbs:&[2,2], expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"a , b , c", crumbs:&[2,3], expected: &[InsertBefore]           }
            , Case{expr:"f a b"    , crumbs:&[0,0], expected: &[Set]                    }
            , Case{expr:"f a b"    , crumbs:&[0,1], expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"f a b"    , crumbs:&[1]  , expected: &[Set,InsertBefore,Erase] }
            , Case{expr:"+ b"      , crumbs:&[0]  , expected: &[Set]                    }
            , Case{expr:"+ b"      , crumbs:&[1]  , expected: &[]                       }
            , Case{expr:"+ b"      , crumbs:&[2]  , expected: &[Set,InsertBefore]       }
            , Case{expr:"a +"      , crumbs:&[0]  , expected: &[Set,InsertBefore]       }
            , Case{expr:"a +"      , crumbs:&[1]  , expected: &[]                       }
            , Case{expr:"a +"      , crumbs:&[2]  , expected: &[Set]                    }
            , Case{expr:"+"        , crumbs:&[0]  , expected: &[Set]                    }
            , Case{expr:"+"        , crumbs:&[1]  , expected: &[]                       }
            , Case{expr:"+"        , crumbs:&[2]  , expected: &[Set]                    }
            ];
        let parser = Parser::new_or_panic();
        for case in cases { case.run(&parser); }
    }
}