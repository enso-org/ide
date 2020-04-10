//! Module with alias analysis — allows telling what identifiers are used and introduced by each
//! node in the graph.

use crate::prelude::*;

use crate::double_representation::node::NodeInfo;

use ast::crumbs::{InfixCrumb, Located};
use ast::crumbs::Crumb;
use crate::double_representation::definition::DefinitionInfo;

#[cfg(test)]
pub mod test_utils;

/// Identifier with its ast crumb location (relative to the node's ast).
pub type LocatedIdentifier = ast::crumbs::Located<NormalizedName>;



// ======================
// === NormalizedName ===
// ======================

/// The identifier name normalized to a lower-case (as the comparisons are case-insensitive).
/// Implements case-insensitive compare with AST.
#[derive(Clone,Debug,Display,Hash,PartialEq,Eq)]
pub struct NormalizedName(String);

impl NormalizedName {
    /// Wraps given string into the normalized name.
    pub fn new(name:impl Str) -> NormalizedName {
        let name = name.as_ref().to_lowercase();
        NormalizedName(name)
    }

    /// If the given AST is an identifier, returns its normalized name.
    pub fn try_from_ast(ast:&Ast) -> Option<NormalizedName> {
        ast::identifier::name(ast).map(NormalizedName::new)
    }
}

/// Tests if Ast is identifier that might reference the same name (case insensitive match).
impl PartialEq<Ast> for NormalizedName {
    fn eq(&self, other:&Ast) -> bool {
        NormalizedName::try_from_ast(other).contains_if(|other_name| {
            other_name == self
        })
    }
}



// =======================
// === IdentifierUsage ===
// =======================

/// Description of how some node is interacting with the graph's scope.
#[derive(Clone,Debug,Default)]
pub struct IdentifierUsage {
    /// Identifiers from the graph's scope that node is using.
    pub introduced : Vec<LocatedIdentifier>,
    /// Identifiers that node introduces into the parent scope.
    pub used       : Vec<LocatedIdentifier>,
}



// ================
// === Analysis ===
// ================

#[derive(Clone,Copy,Debug,Display,PartialEq)]
enum NameKind { Used, Introduced }

#[derive(Clone,Copy,Debug,Display,PartialEq)]
enum Context { NonPattern, Pattern }

#[derive(Clone,Debug,Default)]
struct Scope {
    symbols : IdentifierUsage
}

#[derive(Clone,Debug)]
struct AliasAnalyzer {
    run_focus : Option<NameKind>,
    scopes    : Vec<Scope>,
    context   : Vec<Context>,
    location  : Vec<ast::crumbs::Crumb>,
}

impl AliasAnalyzer {
    fn new() -> AliasAnalyzer {
        AliasAnalyzer {
            run_focus : default(),
            scopes    : vec![default()],
            context   : vec![Context::NonPattern],
            location  : default(),
        }
    }

    fn with_items_added<T,Cs,R,F>
    ( &mut self
    , vec   : impl Fn(&mut Self) -> &mut Vec<T>
    , items : Cs
    , f     : F) -> R
    where
      Cs : IntoIterator<Item:Into<T>>,
      F  : FnOnce(&mut Self) -> R {
        let original_count = vec(self).len();
        vec(self).extend(items.into_iter().map(|item| item.into()));
        let ret = f(self);
        vec(self).truncate(original_count);
        ret
    }

    fn in_new_scope(&mut self, f:impl FnOnce(&mut AliasAnalyzer)) {
        let scope = Scope::default();
        self.with_items_added(|this| &mut this.scopes, std::iter::once(scope), f);
    }

    fn in_new_context(&mut self, context:Context, f:impl FnOnce(&mut AliasAnalyzer)) {
        self.with_items_added(|this| &mut this.context, std::iter::once(context), f);
    }

    fn in_new_location<Cs,F,R>(&mut self, crumbs:Cs, f:F) -> R
    where Cs : IntoIterator<Item:Into<Crumb>>,
           F : FnOnce(&mut AliasAnalyzer) -> R {
        self.with_items_added(|this| &mut this.location, crumbs, f)
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        // TODO explain why absolutely safe and should totally not break tests
        self.scopes.last_mut().unwrap()
    }

