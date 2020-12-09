//! A Wrapper for module which synchronizes opening/closing and all changes with Language Server.

use crate::prelude::*;

use crate::double_representation::definition::DefinitionInfo;
use crate::double_representation::graph::Id;
use crate::model::module::API;
use crate::model::module::NotificationKind;
use crate::model::module::Content;
use crate::model::module::Notification;
use crate::model::module::NodeMetadata;
use crate::model::module::Path;

use ast::IdMap;
use data::text::TextChange;
use data::text::TextLocation;
use enso_protocol::types::Sha3_224;
use enso_protocol::language_server;
use enso_protocol::language_server::TextEdit;
use flo_stream::Subscriber;
use parser::api::SourceFile;
use parser::Parser;



// =======================
// === Content Summary ===
// =======================

/// The minimal information about module's content, required to do properly invalidation of opened
/// module in Language Server.
#[derive(Clone,Debug,Eq,PartialEq)]
struct ContentSummary {
    digest      : Sha3_224,
    end_of_file : TextLocation,
}

impl ContentSummary {
    fn new(text:&str) -> Self {
        Self {
            digest      : Sha3_224::new(text.as_bytes()),
            end_of_file : TextLocation::at_document_end(text)
        }
    }
}

/// The information about module's content. In addition to minimal summery defined in
/// `ContentSummary` it adds information about sections, what enables efficient updates after code
/// and metadata changes.
#[derive(Clone,Debug,Eq,PartialEq,Shrinkwrap)]
struct ParsedContentSummary {
    #[shrinkwrap(main_field)]
    summary  : ContentSummary,
    code     : Range<TextLocation>,
    id_map   : Range<TextLocation>,
    metadata : Range<TextLocation>,
}

impl ParsedContentSummary {
    /// Get summary from `SourceFile`.
    fn from_source(source:&SourceFile) -> Self {
        ParsedContentSummary {
            summary     : ContentSummary::new(&source.content),
            code        : TextLocation::convert_byte_range(&source.content,&source.code),
            id_map      : TextLocation::convert_byte_range(&source.content,&source.id_map),
            metadata    : TextLocation::convert_byte_range(&source.content,&source.metadata),
        }
    }
}

/// The information about state of the module currently held in LanguageServer.
#[derive(Clone,Debug)]
enum LanguageServerContent {
    /// The content is synchronized with our module state after last fully handled notification.
    Synchronized(ParsedContentSummary),
    /// The content is not synchronized with our module state after last fully handled notification,
    /// probably due to connection error when sending update.
    Desynchronized(ContentSummary)
}

impl LanguageServerContent {
    fn summary(&self) -> &ContentSummary {
        match self {
            LanguageServerContent::Synchronized(content)   => &content.summary,
            LanguageServerContent::Desynchronized(content) => content,
        }
    }
}



// ===========================
// === Synchronized Module ===
// ===========================

/// A Module which state is synchronized with Language Server using its textual API.
///
/// This struct owns  `model::Module`, load the state during creation and updates LS about all
/// changes done to it. On drop the module is closed in Language Server.
///
/// See also (enso protocol documentation)
/// [https://github.com/enso-org/enso/blob/main/docs/language-server/protocol-language-server.md].
#[derive(Debug)]
pub struct Module {
    model           : model::module::Plain,
    language_server : Rc<language_server::Connection>,
    logger          : Logger,
}


// === Public API ===

impl Module {
    /// Open the module.
    ///
    /// This function will open the module in Language Server and schedule task which will send
    /// updates about module's change to Language Server.
    pub async fn open
    ( path            : Path
    , language_server : Rc<language_server::Connection>
    , parser          : Parser
    ) -> FallibleResult<Rc<Self>> {
        let logger        = Logger::new(iformat!("Module {path}"));
        let file_path     = path.file_path().clone();
        info!(logger, "Opening module {file_path}");
        let opened = language_server.client.open_text_file(&file_path).await?;
        info!(logger, "Read content of the module {path}, digest is {opened.current_version:?}");
        let end_of_file = TextLocation::at_document_end(&opened.content);
        // TODO[ao] We should not fail here when metadata are malformed, but discard them and set
        //  default instead.
        let source  = parser.parse_with_metadata(opened.content)?;
        let digest  = opened.current_version;
        let summary = ContentSummary {digest,end_of_file};
        let model   = model::module::Plain::new(path,source.ast,source.metadata);
        let this    = Rc::new(Module {model,language_server,logger});
        let content = this.model.serialized_content()?;
        let first_invalidation = this.full_invalidation(&summary,content);
        executor::global::spawn(Self::runner(this.clone_ref(),summary,first_invalidation));
        Ok(this)
    }

