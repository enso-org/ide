use crate::prelude::*;

use crate::model::execution_context::VisualizationUpdateData;
use crate::model::execution_context;
use crate::model::module;
use crate::model::SuggestionDatabase;
use crate::model::traits::*;

use enso_protocol::binary;
use enso_protocol::binary::message::VisualisationContext;
use enso_protocol::language_server;
use enso_protocol::language_server::CapabilityRegistration;
use enso_protocol::language_server::MethodPointer;
use parser::Parser;



// =================================
// === ExecutionContextsRegistry ===
// =================================

// === Errors ===

#[allow(missing_docs)]
#[derive(Clone,Copy,Debug,Fail)]
#[fail(display="No execution context with id {} was found in the registry.", _0)]
pub struct NoSuchExecutionContext(execution_context::Id);


// === Aliases ===

type ExecutionContextWeakMap = WeakValueHashMap<execution_context::Id,Weak<execution_context::Synchronized>>;


// === Definition ===

/// Stores the weak handles to the synchronized execution context models.
/// Implements dispatching the visualization updates.
#[derive(Clone,Debug,Default)]
pub struct ExecutionContextsRegistry(RefCell<ExecutionContextWeakMap>);

impl ExecutionContextsRegistry {
    /// Retrieve the execution context with given Id and calls the given function with it.
    ///
    /// Handles the error of context not being present in the registry.
    pub fn with_context<R>
    (&self, id:execution_context::Id, f:impl FnOnce(Rc<execution_context::Synchronized>) -> FallibleResult<R>)
     -> FallibleResult<R> {
        let ctx = self.0.borrow_mut().get(&id);
        let ctx = ctx.ok_or_else(|| NoSuchExecutionContext(id))?;
        f(ctx)
    }

    /// Route the visualization update into the appropriate execution context.
    pub fn dispatch_visualization_update
    (&self
    , context : VisualisationContext
    , data    : VisualizationUpdateData
    ) -> FallibleResult<()> {
        self.with_context(context.context_id, |ctx| {
            ctx.dispatch_visualization_update(context.visualization_id,data)
        })
    }

    /// Handles the update about expressions being computed.
    pub fn handle_expression_values_computed
    (&self, update:language_server::ExpressionValuesComputed) -> FallibleResult<()> {
        self.with_context(update.context_id, |ctx| {
            ctx.handle_expression_values_computed(update)
        })
    }

    /// Registers a new ExecutionContext. It will be eligible for receiving future updates routed
    /// through `dispatch_visualization_update`.
    pub fn insert(&self, context:Rc<execution_context::Synchronized>) {
        self.0.borrow_mut().insert(context.id(),context);
    }
}

/// Project Model.
///
///
#[allow(missing_docs)]
#[derive(Debug)]
pub struct Project {
    pub name                : ImString,
    pub language_server_rpc : Rc<language_server::Connection>,
    pub language_server_bin : Rc<binary::Connection>,
    pub visualization       : controller::Visualization,
    pub module_registry     : Rc<model::registry::Registry<module::Path,module::Synchronized>>,
    pub execution_contexts  : Rc<ExecutionContextsRegistry>,
    pub suggestion_db       : Rc<SuggestionDatabase>,
    pub parser              : Parser,
    pub logger              : Logger,
}

impl Project {
    /// Create a new project model.
    pub async fn new
    ( parent              : impl AnyLogger
      , language_server_rpc : Rc<language_server::Connection>
      , language_server_bin : Rc<binary::Connection>
      , name                : impl Str
    ) -> FallibleResult<Self> {
        let logger = Logger::sub(parent,"Project Controller");
        info!(logger,"Creating a model of project {name.as_ref()}");
        let binary_protocol_events  = language_server_bin.event_stream();
        let json_rpc_events         = language_server_rpc.events();
        let embedded_visualizations = default();
        let language_server         = language_server_rpc.clone();
        let visualization           = controller::Visualization::new(language_server,embedded_visualizations);
        let name                    = ImString::new(name.into());
        let module_registry         = default();
        let execution_contexts      = default();
        let parser                  = Parser::new_or_panic();
        let language_server         = &*language_server_rpc;
        let suggestion_db           = SuggestionDatabase::create_synchronized(language_server);
        let suggestion_db           = Rc::new(suggestion_db.await?);

        let ret = Project {name,module_registry,execution_contexts,parser,
            language_server_rpc,language_server_bin,logger,visualization,suggestion_db};

        let binary_handler = ret.binary_event_handler();
        crate::executor::global::spawn(binary_protocol_events.for_each(binary_handler));

        let json_rpc_handler = ret.json_event_handler();
        crate::executor::global::spawn(json_rpc_events.for_each(json_rpc_handler));

        ret.acquire_suggestion_db_updates_capability().await?;
        Ok(ret)
    }

