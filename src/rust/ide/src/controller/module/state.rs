//! This module contains all structures which describes Module state (code, ast, metadata).

pub mod registry;

use crate::prelude::*;

use crate::controller::notification;
use crate::double_representation::definition::DefinitionInfo;

use flo_stream::MessagePublisher;
use flo_stream::Subscriber;
use parser::api::SourceFile;
use serde::Serialize;
use serde::Deserialize;



// ============
// == Errors ==
// ============

/// Failure for missing node metadata.
#[derive(Debug,Clone,Copy,Fail)]
#[fail(display="Node with ID {} was not found in metadata.", _0)]
pub struct NodeMetadataNotFound(pub ast::Id);



// ==============
// == Metadata ==
// ==============

/// Mapping between ID and metadata.
#[derive(Debug,Clone,Default,Deserialize,Serialize)]
pub struct Metadata {
    /// Metadata used within ide.
    #[serde(default="default")]
    pub ide : IdeMetadata,
    #[serde(flatten)]
    /// Metadata of other users of SourceFile<Metadata> API.
    /// Ide should not modify this part of metadata.
    rest : serde_json::Value,
}

impl parser::api::Metadata for Metadata {}

/// Metadata that belongs to ide.
#[derive(Debug,Clone,Default,Deserialize,Serialize)]
pub struct IdeMetadata {
    /// Metadata that belongs to nodes.
    node : HashMap<ast::Id,NodeMetadata>
}

/// Metadata of specific node.
#[derive(Debug,Clone,Copy,Default,Serialize,Deserialize,Shrinkwrap)]
pub struct NodeMetadata {
    /// Position in x,y coordinates.
    pub position: Option<Position>
}

/// Used for storing node position.
#[derive(Clone,Copy,Debug,PartialEq,Serialize,Deserialize)]
pub struct Position {
    /// Vector storing coordinates of the visual position.
    pub vector:Vector2<f32>
}

impl Position {
    /// Creates a new position with given coordinates.
    pub fn new(x:f32, y:f32) -> Position {
        let vector = Vector2::new(x,y);
        Position {vector}
    }
}



// ====================
// === Module State ===
// ====================

/// A type describing content of the module: the ast and metadata.
pub type Content = SourceFile<Metadata>;
/// A shared handle for module's state.
pub type Handle = Rc<State>;

/// A structure describing the module state.
///
/// It implements internal mutability pattern, so the state may be shared between different
/// controllers. Each change in module will emit notification for each module representation
/// (text and graphs).
#[derive(Debug)]
pub struct State {
    content             : RefCell<Content>,
    text_notifications  : RefCell<notification::Publisher<notification::Text>>,
    graph_notifications : RefCell<notification::Publisher<notification::Graphs>>,
}

impl Default for State {
    fn default() -> Self {
        let ast = ast::known::Module::new(ast::Module{lines:default()},None);
        Self::new(ast,default())
    }
}

impl State {
    /// Create state with given content.
    pub fn new(ast:ast::known::Module, metadata:Metadata) -> Self {
        State {
            content: RefCell::new(SourceFile{ast,metadata}),
            text_notifications  : default(),
            graph_notifications : default(),
        }
    }

    /// Update whole content of the module.
    pub fn update_whole(&self, content:Content) {
        *self.content.borrow_mut() = content;
        self.notify(notification::Text::Invalidate,notification::Graphs::Invalidate);
    }

    /// Get module sources as a string, which contains both code and metadata.
    pub fn source_as_string(&self) -> FallibleResult<String> {
        Ok(String::try_from(&*self.content.borrow())?)
    }

    /// Get module's ast.
    pub fn ast(&self) -> ast::known::Module {
        self.content.borrow().ast.clone_ref()
    }

    /// Update ast in module controller.
    pub fn update_ast(&self, ast:ast::known::Module) {
        self.content.borrow_mut().ast  = ast.into();
        self.notify(notification::Text::Invalidate,notification::Graphs::Invalidate);
    }

    /// Obtains definition information for given graph id.
    pub fn find_definition
    (&self,id:&double_representation::graph::Id) -> FallibleResult<DefinitionInfo> {
        let ast = self.content.borrow().ast.clone_ref();
        double_representation::definition::traverse_for_definition(&ast,id)
    }

    /// Returns metadata for given node, if present.
    pub fn node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        let data = self.content.borrow().metadata.ide.node.get(&id).cloned();
        data.ok_or_else(|| NodeMetadataNotFound(id).into())
    }

    /// Sets metadata for given node.
    pub fn set_node_metadata(&self, id:ast::Id, data:NodeMetadata) {
        self.content.borrow_mut().metadata.ide.node.insert(id, data);
        let graph_change = notification::Graphs::Invalidate;
        executor::global::spawn(self.graph_notifications.borrow_mut().publish(graph_change));
    }

    /// Removes metadata of given node and returns them.
    pub fn take_node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        let data = self.content.borrow_mut().metadata.ide.node.remove(&id);
        data.ok_or_else(|| NodeMetadataNotFound(id).into())
    }

    /// Subscribe for notifications about text representation changes.
    pub fn subscribe_text_notifications(&self) -> Subscriber<notification::Text> {
        self.text_notifications.borrow_mut().subscribe()
    }

    /// Subscribe for notifications about graph representation changes.
    pub fn subscribe_graph_notifications(&self) -> Subscriber<notification::Graphs> {
        self.graph_notifications.borrow_mut().subscribe()
    }

    fn notify(&self, text_change:notification::Text, graphs_change:notification::Graphs) {
        let code_notify  = self.text_notifications.borrow_mut().publish(text_change);
        let graph_notify = self.graph_notifications.borrow_mut().publish(graphs_change);
        executor::global::spawn(async move { futures::join!(code_notify,graph_notify); });
    }

    /// Create module state from given code, id_map and metadata.
    #[cfg(test)]
    pub fn from_code_or_panic<S:ToString>(code:S, id_map:ast::IdMap, metadata:Metadata) -> Handle {
        let parser = parser::Parser::new_or_panic();
        let ast    = parser.parse(code.to_string(),id_map).unwrap().try_into().unwrap();
        Rc::new(Self::new(ast,metadata))
    }
}

//TODO[ao] add tests here!