    /// Create a module mock.
    pub fn mock(model:model::module::Plain) -> Rc<Self> {
        let logger = Logger::new(iformat!("Mocked Module {model.path()}"));
        let client = language_server::MockClient::default();
        client.expect.close_text_file(|_| Ok(()));
        // We don't expect any other call, because we don't execute `runner()`.
        let language_server = language_server::Connection::new_mock_rc(client);
        Rc::new(Module{model,language_server,logger})
    }
}

impl API for Module {
    fn subscribe(&self) -> Subscriber<Notification> {
        self.model.subscribe()
    }

    fn path(&self) -> &Path {
        self.model.path()
    }

    fn serialized_content(&self) -> FallibleResult<SourceFile> {
        self.model.serialized_content()
    }

    fn ast(&self) -> ast::known::Module {
        self.model.ast()
    }

    fn find_definition(&self, id:&Id) -> FallibleResult<DefinitionInfo> {
        self.model.find_definition(id)
    }

    fn node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        self.model.node_metadata(id)
    }

    fn update_whole(&self, content:Content) -> FallibleResult {
        self.model.update_whole(content)
    }

    fn update_ast(&self, ast: ast::known::Module) -> FallibleResult {
        self.model.update_ast(ast)
    }

    fn apply_code_change
    (&self, change:TextChange, parser:&Parser, new_id_map:IdMap) -> FallibleResult {
        self.model.apply_code_change(change,parser,new_id_map)
    }

    fn set_node_metadata(&self, id:ast::Id, data:NodeMetadata) -> FallibleResult {
        self.model.set_node_metadata(id,data)
    }

    fn remove_node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        self.model.remove_node_metadata(id)
    }

    fn with_node_metadata
    (&self, id:ast::Id, fun:Box<dyn FnOnce(&mut NodeMetadata) + '_>) -> FallibleResult {
        self.model.with_node_metadata(id,fun)
    }
}


// === Synchronizing Language Server ===

impl Module {
    /// The asynchronous task scheduled during struct creation which listens for all module changes
    /// and send proper updates to Language Server.
    async fn runner
    ( self               : Rc<Self>
    , initial_ls_content : ContentSummary
    , first_invalidation : impl Future<Output=FallibleResult<ParsedContentSummary>>) {
        let first_invalidation = first_invalidation.await;
        let mut ls_content     = self.new_ls_content_info(initial_ls_content,first_invalidation);
        let mut subscriber     = self.model.subscribe();
        let weak               = Rc::downgrade(&self);
        drop(self);

        loop {
            let notification = subscriber.next().await;
            let this = weak.upgrade();
            match (notification,this) {
                (Some(notification),Some(this)) => {
                    debug!(this.logger,"Processing a notification: {notification:?}");
                    let result = this.handle_notification(&ls_content,notification).await;
                    ls_content = this.new_ls_content_info(ls_content.summary().clone(),result)
                }
                _ => break,
            }
        }
    }

    /// Get the updated Language Server content summary basing on result of some updating function
    /// (`handle_notification` or `full_invalidation`. If the result is Error, then we assume that
    /// any change was not applied to Language Server state, and mark the state as `Desynchronized`,
    /// so any new update attempt should perform full invalidation.
    fn new_ls_content_info
    (&self, old_content:ContentSummary, new_content:FallibleResult<ParsedContentSummary>)
    -> LanguageServerContent {
        match new_content {
            Ok(new_content) => {
                debug!(self.logger,"Updating the LS content digest to: {new_content.summary:?}");
                LanguageServerContent::Synchronized(new_content)
            },
            Err(err)        => {
                error!(self.logger,"Error during sending text change to Language Server: {err}");
                LanguageServerContent::Desynchronized(old_content)
            }
        }
    }

