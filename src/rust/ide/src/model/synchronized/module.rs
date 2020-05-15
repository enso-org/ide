use crate::prelude::*;

use enso_protocol::types::Sha3_224;
use enso_protocol::language_server;
use data::text::TextLocation;
use parser::api::SerializedSourceFile;
use data::text;
use parser::Parser;
use enso_protocol::language_server::TextEdit;

struct ContentSummary {
    digest          : Sha3_224,
    code            : Range<TextLocation>,
    id_map          : Range<TextLocation>,
    metadata        : Range<TextLocation>,
}

impl ContentSummary {
    fn from_source(source:&SerializedSourceFile) -> Self {
        ContentSummary {
            digest   : Sha3_224::new(source.string.as_bytes()),
            code     : TextLocation::convert_range(&source.string,&source.code),
            id_map   : TextLocation::convert_range(&source.string,&source.id_map),
            metadata : TextLocation::convert_range(&source.string,&source.metadata),
        }
    }
}

pub struct Module {
    path            : controller::module::Path,
    model           : model::Module,
    language_server : Rc<language_server::Connection>,
    parser          : Parser,
    ls_content      : RefCell<ContentSummary>,
    logger          : Logger,
}

impl Module {
    pub async fn open
    ( path            : controller::module::Path
    , language_server : Rc<language_server::Connection>
    , parser          : Parser
    ) -> FallibleResult<Self> {
        let logger     = Logger::new(iformat!("Module {path}"));
        let file_path  = path.file_path().clone();
        let opened     = language_server.client.open_text_file(file_path).await?;
        let current_range = TextLocation::at_document_begin()..TextLocation::at_document_end(&opened.content);
        let source     = parser.parse_with_metadata(opened.content)?;
        let serialized = source.serialize_()?;
        let summary    = ContentSummary::from_source(&serialized);
        // After loading we might alter id_map during parsing. We must inform LanguageServer about
        // that
        let edit = language_server::types::FileEdit {
            path: path.file_path().clone(),
            edits: vec![language_server::types::TextEdit {
                range: current_range.into(),
                text: serialized.string,
            }],
            old_version: opened.current_version,
            new_version: summary.digest.clone()
        };
        language_server.client.apply_text_file_edit(edit).await?;

        let ls_content = RefCell::new(summary);
        let model      = model::Module::new(source.ast,source.metadata);
        Ok(Module {path,model,language_server,parser,ls_content,logger})
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        let file_path       = self.path.file_path().clone();
        let language_server = self.language_server.clone_ref();
        let logger          = self.logger.clone_ref();
        executor::global::spawn(async move {
            let result = language_server.client.close_text_file(file_path.clone()).await;
            if let Err(err) = result {
                error!(logger,"Error when closing module file {file_path}: {err}");
            }
        });
    }
}