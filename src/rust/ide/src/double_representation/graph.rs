//! Code for retrieving graph description from AST.

use crate::prelude::*;

use crate::double_representation::definition;
use crate::double_representation::definition::DefinitionInfo;
use crate::double_representation::definition::DefinitionName;
use crate::double_representation::definition::DefinitionProvider;
use crate::double_representation::node::NodeInfo;

use ast::Ast;
use ast::known;
use utils::fail::FallibleResult;
use ast::known::KnownAst;


// =============
// === Error ===
// =============

#[derive(Fail,Display,Debug)]
struct IDNotFound {id:ast::ID}



// ====================
// === LocationHint ===
// ====================

/// Describes the desired position of the node's line in the graph's code block.
#[derive(Clone,Copy,Debug)]
pub enum LocationHint {
    /// Try placing this node's line before the line described by id.
    Before(ast::ID),
    /// Try placing this node's line after the line described by id.
    After(ast::ID),
    /// Try placing this node's line at the start of the graph's code block.
    Start,
    /// Try placing this node's line at the end of the graph's code block.
    End,
}



// ================
// === Graph Id ===
// ================

/// Crumb describes step that needs to be done when going from context (for graph being a module)
/// to the target.
// TODO [mwu]
//  Currently we support only entering named definitions.
pub type Crumb = DefinitionName;

/// Identifies graph in the module.
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Id {
    /// Sequence of traverses from module root up to the identified graph.
    pub crumbs : Vec<Crumb>,
}



// ===============================
// === Finding Graph In Module ===
// ===============================

#[derive(Fail,Clone,Debug)]
#[fail(display="Definition ID was empty")]
struct CannotFindDefinition(Id);

#[derive(Fail,Clone,Debug)]
#[fail(display="Definition ID was empty")]
struct EmptyDefinitionId;

/// Looks up graph in the module.
pub fn traverse_for_definition
(ast:ast::known::Module, id:&Id) -> FallibleResult<DefinitionInfo> {
    let err            = || CannotFindDefinition(id.clone());
    let mut crumb_iter = id.crumbs.iter();
    let first_crumb    = crumb_iter.next().ok_or(EmptyDefinitionId)?;
    let mut definition = ast.find_definition(first_crumb).ok_or_else(err)?;
    for crumb in crumb_iter {
        definition = definition.find_definition(crumb).ok_or_else(err)?;
    }
    Ok(definition)
}



// =================
// === GraphInfo ===
// =================

/// Description of the graph, based on information available in AST.
#[derive(Clone,Debug)]
pub struct GraphInfo {
    source:DefinitionInfo,
}

impl GraphInfo {
    /// Describe graph of the given definition.
    pub fn from_definition(source:DefinitionInfo) -> GraphInfo {
        GraphInfo {source}
    }

    /// Lists nodes in the given binding's ast (infix expression).
    fn from_function_binding(ast:known::Infix) -> Vec<NodeInfo> {
        let body = ast.rarg.clone();
        if let Ok(body_block) = known::Block::try_new(body.clone()) {
            block_nodes(&body_block)
        } else {
            expression_node(body)
        }
    }

    /// Gets all known nodes in this graph (does not include special pseudo-nodes like graph
    /// inputs and outputs).
    pub fn nodes(&self) -> Vec<NodeInfo> {
        Self::from_function_binding(self.source.ast.clone())
    }

    /// Adds a new node to this graph.
    pub fn add_node
    (&mut self, line_ast:Ast, location_hint:LocationHint) -> FallibleResult<()> {
        let block = self.source.ast.rarg.clone();
        let block = known::Block::try_from(block)?;

        let mut lines = block.all_lines();

        let find_position = |id| {
            lines.iter().find_position(|line| {
                line.elem.as_ref().map(|line_ast| {
                    NodeInfo::from_line_ast(line_ast).map(|node| node.id() == id).unwrap_or(false)
                }).unwrap_or(false)
            }).map(|(index,_)| index).ok_or(IDNotFound{id})
        };

        let index = match location_hint {
            LocationHint::Start      => 0,
            LocationHint::End        => lines.len(),
            LocationHint::After(id)  => find_position(id)? + 1,
            LocationHint::Before(id) => find_position(id)?
        };

        lines.insert(index, ast::BlockLine { elem: Some(line_ast), off: 0 });

        let rarg        = Ast::new(block.with_all_lines(lines), None);
        self.source.ast = KnownAst::new(ast::Infix {rarg, ..self.source.ast.deref().clone()}, None);
        Ok(())
    }

    /// Removes the node from graph.
    pub fn remove_node(&mut self, _node_id:ast::ID) -> FallibleResult<()> {
        todo!()
    }

    /// Sets expression of the given node.
    pub fn edit_node(&self, _node_id:ast::ID, _new_expression:impl Str) -> FallibleResult<()> {
        todo!()
    }
}



