//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.

pub mod mock;

use crate::prelude::*;

use flo_stream::Subscriber;

pub use double_representation::graph::Id;
pub use controller::node::Position;
pub use controller::notification;



// ==============
// === Errors ===
// ==============

#[derive(Clone,Debug,Fail)]
#[fail(display="Node with ID {} was not found.", _0)]
struct NodeNotFound(ast::ID);



// ===================
// === NewNodeInfo ===
// ===================

/// Describes the node to be added.
#[derive(Clone,Debug)]
pub struct NewNodeInfo {
    /// Expression to be placed on the node
    pub expression : String,
    /// Visual node position in the graph scene.
    pub position : Position,
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



// =================
// === Interface ===
// =================

/// Graph controller interface.
pub trait Interface: Sized {
    /// Type of the node controller handle that this graph controller uses.
    type NodeHandle : controller::node::Interface;

    /// Adds a new node to the graph and returns a controller managing this node.
    fn add_node(&self, node:NewNodeInfo) -> FallibleResult<Self::NodeHandle>;

    /// Retrieves a controller to the node with given ID.
    fn get_node(&self, id:ast::ID) -> FallibleResult<Self::NodeHandle>;

    /// Returns controllers for all the nodes in the graph.
    fn get_nodes(&self) -> FallibleResult<Vec<Self::NodeHandle>>;

    /// Removes node with given ID from the graph.
    fn remove_node(&self, id:ast::ID) -> FallibleResult<()>;

    /// Get subscriber receiving controller's notifications.
    fn subscribe(&mut self) -> Subscriber<controller::notification::Graph>;
}



// ==================
// === Controller ===
// ==================

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

//pub struct Handle;
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

    /// Returns information about all nodes in the graph.
    pub fn list_node_infos(&self) -> FallibleResult<Vec<double_representation::node::NodeInfo>> {
        let definition = self.get_definition()?;
        let graph = double_representation::graph::GraphInfo::from_definition(definition);
        Ok(graph.nodes)
    }

    /// Retrieves double rep information about node with given ID.
    pub fn node_info(&self, id:ast::ID) -> FallibleResult<double_representation::node::NodeInfo> {
        let nodes = self.list_node_infos()?;
        nodes.into_iter().find(|node_info| node_info.id() == id).ok_or(NodeNotFound(id).into())
    }

    /// Creates a new controller for node with given ID.
    fn create_node_controller(&self, id:ast::ID) -> controller::node::Controller {
        controller::node::Controller::new(self.clone(),id)
    }
}

impl Interface for Handle {
    type NodeHandle = controller::node::Controller;

    fn add_node(&self, _node:NewNodeInfo) -> FallibleResult<Self::NodeHandle> {
        todo!()
    }

    fn get_node(&self, id:ast::ID) -> FallibleResult<Self::NodeHandle> {
        let _ = self.node_info(id)?;
        Ok(controller::node::Controller::new(self.clone(),id))
    }

    fn get_nodes(&self) -> FallibleResult<Vec<Self::NodeHandle>> {
        let node_infos = self.list_node_infos()?;
        let nodes      = node_infos.into_iter().map(|node_info| {
            self.create_node_controller(node_info.id())
        }).collect();
        Ok(nodes)
    }

    fn remove_node(&self, _id:ast::ID) -> FallibleResult<()> {
        todo!()
    }

    fn subscribe(&mut self) -> Subscriber<notification::Graph> {
        todo!()
    }
}

