//! A Wrapper for module which synchronizes opening/closing and all changes with Language Server.

use crate::prelude::*;

use crate::double_representation::definition::DefinitionInfo;
use crate::double_representation::graph::Id;
use crate::model::module::API;
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
    /// The content is not synchronized with our module state after last fully handled notificaiton,
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
/// [https://github.com/luna/enso/blob/main/docs/language-server/protocol-language-server.md].
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
        executor::global::spawn(Self::runner(this.clone_ref(),summary));
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

    fn update_whole(&self, content:Content) {
        self.model.update_whole(content)
    }

    fn update_ast(&self, ast: ast::known::Module) {
        self.model.update_ast(ast)
    }

    fn apply_code_change
    (&self, change:TextChange, parser:&Parser, new_id_map:IdMap) -> FallibleResult<()> {
        self.model.apply_code_change(change,parser,new_id_map)
    }

    fn set_node_metadata(&self, id:ast::Id, data:NodeMetadata) {
        self.model.set_node_metadata(id,data)
    }

    fn remove_node_metadata(&self, id:ast::Id) -> FallibleResult<NodeMetadata> {
        self.model.remove_node_metadata(id)
    }

    fn with_node_metadata(&self, id:ast::Id, fun:Box<dyn FnOnce(&mut NodeMetadata) + '_>) {
        self.model.with_node_metadata(id,fun)
    }
}


// === Synchronizing Language Server ===

impl Module {
    /// The asynchronous task scheduled during struct creation which listens for all module changes
    /// and send proper updates to Language Server.
    async fn runner(self:Rc<Self>, initial_ls_content: ContentSummary) {
        let first_invalidation = self.full_invalidation(&initial_ls_content).await;
        let mut ls_content     = self.new_ls_content_info(initial_ls_content, first_invalidation);
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
            Ok(new_content) => LanguageServerContent::Synchronized(new_content),
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
        debug!(self.logger,"Handling notification: {content:?}.");
        match content {
            LanguageServerContent::Desynchronized(summary) => self.full_invalidation(summary).await,
            LanguageServerContent::Synchronized(summary)   => match notification {
                Notification::Invalidate => self.full_invalidation(&summary.summary).await,
                Notification::CodeChanged{change,replaced_location} =>
                    self.notify_language_server(&summary.summary, |content| {
                        let code_change = TextEdit {
                            range : replaced_location.into(),
                            text  : change.inserted,
                        };
                        let id_map_change = TextEdit {
                            range : summary.id_map.clone().into(),
                            text  : content.id_map_slice().to_string(),
                        };
                        //id_map goes first, because code change may alter it's position.
                        vec![id_map_change,code_change]
                    }).await,
                Notification::MetadataChanged =>
                    self.notify_language_server(&summary.summary, |content| vec![TextEdit {
                        range : summary.metadata.clone().into(),
                        text  : content.metadata_slice().to_string(),
                    }]).await,
            },
        }
    }

    /// Send update to Language Server with the entire file content. Returns the new content summary
    /// of Language Server state.
    async fn full_invalidation
    (&self, ls_content:&ContentSummary) -> FallibleResult<ParsedContentSummary> {
        debug!(self.logger,"Handling full invalidation: {ls_content:?}.");
        let range = TextLocation::at_document_begin()..ls_content.end_of_file;
        self.notify_language_server(ls_content,|content| vec![TextEdit {
            range : range.into(),
            text  : content.content
        }]).await
    }

