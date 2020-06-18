//! A ExecutionContext model which is synchronized with LanguageServer state.

use crate::prelude::*;

use crate::double_representation::definition::DefinitionName;
use crate::model::execution_context::ComputedValueInfoRegistry;
use crate::model::execution_context::LocalCall;
use crate::model::execution_context::Visualization;
use crate::model::execution_context::VisualizationUpdateData;
use crate::model::execution_context::VisualizationId;

use enso_protocol::language_server;
use enso_protocol::language_server::ExpressionValuesComputed;
#[cfg(test)]
use enso_protocol::language_server::connection::ShareableConnection;
use json_rpc::error::RpcError;



// ==========================
// === Synchronized Model ===
// ==========================

/// An ExecutionContext model synchronized with LanguageServer. It will be automatically removed
/// from LS once dropped.
#[derive(Debug)]
pub struct ExecutionContext {
    id              : model::execution_context::Id,
    model           : model::ExecutionContext,
    module_path     : Rc<model::module::Path>,
    language_server : Rc<language_server::Connection>,
    logger          : Logger,
}

impl ExecutionContext {
    /// The unique identifier of this execution context.
    pub fn id(&self) -> model::execution_context::Id {
        self.id
    }

    /// Create new ExecutionContext. It will be created in LanguageServer and the ExplicitCall
    /// stack frame will be pushed.
    ///
    /// NOTE: By itself this execution context will not be able to receive any updates from the
    /// language server.
    pub fn create
	( parent          : impl AnyLogger
    , language_server : Rc<language_server::Connection>
    , module_path     : Rc<model::module::Path>
    , root_definition : DefinitionName
    ) -> impl Future<Output=FallibleResult<Self>> {
        let logger = Logger::sub(&parent,"ExecutionContext");
        async move {
            info!(logger, "Creating.");
            let id     = language_server.client.create_execution_context().await?.context_id;
            let logger = Logger::sub(&parent,iformat!{"ExecutionContext {id}"});
            let model  = model::ExecutionContext::new(&logger,root_definition);
            info!(logger, "Created. Id:{id}");
            let this = Self { id, module_path, model, language_server, logger };
            this.push_root_frame().await?;
            info!(this.logger, "Pushed root frame");
            Ok(this)
        }
    }

    fn push_root_frame(&self) -> impl Future<Output=FallibleResult<()>> {
        let method_pointer = language_server::MethodPointer {
            file            : self.module_path.file_path().clone(),
            defined_on_type : self.module_path.module_name().to_string(),
            name            : self.model.entry_point.name.item.clone(),
        };
        let this_argument_expression         = default();
        let positional_arguments_expressions = default();

        let call = language_server::ExplicitCall {method_pointer,this_argument_expression,
            positional_arguments_expressions};
        let frame  = language_server::StackItem::ExplicitCall(call);
        let result = self.language_server.push_to_execution_context(&self.id,&frame);
        result.map(|res| res.map_err(|err| err.into()))
    }

    /// Push a new stack item to execution context.
    pub fn push(&self, stack_item: LocalCall) -> impl Future<Output=Result<(),RpcError>> {
        let expression_id = stack_item.call;
        let call          = language_server::LocalCall{expression_id};
        let frame         = language_server::StackItem::LocalCall(call);
        self.model.push(stack_item);
        self.language_server.push_to_execution_context(&self.id,&frame)
    }

    /// Pop the last stack item from this context. It returns error when only root call
    /// remains.
    pub async fn pop(&self) -> FallibleResult<()> {
        self.model.pop()?;
        self.language_server.pop_from_execution_context(&self.id).await?;
        Ok(())
    }

    /// Attach a new visualization for current execution context.
    ///
    /// Returns a stream of visualization update data received from the server.
    pub async fn attach_visualization
    (&self, vis:Visualization) -> FallibleResult<impl Stream<Item=VisualizationUpdateData>> {
        // Note: [mwu]
        //  We must register our visualization in the model first, because Language server can send
        //  us visualization updates through the binary socket before confirming that visualization
        //  has been successfully attached.
        let config = vis.config(self.id);
        let stream = self.model.attach_visualization(vis.clone());
        let result = self.language_server.attach_visualisation(&vis.id, &vis.ast_id, &config).await;
        if let Err(e) = result {
            self.model.detach_visualization(&vis.id)?;
            Err(e.into())
        } else {
            Ok(stream)
        }
    }

    /// Detach visualization from current execution context.
    pub async fn detach_visualization(&self, id:&VisualizationId) -> FallibleResult<Visualization> {
        info!(self.logger,"Scheduling detaching visualization by id: {id}.");
        let vis    = self.model.detach_visualization(id)?;
        let vis_id = *id;
        let exe_id = self.id;
        let ast_id = vis.ast_id;
        let ls     = self.language_server.clone_ref();
        let logger = self.logger.clone_ref();
        executor::global::spawn(async move {
            info!(logger,"About to detach visualization by id: {vis_id}.");
            let result = ls.detach_visualisation(&exe_id,&vis_id,&ast_id).await;
            if result.is_err() {
                error!(logger,"Error when detaching node: {result:?}.");
            }
        });
        Ok(vis)
    }

