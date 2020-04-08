//! Module with alias analysis — allows telling what identifiers are used and introduced by each
//! node in the graph.

use crate::prelude::*;

use crate::double_representation::node::NodeInfo;

use ast::crumbs::InfixCrumb;
use ast::crumbs::Crumb;

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

#[derive(Clone,Copy,Debug,Display)]
enum Context {Graph,AssignmentPattern}

#[derive(Clone,Debug)]
struct AliasAnalyzer {
    usage    : IdentifierUsage,
    context  : Vec<Context>,
    location : ast::crumbs::Crumbs,
}

impl AliasAnalyzer {
    fn new() -> AliasAnalyzer {
        AliasAnalyzer {
            usage    : default(),
            context  : vec![Context::Graph],
            location : default(),
        }
    }

    fn use_identifier(&mut self, identifier:NormalizedName) {
        let identifier = LocatedIdentifier::new(self.location.clone(), identifier);
        self.usage.used.push(identifier)
    }
    fn introduce_identifier(&mut self, identifier:NormalizedName) {
        let identifier = LocatedIdentifier::new(self.location.clone(), identifier);
        self.usage.introduced.push(identifier)
    }

    fn process_ast(&mut self, ast:&Ast) {
        if let Some(name) = NormalizedName::try_from_ast(ast) {
            match self.context.last() {
                Some(Context::AssignmentPattern) => self.introduce_identifier(name),
                Some(Context::Graph)             => self.use_identifier(name),
                _                                => todo!()
            }
        } else {
            for (crumb,ast) in ast.enumerate() {
                self.in_location(crumb, |this| this.process_ast(ast))
            }
        }
    }

    fn in_location<F,R>(&mut self, crumb:impl Into<Crumb>, f:F) -> R
    where F:FnOnce(&mut Self) -> R {
        self.location.push(crumb.into());
        let ret = f(self);
        self.location.pop();
        ret
    }

    fn enter_assignment_pattern(&mut self, ast:&Ast) {
        self.in_location(InfixCrumb::LeftOperand, |this| {
            this.context.push(Context::AssignmentPattern);
            this.process_ast(ast);
            this.context.pop();
        });
    }

    fn enter_assignment_body(&mut self, ast:&Ast) {
        self.in_location(InfixCrumb::RightOperand, |this| this.process_ast(ast));
    }

    fn enter_node(&mut self, node:&NodeInfo) {
        let ast = node.ast();
        if let Some(assignment) = ast::opr::to_assignment(ast) {
            self.enter_assignment_pattern(&assignment.larg);
            self.enter_assignment_body(&assignment.rarg);
        } else {
            self.process_ast(ast)
        }
    }
}

/// Describes identifiers that nodes introduces into the graph and identifiers from graph's scope
/// that node uses. This logic serves as a base for connection discovery.
pub fn analyse_identifier_usage(node:&NodeInfo) -> IdentifierUsage {
    let mut analyzer = AliasAnalyzer::new();
    analyzer.enter_node(node);
    analyzer.usage
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

    use wasm_bindgen_test::wasm_bindgen_test;
    use wasm_bindgen_test::wasm_bindgen_test_configure;

    wasm_bindgen_test_configure!(run_in_browser);

    /// Checks if actual observed sequence of located identifiers matches the expected one.
    /// Expected identifiers are described as code spans in the node's text representation.
    fn validate_identifiers
    (node:&NodeInfo, expected:Vec<Range<usize>>, actual:&Vec<LocatedIdentifier>) {
        let mut checker = IdentifierValidator::new(node,expected);
        checker.validate_identifiers(actual);
    }

    /// Runs the test for the given test case description.
    fn run_case(parser:&parser::Parser, case:Case) {
        let ast    = parser.parse_line(&case.code).unwrap();
        let node   = NodeInfo::from_line_ast(&ast).unwrap();
        let result = analyse_identifier_usage(&node);
        println!("Analysis results: {:?}", result);
        validate_identifiers(&node, case.expected_introduced, &result.introduced);
        validate_identifiers(&node, case.expected_used, &result.used);
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

        let test_cases = vec![
            "»foo«",
            "«five» = 5",
            "«foo» = »bar«",
            "«sum» = »a« »+« »b«",
//            "«foo» a b = a + b",
//            "Foo «a» «b» = »bar«",
//            "a.«hello» = »print« 'Hello'",
//            "«log_name» object = »print« object.»name«",
//            "«log_name» = object -> »print« object.»name«",
//            "«log_name» = object -> »print« $ »name« object",
//            "«^» a n = a * a ^ (n - 1)",
        ];
        for case in test_cases {
            run_markdown_case(&parser,case)
        }


//        let code   = "«sum» = »a« + »b«";
//        run_markdown_case(&parser, code);
    }
}