    /// This is a helper function with all common logic regarding sending the update to
    /// Language Server. Returns the new summary of Language Server state.
    async fn notify_language_server
    ( &self
    , ls_content        : &ContentSummary
    , edits_constructor : impl FnOnce(SourceFile) -> Vec<TextEdit>
    ) -> FallibleResult<ParsedContentSummary> {
        let content = self.model.serialized_content()?;
        let summary = ParsedContentSummary::from_source(&content);
        let edit    = language_server::types::FileEdit {
            path        : self.path().file_path().clone(),
            edits       : edits_constructor(content),
            old_version : ls_content.digest.clone(),
            new_version : summary.digest.clone()
        };
        debug!(self.logger,"Notifying LS with edit: {edit:?}.");
        self.language_server.client.apply_text_file_edit(&edit).await?;
        Ok(summary)
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

    use data::text::TextChange;
    use data::text;
    use enso_protocol::language_server::{CapabilityRegistration, MockClient, FileEdit, TextRange, Position};
    use json_rpc::error::RpcError;
    use json_rpc::expect_call;
    use utils::test::ExpectTuple;
    use utils::test::traits::*;
    use wasm_bindgen_test::wasm_bindgen_test;
    use enso_frp::data::bitfield::BitField128;
    use enso_frp::data::bitfield::BitField;

    // Ensures that subsequent operations form a consistent series of versions.
    #[derive(Clone,Debug)]
    struct LsClientSetup {
        path               : model::module::Path,
        current_ls_code    : Rc<CloneCell<String>>,
        current_ls_version : Rc<CloneCell<Sha3_224>>,
    }

    impl LsClientSetup {
        fn new(path:model::module::Path, initial_code:impl Into<String>) -> Self {
            let current_ls_code    = initial_code.into();
            let current_ls_version = Sha3_224::new(current_ls_code.as_bytes());
            Self {
                path,
                current_ls_code    : Rc::new(CloneCell::new(current_ls_code)),
                current_ls_version : Rc::new(CloneCell::new(current_ls_version)),
            }
        }

        fn new_for_mock_data(data:&crate::test::mock::Unified) -> Self {
            Self::new(data.module_path.clone(),data.get_code())
        }

        fn expect_some_edit
        (&self, client:&mut MockClient, f:impl FnOnce(&FileEdit) -> json_rpc::Result<()> + 'static) {
            let this = self.clone();
            client.expect.apply_text_file_edit(move |edits| {
                let code_so_far = this.current_ls_code.get();
                let ret = f(edits);
                let mut new_contents = code_so_far;
                for edit in &edits.edits {
                    new_contents = apply_edit(&new_contents,edit)
                }

                let new_version  = Sha3_224::new(new_contents.as_bytes());
                println!("Internal Version {} => {}", this.current_ls_version.get(),new_version);
                println!("Declared Version {} => {}", edits.old_version,edits.new_version);
                println!("New contents:\n===\n{}\n===",new_contents);
                assert_eq!(&edits.path,this.path.file_path());
                assert_eq!(edits.old_version,this.current_ls_version.get());
                assert_eq!(edits.new_version,new_version);
                if ret.is_ok() {
                    this.current_ls_code.set(new_contents);
                    this.current_ls_version.set(new_version);
                    println!("Accepted!");
                } else {
                    println!("Rejected!");
                }
                ret
            });
        }

        /// The single text edit with accompanying metadata changes.
        fn expect_edit_w_metadata
        (&self, client:&mut MockClient, f:impl FnOnce(&TextEdit) -> json_rpc::Result<()> + 'static) {
            // let parser : Parser = todo!();
            // let this = self.clone();
            //
            self.expect_some_edit(client, move |edit| {
                if let [_edit_metadata,edit_code] = edit.edits.as_slice() {
                    // TODO assert that first edit actually does touch only metadata

                    // assert_eq!(edit_code.range, TextRange {
                    //     start : Position {line:1, character:13},
                    //     end   : Position {line:1, character:17},
                    // });
                    assert_eq!(edit_code.text, "Test 2");
                    f(edit_code)
                } else {
                    panic!("Expected exactly two edits");
                }
            });
        }

        fn expect_full_invalidation(&self, client:&mut MockClient) {
            self.expect_full_invalidation_result(client,Ok(()))
        }

        fn expect_full_invalidation_err(&self, client:&mut MockClient, error:RpcError) {
            self.expect_full_invalidation_result(client,Err(error))
        }

        fn expect_full_invalidation_result(&self, client:&mut MockClient, result:json_rpc::Result<()>) {
            let this = self.clone();
            self.expect_some_edit(client, move |edits| {
                let (edit,) = edits.edits.iter().expect_tuple();
                assert_eq!(edit.range,this.whole_document_range());
                result
            });
        }

        fn whole_document_range(&self) -> language_server::TextRange {
            let code_so_far = self.current_ls_code.get();
            let end_of_file = TextLocation::at_document_end(&code_so_far);
            language_server::TextRange {
                start : language_server::types::Position { line:0,character:0  },
                end   : end_of_file.into(),
            }
        }
    }

    fn apply_edit(code:&str, edit:&language_server::TextEdit) -> String {
        let start = TextLocation::from(edit.range.start.into()).to_index(code);
        let end = TextLocation::from(edit.range.end.into()).to_index(code);
        data::text::TextChange::replace(start..end,edit.text.clone()).applied(code)
    }


    #[test]
    fn handling_fast_edits() {
        let initial_code = "main =\n    println \"Hello World!\"";
        let mut data = crate::test::mock::Unified::new();
        data.set_code(initial_code);
        let edit_handler = Rc::new(LsClientSetup::new(data.module_path.clone(), initial_code));
        let mut fixture  = data.fixture_customize(|data,client| {
            data.expect_opening_the_module(client);
            data.expect_closing_the_module(client);
            edit_handler.expect_full_invalidation(client);
            edit_handler.expect_some_edit(client, |edit| {
                println!("checking {:?}",edit);
                Ok(())
            });

            edit_handler.expect_some_edit(client, |edit| {
                println!("checking {:?}",edit);
                Ok(())
            });
        });

        let module = fixture.module.clone();
        let parser = data.parser.clone();

        let path = data.module_path.clone();
        let ls   = fixture.project.json_rpc().clone();
        let module = Module::open(path,ls,parser.clone()).boxed_local().expect_ready().unwrap();

        let change = TextChange {
            replaced : text::Index::new(20)..text::Index::new(20),
            inserted : "a".to_string(),
        };
        module.apply_code_change(change.clone(),&Parser::new_or_panic(),default()).unwrap();
        fixture.run_until_stalled();
        module.apply_code_change(change.clone(),&Parser::new_or_panic(),default()).unwrap();
        fixture.run_until_stalled();
    }

    /// Helper structure that runs given test multiple times.
    ///
    /// The test when run is given a handle to the runner object that
    #[derive(Clone,Copy,Debug,Default)]
    pub struct Runner {
        /// Incremented each time when the runnee calls an interruption point.
        /// Reset to 0 after each run.
        current   : u32,
        /// Number of iterations already done.
        /// Used as a seed for the current iteration behavior.
        /// (subsequent bits from the least significant encode whether the n-th interruption point
        /// should actually pass control to the executor).
        iteration : BitField128,
    }

    impl Runner {
        fn run_internal(&mut self, mut f:impl FnMut(&mut Runner)) {
            assert_eq!(self.iteration.raw,0,"Runner object must not be reused.");
            assert_eq!(self.current,0,"Runner object must not be reused.");
            println!("Iteration #{}",0);
            f(self);
            let possibilities_count = 2u128.pow(self.current);
            for i in 1 ..possibilities_count {
                println!("Iteration #{}",i);
                self.iteration.raw = i;
                self.current = 0;
                f(self);
            }
        }

        pub fn perhaps_run_until_stalled(&mut self, fixture:&mut crate::test::mock::Fixture) {
            let index     = self.current;
            self.current += 1;
            let should_run = self.iteration.get_bit(index as usize);
            println!("Should run #{}? {}",index,should_run);
            if should_run {
                fixture.run_until_stalled();
            }
        }

        pub fn run(mut f:impl FnMut(&mut Runner)) {
            let mut runner = Runner::default();
            runner.run_internal(f);
        }
    }

    #[test]
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
            let edit_handler = Rc::new(LsClientSetup::new(data.module_path.clone(), initial_code));
            let mut fixture = data.fixture_customize(|data, client| {
                data.expect_opening_the_module(client);
                data.expect_closing_the_module(client);
                // Opening module and metadata generation.
                edit_handler.expect_full_invalidation(client);
                // Explicit AST update.
                edit_handler.expect_full_invalidation(client);
                // Replacing `Test` with `Test 2`
                edit_handler.expect_some_edit(client, |edits| {
                    println!("!!!!!!!!!, {:#?}", edits);
                    Ok(())
                });
            });

            let parser = data.parser.clone();
            let module = fixture.synchronized_module();

            let new_content = "main =\n    println \"Test\"".to_string();
            let new_ast = parser.parse_module(new_content.clone(), default()).unwrap();
            module.update_ast(new_ast);
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

    #[test]
    fn handling_notification_after_failure() {
        let initial_code = r#"main =
    println "Hello World!""#;
        let mut data = crate::test::mock::Unified::new();
        data.set_code(initial_code);
        let edit_handler = LsClientSetup::new_for_mock_data(&data);
        let mut fixture = data.fixture_customize(|data, client| {
            data.expect_opening_the_module(client);
            data.expect_closing_the_module(client);
            // Opening module and metadata generation.
            edit_handler.expect_full_invalidation(client);
            // Applying code update.
            edit_handler.expect_edit_w_metadata(client,|edit| {
                assert_eq!(edit.text, "Test 2");
                assert_eq!(edit.range, TextRange {
                    start : Position {line:1, character:13},
                    end   : Position {line:1, character:17},
                });
                Err(RpcError::LostConnection)
            });
            // Full synchronization due to failed update in previous edit.
            edit_handler.expect_full_invalidation(client);
        });

        let (module,controller) = fixture.synchronized_module_w_controller();
        fixture.run_until_stalled();
        let change = TextChange {
            replaced : text::Index::new(20)..text::Index::new(24),
            inserted : "Test 2".to_string(),
        };
        controller.apply_code_change(change).unwrap();
        fixture.run_until_stalled();
    }
}
