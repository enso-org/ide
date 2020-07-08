use crate::prelude::*;
use enso_protocol::language_server::{MethodPointer, ExpressionValuesComputed};
use crate::model::execution_context::{LocalCall, VisualizationId, AttachedVisualization, ComputedValueInfoRegistry, Visualization, VisualizationUpdateData};
use websocket::futures::unsync::mpsc::UnboundedReceiver;


// ==============
// === Errors ===
// ==============

/// Error then trying to pop stack item on ExecutionContext when there only root call remains.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Tried to pop an entry point.")]
pub struct PopOnEmptyStack();

/// Error when using an Id that does not correspond to any known visualization.
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Tried to use incorrect visualization Id: {}.",_0)]
pub struct InvalidVisualizationId(VisualizationId);



// =============
// === Model ===
// =============

/// Execution Context Model.
///
/// This model reflects the state of the execution context in Language Server.
/// It consists of the root call (which is a direct call of some function
/// definition), stack of function calls (see `StackItem` definition and docs) and a list of
/// active visualizations. It can also cache all computed values and types of various expression
/// for the context.
///
/// It implements internal mutability pattern, so it may be shared between different
/// controllers.
#[derive(Debug)]
pub struct ExecutionContext {
    logger:Logger,
    /// A name of definition which is a root call of this context.
    pub entry_point:MethodPointer,
    /// Local call stack.
    stack:RefCell<Vec<LocalCall>>,
    /// Set of active visualizations.
    visualizations: RefCell<HashMap<VisualizationId,AttachedVisualization>>,
    /// Storage for information about computed values (like their types).
    pub computed_value_info_registry: ComputedValueInfoRegistry,
}

impl ExecutionContext {
    /// Create new execution context
    pub fn new(logger:impl Into<Logger>, entry_point:MethodPointer) -> Self {
        let logger                       = logger.into();
        let stack                        = default();
        let visualizations               = default();
        let computed_value_info_registry = default();
        Self {logger,entry_point,stack,visualizations, computed_value_info_registry }
    }


}

impl model::execution_context::API for ExecutionContext {
    fn current_method(&self) -> MethodPointer {
        if let Some(top_frame) = self.stack.borrow().last() {
            top_frame.definition.clone()
        } else {
            self.entry_point.clone()
        }
    }

    fn visualization_info(&self, id:VisualizationId) -> FallibleResult<Visualization> {
        let err = || InvalidVisualizationId(id).into();
        self.visualizations.borrow_mut().get(&id).map(|v| v.visualization.clone()).ok_or_else(err)
    }

    fn all_visualizations_info(&self) -> Vec<Visualization> {
        self.visualizations.borrow_mut().values().map(|v| v.visualization.clone()).collect()
    }

    fn active_visualizations(&self) -> Vec<VisualizationId> {
        self.visualizations.borrow().keys().copied().collect_vec()
    }

    fn push(&self, stack_item:LocalCall)  {
        self.stack.borrow_mut().push(stack_item);
        self.computed_value_info_registry.clear();
    }

    fn pop(&self) -> FallibleResult<LocalCall> {
        let ret = self.stack.borrow_mut().pop().ok_or_else(PopOnEmptyStack)?;
        self.computed_value_info_registry.clear();
        Ok(ret)
    }

    fn attach_visualization
    (&self, visualization:Visualization)
    -> futures::channel::mpsc::UnboundedReceiver<VisualizationUpdateData> {
        let id                       = visualization.id;
        let (update_sender,receiver) = futures::channel::mpsc::unbounded();
        let visualization            = AttachedVisualization {visualization,update_sender};
        info!(self.logger,"Inserting to the registry: {id}.");
        self.visualizations.borrow_mut().insert(id,visualization);
        receiver
    }

    fn detach_visualization(&self, id:VisualizationId) -> FallibleResult<Visualization> {
        let err = || InvalidVisualizationId(id);
        info!(self.logger,"Removing from the registry: {id}.");
        Ok(self.visualizations.borrow_mut().remove(&id).ok_or_else(err)?.visualization)
    }

    fn dispatch_visualization_update
    (&self, visualization_id:VisualizationId, data:VisualizationUpdateData) -> FallibleResult<()> {
        if let Some(visualization) = self.visualizations.borrow_mut().get(&visualization_id) {
            // TODO [mwu] Should we consider detaching the visualization if the view has dropped the
            //   channel's receiver? Or we need to provide a way to re-establish the channel.
            let _ = visualization.update_sender.unbounded_send(data);
            debug!(self.logger,"Sending update data to the visualization {visualization_id}.");
            Ok(())
        } else {
            error!(self.logger,"Failed to dispatch update to visualization {visualization_id}. \
            Failed to found such visualization.");
            Err(InvalidVisualizationId(visualization_id).into())
        }
    }
}