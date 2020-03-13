//! Code for retrieving graph description from AST.

use crate::prelude::*;

use crate::controller::graph::{NewNodeInfo, Position};
use crate::double_representation::definition;
use crate::double_representation::definition::DefinitionInfo;
use crate::double_representation::definition::DefinitionName;
use crate::double_representation::definition::DefinitionProvider;
use crate::double_representation::node::NodeInfo;

use ast::Ast;
use ast::IdMap;
use ast::ID;
use ast::known;
use utils::fail::FallibleResult;
use parser::api::IsParser;
use ast::test_utils::expect_single_line;
use data::text::{Span, Index, Size};


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
#[fail(display = "Definition ID was empty")]
struct CannotFindDefinition(Id);

#[derive(Fail,Clone,Debug)]
#[fail(display = "Definition ID was empty")]
struct EmptyDefinitionId;

/// Looks up graph in the module.
pub fn traverse_for_definition
(ast:ast::known::Module, id:&Id) -> FallibleResult<DefinitionInfo> {
    let err            = || CannotFindDefinition(id.clone());
    let mut crumb_iter = id.crumbs.iter();
    let first_crumb    = crumb_iter.next().ok_or(EmptyDefinitionId)?;
    let mut definition = ast.find_definition(first_crumb).ok_or_else(err)?;
    while let Some(crumb) = crumb_iter.next() {
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
    /// Describes all known nodes in this graph (does not include special pseudo-nodes like graph
    /// inputs and outputs).
    pub nodes:Vec<NodeInfo>,
}

impl GraphInfo {
    /// Describe graph of the given definition.
    pub fn from_definition(source:DefinitionInfo) -> GraphInfo {
        let nodes = Self::from_function_binding(source.ast.clone());
        GraphInfo {source,nodes}
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

    /// Adds a new node to this graph.
    pub fn add_node
    (&mut self, new_node:NewNodeInfo, parser:&mut impl IsParser) -> FallibleResult<ID> {
        let index = new_node.next_node.map(|id| {
            self.nodes.iter().find_position(|node| {
                node.id() == id
            }).map(|(index,_)| index + 1).unwrap_or(self.nodes.len())
        }).unwrap_or(self.nodes.len());

        let error_message = format!("Couldn't parse {}", new_node.expression);

        let id            = new_node.id.unwrap_or(ID::new_v4());
        let ix = Index{value:0};
        let size  = Size{value:new_node.expression.len()};
        let span  = Span::new(ix, size);
        let id_map   = IdMap(vec![(span, id)]);
        let node_ast = parser.parse(new_node.expression, id_map)?;
        let line_ast = expect_single_line(&node_ast);
        println!("{:#?}", line_ast);
        let node          = NodeInfo::from_line_ast(&line_ast);
        let node          = node.ok_or(parser::api::Error::ParsingError(error_message))?;
        assert_eq!(node.id(), id);

        self.nodes.insert(index,node);
        Ok(id)
    }

    /// Removes the node from graph.
    pub fn remove_node(&mut self, _node_id:ID) -> FallibleResult<()> {
        todo!()
    }

    /// Sets the visual position of the given node.
    pub fn move_node(&self, _node_id:ID, _new_position:Position) -> FallibleResult<()> {
        todo!()
    }

    /// Sets expression of the given node.
    pub fn edit_node(&self, _node_id:ID, _new_expression:impl Str) -> FallibleResult<()> {
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

    use parser::api::IsParser;
    use wasm_bindgen_test::wasm_bindgen_test;
    use ast::ID;

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
            assert_eq!(graph.nodes.len(), 1);
            let node = &graph.nodes[0];
            assert_eq!(node.expression_text(), "2+2");
            let _ = node.id(); // just to make sure it is available
        }
    }

    #[wasm_bindgen_test]
    fn add_node() {
        basegl::system::web::set_stdout();
        let mut parser = parser::Parser::new_or_panic();
        let program = r"
main =
    foo = node
    foo a = not_node
    node
";
        let mut graph = main_graph(&mut parser, program);
        let next_node  = Some(graph.nodes[0].id());
        let expression = "4 + 4".to_string();
        let id         = Some(ID::new_v4());
        let location   = default();

        assert!(graph.add_node(NewNodeInfo {expression,id,location,next_node},&mut parser).is_ok());

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.nodes[0].expression_text(), "node");
        assert_eq!(graph.nodes[1].expression_text(), "4 + 4");
        assert_eq!(graph.nodes[2].expression_text(), "node");
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
        // TODO [mwu]
        //  Add case like `Int.= a = node` once https://github.com/luna/enso/issues/565 is fixed

        let graph = main_graph(&mut parser, program);
        assert_eq!(graph.nodes.len(), 2);
        for node in graph.nodes.iter() {
            assert_eq!(node.expression_text(), "node");
        }
    }
}
