
use crate::prelude::*;

use crate::controller::node::Interface;
use crate::controller::node::Position;

use ast::Ast;
use ast::ID;
use parser::api::IsParser;
use crate::double_representation::node::NodeInfo;

struct NodeState {
    node_info : double_representation::node::NodeInfo,
    position  : Position,
}

struct Controller {
    state : Rc<RefCell<NodeState>>,
}

impl Controller {
    pub fn new_expr_ast(expression: Ast, position: Position) -> Option<Controller> {
        let node_info  = NodeInfo::new_expression(expression)?;
        let state_data = NodeState {node_info,position};
        let state      = Rc::new(RefCell::new(state_data));
        Some(Controller {state})
    }
}

impl Interface for Controller {
    fn id(&self) -> ID {
        self.state.borrow().node_info.id()
    }

    fn expression(&self) -> FallibleResult<Ast> {
        Ok(self.state.borrow().node_info.expression().clone())
    }

    fn position(&self) -> FallibleResult<Position> {
        Ok(self.state.borrow().position.clone())
    }

    fn set_expression(&self, ast:Ast) -> FallibleResult<()> {
        let new_node_info = NodeInfo::new_expression(ast).unwrap();
        assert_eq!(new_node_info.id(), self.id(), "node's id must not be changed");
        self.state.borrow_mut().node_info = new_node_info;
        Ok(())
    }

    fn set_expression_text(&self, expression:&str) -> FallibleResult<()> {
        let mut parser = parser::Parser::new_or_panic();
        let module     = parser.parse_module(expression.into(),default()).unwrap();
        let expression = module.lines[0].elem.as_ref().unwrap().clone();
        self.set_expression(expression)
    }

    fn set_position(&self, new_position:Position) -> FallibleResult<()> {
        self.state.borrow_mut().position = new_position;
        Ok(())
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
        let mut node = Controller::new_expr_ast(ast.clone_ref(), position).unwrap();

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
