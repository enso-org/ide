//! Graph Controller.
//!
//! This controller provides access to a specific graph. It lives under a module controller, as
//! each graph belongs to some module.


use crate::prelude::*;

use crate::double_representation::definition::DefinitionName;
use crate::controller::FallibleResult;

use flo_stream::Subscriber;


/// Used e.g. for node posittion
// TODO [mwu] use some common dictionary type for positions
type Position = (f64,f64);

/// Crumb describes step that needs to be done when going from context (for graph being a module)
/// to the target.
// TODO [mwu]
//  Currently we support only entering named definitions.
pub type Crumb = DefinitionName;

/// Identifies graph in the module.
#[derive(Clone,Debug,Eq,Hash,PartialEq)]
pub struct Id {
    /// Sequence of traverses from module root up to the identified graph.
    pub crumbs : Vec<Crumb>,
}

/// Describes node in the graph for the view.
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
        module: controller::module::Handle,

        id : Id
//        notifications  : notification::Publisher<Notification>,
        // TODO [mwu] support nested definitions
        // TODO [mwu] notifications
    }

    impl {
//        /// Gets the name of the definition that this graph is body of.
//        pub fn get_name(&self) -> DefinitionName {
//            self.name.clone()
//        }

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
//
//impl Controller {
//    /// Returns information about all nodes in the graph.
//    pub fn list_nodes(&self) -> Vec<Node> {
//        let name = self.get_name();
//        let definition = match self.get_module().find_definition(&name) {
//            Some(def) => def,
//            _ => return default(),
//        };
//
//        let graph = double_representation::graph::GraphInfo::from_definition(definition);
//        graph.nodes.into_iter().map(|node_info| Node {info:node_info}).collect()
//    }
//}

impl Handle {
    /// Creates a new graph controller. Given name should identify a definition in the module's
    /// root scope.
    pub fn new(module:controller::module::Handle, id:Id) -> FallibleResult<Handle> {
        let data = Controller {module,id};
        Ok(Handle::new_from_data(data))
    }

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
    pub fn add_node(&self, info:NewNodeInfo) -> ast::ID {
        todo!()
    }

    /// Removed the node from graph.
    pub fn remove_node(&self, node_id:ast::ID) -> FallibleResult<()> {
        todo!()
    }

    /// Sets the visual position of the given node.
    pub fn move_node(&self, node_id:ast::ID, new_position:Position) -> FallibleResult<()> {
        todo!()
    }

    /// Sets expression of the given node.
    pub fn edit_node(&self, node_id:ast::ID, new_expression:impl Str) -> FallibleResult<()> {
        todo!()
    }
}

/// Describes the node to be added.
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



#[cfg(test)]
mod tests {
    use super::*;

    pub fn ttt() {

    }
}