    fn add_identifier(&mut self, kind:NameKind, identifier:NormalizedName) {
        let matching_focus = self.run_focus.contains(&kind);
        let identifier     = LocatedIdentifier::new(self.location.clone(), identifier);
        let scope_index     = self.scopes.len()-1;


        let symbols        = &mut self.current_scope_mut().symbols;
        if matching_focus {
            let target = match kind {
                NameKind::Used => &mut symbols.used,
                NameKind::Introduced => &mut symbols.introduced,
            };
            println!("Name {} is {} in scope @{}",identifier.item.0,kind,scope_index);
            target.push(identifier)
        }

        /*
        if self.run_focus.contains(&kind) == false {
            return
        }
        let identifier     = LocatedIdentifier::new(self.location.clone(), identifier);
        let scope_index     = self.scopes.len()-1;
        assert!(self.scopes.len() > 0);

        match kind {
            NameKind::Introduced => {
                println!("Name {} is introduced into scope @{}",identifier.item.0,kind,scope_index);
                self.current_scope_mut().symbols.introduced.push(identifier);
            }
            NameKind::Used => {
                for scope in self.scopes.iter_mut().rev() {
                    if scope.intro
                }
            }
        }



        let symbols        = &mut self.current_scope_mut().symbols;
            let target = match kind {
                NameKind::Used => &mut symbols.used,
                NameKind::Introduced => &mut symbols.introduced,
            };
            println!("Name {} is {} in scope @{}",identifier.item.0,kind,scope_index);
            target.push(identifier)
        */
    }


    fn in_context(&self, context:Context) -> bool {
        self.context.last().contains(&&context)
    }

    fn is_in_pattern(&self) -> bool {
        self.in_context(Context::Pattern)
    }

    fn try_adding_name(&mut self, kind:NameKind, ast:&Ast) -> bool {
        if let Some(name) = NormalizedName::try_from_ast(ast) {
            self.add_identifier(kind,name);
            true
        } else {
            false
        }
    }
    fn try_adding_located<T>(&mut self, kind:NameKind, located:&Located<T>) -> bool
    where for<'a> &'a T : Into<&'a Ast> {
        let ast = (&located.item).into();
        self.in_location_of(located, |this| this.try_adding_name(kind,ast))
    }

    fn process_ast(&mut self, ast:&Ast) {
        println!("Processing `{}` in context {}",ast.repr(),self.context.last().unwrap());
        if let Some(assignment) = ast::opr::to_assignment(ast) {
            self.process_assignment(&assignment);
        } else if self.is_in_pattern() {
            // We are in the assignment's pattern. three options:
            // 1) This is a destructuring pattern match with prefix syntax, like `Point x y`.
            // 3) As above but with operator and infix syntax, like `head,tail`.
            // 2) This is a nullary symbol binding, like `foo`.
            // (the possibility of definition has been already excluded)
            if let Some(prefix_chain) = ast::prefix::Chain::try_new(ast) {
                println!("Pattern of infix chain of {}",ast.repr());
                // Arguments introduce names, we ignore function name.
                for Located{crumbs,item} in prefix_chain.enumerate_args() {
                    println!("Argument: crumb {:?} contents {}", crumbs,item.repr());
                    self.in_new_location(crumbs, |this| this.process_ast(&item))
                }
            } else if let Some(infix_chain) = ast::opr::Chain::try_new(ast) {
                for operand in infix_chain.enumerate_operands() {
                    self.in_location_of(operand, |this| this.process_ast(&operand.item))
                }
                for operator in infix_chain.enumerate_operators() {
                    // Operators in infix positions are treated as constructors, i.e. they are used.
                    self.try_adding_located(NameKind::Used,operator);
                }
            } else {
                self.try_adding_name(NameKind::Introduced,ast);
            }
        } else if self.in_context(Context::NonPattern) {
            if let Ok(block) = ast::known::Block::try_from(ast) {
                self.in_new_scope(|this| {
                    for (crumb,ast) in ast.enumerate() {
                        this.in_location(crumb, |this| this.process_ast(ast))
                    }
                })
            } else if self.try_adding_name(NameKind::Used,ast) {
                // Plain identifier: just add and do nothing.
            } else {
                for (crumb,ast) in ast.enumerate() {
                    self.in_location(crumb, |this| this.process_ast(ast))
                }
            }
        }
    }

    fn in_location<F,R>(&mut self, crumb:impl Into<Crumb>, f:F) -> R
    where F:FnOnce(&mut Self) -> R {
        self.in_new_location(std::iter::once(crumb),f)
    }

    fn in_location_of<T,F,R>(&mut self, located_item:&Located<T>, f:F) -> R
    where F:FnOnce(&mut Self) -> R {
        self.in_new_location(located_item.crumbs.iter().cloned(), f)
    }

    fn process_assignment(&mut self, assignment:&ast::known::Infix) {
        self.in_location(InfixCrumb::LeftOperand, |this|
            this.in_new_context(Context::Pattern, |this|
                this.process_ast(&assignment.larg)
            )
        );
        self.in_location(InfixCrumb::RightOperand, |this|
            this.process_ast(&assignment.rarg)
        );
    }

    fn enter_node(&mut self, node:&NodeInfo) {
        self.process_ast(node.ast())
    }
}

