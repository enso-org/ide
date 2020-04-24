//! A module containing all actions provided by SpanTree.
//!
//! The actions are in WIP state - they will be implemented along connection operations.
use crate::prelude::*;

use crate::node;

use ast::Shape::*;
use ast::{Ast, Shifted};
use ast::crumbs::*;
use crate::node::Kind;
use ast::assoc::Assoc;
use ast::opr::{GeneralizedInfix, make_operand, ChainElement, Operand};


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
//
// impl<T:Implementation> Actions for T {
//     fn is_action_available(&self, action:Action) -> bool {
//         match action {
//             Action::Set          => self.set_impl().is_some(),
//             Action::InsertBefore => self.insert_before_impl().is_some(),
//             Action::Erase        => self.erase_impl().is_some(),
//         }
//     }
//
//     fn set(&self, root:&Ast, to:Ast) -> FallibleResult<Ast> {
//         let operation = Action::Set;
//         let action    = self.set_impl().ok_or(ActionNotAvailable{operation})?;
//         action(root,to)
//     }
//
//     fn insert_before(&self, root:&Ast, new:Ast) -> FallibleResult<Ast> {
//         let operation = Action::InsertBefore;
//         let action    = self.insert_before_impl().ok_or(ActionNotAvailable{operation})?;
//         action(root,new)
//     }
//
//     fn erase(&self, root:&Ast) -> FallibleResult<Ast> {
//         let operation = Action::Erase;
//         let action    = self.erase_impl().ok_or(ActionNotAvailable{operation})?;
//         action(root)
//     }
// }
//
//
//
// // ==============================
// // === Actions Implementation ===
// // ==============================
//
// const DEFAULT_OFFSET : usize = 1;
//
// /// Implementation of actions - this is for keeping in one place checking of actions availability
// /// and the performing the action.
// #[allow(missing_docs)]
// pub trait Implementation {
//     fn set_impl<'a>(&'a self)           -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
//     fn insert_before_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast,Ast) -> FallibleResult<Ast> + 'a>>;
//     fn erase_impl<'a>(&'a self)         -> Option<Box<dyn FnOnce(&Ast)     -> FallibleResult<Ast> + 'a>>;
// }
//
// impl<'x> Implementation for node::Ref<'x> {
//     fn set_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast> + 'a>> {
//         match &self.node.kind {
//             Kind::Append  => None,
//             Kind::Missing => Some(Box::new(set_on_missing(self))),
//             _ => match &self.ast_crumbs.last() {
//                 // Operators should be treated in a special way - setting functions in place in
//                 // a operator should replace Infix with Prefix with two applications.
//                 // TODO[ao] Maybe some day...
//                 Some(Crumb::Infix(InfixCrumb::Operator))          |
//                 Some(Crumb::SectionLeft(SectionLeftCrumb::Opr))   |
//                 Some(Crumb::SectionRight(SectionRightCrumb::Opr)) |
//                 Some(Crumb::SectionSides(SectionSidesCrumb))      => None,
//                 _ => Some(Box::new(move |root, new| {
//                     root.set_traversing(self.ast_crumbs.iter().cloned(),new)
//                 }))
//             }
//         }
//     }
//
//     fn insert_before_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast, Ast) -> FallibleResult<Ast> + 'a>> {
//         match &self.node.kind {
//             Kind::Append   |
//             Kind::Target   |
//             Kind::Argument => Some(Box::new(move |root,new| with_changed_parent_ast(root,self,|ast|{
//                 let item = Shifted{wrapped:new,off:DEFAULT_OFFSET};
//                 let mut position = match &self.node.kind {
//                     Kind::Append   =>
//                     Kind::Target   => 0,
//                     Kind::Argument => 1,
//                 }
//                 if let Some(mut chain)  = ast::opr::Chain::try_new(&ast) {
//                     match assoc(chain.operator) {
//                         Assoc::Left  => { chain.push_operand(item) },
//                         Assoc::Right => { chain.push_front_operand(item) },
//                     }
//                     Ok(chain.into_ast())
//                 } else if let Some(mut chain) = ast::prefix::Chain::try_new(&ast) {
//                     chain.args.push(item);
//                     Ok(chain.into_ast())
//                 }
//             })
//
//            )),
//             _              => None,
//         }
//     }
//
//     fn erase_impl<'a>(&'a self) -> Option<Box<dyn FnOnce(&Ast) -> FallibleResult<Ast> + 'a>> {
//         let node_type_erasable = match self.node.kind {
//             node::Kind::Argument | node::Kind::Target => true,
//             _                                         => false
//         };
//         match self.ast_crumbs.last() {
//             _ if !node_type_erasable => None,
//             Some(Crumb::Infix(InfixCrumb::LeftOperand)) => Some(Box::new(move |root| {
//                 let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
//                 let parent       = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
//                 root.set_traversing(parent_crumb,parent.rarg.clone_ref())
//             })),
//             Some(Crumb::Infix(InfixCrumb::RightOperand)) => Some(Box::new(move |root| {
//                 let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
//                 let parent       = ast::known::Infix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
//                 root.set_traversing(parent_crumb,parent.larg.clone_ref())
//             })),
//             Some(Crumb::Prefix(PrefixCrumb::Arg)) => Some(Box::new(move |root| {
//                 let parent_crumb = &self.ast_crumbs[..self.ast_crumbs.len()-1];
//                 let parent       = ast::known::Prefix::try_new(root.get_traversing(parent_crumb)?.clone_ref())?;
//                 root.set_traversing(parent_crumb,parent.func.clone_ref())
//             })),
//             _ => None
//         }
//     }
// }
//
// fn with_changed_parent_ast<F>(root:&Ast, node:&node::Ref, f:F) -> FallibleResult<Ast>
// where F : FnOnce(&Ast) -> FallibleResult<Ast> {
//     let parent_crumbs = match node.node.kind {
//         Kind::Missing | Kind::Argument => &node.ast_crumbs,
//         _                              => &node.ast_crumbs[..node.ast_crumbs.len()-1]
//     };
//     let parent = root.get_traversing(parent_crumbs)?;
//     root.set_traversing(parent_crumbs,f(parent)?)
// }
//
// fn set_on_missing<'a,'b>(node:&'a node::Ref<'b>) -> impl Fn(&Ast,Ast) -> FallibleResult<Ast> + 'a {
//     move |root,new| {
//         // The AST crumbs of missing nodes points to the Section AST node
//         let ast        = root.get_traversing(&node.ast_crumbs)?.clone_ref();
//         let mut infix  = GeneralizedInfix::try_new(&ast).ok_or(AstSpanTreeMismatch)?;
//         let node_index = node.crumbs.last();
//         match node_index {
//             Some(0) => { infix.left  = make_operand(new,DEFAULT_OFFSET); },
//             Some(2) => { infix.right = make_operand(new,DEFAULT_OFFSET); },
//             _       => return Err(AstSpanTreeMismatch.into()),
//         }
//         root.set_traversing(&node.ast_crumbs,infix.into_ast())
//     }
// }
//
// fn append<'a,'b>(node:&'a node::Ref<'b>) -> impl Fn(&Ast,Ast) -> FallibleResult<Ast> + 'a {
//     move |root,new| {
//         let ast  = root.get_traversing(&node.ast_crumbs)?.clone_ref();
//         let item = Shifted{wrapped:new,off:DEFAULT_OFFSET};
//         let new_ast = if let Some(mut chain)  = ast::opr::Chain::try_new(&ast) {
//             match assoc(chain.operator) {
//                 Assoc::Left  => { chain.push_operand(item) },
//                 Assoc::Right => { chain.push_front_operand(item) },
//             }
//             chain.into_ast()
//         } else if let Some(mut chain) = ast::prefix::Chain::try_new(&ast) {
//             chain.args.push(item);
//             chain.into_ast()
//         };
//         root.set_traversing(&node.ast_crumbs, new_ast)
//     }
// }
//
// fn insert_before_target<'a,'b>(node:&'a node::Ref<'b>) -> impl Fn(&Ast,Ast) -> FallibleResult<Ast> + 'a {
//     move |root,new| {
//         let parent_crumb = &node.ast_crumbs[..node.ast_crumbs.len()];
//         let parent       = root.get_traversing(parent_crumb)?;
//         let item         = Shifted{wrapped:new,off:DEFAULT_OFFSET};
//         let new_ast      = if let Some(mut chain) = ast::opr::Chain::try_new(&parent) {
//             match assoc(chain.operator) {
//                 Assoc::Left  => { chain.push_front_operand(item) },
//                 Assoc::Right => { chain.insert_operand(1,item)   },
//             }
//             chain.into_ast()
//         } else if let Some(mut chain) = ast::prefix::Chain::try_new(&ast) {
//             chain.args.insert(0,item);
//             chain.into_ast();
//         };
//         root.set_traversing(parent_crumb,new_ast)
//     }
// }
//
// fn insert_before_argument<'a,'b>(node:&'a node::Ref<'b>) -> impl Fn(&Ast,Ast) -> FallibleResult<Ast> + 'a {
//     move |root,new| {
//         let parent_crumb = &node.ast_crumbs[..node.ast_crumbs.len()];
//         let parent       = root.get_traversing(parent_crumb)?;
//         let item         = Shifted{wrapped:new,off:DEFAULT_OFFSET};
//         let new_ast      = if let Some(mut chain) = ast::opr::Chain::try_new(&parent) {
//             match assoc(chain.operator) {
//                 Assoc::Left  => { chain.insert_operand(chain.args.len()-1,item) },
//                 Assoc::Right => { chain.push_front_operand(item)   },
//             }
//             chain.into_ast()
//         } else if let Some(mut chain) = ast::prefix::Chain::try_new(&ast) {
//             chain.args.insert(0,item);
//             chain.into_ast();
//         };
//         root.set_traversing(parent_crumb,new_ast)
//     }
// }
//
//
//
// // =============
// // === Tests ===
// // =============
//
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     use Action::*;
//
//     use wasm_bindgen_test::wasm_bindgen_test;
//     use parser::Parser;
//     use ast::HasRepr;
//
//     #[wasm_bindgen_test]
//     fn actions_in_span_tree() {
//         #[derive(Debug)]
//         struct Case {
//             expr    : &'static str,
//             crumbs  : &'static [usize],
//             action  : Action,
//             expected: &'static str,
//         }
//
//         impl Case {
//             fn run(&self, parser:&Parser) {
//                 let ast    = parser.parse_line(self.expr).unwrap();
//                 let tree   = ast.generate_tree().unwrap();
//                 let node   = tree.root_ref().traverse_subnode(self.crumbs.iter().cloned()).unwrap();
//                 let arg    = Ast::new(ast::Var {name:"foo".to_string()},None);
//                 let result = match &self.action {
//                     Set          => node.set(&ast,arg),
//                     InsertBefore => node.insert_before(&ast,arg),
//                     Erase        => node.erase(&ast),
//                 }.unwrap();
//                 let result_repr = result.repr();
//                 assert_eq!(result_repr,self.expected,"Wrong answer for case {:?}",self);
//             }
//         }
//
//         let cases:&[Case] = &
//             // Setting
//             [ Case{expr:"a + b"    , crumbs:&[]   , action:Set         , expected:"foo"            }
//             , Case{expr:"a + b"    , crumbs:&[0]  , action:Set         , expected:"foo + b"        }
//             , Case{expr:"a + b"    , crumbs:&[2]  , action:Set         , expected:"a + foo"        }
//             , Case{expr:"a + b + c", crumbs:&[0,0], action:Set         , expected:"foo + b + c"    }
//             , Case{expr:"a + b + c", crumbs:&[0,2], action:Set         , expected:"a + foo + c"    }
//             , Case{expr:"a , b , c", crumbs:&[0]  , action:Set         , expected:"foo , b , c"    }
//             , Case{expr:"a , b , c", crumbs:&[2,0], action:Set         , expected:"a , foo , c"    }
//             , Case{expr:"a , b , c", crumbs:&[2,2], action:Set         , expected:"a , b , foo"    }
//             , Case{expr:"f a b"    , crumbs:&[0,0], action:Set         , expected:"foo a b"        }
//             , Case{expr:"f a b"    , crumbs:&[0,1], action:Set         , expected:"f foo b"        }
//             , Case{expr:"f a b"    , crumbs:&[1]  , action:Set         , expected:"f a foo"        }
//             , Case{expr:"+ b"      , crumbs:&[0]  , action:Set         , expected:"foo + b"        }
//             , Case{expr:"+ b"      , crumbs:&[2]  , action:Set         , expected:"+ foo"          }
//             , Case{expr:"a +"      , crumbs:&[0]  , action:Set         , expected:"foo +"          }
//             , Case{expr:"a +"      , crumbs:&[2]  , action:Set         , expected:"a + foo"        }
//             , Case{expr:"+"        , crumbs:&[0]  , action:Set         , expected:"foo +"          }
//             , Case{expr:"+"        , crumbs:&[2]  , action:Set         , expected:"+ foo"          }
//             // Inserting Before
//             , Case{expr:"a + b"    , crumbs:&[0]  , action:InsertBefore, expected:"foo + a + b"    }
//             , Case{expr:"a + b"    , crumbs:&[2]  , action:InsertBefore, expected:"a + foo + b"    }
//             , Case{expr:"a + b"    , crumbs:&[3]  , action:InsertBefore, expected:"a + b + foo"    }
//             , Case{expr:"+ b"      , crumbs:&[3]  , action:InsertBefore, expected:"+ b + foo"      }
//             , Case{expr:"a + b + c", crumbs:&[0,0], action:InsertBefore, expected:"foo + a + b + c"}
//             , Case{expr:"a + b + c", crumbs:&[2]  , action:InsertBefore, expected:"a + b + foo + c"}
//             , Case{expr:"a , b , c", crumbs:&[0]  , action:InsertBefore, expected:"foo , a , b , c"}
//             , Case{expr:"a , b , c", crumbs:&[2,0], action:InsertBefore, expected:"a , foo , b , c"}
//             , Case{expr:"a , b , c", crumbs:&[2,2], action:InsertBefore, expected:"a , b , foo , c"}
//             , Case{expr:"a , b , c", crumbs:&[2,3], action:InsertBefore, expected:"a , b , c , foo"}
//             , Case{expr:", b"      , crumbs:&[3]  , action:InsertBefore, expected:", b , foo"      }
//             , Case{expr:"f a b"    , crumbs:&[0,1], action:InsertBefore, expected:"f foo a b"      }
//             , Case{expr:"f a b"    , crumbs:&[1]  , action:InsertBefore, expected:"f a foo b"      }
//             , Case{expr:"f a b"    , crumbs:&[2]  , action:InsertBefore, expected:"f a b foo"      }
//             // Erasing
//             , Case{expr:"a + b + c", crumbs:&[0,0], action:Erase       , expected:"b + c"          }
//             , Case{expr:"a + b + c", crumbs:&[0,2], action:Erase       , expected:"a + c"          }
//             , Case{expr:"a + b + c", crumbs:&[2]  , action:Erase       , expected:"a + b"          }
//             , Case{expr:"a , b , c", crumbs:&[0]  , action:Erase       , expected:"b , c"          }
//             , Case{expr:"a , b , c", crumbs:&[2,0], action:Erase       , expected:"a , c"          }
//             , Case{expr:"a , b , c", crumbs:&[2,2], action:Erase       , expected:"a , b"          }
//             , Case{expr:"f a b"    , crumbs:&[0,1], action:Erase       , expected:"f b"            }
//             , Case{expr:"f a b"    , crumbs:&[1]  , action:Erase       , expected:"f a"            }
//             ];
//         let parser = Parser::new_or_panic();
//         for case in cases { case.run(&parser); }
//     }
//
//     #[wasm_bindgen_test]
//     fn possible_actions_in_span_tree() {
//         #[derive(Debug)]
//         struct Case {
//             expr     : &'static str,
//             crumbs   : &'static [usize],
//             expected : &'static [Action],
//         }
//
//         impl Case {
//             fn run(&self, parser:&Parser) {
//                 let ast    = parser.parse_line(self.expr).unwrap();
//                 let tree   = ast.generate_tree().unwrap();
//                 let node   = tree.root_ref().traverse_subnode(self.crumbs.iter().cloned()).unwrap();
//                 let expected:HashSet<Action> = self.expected.iter().cloned().collect();
//                 for action in &[Set,InsertBefore,Erase] {
//                     assert_eq!(node.is_action_available(*action), expected.contains(action),
//                     "Availability mismatch for action {:?} in case {:?}",action,self)
//                 }
//             }
//         }
//         let cases:&[Case] = &
//             [ Case{expr:"abc"      , crumbs:&[]   , expected: &[Set]                    }
//             , Case{expr:"a + b"    , crumbs:&[]   , expected: &[Set]                    }
//             , Case{expr:"a + b"    , crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a + b"    , crumbs:&[1]  , expected: &[]                       }
//             , Case{expr:"a + b"    , crumbs:&[2]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a + b"    , crumbs:&[3]  , expected: &[InsertBefore]           }
//             , Case{expr:"a + b + c", crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a + b + c", crumbs:&[0,0], expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a + b + c", crumbs:&[0,1], expected: &[]                       }
//             , Case{expr:"a + b + c", crumbs:&[0,2], expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a , b , c", crumbs:&[0]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a , b , c", crumbs:&[1]  , expected: &[]                       }
//             , Case{expr:"a , b , c", crumbs:&[2]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a , b , c", crumbs:&[2,0], expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a , b , c", crumbs:&[2,1], expected: &[]                       }
//             , Case{expr:"a , b , c", crumbs:&[2,2], expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"a , b , c", crumbs:&[2,3], expected: &[InsertBefore]           }
//             , Case{expr:"f a b"    , crumbs:&[0,0], expected: &[Set]                    }
//             , Case{expr:"f a b"    , crumbs:&[0,1], expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"f a b"    , crumbs:&[1]  , expected: &[Set,InsertBefore,Erase] }
//             , Case{expr:"+ b"      , crumbs:&[0]  , expected: &[Set]                    }
//             , Case{expr:"+ b"      , crumbs:&[1]  , expected: &[]                       }
//             , Case{expr:"+ b"      , crumbs:&[2]  , expected: &[Set,InsertBefore]       }
//             , Case{expr:"a +"      , crumbs:&[0]  , expected: &[Set,InsertBefore]       }
//             , Case{expr:"a +"      , crumbs:&[1]  , expected: &[]                       }
//             , Case{expr:"a +"      , crumbs:&[2]  , expected: &[Set]                    }
//             , Case{expr:"+"        , crumbs:&[0]  , expected: &[Set]                    }
//             , Case{expr:"+"        , crumbs:&[1]  , expected: &[]                       }
//             , Case{expr:"+"        , crumbs:&[2]  , expected: &[Set]                    }
//             ];
//         let parser = Parser::new_or_panic();
//         for case in cases { case.run(&parser); }
//     }
// }