    /// Send to LanguageServer update about received notification about module. Returns the new
    /// content summery of Language Server state.
    async fn handle_notification
    (&self, content:&LanguageServerContent, notification:Notification)
    -> FallibleResult<ParsedContentSummary> {
        let Notification{new_file,kind} = notification;
        debug!(self.logger,"Handling notification: {content:?}.");
        match content {
            LanguageServerContent::Desynchronized(summary) =>
                self.full_invalidation(summary,new_file).await,
            LanguageServerContent::Synchronized(summary) => match kind {
                NotificationKind::Invalidate =>
                    self.full_invalidation(&summary.summary,new_file).await,
                NotificationKind::CodeChanged{change,replaced_location} => {
                    let code_change = TextEdit {
                        range: replaced_location.into(),
                        text: change.inserted,
                    };
                    let id_map_change = TextEdit {
                        range: summary.id_map.clone().into(),
                        text: new_file.id_map_slice().to_string(),
                    };
                    //id_map goes first, because code change may alter it's position.
                    let edits = vec![id_map_change, code_change];
                    self.notify_language_server(&summary.summary,&new_file,edits).await
                }
                NotificationKind::MetadataChanged => {
                    let edits = vec![TextEdit {
                        range: summary.metadata.clone().into(),
                        text: new_file.metadata_slice().to_string(),
                    }];
                    self.notify_language_server(&summary.summary,&new_file,edits).await
                }
            },
        }
    }

    /// Send update to Language Server with the entire file content. Returns the new content summary
    /// of Language Server state.
    fn full_invalidation
    (&self, ls_content:&ContentSummary, new_file:SourceFile)
    -> impl Future<Output=FallibleResult<ParsedContentSummary>> + 'static {
        debug!(self.logger,"Handling full invalidation: {ls_content:?}.");
        let range = TextLocation::at_document_begin()..ls_content.end_of_file;
        let edits = vec![TextEdit {
            range : range.into(),
            text  : new_file.content.clone(),
        }];
        self.notify_language_server(ls_content,&new_file,edits)
    }

    /// This is a helper function with all common logic regarding sending the update to
    /// Language Server. Returns the new summary of Language Server state.
    fn notify_language_server
    ( &self
    , ls_content        : &ContentSummary
    , new_file          : &SourceFile
    , edits             : Vec<TextEdit>
    ) -> impl Future<Output=FallibleResult<ParsedContentSummary>> + 'static  {
        let summary = ParsedContentSummary::from_source(&new_file);
        let edit    = language_server::types::FileEdit {
            edits,
            path        : self.path().file_path().clone(),
            old_version : ls_content.digest.clone(),
            new_version : Sha3_224::new(new_file.content.as_bytes()),
        };
        debug!(self.logger,"Notifying LS with edit: {edit:#?}.");
        let ls_future_reply = self.language_server.client.apply_text_file_edit(&edit);
        async {
            ls_future_reply.await?;
            Ok(summary)
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let file_path       = self.path().file_path().clone();
        let language_server = self.language_server.clone_ref();
        let logger          = self.logger.clone_ref();
        executor::global::spawn(async move {
            let result = language_server.client.close_text_file(&file_path).await;
            if let Err(err) = result {
                error!(logger,"Error when closing module file {file_path}: {err}");
            }
        });
    }
}

impl Deref for Module {
    type Target = model::module::Plain;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
pub mod test {
    use super::*;

    use crate::test::Runner;

    use data::text;
    use data::text::TextChange;
    use enso_protocol::language_server::FileEdit;
    use enso_protocol::language_server::MockClient;
    use enso_protocol::language_server::Position;
    use enso_protocol::language_server::TextRange;
    use json_rpc::error::RpcError;
    use utils::test::ExpectTuple;
    use wasm_bindgen_test::wasm_bindgen_test;


    // === LsClientSetup ===

    // Ensures that subsequent LS text operations form a consistent series of versions.
    #[derive(Clone,Debug)]
    struct LsClientSetup {
        logger              : Logger,
        path                : Path,
        current_ls_content  : Rc<CloneCell<String>>,
        current_ls_version  : Rc<CloneCell<Sha3_224>>,
    }

