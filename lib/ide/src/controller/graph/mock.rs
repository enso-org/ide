//! Module with mock implementation of graph controller interface.
//!
//! Should not be used outside tests and/or debug scenes.


use crate::prelude::*;

use crate::controller::graph::Interface;
use crate::controller::graph::NodeNotFound;
use crate::controller::notification;

use flo_stream::MessagePublisher;
use flo_stream::Subscriber;

/// State of the mock graph controller.
pub struct MockGraph {
    nodes                  : HashMap<ast::ID,Rc<dyn controller::node::Interface>>,
    notification_publisher : notification::Publisher<notification::Graph>,
}

impl MockGraph {
    pub fn invalidate(&mut self) {
        let notification = notification::Graph::Invalidate;
        self.notification_publisher.publish(notification);
    }
}

impl Debug for MockGraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<Mock Graph Controller>")
    }
}

#[derive(Clone,Debug)]
pub struct Handle(pub Rc<RefCell<MockGraph>>);

impl Interface for Handle {
    fn get_node(&self, id:ast::ID) -> FallibleResult<Rc<dyn controller::node::Interface>> {
        self.0.borrow().nodes.get(&id).cloned().ok_or_else(|| NodeNotFound(id).into())
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