/// Describes identifiers that nodes introduces into the graph and identifiers from graph's scope
/// that node uses. This logic serves as a base for connection discovery.
pub fn analyse_identifier_usage(node:&NodeInfo) -> IdentifierUsage {
    println!("\n===============================================================================\n");
    println!("Case: {}",node.ast().repr());
    let mut analyzer = AliasAnalyzer::new();
    analyzer.run_focus = Some(NameKind::Introduced);
    analyzer.enter_node(node);
    analyzer.run_focus = Some(NameKind::Used);
    analyzer.enter_node(node);
    analyzer.scopes.last().unwrap().symbols.clone() // TODO mvoe out
}

///// Hardcoded proper result for `sum = a + b`.
///// TODO [mwu] remove when real implementation is present
//fn analyse_identifier_usage_mock(_:&NodeInfo) -> IdentifierUsage {
//    use ast::crumbs::InfixCrumb::LeftOperand;
//    use ast::crumbs::InfixCrumb::RightOperand;
//    let sum        = NormalizedName::new("sum");
//    let a          = NormalizedName::new("a");
//    let b          = NormalizedName::new("b");
//    let introduced = vec![LocatedIdentifier::new(&[LeftOperand], sum)];
//    let used       = vec![
//        LocatedIdentifier::new(&[RightOperand, LeftOperand],  a),
//        LocatedIdentifier::new(&[RightOperand, RightOperand], b),
//    ];
//    IdentifierUsage {introduced,used}
//}
//


// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*;

//    use wasm_bindgen_test::wasm_bindgen_test;
//    use wasm_bindgen_test::wasm_bindgen_test_configure;
//
//    wasm_bindgen_test_configure!(run_in_browser);

    /// Checks if actual observed sequence of located identifiers matches the expected one.
    /// Expected identifiers are described as code spans in the node's text representation.
    fn validate_identifiers
    (name:impl Str, node:&NodeInfo, expected:Vec<Range<usize>>, actual:&Vec<LocatedIdentifier>) {
        let mut checker = IdentifierValidator::new(name,node,expected);
        checker.validate_identifiers(actual);
    }

    /// Runs the test for the given test case description.
    fn run_case(parser:&parser::Parser, case:Case) {
        let ast    = parser.parse_line(&case.code).unwrap();
        let node   = NodeInfo::from_line_ast(&ast).unwrap();
        let result = analyse_identifier_usage(&node);
        println!("Analysis results: {:?}", result);
        validate_identifiers("introduced",&node, case.expected_introduced, &result.introduced);
        validate_identifiers("used",      &node, case.expected_used,       &result.used);
    }

    /// Runs the test for the test case expressed using markdown notation. See `Case` for details.
    fn run_markdown_case(parser:&parser::Parser, marked_code:impl Str) {
        println!("Running test case for {}", marked_code.as_ref());
        let case = Case::from_markdown(marked_code);
        run_case(parser,case)
    }


    #[test]
    fn test_alias_analysis() {
        let parser = parser::Parser::new_or_panic();

        // Removed cases
//            "«foo» a b = a »+« b",  // this we don't care, because this is not a node
//            "«log_name» object = »print« object.»name«",
//            "«^» a n = a * a ^ (n - 1)",

        let test_cases = vec![
            "»foo«",
            "«five» = 5",
            "«foo» = »bar«",
            "«sum» = »a« »+« »b«",
            "Point «x» «u» = »point«",
            "«x» »,« «y» = »pair«",

            r"«inc» =
                »foo« »+« 1",

            r"«inc» =
                foo = 2
                foo »+« 1",

//            "a.«hello» = »print« 'Hello'",
//            "«log_name» = object -> »print« object.»name«",
//            "«log_name» = object -> »print« $ »name« object",
        ];
        for case in test_cases {
            run_markdown_case(&parser,case)
        }


//        let code   = "«sum» = »a« + »b«";
//        run_markdown_case(&parser, code);
    }
}
