//! Visualisation controller.
//!
//! Ths Visualisation Controller is Responsible identifying all the available visualisation natively
//! embedded in IDE and available within the project's `visualisation` folder. The
//! `Visualisation Controller` lives as long as the `Project Controller`.

use crate::prelude::*;

use std::rc::Rc;
use enso_protocol::language_server;



// =============
// === Error ===
// =============

/// Enumeration of errors used in `Visualisation Controller`.
#[derive(Debug,Fail)]
#[allow(missing_docs)]
pub enum VisualiserError {
    #[fail(display = "Visualiser \"{}\" not found.", identifier)]
    NotFound {
        identifier : VisualiserIdentifier
    }
}



// ===============================
// === VisualisationIdentifier ===
// ===============================

/// This enum is used to identify a visualiser both in the project folder or natively embedded in
/// IDE.
#[derive(Debug,Clone,PartialEq,Eq,Display)]
#[allow(missing_docs)]
pub enum VisualiserIdentifier {
    Embedded(String),
    File(language_server::Path)
}



// ==============
// === Handle ===
// ==============

/// Visualisation Controller's state.
#[derive(Debug,Clone)]
pub struct Handle {
    language_server_rpc  : Rc<language_server::Connection>,
    embedded_visualisers : HashMap<String,String>
}

impl Handle {
    /// Creates a new visualisation controller.
    pub fn new
    ( language_server_rpc  : Rc<language_server::Connection>
    , embedded_visualisers : HashMap<String,String>) -> Self {
        Self {language_server_rpc,embedded_visualisers}
    }

    /// Get a list of all available visualisers.
    pub async fn list_visualisers(&self) -> FallibleResult<Vec<VisualiserIdentifier>> {
        let root_id    = self.language_server_rpc.content_root();
        let vis_path   = language_server::Path{root_id,segments:vec!["visualisation".into()]};
        let file_list  = self.language_server_rpc.file_list(vis_path).await?;
        let result     = self.embedded_visualisers.keys();
        let result     = result.map(|identifier| VisualiserIdentifier::Embedded(identifier.clone()));
        let mut result = result.collect::<Vec<VisualiserIdentifier>>();
        result.extend(file_list.paths.iter().filter_map(|object| {
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
        }));
        Ok(result)
    }

    /// Load the source code of the specified visualiser.
    pub async fn load_visualiser(&self, visualiser:&VisualiserIdentifier) -> FallibleResult<String> {
        match visualiser {
            VisualiserIdentifier::Embedded(identifier) => {
                let result = self.embedded_visualisers.get(identifier);
                let error  = || {
                    failure::Error::from(VisualiserError::NotFound{identifier:visualiser.clone()})
                };
                result.cloned().ok_or_else(error)
            },
            VisualiserIdentifier::File(path) =>
                Ok(self.language_server_rpc.read_file(path.clone()).await?.contents)
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use std::future::Future;
    use utils::test::poll_future_output;
    use enso_protocol::language_server::FileSystemObject;
    use enso_protocol::language_server::Path;

    fn result<T,F:Future<Output = FallibleResult<T>>>(fut:F) -> FallibleResult<T> {
        let mut fut = Box::pin(fut);
        poll_future_output(&mut fut).expect("Promise isn't ready")
    }

    #[test]
    fn list_and_load() {
        let mock_client     = language_server::MockClient::default();

        let root_id = uuid::Uuid::default();
        let path    = Path{root_id,segments:vec!["visualisation".into()]};
        let paths   = vec![
            FileSystemObject::File{path:path.clone(),name:"histogram.js".into()},
            FileSystemObject::File{path:path.clone(),name:"graph.js".into()},
        ];
        let file_list_result = language_server::response::FileList{paths};
        mock_client.set_file_list_result(path, Ok(file_list_result));

        let path0 = Path{root_id,segments:vec!["visualisation".into(),"histogram.js".into()]};
        let path1 = Path{root_id,segments:vec!["visualisation".into(),"graph.js".into()]};
        let file_content0 = "<histogram code>".to_string();
        let file_content1 = "<graph code>".to_string();
        let result0 = language_server::response::Read{contents:file_content0.clone()};
        let result1 = language_server::response::Read{contents:file_content1.clone()};
        mock_client.set_file_read_result(path0.clone(),Ok(result0));
        mock_client.set_file_read_result(path1.clone(),Ok(result1));

        let language_server          = language_server::Connection::new_mock_rc(mock_client);
        let mut embedded_visualisers = HashMap::new();
        let embedded_content         = "<extremely fast point cloud code>".to_string();
        embedded_visualisers.insert("PointCloud".to_string(),embedded_content.clone());
        let vis_controller           = Handle::new(language_server,embedded_visualisers);

        let visualisers = result(vis_controller.list_visualisers()).expect("Couldn't list visualisers.");

        assert_eq!(visualisers[0],VisualiserIdentifier::Embedded("PointCloud".to_string()));
        assert_eq!(visualisers[1],VisualiserIdentifier::File(path0));
        assert_eq!(visualisers[2],VisualiserIdentifier::File(path1));

        let content = vec![embedded_content,file_content0,file_content1];
        let zipped  = visualisers.iter().zip(content.iter());
        for (visualiser,content) in zipped {
            let loaded_content = result(vis_controller.load_visualiser(&visualiser));
            let loaded_content = loaded_content.expect("Couldn't load visualiser's content.");
            assert_eq!(*content,loaded_content);
        }
    }
}