// =====================
// === Listing nodes ===
// =====================

/// Collects information about nodes in given code `Block`.
pub fn block_nodes(ast:&known::Block) -> Vec<NodeInfo> {
    ast.iter().flat_map(|line_ast| {
        // If this can be a definition, then don't treat it as a node.
        match definition::DefinitionInfo::from_line_ast(line_ast, definition::ScopeKind::NonRoot) {
            None    => NodeInfo::from_line_ast(line_ast),
            Some(_) => None
        }
    }).collect()
}

/// Collects information about nodes in given trivial definition body.
pub fn expression_node(ast:Ast) -> Vec<NodeInfo> {
    NodeInfo::new_expression(ast).into_iter().collect()
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use crate::double_representation::definition::DefinitionName;
    use crate::double_representation::definition::DefinitionProvider;

    use ast::HasRepr;
    use parser::api::IsParser;
    use wasm_bindgen_test::wasm_bindgen_test;
    use ast::test_utils::expect_single_line;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Takes a program with main definition in root and returns main's graph.
    fn main_graph(parser:&mut impl IsParser, program:impl Str) -> GraphInfo {
        let module = parser.parse_module(program.into(), default()).unwrap();
        let name   = DefinitionName::new_plain("main");
        let main   = module.find_definition(&name).unwrap();
        GraphInfo::from_definition(main)
    }

    #[wasm_bindgen_test]
    fn detect_a_node() {
        let mut parser = parser::Parser::new_or_panic();
        // Each of these programs should have a `main` definition with a single `2+2` node.
        let programs = vec![
            "main = 2+2",
            "main = \n    2+2",
            "main = \n    foo = 2+2",
            "main = \n    foo = 2+2\n    bar b = 2+2", // `bar` is a definition, not a node
        ];
        for program in programs {
            let graph = main_graph(&mut parser, program);
            let nodes = graph.nodes();
            assert_eq!(nodes.len(), 1);
            let node = &nodes[0];
            assert_eq!(node.expression().repr(), "2+2");
            let _ = node.id(); // just to make sure it is available
        }
    }

    fn create_graph(parser:&mut impl IsParser) -> GraphInfo {
        let program = r"
main =
    foo = node
    foo a = not_node
    node
";

        main_graph(parser, program)
    }

    fn create_node_ast(parser:&mut impl IsParser, expression:&str) -> (Ast,ast::ID) {
        let id         = ast::ID::new_v4();
        let node_ast   = parser.parse(expression.to_string(), default()).unwrap();
        let line_ast   = expect_single_line(&node_ast).with_id(id);
        (line_ast,id)
    }

    #[wasm_bindgen_test]
    fn add_node() {
        // TODO [dg] Also add test for binding node when it's possible to update its id.
        let mut parser = parser::Parser::new_or_panic();

        let mut graph     = create_graph(&mut parser);
        let (line_ast0,id0) = create_node_ast(&mut parser, "4 + 4");
        let (line_ast1,id1) = create_node_ast(&mut parser, "a + b");
        let (line_ast2,id2) = create_node_ast(&mut parser, "x * x");
        let (line_ast3,id3) = create_node_ast(&mut parser, "x / x");

        assert!(graph.add_node(line_ast0, LocationHint::Start).is_ok());
        assert!(graph.add_node(line_ast1, LocationHint::Before(graph.nodes()[0].id())).is_ok());
        assert!(graph.add_node(line_ast2, LocationHint::After(graph.nodes()[1].id())).is_ok());
        assert!(graph.add_node(line_ast3, LocationHint::End).is_ok());

        let nodes = graph.nodes();
        assert_eq!(nodes.len(), 6);
        assert_eq!(nodes[0].expression().repr(), "a + b");
        assert_eq!(nodes[0].id(), id1);
        assert_eq!(nodes[1].expression().repr(), "4 + 4");
        assert_eq!(nodes[1].id(), id0);
        assert_eq!(nodes[2].expression().repr(), "x * x");
        assert_eq!(nodes[2].id(), id2);
        assert_eq!(nodes[3].expression().repr(), "node");
        assert_eq!(nodes[4].expression().repr(), "node");
        assert_eq!(nodes[5].expression().repr(), "x / x");
        assert_eq!(nodes[5].id(), id3);
    }

    #[wasm_bindgen_test]
    fn multiple_node_graph() {
        let mut parser = parser::Parser::new_or_panic();
        let program = r"
main =
    foo = node
    foo a = not_node
    Int.= a = node
    node
";
        let graph = main_graph(&mut parser, program);
        let nodes = graph.nodes();
        assert_eq!(nodes.len(), 2);
        for node in nodes.iter() {
            assert_eq!(node.expression().repr(), "node");
        }
    }
}
