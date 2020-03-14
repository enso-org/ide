//! Module with mock implementation of graph controller interface.
//!
//! Should not be used outside tests and/or debug scenes.


use crate::prelude::*;

use crate::controller::graph::Interface;
use crate::controller::graph::NewNodeInfo;
use crate::controller::graph::NodeNotFound;
use crate::controller::graph::NodeHandle;
use crate::controller::notification;
use crate::executor::global::spawn;

use flo_stream::MessagePublisher;
use flo_stream::Subscriber;

/// State of the mock graph controller.
pub struct MockGraph {
    nodes                  : HashMap<ast::ID,Rc<controller::node::mock::Handle>>,
    notification_publisher : notification::Publisher<notification::Graph>,
}

impl MockGraph {
    /// Emits Invalidate notification.
    pub fn invalidate(&mut self) {
        let notification = notification::Graph::Invalidate;
        spawn(self.notification_publisher.publish(notification))
    }
}

impl Debug for MockGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Mock Graph Controller>")
    }
}

/// Mock graph controller.
#[derive(Clone,Debug)]
pub struct Handle(pub Rc<RefCell<MockGraph>>);

impl Interface for Handle {
    fn add_node(&self, _node:NewNodeInfo) -> FallibleResult<NodeHandle> {
//        let ast  = node.expression
//        let node = controller::node::mock::Handle::new_expr_ast(ast,position);
        todo!()
    }

    fn get_node(&self, id:ast::ID) -> FallibleResult<NodeHandle> {
        let node_result = self.0.borrow().nodes.get(&id).cloned();
        Ok(node_result.ok_or_else(|| NodeNotFound(id))?)
    }

    fn get_nodes(&self) -> FallibleResult<Vec<NodeHandle>> {
        let mut ret: Vec<NodeHandle> = default();
        for node in self.0.borrow_mut().nodes.values() {
            ret.push(node.clone())
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
