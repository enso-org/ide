//! Module with plain Execution Context Model (without any synchronization).

use crate::prelude::*;

use crate::model::execution_context::AttachedVisualization;
use crate::model::execution_context::ComputedValueInfoRegistry;
use crate::model::execution_context::LocalCall;
use crate::model::execution_context::Visualization;
use crate::model::execution_context::VisualizationId;
use crate::model::execution_context::VisualizationUpdateData;

use enso_protocol::language_server::MethodPointer;
use futures::future::LocalBoxFuture;


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

    /// Push a new stack item to execution context.
    ///
    /// This function shadows the asynchronous version from API trait.
    pub fn push(&self, stack_item:LocalCall)  {
        self.stack.borrow_mut().push(stack_item);
        self.computed_value_info_registry.clear();
    }

    /// Pop the last stack item from this context. It returns error when only root call remains.
    ///
    /// This function shadows the asynchronous version from API trait.
    pub fn pop(&self) -> FallibleResult<LocalCall> {
        let ret = self.stack.borrow_mut().pop().ok_or_else(PopOnEmptyStack)?;
        self.computed_value_info_registry.clear();
        Ok(ret)
    }

    /// Attach a new visualization for current execution context. Returns a stream of visualization
    /// update data received from the server.
    ///
    /// This function shadows the asynchronous version from API trait.
    pub fn attach_visualization
    (&self, visualization:Visualization)
     -> futures::channel::mpsc::UnboundedReceiver<VisualizationUpdateData> {
        let id                       = visualization.id;
        let (update_sender,receiver) = futures::channel::mpsc::unbounded();
        let visualization            = AttachedVisualization {visualization,update_sender};
        info!(self.logger,"Inserting to the registry: {id}.");
        self.visualizations.borrow_mut().insert(id,visualization);
        receiver
    }

    /// Detach the visualization from this execution context.
    ///
    /// This function shadows the asynchronous version from API trait.
    pub fn detach_visualization
    (&self, id:VisualizationId) -> FallibleResult<Visualization> {
        let err = || InvalidVisualizationId(id);
        info!(self.logger,"Removing from the registry: {id}.");
        let removed = self.visualizations.borrow_mut().remove(&id).ok_or_else(err)?;
        Ok(removed.visualization)
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

    fn computed_value_info_registry(&self) -> &ComputedValueInfoRegistry {
        &self.computed_value_info_registry
    }

    fn stack_items<'a>(&'a self) -> Box<dyn Iterator<Item=LocalCall> + 'a> {
        let stack_size = self.stack.borrow().len();
        Box::new((0..stack_size).filter_map(move |i| self.stack.borrow().get(i).cloned()))
    }

    fn push(&self, stack_item:LocalCall) -> LocalBoxFuture<'_, FallibleResult<()>> {
        self.push(stack_item);
        futures::future::ready(Ok(())).boxed_local()
    }

    fn pop(&self) -> LocalBoxFuture<'_, FallibleResult<LocalCall>> {
        futures::future::ready(self.pop()).boxed_local()
    }

    fn attach_visualization
    (&self, visualization:Visualization)
    -> LocalBoxFuture<FallibleResult<futures::channel::mpsc::UnboundedReceiver<VisualizationUpdateData>>> {
        futures::future::ready(Ok(self.attach_visualization(visualization))).boxed_local()
    }

    fn detach_visualization
    (&self, id:VisualizationId) -> LocalBoxFuture<'_, FallibleResult<Visualization>> {
        futures::future::ready(self.detach_visualization(id)).boxed_local()
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



#[cfg(test)]
pub mod test {
    use super::*;

    use crate::double_representation::definition::DefinitionName;

    #[derive(Clone,Derivative)]
    #[derivative(Debug)]
    pub struct MockData {
        pub module_path     : model::module::Path,
        pub context_id      : model::execution_context::Id,
        pub root_definition : DefinitionName,
        pub project_name    : String,
    }

    impl Default for MockData {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockData {
        pub fn new() -> MockData {
            MockData {
                context_id      : model::execution_context::Id::new_v4(),
                module_path     : crate::test::mock::data::module_path(),
                root_definition : crate::test::mock::data::definition_name(),
                project_name    : crate::test::mock::data::PROJECT_NAME.to_owned(),
            }
        }

        pub fn module_qualified_name(&self) -> model::module::QualifiedName {
            self.module_path.qualified_module_name(&self.project_name)
        }

        pub fn definition_id(&self) -> model::execution_context::DefinitionId {
            model::execution_context::DefinitionId::new_single_crumb(self.root_definition.clone())
        }

        pub fn main_method_pointer(&self) -> MethodPointer {
            MethodPointer {
                file            : self.module_path.file_path().clone(),
                defined_on_type : self.module_path.module_name().to_string(),
                name            : self.root_definition.to_string(),
            }
        }

        pub fn create(&self) -> ExecutionContext {
            let logger = Logger::new("Mocked Execution Context");
            ExecutionContext::new(logger,self.main_method_pointer())
        }
    }
}
