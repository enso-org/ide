//! Code for node discovery and other node-related tasks.

use crate::prelude::*;

use ast::Ast;
use ast::Id;
use ast::crumbs::Crumbable;
use ast::known;



// ================
// === NodeInfo ===
// ================

/// Description of the node that consists of all information locally available about node.
/// Nodes are required to bear IDs. This enum should never contain an ast of node without id set.
#[derive(Clone,Debug)]
#[allow(missing_docs)]
pub enum NodeInfo {
    /// Code with assignment, e.g. `foo = 2 + 2`
    Binding { infix: known::Infix },
    /// Code without assignment (no variable binding), e.g. `2 + 2`.
    Expression { ast: Ast },
}

impl NodeInfo {
    /// Tries to interpret the whole binding as a node. Right-hand side will become node's
    /// expression.
    pub fn new_binding(infix:known::Infix) -> Option<NodeInfo> {
        infix.rarg.id?;
        Some(NodeInfo::Binding {infix})
    }

    /// Tries to interpret AST as node, treating whole AST as an expression.
    pub fn new_expression(ast:Ast) -> Option<NodeInfo> {
        ast.id?;
        Some(NodeInfo::Expression {ast})
    }

    /// Tries to interpret AST as node, treating whole AST as an expression.
    pub fn from_line_ast(ast:&Ast) -> Option<NodeInfo> {
        if let Some(infix) = ast::opr::to_assignment(ast) {
            Self::new_binding(infix)
        } else {
            Self::new_expression(ast.clone())
        }
    }

    /// Node's unique ID.
    pub fn id(&self) -> Id {
        // Panic must not happen, as the only available constructors checks that
        // there is an ID present.
        self.expression().id.expect("Node AST must bear an ID")
    }

    /// Updates the node's AST so the node bears the given ID.
    pub fn set_id(&mut self, new_id:Id) {
        match self {
            NodeInfo::Binding{ref mut infix} => {
                let new_rarg = infix.rarg.with_id(new_id);
                let set      = infix.set(&ast::crumbs::InfixCrumb::RightOperand.into(),new_rarg);
                set.expect("Internal error: setting infix operand should always succeed.");
            }
            NodeInfo::Expression{ref mut ast} => {
                *ast = ast.with_id(new_id);
            }
        };
    }

    /// AST of the node's expression.
    pub fn expression(&self) -> &Ast {
        match self {
            NodeInfo::Binding   {infix} => &infix.rarg,
            NodeInfo::Expression{ast}   => &ast,
        }
    }

    /// Mutable AST of the node's expression. Maintains ID.
    pub fn set_expression(&mut self, expression:Ast) {
        let id = self.id();
        match self {
            NodeInfo::Binding{ref mut infix} => {
                let rarg = expression;
                let old_infix = infix.shape().clone();
                *infix = known::Infix::new(ast::Infix {rarg,..old_infix}, infix.id());
            }
            NodeInfo::Expression{ref mut ast}   => { *ast = expression; },
        }
        // Id might have been overwritten by the AST we have set. Now we restore it.
        self.set_id(id);
    }

    /// The whole AST of node.
    pub fn ast(&self) -> &Ast {
        match self {
            NodeInfo::Binding   {infix} => infix.into(),
            NodeInfo::Expression{ast}   => ast,
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use ast::HasRepr;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

    fn expect_node(ast:Ast, expression_text:&str, id: Id) {
        let node_info = NodeInfo::from_line_ast(&ast).expect("expected a node");
        assert_eq!(node_info.expression().repr(),expression_text);
        assert_eq!(node_info.id(), id);
    }

    #[wasm_bindgen_test]
    fn expression_node_test() {
        // expression: `4`
        let id = Id::new_v4();
        let ast = Ast::new(ast::Number { base:None, int: "4".into()}, Some(id));
        expect_node(ast,"4",id);
    }

    #[wasm_bindgen_test]
    fn binding_node_test() {
        // expression: `foo = 4`
        let id = Id::new_v4();
        let number = ast::Number { base:None, int: "4".into()};
        let larg   = Ast::var("foo");
        let loff   = 1;
        let opr    = Ast::opr("=");
        let roff   = 1;
        let rarg   = Ast::new(number, Some(id));
        let ast    = Ast::new(ast::Infix {larg,loff,opr,roff,rarg}, None);
        expect_node(ast,"4",id);
    }
}