    /// Create a project model from owned LS connections.
    pub fn from_connections
    ( parent              : impl AnyLogger
    , language_server_rpc : language_server::Connection
    , language_server_bin : binary::Connection
    , project_name        : impl Str
    ) -> impl Future<Output=FallibleResult<Self>> {
        let language_server_rpc = Rc::new(language_server_rpc);
        let language_server_bin = Rc::new(language_server_bin);
        Self::new(parent,language_server_rpc,language_server_bin,project_name)
    }

    /// Returns a handling function capable of processing updates from the binary protocol.
    /// Such function will be then typically used to process events stream from the binary
    /// connection handler.
    pub fn binary_event_handler
    (&self) -> impl Fn(enso_protocol::binary::Event) -> futures::future::Ready<()> {
        let logger                  = self.logger.clone_ref();
        let weak_execution_contexts = Rc::downgrade(&self.execution_contexts);
        move |event| {
            debug!(logger, "Received an event from the binary protocol: {event:?}");
            use enso_protocol::binary::client::Event;
            use enso_protocol::binary::Notification;
            match event {
                Event::Notification(Notification::VisualizationUpdate {context,data}) => {
                    let data = VisualizationUpdateData::new(data);
                    if let Some(execution_contexts) = weak_execution_contexts.upgrade() {
                        let result = execution_contexts.dispatch_visualization_update(context,data);
                        if let Err(error) = result {
                            error!(logger,"Failed to handle the visualization update: {error}.");
                        }
                    } else {
                        error!(logger,"Received a visualization update despite project being \
                        already dropped.");
                    }
                }
                Event::Closed => {
                    error!(logger,"Lost binary connection with the Language Server!");
                    // TODO [wmu]
                    //  The problem should be reported to the user and the connection should be
                    //  reestablished, see https://github.com/luna/ide/issues/145
                }
                Event::Error(error) => {
                    error!(logger,"Error emitted by the binary data connection: {error}.");
                }
            }
            futures::future::ready(())
        }
    }

    /// Returns a handling function capable of processing updates from the json-rpc protocol.
    /// Such function will be then typically used to process events stream from the json-rpc
    /// connection handler.
    pub fn json_event_handler
    (&self) -> impl Fn(enso_protocol::language_server::Event) -> futures::future::Ready<()> {
        // TODO [mwu]
        //  This handler for JSON-RPC notifications is very similar to the function above that handles
        //  binary protocol notifications. However, it is not practical to generalize them, as the
        //  underlying RPC handlers and their types are separate.
        //  This generalization should be reconsidered once the old JSON-RPC handler is phased out.
        //  See: https://github.com/luna/ide/issues/587
        let logger                  = self.logger.clone_ref();
        let weak_execution_contexts = Rc::downgrade(&self.execution_contexts);
        let weak_suggestion_db      = Rc::downgrade(&self.suggestion_db);
        move |event| {
            debug!(logger, "Received an event from the json-rpc protocol: {event:?}");
            use enso_protocol::language_server::Event;
            use enso_protocol::language_server::Notification;
            match event {
                Event::Notification(Notification::ExpressionValuesComputed(update)) => {
                    if let Some(execution_contexts) = weak_execution_contexts.upgrade() {
                        let result = execution_contexts.handle_expression_values_computed(update);
                        if let Err(error) = result {
                            error!(logger,"Failed to handle the expression values computed update: \
                            {error}.");
                        }
                    } else {
                        error!(logger,"Received a `ExpressionValuesComputed` update despite \
                        execution context being already dropped.");
                    }
                }
                Event::Notification(Notification::SuggestionDatabaseUpdate(update)) => {
                    if let Some(suggestion_db) = weak_suggestion_db.upgrade() {
                        suggestion_db.apply_update_event(update);
                    }
                }
                Event::Closed => {
                    error!(logger,"Lost JSON-RPC connection with the Language Server!");
                    // TODO [wmu]
                    //  The problem should be reported to the user and the connection should be
                    //  reestablished, see https://github.com/luna/ide/issues/145
                }
                Event::Error(error) => {
                    error!(logger,"Error emitted by the binary data connection: {error}.");
                }
                _ => {}
            }
            futures::future::ready(())
        }
    }

    fn acquire_suggestion_db_updates_capability(&self) -> impl Future<Output=json_rpc::Result<()>> {
        let capability = CapabilityRegistration::create_receives_suggestions_database_updates();
        self.language_server_rpc.acquire_capability(&capability.method,&capability.register_options)
    }

    fn load_module(&self, path:module::Path)
    -> impl Future<Output=FallibleResult<Rc<module::Synchronized>>> {
        let language_server = self.language_server_rpc.clone_ref();
        let parser          = self.parser.clone_ref();
        module::Synchronized::open(path,language_server,parser)
    }
}

impl model::project::API for Project {
    fn name(&self) -> ImString {
        self.name.clone_ref()
    }

    fn json_rpc(&self) -> Rc<language_server::Connection> {
        self.language_server_rpc.clone_ref()
    }

    fn binary_rpc(&self) -> Rc<binary::Connection> {
        self.language_server_bin.clone_ref()
    }

