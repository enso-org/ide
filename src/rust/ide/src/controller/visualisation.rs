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
pub enum VisualisationError {
    #[fail(display = "Visualisation \"{}\" not found.", identifier)]
    NotFound {
        identifier : VisualisationIdentifier
    }
}



// ===============================
// === VisualisationIdentifier ===
// ===============================

/// This enum is used to identify a visualiser both in the project folder or natively embedded in
/// IDE.
#[derive(Clone,Debug,Display,Eq,PartialEq)]
#[allow(missing_docs)]
pub enum VisualisationIdentifier {
    Embedded(String),
    File(language_server::Path)
}



// ==============================
// === EmbeddedVisualisations ===
// ==============================

#[allow(missing_docs)]
pub type EmbeddedVisualisationName   = String;
#[allow(missing_docs)]
pub type EmbeddedVisualisationSource = String;

/// Embedded visualisations mapped from name to source code.
#[derive(Shrinkwrap,Debug,Clone,Default)]
#[shrinkwrap(mutable)]
pub struct EmbeddedVisualisations {
    #[allow(missing_docs)]
    pub map : HashMap<EmbeddedVisualisationName,EmbeddedVisualisationSource>
}




// ==============
// === Handle ===
// ==============

const VISUALISATION_FOLDER : &str = "visualisation";

/// Visualisation Controller's state.
#[derive(Debug,Clone)]
pub struct Handle {
    language_server_rpc     : Rc<language_server::Connection>,
    embedded_visualisations : EmbeddedVisualisations
}

impl Handle {
    /// Creates a new visualisation controller.
    pub fn new
    ( language_server_rpc     : Rc<language_server::Connection>
    , embedded_visualisations : EmbeddedVisualisations) -> Self {
        Self {language_server_rpc,embedded_visualisations}
    }

    async fn list_file_visualisations(&self) -> FallibleResult<Vec<VisualisationIdentifier>> {
        let root_id   = self.language_server_rpc.content_root();
        let path      = language_server::Path::new(root_id,&[VISUALISATION_FOLDER]);
        let file_list = self.language_server_rpc.file_list(&path).await?;
        let result    = file_list.paths.iter().filter_map(|object| {
            if let language_server::FileSystemObject::File{name,path} = object {
                let mut path = path.clone();
                path.segments.push(name.to_string());
                let root_id    = path.root_id;
                let segments   = path.segments;
                let path       = language_server::Path{root_id,segments};
                let identifier = VisualisationIdentifier::File(path);
                Some(identifier)
            } else {
                None
            }
        }).collect();
        Ok(result)
    }

    fn list_embedded_visualisations(&self) -> Vec<VisualisationIdentifier> {
        let result = self.embedded_visualisations.keys().cloned();
        let result = result.map(VisualisationIdentifier::Embedded);
        result.collect()
    }

    /// Get a list of all available visualisations.
    pub async fn list_visualisations(&self) -> FallibleResult<Vec<VisualisationIdentifier>> {
        let mut visualisations = self.list_embedded_visualisations();
        visualisations.extend_from_slice(&self.list_file_visualisations().await?);
        Ok(visualisations)
    }

    /// Load the source code of the specified visualisation.
    pub async fn load_visualisation
    (&self, visualisation:&VisualisationIdentifier) -> FallibleResult<String> {
        match visualisation {
            VisualisationIdentifier::Embedded(identifier) => {
                let result     = self.embedded_visualisations.get(identifier);
                let identifier = visualisation.clone();
                let error      = || VisualisationError::NotFound{identifier}.into();
                result.cloned().ok_or_else(error)
            },
            VisualisationIdentifier::File(path) =>
                Ok(self.language_server_rpc.read_file(&path).await?.contents)
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
        let mock_client = language_server::MockClient::default();

        let root_id = uuid::Uuid::default();
        let path    = Path{root_id,segments:vec!["visualisation".into()]};

        let path0 = Path::new(root_id,&["visualisation","histogram.js"]);
        let path1 = Path::new(root_id,&["visualisation","graph.js"]);

        let paths   = vec![
            FileSystemObject::new_file(path0.clone()),
            FileSystemObject::new_file(path1.clone()),
        ];
        let file_list_result = language_server::response::FileList{paths};
        mock_client.set_file_list_result(path, Ok(file_list_result));

        let file_content0 = "<histogram code>".to_string();
        let file_content1 = "<graph code>".to_string();
        let result0 = language_server::response::Read{contents:file_content0.clone()};
        let result1 = language_server::response::Read{contents:file_content1.clone()};
        mock_client.set_file_read_result(path0.clone(),Ok(result0));
        mock_client.set_file_read_result(path1.clone(),Ok(result1));

        let language_server             = language_server::Connection::new_mock_rc(mock_client);
        let mut embedded_visualisations = EmbeddedVisualisations::default();
        let embedded_content            = "<extremely fast point cloud code>".to_string();
        embedded_visualisations.insert("PointCloud".to_string(),embedded_content.clone());
        let vis_controller           = Handle::new(language_server,embedded_visualisations);

        let visualisations = result(vis_controller.list_visualisations());
        let visualisations = visualisations.expect("Couldn't list visualisations.");

        assert_eq!(visualisations[0],VisualisationIdentifier::Embedded("PointCloud".to_string()));
        assert_eq!(visualisations[1],VisualisationIdentifier::File(path0));
        assert_eq!(visualisations[2],VisualisationIdentifier::File(path1));

        let content = vec![embedded_content,file_content0,file_content1];
        let zipped  = visualisations.iter().zip(content.iter());
        for (visualisation,content) in zipped {
            let loaded_content = result(vis_controller.load_visualisation(&visualisation));
            let loaded_content = loaded_content.expect("Couldn't load visualisation's content.");
            assert_eq!(*content,loaded_content);
        }
    }
}