    impl LsClientSetup {
        fn new(parent:impl AnyLogger, path:Path, initial_content:impl Into<String>) -> Self {
            let current_ls_content = initial_content.into();
            let current_ls_version = Sha3_224::new(current_ls_content.as_bytes());
            let logger             = Logger::sub(parent,"LsClientSetup");
            debug!(logger,"Initial content:\n===\n{current_ls_content}\n===");
            Self {
                path,logger,
                current_ls_content : Rc::new(CloneCell::new(current_ls_content)),
                current_ls_version : Rc::new(CloneCell::new(current_ls_version)),
            }
        }

        /// Create a setup initialized with the data from the given mock description.
        fn new_for_mock_data(data:&crate::test::mock::Unified) -> Self {
            Self::new(&data.logger,data.module_path.clone(),data.get_code())
        }

        /// Set up general text edit expectation in the mock client.
        ///
        /// This edit can be anything (full invalidation, code or metadata edit). The given function
        /// should perform any necessary assertions on the `FileEdit` and provide the result that
        /// the mock client will reply with.
        fn expect_some_edit
        (&self, client:&mut MockClient, f:impl FnOnce(&FileEdit) -> json_rpc::Result<()> + 'static) {
            let this = self.clone();
            client.expect.apply_text_file_edit(move |edits| {
                let content_so_far = this.current_ls_content.get();
                let result         = f(edits);
                let new_content    = apply_edits(content_so_far, &edits);
                let actual_old     = this.current_ls_version.get();
                let actual_new     = Sha3_224::new(new_content.as_bytes());
                debug!(this.logger,"Actual digest:   {actual_old} => {actual_new}");
                debug!(this.logger,"Declared digest: {edits.old_version} => {edits.new_version}");
                debug!(this.logger,"New content:\n===\n{new_content}\n===");
                assert_eq!(&edits.path,this.path.file_path());
                assert_eq!(edits.old_version,actual_old);
                assert_eq!(edits.new_version,actual_new);
                if result.is_ok() {
                    this.current_ls_content.set(new_content);
                    this.current_ls_version.set(actual_new);
                    debug!(this.logger,"Accepted!");
                } else {
                    debug!(this.logger,"Rejected!");
                }
                result
            });
        }

        /// The single text edit with accompanying metadata idmap changes.
        fn expect_edit_w_metadata
        (&self, client:&mut MockClient, f:impl FnOnce(&TextEdit) -> json_rpc::Result<()> + 'static) {
            let this = self.clone();
            self.expect_some_edit(client, move |edit| {
                if let [edit_idmap,edit_code] = edit.edits.as_slice() {
                    let code_so_far = this.current_ls_content.get();
                    let file_so_far = SourceFile::new(code_so_far);
                    // TODO [mwu]
                    //  Currently this assumes that the whole idmap is replaced at each edit.
                    //  This code should be adjusted, if partial metadata updates are implemented.
                    let idmap_range = TextLocation::convert_byte_range(&file_so_far.content,
                        &file_so_far.id_map);
                    let idmap_range = TextRange::from(idmap_range);
                    assert_eq!(edit_idmap.range, idmap_range);
                    assert!(SourceFile::looks_like_idmap(&edit_idmap.text));
                    f(edit_code)
                } else {
                    // This test assumes that expected single file edit consists from two text
                    // edits: first for idmap adjustment and the second for the actual code edit.
                    panic!("Expected exactly two edits.");
                }
            });
        }

        /// Set up expectation that the content will be fully invalidated. The mock client will
        /// report a success.
        fn expect_full_invalidation(&self, client:&mut MockClient) {
            self.expect_full_invalidation_result(client,Ok(()))
        }

        /// Set up expectation that the content will be fully invalidated. The mock client will
        /// report a given result.
        fn expect_full_invalidation_result
        (&self, client:&mut MockClient, result:json_rpc::Result<()>) {
            let this = self.clone();
            self.expect_some_edit(client, move |edits| {
                let (edit,) = edits.edits.iter().expect_tuple();
                assert_eq!(edit.range,this.whole_document_range());
                result
            });
        }

        fn whole_document_range(&self) -> TextRange {
            let code_so_far = self.current_ls_content.get();
            let end_of_file = TextLocation::at_document_end(&code_so_far);
            TextRange {
                start : Position { line:0,character:0  },
                end   : end_of_file.into(),
            }
        }
    }

