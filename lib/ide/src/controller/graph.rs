//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.

use crate::prelude::*;

use flo_stream::Subscriber;

pub use double_representation::graph::Id;



// ============
// === Node ===
// ============

/// Used e.g. for node posittion
// TODO [mwu] use some common dictionary type for positions
type Position = (f64,f64);

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
    pub fn expression_text(&self) -> String {
        self.info.expression_text()
    }

    /// Text representation of the node's expression.
    pub fn expression_ast(&self) -> ast::Ast {
        self.info.expression_ast().clone_ref()
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
    pub fn list_nodes(&self) -> FallibleResult<Vec<Node>> {
        let definition = self.get_definition()?;
        let graph = double_representation::graph::GraphInfo::from_definition(definition);
        let ret = graph.nodes.into_iter().map(|node_info| Node { info: node_info });
        Ok(ret.collect())
    }

    /// Adds a new node to this graph.
    pub fn add_node(&self, _info:NewNodeInfo) -> ast::ID {
        todo!()
    }

    /// Removed the node from graph.
    pub fn remove_node(&self, _node_id:ast::ID) -> FallibleResult<()> {
        todo!()
    }

    /// Sets the visual position of the given node.
    pub fn move_node(&self, _node_id:ast::ID, _new_position:Position) -> FallibleResult<()> {
        todo!()
    }

    /// Sets expression of the given node.
    pub fn edit_node(&self, _node_id:ast::ID, _new_expression:impl Str) -> FallibleResult<()> {
        todo!()
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

