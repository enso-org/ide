//! Module with mock implementation of graph controller interface.
//!
//! Should not be used outside tests and/or debug scenes.

use crate::prelude::*;

use crate::controller::node::Interface;
use crate::controller::node::Position;
use crate::controller::notification;
use crate::double_representation::node::NodeInfo;

use ast::Ast;
use ast::ID;
use flo_stream::MessagePublisher;
use flo_stream::Subscriber;
use parser::api::IsParser;

/// Internal state storage of the mock node.
#[derive(Derivative)]
#[derivative(Debug)]
struct NodeState {
    node_info              : NodeInfo,
    position               : Position,
    graph                  : Option<controller::graph::mock::Handle>,
    #[derivative(Debug="ignore")]
    notification_publisher : notification::Publisher<notification::Node>,
}

/// Mock node controller.
#[derive(Clone,Debug)]
pub struct Handle {
    state : Rc<RefCell<NodeState>>,
}

impl Handle {
    /// Creates a mock node controller for given node state.
    pub fn new_expr_ast(expression: Ast, position: Position) -> Option<Handle> {
        let node_info  = NodeInfo::new_expression(expression)?;
        let graph      = None;
        let notification_publisher = default();
        let state_data = NodeState {node_info,position,graph,notification_publisher};
        let state      = Rc::new(RefCell::new(state_data));
        Some(Handle {state})
    }

    fn invalidate_graph(&self) {
        if let Some(ref graph) = self.state.borrow_mut().graph {
            graph.0.borrow_mut().invalidate();
        }
    }
}

impl Interface for Handle {
    fn id(&self) -> ID {
        self.state.borrow().node_info.id()
    }

    fn expression(&self) -> FallibleResult<Ast> {
        Ok(self.state.borrow().node_info.expression().clone())
    }

    fn position(&self) -> FallibleResult<Position> {
        Ok(self.state.borrow().position)
    }

    fn set_expression(&self, ast:Ast) -> FallibleResult<()> {
        let new_node_info = NodeInfo::new_expression(ast).unwrap();
        assert_eq!(new_node_info.id(), self.id(), "node's id must not be changed");
        self.state.borrow_mut().node_info = new_node_info;
        self.invalidate_graph();
        Ok(())
    }

    fn set_expression_text(&self, expression:&str) -> FallibleResult<()> {
        let mut parser = parser::Parser::new_or_panic();
        let module     = parser.parse_module(expression.into(),default()).unwrap();
        let expression = module.lines[0].elem.as_ref().unwrap().clone();
        self.invalidate_graph();
        self.set_expression(expression)
    }

    fn set_position(&self, new_position:Position) -> FallibleResult<()> {
        self.state.borrow_mut().position = new_position;
        self.invalidate_graph();
        Ok(())
    }

    fn subscribe(&mut self) -> Subscriber<controller::notification::Node> {
        self.state.borrow_mut().notification_publisher.subscribe()
    }
}



#[cfg(test)]
mod test {
    use super::*;

    use ast::HasRepr;
    use uuid::Uuid;

    #[test]
    fn mock_node_controller() -> FallibleResult<()> {
        let id   = Uuid::new_v4();
        let ast  = ast::Ast::var_with_id("foo",id);
        let position = default();
        let mut node = Handle::new_expr_ast(ast.clone_ref(), position).unwrap();

        // limit ourselves to trait-based api
        let node: &mut dyn Interface = &mut node;
        assert_eq!(node.id(), id);
        assert_eq!(node.expression()?.repr(), ast.repr());
        assert_eq!(node.position()?, position);

        let ast = ast::Ast::var_with_id("bar",id);
        node.set_expression(ast.clone_ref())?;
        assert_eq!(node.expression()?.repr(), ast.repr());

        let position = Position::new(10.0,20.0);
        node.set_position(position)?;
        assert_eq!(node.position()?, position);
        assert_eq!(node.position()?.vector.x, 10.0);
        assert_eq!(node.position()?.vector.y, 20.0);

        let ast_text = "2+2";
        node.set_expression_text(ast_text)?;
        assert_eq!(node.expression()?.repr(), ast_text);

        // id wasn't broken during all these changes
        assert_eq!(node.id(), id);

        Ok(())
    }
}
