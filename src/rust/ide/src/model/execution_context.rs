use crate::prelude::*;

use crate::double_representation::definition::DefinitionName;

use enso_protocol::language_server;
use json_rpc::error::RpcError;



pub type Id  = language_server::ContextId;
pub type DefinitionId = crate::double_representation::definition::Id;
pub type ExpressionId = ast::Id;

#[derive(Clone,Debug)]
pub struct StackItem {
    pub call       : ExpressionId,
    pub definition : DefinitionId,
}

#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Received invalid response from LanguageServer: {:?}",response)]
pub struct InvalidLanguageServerResponse<T:Debug+Send+Sync+'static> {
    response:T
}

#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="Pop on empty stack")]
pub struct PopOnEmptyStack {}

#[derive(Debug)]
pub struct ExecutionContext {
    id              : Id,
    module_path     : Rc<language_server::Path>,
    root_definition : DefinitionName,
    stack           : RefCell<Vec<StackItem>>,
    language_server : Rc<language_server::Connection>,
    logger          : Logger,
}

impl ExecutionContext {
    pub async fn create
    ( language_server : Rc<language_server::Connection>
    , module_path     : Rc<language_server::Path>
    , root_definition : DefinitionName
    ) -> FallibleResult<Self> {
        let logger = Logger::new("ExecutionContext");
        warning!(logger,"Creating Execution Context.");
        let id = language_server.client.create_execution_context().await?.context_id;
        warning!(logger,"Execution Context created. Id:{id}");
        let stack = default();
        let this  = Self {id,module_path,root_definition,stack,language_server,logger};
        this.push_root_frame().await?;
        warning!(this.logger,"Pushed root frame");
        Ok(this)
    }

    async fn push_root_frame(&self) -> FallibleResult<()> {
        let method_pointer = language_server::MethodPointer {
            file            : self.module_path.deref().clone(),
            defined_on_type : "Main".to_string(), //TODO[ao]
            name            : self.root_definition.name.item.clone(),
        };
        let this_argument_expression        = default();
        let positional_arguments_expression = default();
        let call = language_server::ExplicitCall {method_pointer,this_argument_expression,
            positional_arguments_expression};
        let frame = language_server::StackItem::ExplicitCall(call);
        self.language_server.push_execution_context(self.id,frame).await?;
        Ok(())
    }

    pub fn push(&self, stack_item:StackItem) -> impl Future<Output=Result<(),RpcError>> {
        let call = language_server::LocalCall{
            expression_id : stack_item.call
        };
        let frame = language_server::StackItem::LocalCall(call);
        self.stack.borrow_mut().push(stack_item);
        self.language_server.push_execution_context(self.id,frame)
    }

    pub async fn pop(&self) -> FallibleResult<()> {
        let stack_item = self.stack.borrow_mut().pop();
        if stack_item.is_some() {
            self.language_server.pop_execution_context(self.id).await?;
            Ok(())
        } else {
            Err(PopOnEmptyStack{}.into())
        }
    }

    pub fn new_mock(path:language_server::Path, root_def:DefinitionName) -> Self {
        ExecutionContext {
            id              : Id::new_v4(),
            module_path     : Rc::new(path),
            root_definition : root_def,
            stack           : default(),
            language_server : language_server::Connection::new_mock_rc(default()),
            logger          : Logger::new("ExecuctionContext mock"),
        }
    }
}

impl Drop for ExecutionContext {
    fn drop(&mut self) {
        let id     = self.id;
        let ls     = self.language_server.clone_ref();
        let logger = self.logger.clone_ref();
        executor::global::spawn(async move {
            let result = ls.client.destroy_execution_context(id).await;
            if result.is_err() {
                error!(logger,"Error when destroying Execution Context: {result:?}.");
            }
        });
    }
}
