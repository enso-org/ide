use crate::prelude::*;

use crate::double_representation::definition::DefinitionInfo;
use crate::double_representation::node::{Id, NodeInfo};

use ast::crumbs::{Crumbs, Located};
use crate::double_representation::alias_analysis::{AliasAnalyzer, NormalizedName, Scope};


#[cfg(test)]
pub mod test_utils;

//////////////////////////

type Block = ast::Block<Ast>;

type Endpoint    = (Id,Crumbs);
type Source      = (Id,Crumbs);
type Destination = (Id,Crumbs);

type Connection = (Source,Destination);

/// Lists all the connection in the graph for the given code block.
pub fn list_block(block:&Block) -> Vec<Connection> {
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
            ret.push((source,destination))
        }
    };
    ret
}

/// Lists all the connection in the single-expression definition body.
pub fn list_expression(ast:&Ast) -> Vec<Connection> {
    // At this points single-expression graphs do not have any connection.
    // This will change when there will be input/output pseudo-nodes.
    default()
}

pub fn list(def:&DefinitionInfo) -> Vec<Connection> {
    let body = def.body();
    match body.shape() {
        ast::Shape::Block(block) => list_block(block),
        _                        => list_expression(&body),
    }
}


struct CrumbTree<T> {
    item     : T,
    children : Vec<ast::crumbs::Located<Self>>,
}

fn block_line_endpoint(block:&ast::Block<Ast>, mut crumbs:Crumbs) -> Endpoint {
    match crumbs.first() {
        Some(ast::crumbs::Crumb::Block(block_crumb)) => {
            let line_ast = block.get(block_crumb).unwrap();
            let node     = NodeInfo::from_line_ast(line_ast).unwrap();
            crumbs.pop_front();
            (node.id(),crumbs)
        }
        _ => panic!("not implemented"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use parser::Parser;
    use crate::double_representation::definition::ScopeKind;

    #[test]
    pub fn connection_listing_test() {
        let program = r"main =
    a = 2
    b = 2
    c = a + b";

        let parser = Parser::new_or_panic();
        let module = parser.parse_module(program,default()).unwrap();

        let def = DefinitionInfo::from_root_line(&module.lines[0]).unwrap();
        list(&def);
    }
}