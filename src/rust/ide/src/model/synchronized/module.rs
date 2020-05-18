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

    /// Create a module mock, which will not call any method of language server.
    #[cfg(test)]
    pub fn mock(path:controller::module::Path, model:model::Module) -> Rc<Self> {
        let logger = Logger::new(iformat!("Mocked Module {path}"));
        // We don't take client as arg, because it should not be called at all.
        let language_server = language_server::Connection::new_mock_rc(default());
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
    pub use wasm_bindgen_test::wasm_bindgen_test;
    use crate::executor::test_utils::TestWithLocalPoolExecutor;

    #[wasm_bindgen_test]
    fn open_module_and_listen_for_notifications() {
        let mut test = TestWithLocalPoolExecutor::set_up();
        test.run_task(async {

        })
    }
}
