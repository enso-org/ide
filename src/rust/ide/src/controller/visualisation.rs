//! Visualisation controller.
//!
//! Ths Visualisation Controller is Responsible identifying all the available visualisation natively
//! embedded in IDE and available within the project's `visualisation` folder. The
//! `Visualisation Controller` lives as long as the `Project Controller`.

use crate::prelude::*;

use std::rc::Rc;
use enso_protocol::language_server;



// ===============================
// === VisualisationIdentifier ===
// ===============================

/// This enum is used to identify a visualiser both in the project folder or natively embedded in
/// IDE.
#[derive(Debug,Clone)]
#[allow(missing_docs)]
pub enum VisualiserIdentifier {
    Embedded,
    File(language_server::Path)
}



// ==============
// === Handle ===
// ==============

/// Visualisation Controller's state.
#[derive(Debug,Clone)]
pub struct Handle {
    language_server_rpc : Rc<language_server::Connection>
}

impl Handle {
    /// Creates a new visualisation controller.
    pub fn new(language_server_rpc:Rc<language_server::Connection>) -> Self {
        Self {language_server_rpc}
    }

    /// Get a list of all available visualisers.
    pub async fn list_visualisers(&self) -> FallibleResult<Vec<VisualiserIdentifier>> {
        let root_id   = self.language_server_rpc.content_root();
        let path      = language_server::Path{root_id,segments:vec!["visualisation".into()]};
        let file_list = self.language_server_rpc.file_list(path).await?;
        let result    = file_list.paths.iter().filter_map(|object| {
            if let language_server::FileSystemObject::File{name,path} = object {
                let mut path = path.clone();
                path.segments.push(name.to_string());
                let root_id                  = path.root_id;
                let segments                 = path.segments;
                let path                     = language_server::Path{root_id,segments};
                let visualisation_identifier = VisualiserIdentifier::File(path);
                Some(visualisation_identifier)
            } else {
                None
            }
        }).collect();
        Ok(result)
    }

    /// Load the source code of the specified visualiser.
    pub async fn load_visualiser(&self, visualiser:&VisualiserIdentifier) -> FallibleResult<String> {
        let result = match visualiser {
            VisualiserIdentifier::Embedded   => unimplemented!(),
            VisualiserIdentifier::File(path) => {
                self.language_server_rpc.read_file(path.clone()).await?.contents
            }
        };
        Ok(result)
    }
}

// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_controller() {
        let mock_client     = language_server::MockClient::default();
        let language_server = language_server::Connection::new_mock_rc(mock_client);
        let vis_controller  = Handle::new(language_server);

        let visualisers = vis_controller.list_visualisers().expect("Couldn't list visualisers.");

        let content = vec!["Content A".to_string(), "Content B".to_string()];
        let zipped  = visualisers.iter().zip(content.iter());
        zipped.map(|visualiser,content| {
            let loaded_content = vis_controller.load_visualiser(&visualiser);
            let loaded_content = loaded_content.expect("Couldn't load visualiser's content.");
            assert!(content,loaded_content);
        });
    }
}
