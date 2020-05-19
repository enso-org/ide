//! A Wrapper for module which synchronizes opening/closing and all changes with Language Server.

use crate::prelude::*;

use crate::model::module::Notification;

use enso_protocol::types::Sha3_224;
use enso_protocol::language_server;
use data::text::TextLocation;
use parser::api::SerializedSourceFile;
use parser::Parser;
use enso_protocol::language_server::TextEdit;



// =======================
// === Content Summary ===
// =======================

/// The minimal information about module's content, required to do properly invalidation of opened
/// module in Language Server.
#[derive(Clone,Debug)]
struct ContentSummary {
    digest      : Sha3_224,
    end_of_file : TextLocation,
}

/// The information about module's content, which was parsed at least once. In addition to minimal
/// summery defined in `ContentSummary` it adds information about sections, what enables efficient
/// updates after code and metadata changes.
#[derive(Clone,Debug,Shrinkwrap)]
struct ParsedContentSummary {
    #[shrinkwrap(main_field)]
    summary  : ContentSummary,
    code     : Range<TextLocation>,
    id_map   : Range<TextLocation>,
    metadata : Range<TextLocation>,
}

impl ParsedContentSummary {
    /// Get summary from `SerializedSourceFile`.
    fn from_source(source:&SerializedSourceFile) -> Self {
        let summary = ContentSummary {
            digest      : Sha3_224::new(source.string.as_bytes()),
            end_of_file : TextLocation::at_document_end(&source.string)
        };
        ParsedContentSummary {
            summary,
            code        : TextLocation::convert_range(&source.string,&source.code),
            id_map      : TextLocation::convert_range(&source.string,&source.id_map),
            metadata    : TextLocation::convert_range(&source.string,&source.metadata),
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
/// [https://github.com/luna/enso/blob/master/docs/language-server/protocol-language-server.md].
#[derive(Debug)]
pub struct Module {
    path            : controller::module::Path,
    /// The module handle.
    pub model       : model::Module,
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
    ( path            : controller::module::Path
    , language_server : Rc<language_server::Connection>
    , parser          : Parser
    ) -> FallibleResult<Rc<Self>> {
        let logger        = Logger::new(iformat!("Module {path}"));
        let file_path     = path.file_path().clone();
        info!(logger, "Opening module {file_path}");
        let opened = language_server.client.open_text_file(&file_path).await?;
        trace!(logger, "Read content of module {path}, digest is {opened.current_version:?}");
        let end_of_file = TextLocation::at_document_end(&opened.content);
        // TODO[ao] We should not fail here when metadata are malformed, but discard them and set
        //  default instead.
        let source  = parser.parse_with_metadata(opened.content)?;
        let digest  = opened.current_version;
        let summary = ContentSummary {digest,end_of_file};
        let model   = model::Module::new(source.ast,source.metadata);
        let this    = Rc::new(Module {path,model,language_server,logger});
        executor::global::spawn(Self::runner(this.clone_ref(),summary));
        Ok(this)
    }

    /// Create a module mock.
    #[cfg(test)]
    pub fn mock(path:controller::module::Path, model:model::Module) -> Rc<Self> {
        let logger = Logger::new(iformat!("Mocked Module {path}"));
        let client = language_server::MockClient::default();
        client.expect.close_text_file(|_| Ok(()));
        // We don't expect any other call, because we don't execute `runner()`.
        let language_server = language_server::Connection::new_mock_rc(client);
        Rc::new(Module{path,model,language_server,logger})
    }
}


// === Synchronizing Language Server ===

impl Module {
    /// The asynchronous task scheduled during struct creation which listens for all module changes
    /// and send proper updates to Language Server.
    async fn runner(this:Rc<Self>, initial_ls_content: ContentSummary) {
        let first_invalidation = this.full_invalidation(&initial_ls_content).await;
        let mut ls_content     = this.new_ls_content_info(initial_ls_content, first_invalidation);
        let mut subscriber     = this.model.subscribe();
        let weak               = Rc::downgrade(&this);
        drop(this);

        loop {
            let notification = subscriber.next().await;
            let this = weak.upgrade();
            match (notification,this) {
                (Some(notification),Some(this)) => {
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

    /// Handle received notification. Returns the new content summery of Language Server state.
    async fn handle_notification
    (&self, content:&LanguageServerContent, notification:Notification)
    -> FallibleResult<ParsedContentSummary> {
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
                        vec![code_change,id_map_change]
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
        let range = TextLocation::at_document_begin()..ls_content.end_of_file;
        self.notify_language_server(ls_content,|content| vec![TextEdit {
            range : range.into(),
            text  : content.string
        }]).await
    }

    /// This is a helper function with all common logic regarding sending the update to
    /// Language Server. Returns the new summary of Language Server state.
    async fn notify_language_server
    ( &self
    , ls_content        : &ContentSummary
    , edits_constructor : impl FnOnce(SerializedSourceFile) -> Vec<TextEdit>
    ) -> FallibleResult<ParsedContentSummary> {
        let content = self.model.serialized_content()?;
        let summary = ParsedContentSummary::from_source(&content);
        let edit    = language_server::types::FileEdit {
            path        : self.path.file_path().clone(),
            edits       : edits_constructor(content),
            old_version : ls_content.digest.clone(),
            new_version : summary.digest.clone()
        };
        self.language_server.client.apply_text_file_edit(&edit).await?;
        Ok(summary)
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let file_path       = self.path.file_path().clone();
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
    type Target = model::Module;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}



// ============
// === Test ===
// ============

#[cfg(test)]
mod test {
    use super::*;

    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    use json_rpc::expect_call;
    use wasm_bindgen_test::wasm_bindgen_test;
    use utils::test::ExpectTuple;

    struct LsClientSetup {
        file_path          : language_server::Path,
        current_ls_code    : Rc<CloneCell<String>>,
        current_ls_version : Rc<CloneCell<Sha3_224>>,
        client             : language_server::MockClient,
    }

    impl LsClientSetup {
        fn new(file_path:language_server::Path, initial_content:impl Str) -> Self {
            let initial_content = initial_content.into();
            let initial_version = Sha3_224::new(initial_content.as_bytes());
            let this = LsClientSetup {
                file_path,
                current_ls_code    : Rc::new(CloneCell::new(initial_content)),
                current_ls_version : Rc::new(CloneCell::new(initial_version)),
                client             : default(),
            };
            this.expect_open_and_initial_update();
            this
        }

        fn expect_open_and_initial_update(&self) {
            let open_response = language_server::response::OpenTextFile {
                write_capability : self.create_can_edit_capability(),
                content          : self.current_ls_code.get(),
                current_version  : self.current_ls_version.get(),
            };
            let client = &self.client;
            expect_call!(client.open_text_file(path=self.file_path.clone()) => Ok(open_response));

            let old_code = self.current_ls_code.get();
            self.expect_invalidate(move |new_code| {
                let code_end = old_code.find("#### METADATA ####").unwrap_or(old_code.len());
                assert_eq!(old_code[..code_end], new_code[..code_end]);
            });
        }

        fn expect_invalidate<CodeChecker>(&self, code_checker:CodeChecker)
        where CodeChecker : FnOnce(&str) + 'static {
            let path       = self.file_path.clone();
            let ls_version = self.current_ls_version.clone_ref();
            let ls_code    = self.current_ls_code.clone_ref();
            self.client.expect.apply_text_file_edit(move |edit| {
                assert_eq!(edit.path       , path);
                assert_eq!(edit.old_version, ls_version.get());
                let end_of_file  = TextLocation::at_document_end(ls_code.get());
                let (text_edit,) = edit.edits.iter().expect_tuple();
                let expected_range = language_server::types::TextRange {
                    start : language_server::types::Position { line:0,character:0  },
                    end   : end_of_file.into(),
                };
                assert_eq!(text_edit.range, expected_range);
                assert_eq!(edit.new_version, Sha3_224::new(text_edit.text.as_bytes()));
                code_checker(text_edit.text.as_str());
                ls_code.set(text_edit.text.clone());
                ls_version.set(edit.new_version.clone());
                Ok(())
            });
        }

        fn create_can_edit_capability
        (&self) -> Option<language_server::types::CapabilityRegistration> {
            use language_server::types::*;
            let method           = "text/canEdit".to_string();
            let path             = self.file_path.clone();
            let register_options = RegisterOptions::ReceivesTreeUpdates(ReceivesTreeUpdates {path});
            Some(CapabilityRegistration {method,register_options})
        }

        fn finish(self) -> Rc<language_server::Connection> {
            let client = self.client;
            expect_call!(client.close_text_file(path=self.file_path) => Ok(()));
            language_server::Connection::new_mock_rc(client)
        }
    }

    #[wasm_bindgen_test]
    fn handling_notifications_in_runner() {
        let path            = controller::module::Path::from_module_name("TestModule");
        let parser          = Parser::new_or_panic();
        let initial_content = "main =\n    println \"Hello World!\"";
        let new_content     = "main =\n    println \"Test\"".to_string();
        let new_ast         = parser.parse_module(new_content.clone(),default()).unwrap();
        let setup           = LsClientSetup::new(path.file_path().clone(),initial_content);
        setup.expect_invalidate(move |code| { assert!(code.starts_with(new_content.as_str())) });
        let connection                             = setup.finish();
        let (barrier_snd,barrier_rcv)              = futures::channel::oneshot::channel::<()>();
        let mut test                               = TestWithLocalPoolExecutor::set_up();
        let module:Rc<RefCell<Option<Rc<Module>>>> = default();
        let module_ref                             = module.clone();
        test.run_task(async move {
            let module = Module::open(path,connection,Parser::new_or_panic()).await.unwrap();
            barrier_rcv.await.unwrap();
            module.model.update_ast(new_ast);
            *module_ref.borrow_mut() = Some(module); // Keep the module after this task.
        });
        test.when_stalled(move || barrier_snd.send(()).unwrap());
        test.when_stalled(move || *module.borrow_mut() = None);
    }




}
