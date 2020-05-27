//! A module with Executed Graph Controller.
//!
//! This controller provides operations on a specific graph with some execution context - these
//! operations usually involves retrieving values on nodes: that's are i.e. operations on
//! visualisations, retrieving types on ports, etc.
use crate::prelude::*;

use crate::model::execution_context::Visualization;
use crate::model::execution_context::VisualizationId;
use crate::model::execution_context::VisualizationUpdateData;
use crate::model::synchronized::ExecutionContext;

/// Handle providing executed graph controller interface.
#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    /// A handle to basic graph operations.
    pub graph     : controller::Graph,
    execution_ctx : Rc<ExecutionContext>,
}

impl Handle {
    /// Create handle for given graph and execution context.
    ///
    /// This takes ownership of execution context which will be shared between all copies of this
    /// handle; when all copies will be dropped, the execution context will be dropped as well
    /// (and will then removed from LanguageServer).
    pub fn new(graph:controller::Graph, execution_ctx:Rc<ExecutionContext>) -> Self {
        Handle{graph,execution_ctx}
    }

    /// See `attach_visualization` in `ExecutionContext`.
    pub async fn attach_visualization
    (&self, visualization:Visualization)
    -> FallibleResult<impl Stream<Item=VisualizationUpdateData>> {
        self.execution_ctx.attach_visualization(visualization).await
    }

    /// See `detach_visualization` in `ExecutionContext`.
    pub async fn detach_visualization(&self, id:&VisualizationId) -> FallibleResult<Visualization> {
        self.execution_ctx.detach_visualization(id).await
    }

    // TODO [mwu] Here goes the type/short_rep value access API
}

impl Deref for Handle {
    type Target = controller::Graph;

    fn deref(&self) -> &Self::Target {
        &self.graph
    }
}