    /// Dispatch the visualization update data (typically received from as LS binary notification)
    /// to the respective's visualization update channel.
    pub fn dispatch_visualization_update
    (&self, visualization_id:VisualizationId, data:VisualizationUpdateData) -> FallibleResult<()> {
        debug!(self.logger, "Dispatching visualization update through the context {self.id()}");
        self.model.dispatch_visualization_update(visualization_id,data)
    }

    /// Handles the update about expressions being computed.
    pub fn handle_expression_values_computed
    (&self, notification:ExpressionValuesComputed) -> FallibleResult<()> {
        self.model.handle_expression_values_computed(notification)
    }

    /// Access the registry of computed values information, like types or called method pointers.
    pub fn computed_value_info_registry(&self) -> &ComputedValueInfoRegistry {
        &self.model.computed_value_info_registry
    }


    /// Create a mock which does no call on `language_server` during construction.
    #[cfg(test)]
    pub fn new_mock
    ( id              : model::execution_context::Id
    , path            : model::module::Path
    , model           : model::ExecutionContext
    , language_server : impl ShareableConnection
    ) -> Self {
        let module_path     = Rc::new(path);
        let logger          = Logger::new("ExecuctionContext mock");
        let language_server = language_server.connection();
        ExecutionContext {id,model,module_path,language_server,logger}
    }
}

impl Drop for ExecutionContext {
    fn drop(&mut self) {
        let id     = self.id;
        let ls     = self.language_server.clone_ref();
        let logger = self.logger.clone_ref();
        executor::global::spawn(async move {
            let result = ls.client.destroy_execution_context(&id).await;
            if result.is_err() {
                error!(logger,"Error when destroying Execution Context: {result:?}.");
            }
        });
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
pub mod tests {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;
    use crate::model::module::QualifiedName as ModuleQualifiedName;

    use enso_protocol::language_server::CapabilityRegistration;
    use json_rpc::expect_call;
    use language_server::response;
    use utils::test::ExpectTuple;
    use utils::test::stream::StreamTestExt;

    trait ModelCustomizer = Fn(&mut model::ExecutionContext, &MockData) + 'static;
    //trait ContextProvider = FnOnce(&mut model::ExecutionContext, &MockData) + 'static;

    /// Set of data needed to create and operate mock execution context.
    #[derive(Clone,Derivative)]
    #[derivative(Debug)]
    pub struct MockData {
        pub module_path     : model::module::Path,
        pub context_id      : model::execution_context::Id,
        pub root_definition : DefinitionName,
        pub project_name    : String,
        #[derivative(Debug="ignore")]
        pub customize_model : Rc<dyn ModelCustomizer>,
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
                module_path     : model::module::Path::from_mock_module_name("Test"),
                root_definition : DefinitionName::new_plain("main"),
                project_name    : "MockProject".to_string(),
                customize_model : Rc::new(|_,_| {})
            }
        }

        /// Allows customizing initial state of the context's model.
        pub fn customize_model(&mut self, f:impl Fn(&mut model::ExecutionContext, &MockData) + 'static) {
            self.customize_model = Rc::new(f);
        }

        pub fn create_model(&self) -> model::ExecutionContext {
            let logger = Logger::new("MockExecutionContextModel");
            model::ExecutionContext::new(logger, self.root_definition.clone())
        }

        pub fn create_context(&self, mut ls:language_server::MockClient) -> ExecutionContext {
            self.context_provider(&mut ls)(ls.connection())
        }

        pub fn context_provider
        (&self, ls:&mut language_server::MockClient)
        -> impl FnOnce(Rc<language_server::Connection>) -> ExecutionContext {
            let id = self.context_id;
            expect_call!(ls.destroy_execution_context(id) => Ok(()));

            let data = self.clone();
            move |ls| {
                let mut model = data.create_model();
                (data.customize_model)(&mut model,&data);
                ExecutionContext::new_mock(data.context_id, data.module_path.clone(), model, ls)
            }
        }

        pub fn module_qualified_name(&self) -> ModuleQualifiedName {
            ModuleQualifiedName::from_path(&self.module_path,&self.project_name)
        }

        pub fn definition_id(&self) -> model::execution_context::DefinitionId {
            model::execution_context::DefinitionId::new_single_crumb(self.root_definition.clone())
        }

        pub fn main_method_pointer(&self) -> language_server::MethodPointer {
            language_server::MethodPointer {
                file            : self.module_path.file_path().clone(),
                defined_on_type : self.module_path.module_name().to_string(),
                name            : self.root_definition.to_string(),
            }
        }
    }

