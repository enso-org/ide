use crate::prelude::*;
use crate::model::execution_context::ExecutionContext;

#[derive(Clone,CloneRef,Debug)]
pub struct Handle {
    pub graph         : controller::Graph,
    execution_ctx : Rc<ExecutionContext>,
}

impl Handle {
    pub fn new(graph:controller::Graph, execution_ctx:ExecutionContext) -> Self {
        let execution_ctx = Rc::new(execution_ctx);
        Handle{graph,execution_ctx}
    }

    //TODO[ao] Here goes the methods requiring ContextId
}