    fn apply_edit(code:&str, edit:&TextEdit) -> String {
        let start = TextLocation::from(edit.range.start.into()).to_index(code);
        let end   = TextLocation::from(edit.range.end.into()).to_index(code);
        data::text::TextChange::replace(start..end,edit.text.clone()).applied(code)
    }

    fn apply_edits(code:impl Into<String>, file_edit:&FileEdit) -> String {
        let initial = code.into();
        file_edit.edits.iter().fold(initial, |content,edit| apply_edit(&content,edit))
    }


    // === Test cases ===

    #[wasm_bindgen_test]
    fn handling_notifications() {
        // The test starts with code as below. Then it replaces the whole AST to print "Test".
        // Then partial edit happens to change Test into Test 2.
        // Tested logic is:
        // * there is an initial invalidation after opening the module
        // * replacing AST causes invalidation
        // * localized text edit emits similarly localized synchronization updates.
        let initial_code = "main =\n    println \"Hello World!\"";
        let mut data = crate::test::mock::Unified::new();
        data.set_code(initial_code);
        // We do actually care about sharing `data` between `test` invocations, as it stores the
        // Parser which is time-consuming to construct.
        let test = |runner:&mut Runner| {
            let module_path  = data.module_path.clone();
            let edit_handler = Rc::new(LsClientSetup::new(&data.logger,module_path,initial_code));
            let mut fixture  = data.fixture_customize(|data, client| {
                data.expect_opening_module(client);
                data.expect_closing_module(client);
                // Opening module and metadata generation.
                edit_handler.expect_full_invalidation(client);
                // Explicit AST update.
                edit_handler.expect_full_invalidation(client);
                // Replacing `Test` with `Test 2`
                edit_handler.expect_some_edit(client, |edits| {
                    let edit_code = &edits.edits[1];
                    assert_eq!(edit_code.text, "Test 2");
                    assert_eq!(edit_code.range, TextRange {
                        start : Position { line: 1, character: 13 },
                        end   : Position { line: 1, character: 17 },
                    });
                    Ok(())
                });
            });

            let parser = data.parser.clone();
            let module = fixture.synchronized_module();

            let new_content = "main =\n    println \"Test\"".to_string();
            let new_ast     = parser.parse_module(new_content.clone(), default()).unwrap();
            module.update_ast(new_ast).unwrap();
            runner.perhaps_run_until_stalled(&mut fixture);
            let change = TextChange {
                replaced : text::Index::new(20)..text::Index::new(24),
                inserted : "Test 2".to_string(),
            };
            module.apply_code_change(change, &Parser::new_or_panic(), default()).unwrap();
            runner.perhaps_run_until_stalled(&mut fixture);
        };

        Runner::run(test);
    }

    #[wasm_bindgen_test]
    fn handling_notification_after_failure() {
        let initial_code = "main =\n    println \"Hello World!\"";
        let mut data     = crate::test::mock::Unified::new();
        data.set_code(initial_code);

        let test = |runner:&mut Runner| {
            let edit_handler = LsClientSetup::new_for_mock_data(&data);
            let mut fixture  = data.fixture_customize(|data, client| {
                data.expect_opening_module(client);
                data.expect_closing_module(client);
                // Opening module and metadata generation.
                edit_handler.expect_full_invalidation(client);
                // Applying code update.
                edit_handler.expect_edit_w_metadata(client, |edit| {
                    assert_eq!(edit.text, "Test 2");
                    assert_eq!(edit.range, TextRange {
                        start : Position { line: 1, character: 13 },
                        end   : Position { line: 1, character: 17 },
                    });
                    Err(RpcError::LostConnection)
                });
                // Full synchronization due to failed update in previous edit.
                edit_handler.expect_full_invalidation(client);
            });

            let (_module, controller) = fixture.synchronized_module_w_controller();
            runner.perhaps_run_until_stalled(&mut fixture);
            let change = TextChange {
                replaced: text::Index::new(20)..text::Index::new(24),
                inserted: "Test 2".to_string(),
            };
            controller.apply_code_change(change).unwrap();
            runner.perhaps_run_until_stalled(&mut fixture);
        };
        Runner::run(test);
    }
}