    #[test]
    fn creating_context() {
        let mock_data  = MockData::new();
        let context_id = mock_data.context_id;
        let ls_client  = language_server::MockClient::default();
        let can_modify =
            CapabilityRegistration::create_can_modify_execution_context(context_id);
        let receives_updates =
            CapabilityRegistration::create_receives_execution_context_updates(context_id);
        ls_client.expect.create_execution_context(move || Ok(response::CreateExecutionContext {
            context_id,can_modify,receives_updates,
        }));
        let root_frame = language_server::ExplicitCall {
            method_pointer                   : mock_data.main_method_pointer(),
            this_argument_expression         : None,
            positional_arguments_expressions : vec![]
        };
        let stack_item = language_server::StackItem::ExplicitCall(root_frame);
        expect_call!(ls_client.push_to_execution_context(context_id,stack_item) => Ok(()));
        expect_call!(ls_client.destroy_execution_context(context_id) => Ok(()));
        ls_client.require_all_calls();
        let connection = language_server::Connection::new_mock_rc(ls_client);

        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            let logger  = Logger::default();
            let path    = Rc::new(mock_data.module_path);
            let def     = mock_data.root_definition;
            let context = ExecutionContext::create(logger,connection,path.clone_ref(),def);
            let context = context.await.unwrap();
            assert_eq!(context_id             , context.id);
            assert_eq!(path                   , context.module_path);
            assert_eq!(Vec::<LocalCall>::new(), context.model.stack_items().collect_vec());
        })
    }

    #[test]
    fn pushing_stack_item() {
        let mock_data           = MockData::new();
        let ls                  = language_server::MockClient::default();
        let id                  = mock_data.context_id;
        let expression_id       = model::execution_context::ExpressionId::new_v4();
        let expected_call_frame = language_server::LocalCall{expression_id};
        let expected_stack_item = language_server::StackItem::LocalCall(expected_call_frame);
        expect_call!(ls.push_to_execution_context(id,expected_stack_item) => Ok(()));
        let context = mock_data.create_context(ls);
        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            let item = LocalCall {
                call       : expression_id,
                definition : mock_data.definition_id(),
            };
            context.push(item.clone()).await.unwrap();
            assert_eq!((item,), context.model.stack_items().expect_tuple());
        })
    }

    #[test]
    fn popping_stack_item() {
        let mock_data = MockData {
            customize_model : Rc::new(|model,data| {
                let item = LocalCall {
                    call       : model::execution_context::ExpressionId::new_v4(),
                    definition : data.definition_id(),
                };
                model.push(item);
            }),
            ..default()
        };

        let ls        = language_server::MockClient::default();
        let id        = mock_data.context_id;
        expect_call!(ls.pop_from_execution_context(id) => Ok(()));
        let context  = mock_data.create_context(ls);

        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            context.pop().await.unwrap();
            assert_eq!(Vec::<LocalCall>::new(), context.model.stack_items().collect_vec());
            // Pop on empty stack.
            assert!(context.pop().await.is_err());
        })
    }

    #[test]
    fn attaching_visualizations_and_notifying() {
        let mock_data = MockData::new();
        let ls        = language_server::MockClient::default();
        let vis       = Visualization {
            id                   : model::execution_context::VisualizationId::new_v4(),
            ast_id               : model::execution_context::ExpressionId::new_v4(),
            expression           : "".to_string(),
            visualisation_module : mock_data.module_qualified_name(),
        };
        let exe_id = mock_data.context_id;
        let vis_id = vis.id;
        let ast_id = vis.ast_id;
        let config = vis.config(exe_id);

        expect_call!(ls.attach_visualisation(vis_id,ast_id,config) => Ok(()));
        expect_call!(ls.detach_visualisation(exe_id,vis_id,ast_id) => Ok(()));

        let context   = mock_data.create_context(ls);

        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async move {
            let wrong_id   = model::execution_context::VisualizationId::new_v4();
            let events     = context.attach_visualization(vis.clone()).await.unwrap();
            let mut events = events.boxed_local();
            events.expect_pending();

            let update = VisualizationUpdateData::new(vec![1,2,3]);
            context.dispatch_visualization_update(vis.id,update.clone()).unwrap();
            assert_eq!(events.expect_next(),update);

            events.expect_pending();
            let other_vis_id = VisualizationId::new_v4();
            context.dispatch_visualization_update(other_vis_id,update.clone()).unwrap_err();
            events.expect_pending();
            assert!(context.detach_visualization(&wrong_id).await.is_err());
            events.expect_pending();
            assert!(context.detach_visualization(&vis.id).await.is_ok());
            events.expect_terminated();
            assert!(context.detach_visualization(&vis.id).await.is_err());
            context.dispatch_visualization_update(vis.id,update.clone()).unwrap_err();
        });
    }
}
