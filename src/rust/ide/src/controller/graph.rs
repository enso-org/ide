//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.

use crate::prelude::*;

use flo_stream::Subscriber;

pub use double_representation::graph::Id;
pub use controller::notification;


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
#[derive(Debug)]
pub struct Handle {
    /// Controller of the module which this graph belongs to.
    module    : controller::module::Handle,
    id        : Id,
    publisher : controller::notification::Publisher<controller::notification::Graph>,
    // TODO [mwu] support nested definitions
}

impl Clone for Handle {
    fn clone(&self) -> Self {
        Self::new_unchecked(self.module.clone(), self.id.clone())
    }
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
}

//pub struct Handle;
impl Handle {
    /// Creates a new controller. Does not check if id is valid.
    pub fn new_unchecked(module:controller::module::Handle, id:Id) -> Handle {
//        let _graphs_notifications = module.subscribe_graph_notifications();
//        executor::global::spawn(process_stream_with_handle(_graphs_notifications,weak,|notification,this| {
//            this.with_borrowed(move |data| data.notifications.publish(notification))
//        }));
        // TODO [mwu] wire notifications together
        let publisher = default();
        Handle {module,id,publisher}
    }

    /// Creates a new graph controller. Given ID should uniquely identify a definition in the
    /// module. Fails if ID cannot be resolved.
    pub fn new(module:controller::module::Handle, id:Id) -> FallibleResult<Handle> {
        let ret = Self::new_unchecked(module,id);
        let _ = ret.get_definition()?; // make sure that definition exists
        Ok(ret)
    }

    /// Retrieves double rep information about definition providing this graph.
    pub fn get_definition
    (&self) -> FallibleResult<double_representation::definition::DefinitionInfo> {
        let module = self.get_module();
        let id     = self.get_id();
        module.find_definition(&id)
    }

    /// Returns double rep information about all nodes in the graph.
    pub fn list_node_infos(&self) -> FallibleResult<Vec<double_representation::node::NodeInfo>> {
        let definition = self.get_definition()?;
        let graph      = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes)
    }

    /// Retrieves double rep information about node with given ID.
    pub fn node_info(&self, id:ast::ID) -> FallibleResult<double_representation::node::NodeInfo> {
        let nodes = self.list_node_infos()?;
        let node  = nodes.into_iter().find(|node_info| node_info.id() == id);
        node.ok_or(NodeNotFound(id).into())
    }
}

//impl Interface for Handle {
impl Handle {
    /// Gets information about node with given id.
    ///
    /// Note that it is more efficient to use `get_nodes` to obtain all information at once,
    /// rather then repeatedly call this method.
    pub fn get_node(&self, id:ast::ID) -> FallibleResult<Node> {
        let info = self.node_info(id)?;
        let metadata = self.get_node_metadata(id)?;
        Ok( Node{info,metadata})
    }

    /// Returns information about all the nodes currently present in this graph.
    pub fn get_nodes(&self) -> FallibleResult<Vec<Node>> {
        let node_infos = self.list_node_infos()?;
        let mut nodes = Vec::new();
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
    pub fn subscribe(&mut self) -> Subscriber<notification::Graph> {
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

