//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.

pub mod mock;

use crate::prelude::*;

use flo_stream::Subscriber;

pub use double_representation::graph::Id;
pub use controller::node::Position;


#[derive(Clone,Debug,Fail)]
#[fail(display = "Node by ID {} was not found.", _0)]
struct NodeNotFound(ast::ID);

pub trait Interface {
    fn get_node(&self, id:ast::ID) -> FallibleResult<Rc<dyn controller::node::Interface>>;
    fn remove_node(&self, id:ast::ID) -> FallibleResult<()>;

    /// Get subscriber receiving controller's notifications.
    fn subscribe(&mut self) -> Subscriber<controller::notification::Graph>;
}


// ============
// === Node ===
// ============

/// Describes node in the graph for the view.
///
/// Currently just a thin wrapper over double rep info, in future will be enriched with information
/// received from the Language server/
#[derive(Clone,Debug)]
pub struct Node {
    info : double_representation::node::NodeInfo,
}

impl Node {
    /// Node's unique ID.
    pub fn id(&self) -> ast::ID {
        self.info.id()
    }

    /// Text representation of the node's expression.
    pub fn expression(&self) -> ast::Ast {
        self.info.expression().clone_ref()
    }

    /// Nodes visual position in the scene.
    pub fn position() -> Position {
        default()
    }
}



// ====================
// === Notification ===
// ====================

/// Notification about change in this graph's scope.
#[derive(Clone,Copy,Debug)]
pub enum Notification {
    /// Invalidates the whole graph, it should be re-retrieved.
    Invalidate
}

shared! { Handle
    /// State data of the module controller.
    #[derive(Debug)]
    pub struct Controller {
        /// Controller of the module which this graph belongs to.
        module : controller::module::Handle,
        id     : Id
        // TODO [mwu] support nested definitions
        // TODO [mwu] notifications
    }

    impl {
        /// Gets a handle to a controller of the module that this definition belongs to.
        pub fn get_module(&self) -> controller::module::Handle {
            self.module.clone()
        }

        /// Gets a handle to a controller of the module that this definition belongs to.
        pub fn get_id(&self) -> Id {
            self.id.clone()
        }
    }
}

//struct Handle;
impl Handle {
    /// Creates a new graph controller. Given ID should uniquely identify a definition in the
    /// module.
    pub fn new(module:controller::module::Handle, id:Id) -> FallibleResult<Handle> {
        let data = Controller {module,id};
        let ret = Handle::new_from_data(data);
        let _ = ret.get_definition()?; // make sure that definition exists
        Ok(ret)
    }

    /// Retrieves information about definition providing this graph.
    pub fn get_definition
    (&self) -> FallibleResult<double_representation::definition::DefinitionInfo> {
        let module = self.get_module();
        let id     = self.get_id();
        module.find_definition(&id)
    }

    /// Get subscriber receiving notifications about changes in graph.
    pub fn subscribe_notifications(&self) -> Subscriber<Notification> {
        todo!() // TODO implement once https://github.com/luna/ide/pull/231/ is ready
    }

    /// Returns information about all nodes in the graph.
    pub fn list_node_infos(&self) -> FallibleResult<Vec<double_representation::node::NodeInfo>> {
        let definition = self.get_definition()?;
        let graph = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes)
    }

    pub fn node_info(&self, id:ast::ID) -> FallibleResult<double_representation::node::NodeInfo> {
        let nodes = self.list_node_infos()?;
        Ok(nodes.into_iter().find(|node_info| node_info.id() == id).ok_or(NodeNotFound(id))?)
    }
}



// ====================
// === Notification ===
// ====================

/// Describes the node to be added.
#[derive(Clone,Debug)]
pub struct NewNodeInfo {
    /// Expression to be placed on the node
    pub expression:String,
    /// Visual node position in the graph scene.
    pub location:Position,
    /// ID to be given to the node.
    pub id:Option<ast::ID>,
    /// ID of the node that this node should be placed before. If `None`, it will be placed at the
    /// block's end.
    pub next_node:Option<ast::ID>,
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
//    use super::*;

}

