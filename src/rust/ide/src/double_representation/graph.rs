//! Code for retrieving graph description from AST.

use crate::prelude::*;

use crate::double_representation::definition;
use crate::double_representation::definition::DefinitionInfo;
//use crate::double_representation::definition::DefinitionName;
//use crate::double_representation::definition::DefinitionProvider;
use crate::double_representation::node::NodeInfo;

use ast::Ast;
use ast::BlockLine;
use ast::known;
use utils::fail::FallibleResult;

pub type Id = double_representation::definition::Id;



// =============
// === Error ===
// =============

#[derive(Fail,Debug)]
#[fail(display="ID was not found.")]
struct IdNotFound {id:ast::Id}



// ====================
// === LocationHint ===
// ====================

/// Describes the desired position of the node's line in the graph's code block.
#[derive(Clone,Copy,Debug)]
pub enum LocationHint {
    /// Try placing this node's line before the line described by id.
    Before(ast::Id),
    /// Try placing this node's line after the line described by id.
    After(ast::Id),
    /// Try placing this node's line at the start of the graph's code block.
    Start,
    /// Try placing this node's line at the end of the graph's code block.
    End,
}



// =================
// === GraphInfo ===
// =================

/// Description of the graph, based on information available in AST.
#[derive(Clone,Debug)]
pub struct GraphInfo {
    pub source:DefinitionInfo,
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

    /// Gets the AST of this graph definition.
    pub fn ast(&self) -> Ast {
        self.source.ast.clone().into()
    }

    /// Gets all known nodes in this graph (does not include special pseudo-nodes like graph
    /// inputs and outputs).
    pub fn nodes(&self) -> Vec<NodeInfo> {
        Self::from_function_binding(self.source.ast.clone())
    }

    fn is_node_by_id(line:&BlockLine<Option<Ast>>, id:ast::Id) -> bool {
        let node_info  = line.elem.as_ref().and_then(NodeInfo::from_line_ast);
        let id_matches = node_info.map(|node| node.id() == id);
        id_matches.unwrap_or(false)
    }

    /// Searches for `NodeInfo` with the associated `id` index in `lines`. Returns an error if
    /// the Id is not found.
    pub fn find_node_index_in_lines
    (lines:&[BlockLine<Option<Ast>>], id:ast::Id) -> FallibleResult<usize> {
        let position = lines.iter().position(|line| Self::is_node_by_id(&line,id));
        position.ok_or_else(|| IdNotFound{id}.into())
    }

    /// Adds a new node to this graph.
    pub fn add_node
    (&mut self, line_ast:Ast, location_hint:LocationHint) -> FallibleResult<()> {
        let mut lines = self.source.block_lines()?;

        let index = match location_hint {
            LocationHint::Start      => 0,
            LocationHint::End        => lines.len(),
            LocationHint::After(id)  => Self::find_node_index_in_lines(&lines, id)? + 1,
            LocationHint::Before(id) => Self::find_node_index_in_lines(&lines, id)?
        };

        let elem = Some(line_ast);
        let off  = 0;
        lines.insert(index,BlockLine{elem,off});

        self.source.set_block_lines(lines)?;
        Ok(())
    }

    /// Removes the node from graph.
    pub fn remove_node(&mut self, _node_id:ast::Id) -> FallibleResult<()> {
        todo!()
    }

    /// Sets expression of the given node.
    pub fn edit_node(&self, _node_id:ast::Id, _new_expression:impl Str) -> FallibleResult<()> {
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
    use crate::double_representation::definition::traverse_for_definition;

    use ast::HasRepr;
    use parser::api::IsParser;
    use wasm_bindgen_test::wasm_bindgen_test;
    use ast::test_utils::expect_single_line;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    /// Takes a program with main definition in root and returns main's graph.
    fn main_graph(parser:&mut impl IsParser, program:impl Str) -> GraphInfo {
        let module = parser.parse_module(program.into(), default()).unwrap();
        let name   = DefinitionName::new_plain("main");
        let main   = module.def_iter().find_definition(&name).unwrap();
        GraphInfo::from_definition(main.item)
    }

    fn find_graph(parser:&mut impl IsParser, program:impl Str, name:impl Str) -> GraphInfo {
        let module     = parser.parse_module(program.into(), default()).unwrap();
        let crumbs     = name.into().split(".").map(|name| {
            DefinitionName::new_plain(name)
        }).collect();
        let id         = Id{crumbs};
        let definition = traverse_for_definition(&module,&id).unwrap();
        GraphInfo::from_definition(definition)
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

    fn create_node_ast(parser:&mut impl IsParser, expression:&str) -> (Ast,ast::Id) {
        let node_ast = parser.parse(expression.to_string(), default()).unwrap();
        let line_ast = expect_single_line(&node_ast).clone();
        let id       = line_ast.id.expect("line_ast should have an ID");
        (line_ast,id)
    }

    #[wasm_bindgen_test]
    fn add_node_to_graph_with_single_line() {
        let program = "main = print \"hello\"";
        let mut parser = parser::Parser::new_or_panic();
        let mut graph = main_graph(&mut parser, program);

        let nodes = graph.nodes();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].expression().repr(), "print \"hello\"");

        let expr0 = "a + 2";
        let expr1 = "b + 3";
        let (line_ast0,id0) = create_node_ast(&mut parser, expr0);
        let (line_ast1,id1) = create_node_ast(&mut parser, expr1);

        assert!(graph.add_node(line_ast0, LocationHint::Start).is_ok());
        assert_eq!(graph.nodes().len(), 2);
        assert!(graph.add_node(line_ast1, LocationHint::Before(graph.nodes()[0].id())).is_ok());

        let nodes = graph.nodes();
        assert_eq!(nodes.len(), 3);
        assert_eq!(nodes[0].expression().repr(), expr1);
        assert_eq!(nodes[0].id(), id1);
        assert_eq!(nodes[1].expression().repr(), expr0);
        assert_eq!(nodes[1].id(), id0);
        assert_eq!(nodes[2].expression().repr(), "print \"hello\"");
    }

    #[wasm_bindgen_test]
    fn add_node_to_graph_with_multiple_lines() {
        // TODO [dg] Also add test for binding node when it's possible to update its id.
        let program = r#"
main =

    foo = node

    foo a = not_node

    print "hello"

"#;
        let mut parser = parser::Parser::new_or_panic();
        let mut graph = main_graph(&mut parser, program);

        let (line_ast0,id0) = create_node_ast(&mut parser, "4 + 4");
        let (line_ast1,id1) = create_node_ast(&mut parser, "a + b");
        let (line_ast2,id2) = create_node_ast(&mut parser, "x * x");
        let (line_ast3,id3) = create_node_ast(&mut parser, "x / x");
        let (line_ast4,id4) = create_node_ast(&mut parser, "2 - 2");

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
        assert_eq!(nodes[4].expression().repr(), "print \"hello\"");
        assert_eq!(nodes[5].expression().repr(), "x / x");
        assert_eq!(nodes[5].id(), id3);

        let mut graph = find_graph(&mut parser, program, "main.foo");

        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.add_node(line_ast4, LocationHint::Start).is_ok());
        assert_eq!(graph.nodes().len(), 2);
        assert_eq!(graph.nodes()[0].expression().repr(), "2 - 2");
        assert_eq!(graph.nodes()[0].id(), id4);
        assert_eq!(graph.nodes()[1].expression().repr(), "not_node");
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
