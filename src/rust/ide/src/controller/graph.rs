//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.


use crate::prelude::*;

pub use crate::double_representation::graph::Id;

use flo_stream::MessagePublisher;
use flo_stream::Subscriber;
use utils::channel::process_stream_with_handle;



// ==============
// === Errors ===
// ==============

/// Error raised when node with given Id was not found in the graph's body.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Node with Id {} was not found.", _0)]
pub struct NodeNotFound(ast::ID);



// ============
// === Node ===
// ============

/// TODO: replace with usage of the structure to be provided by Josef
#[derive(Clone,Copy,Debug)]
pub struct NodeMetadata; // here goes position

/// Error raised when node with given Id was not found in the graph's body.
#[derive(Clone,Debug)]
pub struct Node {
    /// Information based on AST, from double_representation module.
    pub info : double_representation::node::NodeInfo,
    /// Information about this node stored in the module's metadata.
    pub metadata : NodeMetadata
}



// ===================
// === NewNodeInfo ===
// ===================

/// Describes the node to be added.
#[derive(Clone,Debug)]
pub struct NewNodeInfo {
    /// Expression to be placed on the node
    pub expression : String,
    /// Visual node position in the graph scene.
    pub metadata : NodeMetadata,
    /// ID to be given to the node.
    pub id : Option<ast::ID>,
    /// Where line created by adding this node should appear.
    pub location_hint : LocationHint
}

/// Describes the desired position of the node's line in the graph's code block.
#[derive(Clone,Copy,Debug)]
pub enum LocationHint {
    /// Try placing this node's line before the line described by id.
    Before(ast::ID),
    /// Try placing this node's line after the line described by id.
    After(ast::ID),
    /// Try placing this node's line at the start of the graph's code block.
    Start,
    /// Try placing this node's line at the end of the graph's code block.
    End,
}



// ==================
// === Controller ===
// ==================

/// State data of the module controller.
#[derive(Clone,Debug)]
pub struct Handle {
    /// Controller of the module which this graph belongs to.
    module    : controller::module::Handle,
    id        : Id,
    /// Publisher. When creating a controller, it sets up task to emit notifications through this
    /// publisher to relay changes from the module controller.
    publisher : Rc<RefCell<controller::notification::Publisher<controller::notification::Graph>>>,
}

impl Handle {
    /// Gets a handle to a controller of the module that this definition belongs to.
    pub fn get_module(&self) -> controller::module::Handle {
        self.module.clone()
    }

    /// Gets a handle to a controller of the module that this definition belongs to.
    pub fn get_id(&self) -> Id {
        self.id.clone()
    }

    /// Creates a new controller. Does not check if id is valid.
    ///
    /// Requires global executor to spawn the events relay task.
    pub fn new_unchecked(module:controller::module::Handle, id:Id) -> Handle {
        let graphs_notifications = module.subscribe_graph_notifications();
        let publisher = default();
        let ret       = Handle {module,id,publisher};
        let weak      = Rc::downgrade(&ret.publisher);
        let relay_notifications = process_stream_with_handle(graphs_notifications,weak,
            |notification,this| {
                match notification {
                    controller::notification::Graphs::Invalidate =>
                    this.borrow_mut().publish(controller::notification::Graph::Invalidate),
            }
        });
        executor::global::spawn(relay_notifications);
        ret
    }

    /// Creates a new graph controller. Given ID should uniquely identify a definition in the
    /// module. Fails if ID cannot be resolved.
    ///
    /// Requires global executor to spawn the events relay task.
    pub fn new(module:controller::module::Handle, id:Id) -> FallibleResult<Handle> {
        let ret = Self::new_unchecked(module,id);
        let _ = ret.get_graph_definition_info()?; // make sure that definition exists
        Ok(ret)
    }

    /// Retrieves double rep information about definition providing this graph.
    pub fn get_graph_definition_info
    (&self) -> FallibleResult<double_representation::definition::DefinitionInfo> {
        let module = self.get_module();
        let id     = self.get_id();
        module.find_definition(&id)
    }

    /// Returns double rep information about all nodes in the graph.
    pub fn get_all_node_infos
    (&self) -> FallibleResult<Vec<double_representation::node::NodeInfo>> {
        let definition = self.get_graph_definition_info()?;
        let graph      = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes)
    }

    /// Retrieves double rep information about node with given ID.
    pub fn get_node_info
    (&self, id:ast::ID) -> FallibleResult<double_representation::node::NodeInfo> {
        let nodes = self.get_all_node_infos()?;
        let node  = nodes.into_iter().find(|node_info| node_info.id() == id);
        node.ok_or_else(|| NodeNotFound(id).into())
    }

    /// Gets information about node with given id.
    ///
    /// Note that it is more efficient to use `get_nodes` to obtain all information at once,
    /// rather then repeatedly call this method.
    pub fn get_node(&self, id:ast::ID) -> FallibleResult<Node> {
        let info     = self.get_node_info(id)?;
        let metadata = self.get_node_metadata(id)?;
        Ok(Node {info,metadata})
    }

    /// Returns information about all the nodes currently present in this graph.
    pub fn get_nodes(&self) -> FallibleResult<Vec<Node>> {
        let node_infos = self.get_all_node_infos()?;
        let mut nodes  = Vec::new();
        for info in node_infos {
            let metadata = self.get_node_metadata(info.id())?;
            nodes.push(Node {info,metadata})
        }

        Ok(nodes)
    }

    /// Adds a new node to the graph and returns information about created node.
    pub fn add_node(&self, _node:NewNodeInfo) -> FallibleResult<Node> {
        todo!()
    }

    /// Removes the node with given Id.
    pub fn remove_node(&self, _id:ast::ID) -> FallibleResult<()> {
        todo!()
    }

    /// Subscribe to updates about changes in this graph.
    pub fn subscribe(&mut self) -> Subscriber<controller::notification::Graph> {
        todo!()
    }

    /// Retrieves metadata for the given node.
    pub fn get_node_metadata(&self, _id:ast::ID) -> FallibleResult<NodeMetadata> {
        todo!()
    }

    /// Update metadata for the given node.
    pub fn update_node_metadata<F>(&self, _id:ast::ID, _updater:F) -> FallibleResult<NodeMetadata>
    where F : FnOnce(&mut NodeMetadata) {
        todo!()
    }
}


#[cfg(test)]
mod tests {
//    use super::*;

    #[test]
    fn test_graph_controller() {
    }
}
