//! Code related to connection discovery and operations.

use crate::prelude::*;

use crate::double_representation::node::Id;
use crate::double_representation::node::NodeInfo;

use ast::crumbs::Crumbs;
use crate::double_representation::alias_analysis::AliasAnalyzer;
use crate::double_representation::alias_analysis::NormalizedName;


#[cfg(test)]
pub mod test_utils;

/// Connection source, i.e. the port generating the data / identifier introducer.
pub type Source      = Endpoint;

/// Connection destination, i.e. the port receiving data / identifier user.
pub type Destination = Endpoint;

/// Describes a connection between two endpoints: from `source` to `destination`.
#[derive(Clone,Debug,PartialEq)]
pub struct Connection {
    #[allow(missing_docs)]
    pub source:Source,
    #[allow(missing_docs)]
    pub destination:Destination,
}

/// A connection endpoint.
#[derive(Clone,Debug,PartialEq)]
pub struct Endpoint {
    /// Id of the node where the endpoint is located.
    pub node : Id,
    /// Crumbs to the AST creating this endpoint. These crumbs are relative to the node's AST,
    /// not just expression, if the node is binding, there'll crumb for left/right operand first.
    pub crumbs : Crumbs,
}

/// Lists all the connection in the graph for the given code block.
pub fn list_block(block:&ast::Block<Ast>) -> Vec<Connection> {
    let mut analyzer = AliasAnalyzer::default();
    analyzer.process_subtrees(block);

    let introduced_iter = analyzer.root_scope.symbols.introduced.into_iter();

    let introduced : HashMap<NormalizedName,Endpoint> = introduced_iter.map(|name| {
        let endpoint = block_line_endpoint(block,name.crumbs);
        (name.item,endpoint)
    }).collect();

    let mut ret: Vec<Connection> = Vec::new();
    for name in analyzer.root_scope.symbols.used {
        if let Some(source) = introduced.get(&name).cloned() {
            let destination = block_line_endpoint(block,name.crumbs);
            println!("Connection for name {}", name.item);
            ret.push(Connection {source,destination})
        }
    };
    ret
}

/// Lists all the connection in the single-expression definition body.
pub fn list_expression(_ast:&Ast) -> Vec<Connection> {
    // At this points single-expression graphs do not have any connection.
    // This will change when there will be input/output pseudo-nodes.
    vec![]
}

/// Lists connections in the given body. For now it only makes sense for block shape.
pub fn list(definition_body:&ast::known::Infix) -> Vec<Connection> {
    let body = &definition_body.rarg;
    match body.shape() {
        ast::Shape::Block(block) => list_block(block),
        _                        => list_expression(body),
    }
}

fn block_line_endpoint(block:&ast::Block<Ast>, mut crumbs:Crumbs) -> Endpoint {
    match crumbs.first() {
        Some(ast::crumbs::Crumb::Block(block_crumb)) => {
            let line_ast = block.get(block_crumb).unwrap();
            let node     = NodeInfo::from_line_ast(line_ast).unwrap().id();
            crumbs.pop_front();
            Endpoint {node,crumbs}
        }
        _ => panic!("not implemented"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::Parser;
    use crate::double_representation::definition::DefinitionInfo;
    use crate::double_representation::graph::GraphInfo;
    use ast::crumbs;
    use ast::crumbs::Crumb;
    use ast::crumbs::InfixCrumb;

    struct TestRun {
        definition  : DefinitionInfo,
        graph       : GraphInfo,
        connections : Vec<Connection>
    }

    impl TestRun {
        fn from_definition(definition:DefinitionInfo) -> TestRun {
            let graph   = GraphInfo::from_definition(definition.clone());
            let repr_of = |connection:&Connection| {
                let endpoint = &connection.source;
                let node     = graph.find_node(endpoint.node).unwrap();
                let ast      = node.ast().get_traversing(&endpoint.crumbs).unwrap();
                ast.repr()
            };

            let mut connections = graph.connections();
            connections.sort_by(|lhs,rhs| {
                repr_of(&lhs).cmp(&repr_of(&rhs))
            });

            TestRun {definition,graph,connections}
        }

        fn from_main_def(code:impl Str) -> TestRun {
            let parser = Parser::new_or_panic();
            let module = parser.parse_module(code,default()).unwrap();
            let definition = DefinitionInfo::from_root_line(&module.lines[0]).unwrap();
            Self::from_definition(definition)
        }

        fn from_block(code:impl Str) -> TestRun {
            let body = code.as_ref().lines().map(|line| format!("    {}", line.trim())).join("\n");
            let definition_code = format!("main =\n{}",body);
            Self::from_main_def(definition_code)
        }

        fn endpoint_node_repr(&self, endpoint:&Endpoint) -> String {
            self.graph.find_node(endpoint.node).unwrap().ast().clone().repr()
        }
    }

    #[test]
    pub fn connection_listing_test_plain() {
        use InfixCrumb::LeftOperand;
        use InfixCrumb::RightOperand;

        let code_block = r"
d,e = p
a = d
b = e
c = a + b
fun a = a b
f = fun 2";


        let run = TestRun::from_block(code_block);
        let c = &run.connections[0];
        assert_eq!(run.endpoint_node_repr(&c.source), "a = d");
        assert_eq!(&c.source.crumbs, &crumbs![LeftOperand]);
        assert_eq!(run.endpoint_node_repr(&c.destination), "c = a + b");
        assert_eq!(&c.destination.crumbs, &crumbs![RightOperand,LeftOperand]);

        let c = &run.connections[1];
        assert_eq!(run.endpoint_node_repr(&c.source), "b = e");
        assert_eq!(&c.source.crumbs, &crumbs![LeftOperand]);
        assert_eq!(run.endpoint_node_repr(&c.destination), "c = a + b");
        assert_eq!(&c.destination.crumbs, &crumbs![RightOperand,RightOperand]);

        let c = &run.connections[2];
        assert_eq!(run.endpoint_node_repr(&c.source), "d,e = p");
        assert_eq!(&c.source.crumbs, &crumbs![LeftOperand,LeftOperand]);
        assert_eq!(run.endpoint_node_repr(&c.destination), "a = d");
        assert_eq!(&c.destination.crumbs, &crumbs![RightOperand]);

        let c = &run.connections[3];
        assert_eq!(run.endpoint_node_repr(&c.source), "d,e = p");
        assert_eq!(&c.source.crumbs, &crumbs![LeftOperand,RightOperand]);
        assert_eq!(run.endpoint_node_repr(&c.destination), "b = e");
        assert_eq!(&c.destination.crumbs, &crumbs![RightOperand]);

        // Note that line `fun a = a b` des not introduce any connections, as it is a definition.

        assert_eq!(run.connections.len(),4);
    }

    #[test]
    pub fn inline_definition() {
        let run = TestRun::from_main_def("main = a");
        assert!(run.connections.is_empty());
    }
}
