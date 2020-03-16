//! Module with mock implementation of graph controller interface.
//!
//! Should not be used outside tests and/or debug scenes.


use crate::prelude::*;

use crate::controller::graph::Interface;
use crate::controller::graph::NewNodeInfo;
use crate::controller::graph::NodeNotFound;
use crate::controller::node::Interface as NodeInterface;
use crate::controller::node::Position;
use crate::controller::notification;
use crate::executor::global::spawn;

use ast::Ast;
use flo_stream::MessagePublisher;
use flo_stream::Subscriber;
use parser::api::IsParser;

/// State of the mock graph controller.
#[derive(Default)]
pub struct MockGraph {
    nodes                  : HashMap<ast::ID,controller::node::mock::Handle>,
    notification_publisher : notification::Publisher<notification::Graph>,
}

impl MockGraph {
    /// Create a new mock graph controller.
    pub fn new() -> MockGraph {
        default()
    }

    /// Emits Invalidate notification.
    pub fn invalidate(&mut self) {
        let notification = notification::Graph::Invalidate;
        spawn(self.notification_publisher.publish(notification))
    }

    /// Adds mock node with given Ast as expression and visually located at given position.
    pub fn insert_node
    (&mut self, node:controller::node::mock::Handle)
    -> FallibleResult<Box<dyn controller::node::Interface>> {
        let id   = node.id();
        assert_eq!(self.nodes.contains_key(&id), false, "Node IDs must be unique.");
        self.nodes.insert(id,node.clone());
        Ok(Box::new(node))
    }
}

impl Debug for MockGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Mock Graph Controller>")
    }
}

/// Mock graph controller.
#[derive(Clone,Debug,Default)]
pub struct Handle(pub Rc<RefCell<MockGraph>>);

impl Handle {
    /// Creates a new mock graph controller.
    pub fn new() -> Handle {
        default()
    }

    /// Adds mock node with given Ast as expression and visually located at given position.
    pub fn add_node_ast
    (&self, expression: Ast, position: Position)
     -> FallibleResult<Box<dyn controller::node::Interface>> {
        let node = controller::node::mock::Handle::new_expr_ast(expression,position).unwrap();
        self.0.borrow_mut().insert_node(node)
    }
}

impl Interface for Handle {
    fn add_node(&self, node:NewNodeInfo) -> FallibleResult<Box<dyn controller::node::Interface>> {
        let mut parser = parser::Parser::new_or_panic();
        let module     = parser.parse_module(node.expression,default()).unwrap();
        let ast        = module.lines[0].elem.as_ref().unwrap().with_id(ast::ID::new_v4());
        self.add_node_ast(ast, node.position)
    }

    fn get_node(&self, id:ast::ID) -> FallibleResult<Box<dyn controller::node::Interface>> {
        let node_result = self.0.borrow().nodes.get(&id).cloned();
        let node = node_result.ok_or_else(|| NodeNotFound(id))?;
        Ok(Box::new(node))
    }

    fn get_nodes(&self) -> FallibleResult<Vec<Box<dyn controller::node::Interface>>> {
        let mut ret: Vec<Box<dyn controller::node::Interface>> = default();
        for node in self.0.borrow_mut().nodes.values() {
            ret.push(Box::new(node.clone()))
        }
        Ok(ret)
    }

    fn remove_node(&self, id:ast::ID) -> FallibleResult<()> {
        if let Some(_) = self.0.borrow_mut().nodes.remove(&id) {
            Ok(())
        } else {
            Err(NodeNotFound(id).into())
        }
    }

    fn subscribe(&mut self) -> Subscriber<notification::Graph> {
        self.0.borrow_mut().notification_publisher.subscribe()
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use uuid::Uuid;

    #[test]
    fn mock_graph_controller() -> FallibleResult<()> {
        let graph = controller::graph::mock::Handle::new();

        let id   = Uuid::new_v4();
        let ast  = ast::Ast::var_with_id("foo",id);
        let node = graph.add_node_ast(ast, Position::new(10.0,20.0))?;
        node.set_position(Position::new(50.0, 20.0))?;
        assert_eq!(graph.get_nodes().unwrap().len(), 1);

        for node in graph.get_nodes()? {
            let pos = node.position()?;
            let new_pos = Position::new(pos.vector.x, pos.vector.x * 10.0);
            node.set_position(new_pos)?;
        }

        for node in graph.get_nodes()? {
            let id   = node.id();
            let expr = node.expression()?;
            let pos  = node.position()?;
            println!("Node with id {} has expression {} and position {:?}", id, String::from(expr), pos);
        }

        Ok(())
    }
}