    fn parser(&self) -> &Parser {
        &self.parser
    }

    fn visualization(&self) -> &controller::Visualization {
        &self.visualization
    }

    fn module(&self, path: module::Path) -> BoxFuture<FallibleResult<model::Module>> {
        async move {
            info!(self.logger,"Obtaining module for {path}");
            let model_loader        = self.load_module(path.clone());
            let model:model::Module = self.module_registry.get_or_load(path,model_loader).await?;
            Ok(model)
        }.boxed_local()
    }

    fn create_execution_context
    (&self, root_definition:MethodPointer) -> BoxFuture<FallibleResult<model::ExecutionContext>> {
        async move {
            let logger  = &self.logger;
            let ls_rpc  = self.language_server_rpc.clone_ref();
            let context = execution_context::Synchronized::create(&logger,ls_rpc,root_definition);
            let context = Rc::new(context.await?);
            self.execution_contexts.insert(context.clone_ref());
            let context:model::ExecutionContext = context;
            Ok(context)
        }.boxed_local()
    }

    fn content_root_id(&self) -> Uuid {
        self.language_server_rpc.content_root()
    }
}

// =============
// === Tests ===
// =============

#[cfg(test)]
mod test {
    use super::*;

    use crate::constants::DEFAULT_PROJECT_NAME;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    use enso_protocol::types::Sha3_224;
    use enso_protocol::language_server::response;
    use json_rpc::expect_call;

    struct Fixture {
        test                 : TestWithLocalPoolExecutor,
        project              : Project,
        binary_events_sender : futures::channel::mpsc::UnboundedSender<binary::client::Event>,
    }

    impl Fixture {
        fn new
        ( setup_mock_json   : impl FnOnce(&mut language_server::MockClient)
        , setup_mock_binary : impl FnOnce(&mut enso_protocol::binary::MockClient)
        ) -> Self {
            let mut test          = TestWithLocalPoolExecutor::set_up();
            let mut json_client   = language_server::MockClient::default();
            let mut binary_client = enso_protocol::binary::MockClient::default();

            let (binary_events_sender,binary_events) = futures::channel::mpsc::unbounded();
            binary_client.expect_event_stream().return_once(|| {
                binary_events.boxed_local()
            });

            let initial_suggestions_db = language_server::response::GetSuggestionDatabase {
                entries: vec![],
                current_version: 0
            };
            expect_call!(json_client.get_suggestions_database() => Ok(initial_suggestions_db));
            let capability_reg = CapabilityRegistration::create_receives_suggestions_database_updates();
            let method         = capability_reg.method;
            let options        = capability_reg.register_options;
            expect_call!(json_client.acquire_capability(method,options) => Ok(()));

            setup_mock_json(&mut json_client);
            setup_mock_binary(&mut binary_client);
            let json_connection   = language_server::Connection::new_mock(json_client);
            let binary_connection = binary::Connection::new_mock(binary_client);
            let logger            = Logger::default();
            let project_fut       = model::Project::from_connections(logger,json_connection,
                binary_connection,DEFAULT_PROJECT_NAME).boxed_local();
            let project = test.expect_completion(project_fut);
            Fixture {test,project,binary_events_sender}
        }
    }

    // #[wasm_bindgen_test]
    // fn obtain_module_controller() {
    //     let path         = module::Path::from_mock_module_name("TestModule");
    //     let another_path = module::Path::from_mock_module_name("TestModule2");
    //     let Fixture{mut test,project,..} = Fixture::new(|ls_json| {
    //         mock_calls_for_opening_text_file(ls_json,path.file_path().clone(),"2+2");
    //         mock_calls_for_opening_text_file(ls_json,another_path.file_path().clone(),"22+2");
    //     }, |_|{});
    //
    //     test.run_task(async move {
    //         use controller::Module;
    //         let log            = Logger::new("Test");
    //         let module         = Module::new(&log,path.clone(),&project).await.unwrap();
    //         let same_module    = Module::new(&log,path.clone(),&project).await.unwrap();
    //         let another_module = Module::new(&log,another_path.clone(),&project).await.unwrap();
    //
    //         assert_eq!(path,         module.model.path);
    //         assert_eq!(another_path, another_module.model.path);
    //         assert!(Rc::ptr_eq(&module.model, &same_module.model));
    //     });
    // }

    fn mock_calls_for_opening_text_file
    (client:&language_server::MockClient, path:language_server::Path, content:&str) {
        let content          = content.to_string();
        let current_version  = Sha3_224::new(content.as_bytes());
        let write_capability = CapabilityRegistration::create_can_edit_text_file(path.clone());
        let write_capability = Some(write_capability);
        let open_response    = response::OpenTextFile {content,current_version,write_capability};
        expect_call!(client.open_text_file(path=path.clone()) => Ok(open_response));
        client.expect.apply_text_file_edit(|_| Ok(()));
        expect_call!(client.close_text_file(path) => Ok(()));
    }
}