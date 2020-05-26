//! This module consists of all structures describing Execution Context.

use crate::prelude::*;

use crate::double_representation::definition::DefinitionName;

use enso_protocol::language_server;
use enso_protocol::language_server::VisualisationConfiguration;

use std::collections::HashMap;
use uuid::Uuid;



// ===============
// === Aliases ===
// ===============

/// An identifier of called definition in module.
pub type DefinitionId = crate::double_representation::definition::Id;

/// An identifier of expression.
pub type ExpressionId = ast::Id;



// ===============================
// === VisualizationUpdateData ===
// ===============================

/// An update data that notification receives from the interpreter. Owns the binary data generated
/// for visualization by the Language Server.
///
/// Binary data can be accessed through `Deref` or `AsRef` implementations.
///
/// The inner storage is private and users should not make any assumptions about it.
#[derive(Clone,Debug)]
pub struct VisualizationUpdateData(Vec<u8>);

impl VisualizationUpdateData {
    /// Wraps given vector with binary data into a visualization update data.
    pub fn new(data:Vec<u8>) -> VisualizationUpdateData {
        VisualizationUpdateData(data)
    }
}

impl AsRef<[u8]> for VisualizationUpdateData {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Deref for VisualizationUpdateData {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}



// ==============
// === Errors ===
// ==============

/// Error then trying to pop stack item on ExecutionContext when there only root call remains.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Tried to pop an entry point.")]
pub struct PopOnEmptyStack();

/// Error when using an Id that does not correspond to any known visualization.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Tried to use incorrect visualization Id.")]
pub struct InvalidVisualizationId();



// =================
// === StackItem ===
// =================

/// A specific function call occurring within another function's definition body.
///
/// This is a single item in ExecutionContext stack.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct LocalCall {
    /// An expression being a call.
    pub call       : ExpressionId,
    /// A definition of function called in `call` expression.
    pub definition : DefinitionId,
}



// =====================
// === Visualization ===
// =====================

/// Unique Id for visualization.
pub type VisualizationId = Uuid;

/// Visualization marker for specific Ast node with preprocessing function.
#[derive(Clone,Debug)]
pub struct Visualization {
    /// Unique identifier of this visualization.
    pub id: VisualizationId,
    /// Node that is to be visualized.
    pub ast_id: ExpressionId,
    /// An enso lambda that will transform the data into expected format, e.g. `a -> a.json`.
    pub expression: String,
}

impl Visualization {
    /// Creates a `VisualisationConfiguration` that is used in communication with language server.
    pub fn config
    (&self, execution_context_id:Uuid, visualisation_module:String) -> VisualisationConfiguration {
        let expression = self.expression.clone();
        VisualisationConfiguration{execution_context_id,visualisation_module,expression}
    }
}

/// An identifier of ExecutionContext.
pub type Id  = language_server::ContextId;



// =============================
// === AttachedVisualization ===
// =============================

/// The information about active visualization. Includes the channel endpoint allowing sending
/// the visualization update's data to the visualization's attacher (presumably the view).
#[derive(Clone,Debug)]
pub struct AttachedVisualization {
    visualization : Visualization,
    update_sender : futures::channel::mpsc::UnboundedSender<VisualizationUpdateData>,
}



// =============
// === Model ===
// =============

/// Execution Context Model.
///
/// The execution context consists of the root call (which is a direct call of some function
/// definition), stack of function calls (see `StackItem` definition and docs) and a list of
/// active visualizations.
///
/// It implements internal mutability pattern, so the state may be shared between different
/// controllers.
#[derive(Debug)]
pub struct ExecutionContext {
    /// A name of definition which is a root call of this context.
    pub entry_point:DefinitionName,
    /// Local call stack.
    stack:RefCell<Vec<LocalCall>>,
    /// Set of active visualizations.
    visualizations: RefCell<HashMap<VisualizationId,AttachedVisualization>>,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(entry_point:DefinitionName) -> Self {
        let stack          = default();
        let visualizations = default();
        Self {entry_point,stack,visualizations}
    }

    /// Push a new stack item to execution context.
    pub fn push(&self, stack_item:LocalCall) {
        self.stack.borrow_mut().push(stack_item);
    }

    /// Pop the last stack item from this context. It returns error when only root call
    /// remains.
    pub fn pop(&self) -> FallibleResult<()> {
        self.stack.borrow_mut().pop().ok_or_else(PopOnEmptyStack)?;
        Ok(())
    }

    /// Attaches a new visualization for current execution context.
    ///
    /// Returns a stream of visualization update data received from the server.
    pub fn attach_visualization
    (&self, visualization:Visualization) -> impl Stream<Item=VisualizationUpdateData> {
        let id                       = visualization.id;
        let (update_sender,receiver) = futures::channel::mpsc::unbounded();
        let visualization            = AttachedVisualization {visualization,update_sender};
        self.visualizations.borrow_mut().insert(id,visualization);
        receiver
    }

    /// Detaches visualization from current execution context.
    pub fn detach_visualization(&self, id:&VisualizationId) -> FallibleResult<Visualization> {
        let err = InvalidVisualizationId;
        Ok(self.visualizations.borrow_mut().remove(id).ok_or_else(err)?.visualization)
    }

    /// Get an iterator over stack items.
    ///
    /// Because this struct implements _internal mutability pattern_, the stack can actually change
    /// during iteration. It should not panic, however might give an unpredictable result.
    pub fn stack_items<'a>(&'a self) -> impl Iterator<Item=LocalCall> + 'a {
        let stack_size = self.stack.borrow().len();
        (0..stack_size).filter_map(move |i| self.stack.borrow().get(i).cloned())
    }

    /// Dispatches the visualization update data (typically received from as LS binary notification)
    /// to the respective's visualization update channel.
    pub fn dispatch_update(&self, visualization_id:VisualizationId, data:VisualizationUpdateData) {
        if let Some(visualization) = self.visualizations.borrow_mut().get(&visualization_id) {
            // TODO [mwu] Should we consider detaching the visualization if the view has dropped the
            //   channel's receiver? Or we need to provide a way to re-establish the channel.
            let _ = visualization.update_sender.unbounded_send(data);
        }
    }
}
