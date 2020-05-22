//! Visualization controller.
//!
//! Ths Visualization Controller is Responsible identifying all the available visualization natively
//! embedded in IDE and available within the project's `visualization` folder. The
//! `Visualization Controller` lives as long as the `Project Controller`.

use crate::prelude::*;

use std::rc::Rc;
use enso_protocol::language_server;
use graph_editor::GraphEditor;
use enso_frp::stream::EventEmitter;
use graph_editor::component::visualization::{ClassHandle, Class, NativeConstructorClass, ClassAttributes, Visualization, JsRenderer, JsSourceClass};
use ensogl::display::Scene;


// =============
// === Error ===
// =============

/// Enumeration of errors used in `Visualization Controller`.
#[derive(Debug,Fail)]
#[allow(missing_docs)]
pub enum VisualizationError {
    #[fail(display = "Visualization \"{}\" not found.", identifier)]
    NotFound {
        identifier : VisualizationIdentifier
    },
    #[fail(display = "JavaScript visualization \"{}\" failed to compile.", identifier)]
    CompileError {
        identifier : VisualizationIdentifier
    }
}



// ===============================
// === VisualizationIdentifier ===
// ===============================

/// This enum is used to identify a visualiser both in the project folder or natively embedded in
/// IDE.
#[derive(Clone,Debug,Display,Eq,PartialEq)]
#[allow(missing_docs)]
pub enum VisualizationIdentifier {
    Embedded(String),
    File(language_server::Path)
}



// ==============================
// === EmbeddedVisualizations ===
// ==============================

#[allow(missing_docs)]
pub type EmbeddedVisualizationName   = String;
#[allow(missing_docs)]
pub type EmbeddedVisualizationSource = String;

/// Embedded visualizations mapped from name to source code.
#[derive(Shrinkwrap,Debug,Clone,Default)]
#[shrinkwrap(mutable)]
pub struct EmbeddedVisualizations {
    #[allow(missing_docs)]
    pub map : HashMap<EmbeddedVisualizationName,EmbeddedVisualizationSource>
}




// ==============
// === Handle ===
// ==============

const VISUALISATION_FOLDER : &str = "visualization";

/// Visualization Controller's state.
#[derive(Debug,Clone)]
pub struct Handle {
    language_server_rpc     : Rc<language_server::Connection>,
    embedded_visualizations : EmbeddedVisualizations,
    graph_editor            : RefCell<Option<GraphEditor>>
}

impl Handle {
    /// Creates a new visualization controller.
    pub fn new
    ( language_server_rpc     : Rc<language_server::Connection>
    , embedded_visualizations : EmbeddedVisualizations) -> Self {
        let graph_editor = RefCell::new(None);
        Self {language_server_rpc,embedded_visualizations,graph_editor}
    }

    /// Sets the GraphEditor to report about visualizations availability.
    pub async fn set_graph_editor(&self, graph_editor:Option<GraphEditor>) -> FallibleResult<()> {
        let identifiers = self.list_visualizations().await;
        let identifiers = identifiers.unwrap_or_default();
        for identifier in identifiers {
            let visualization = self.load_visualization(&identifier).await;
            visualization.map(|visualization| {
                graph_editor.as_ref().map(|graph_editor| {
                    let class_handle = &Some(Rc::new(ClassHandle::new(visualization)));
                    graph_editor.frp.register_visualization_class.emit_event(class_handle);
                });
            })?;
        }
        *self.graph_editor.borrow_mut() = graph_editor;
        Ok(())
    }

    async fn list_file_visualizations(&self) -> FallibleResult<Vec<VisualizationIdentifier>> {
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
                let identifier = VisualizationIdentifier::File(path);
                Some(identifier)
            } else {
                None
            }
        }).collect();
        Ok(result)
    }

    fn list_embedded_visualizations(&self) -> Vec<VisualizationIdentifier> {
        let result = self.embedded_visualizations.keys().cloned();
        let result = result.map(VisualizationIdentifier::Embedded);
        result.collect()
    }

    /// Get a list of all available visualizations.
    pub async fn list_visualizations(&self) -> FallibleResult<Vec<VisualizationIdentifier>> {
        let mut visualizations = self.list_embedded_visualizations();
        visualizations.extend_from_slice(&self.list_file_visualizations().await?);
        Ok(visualizations)
    }

    /// Load the source code of the specified visualization.
    pub async fn load_visualization
    (&self, visualization:&VisualizationIdentifier) -> FallibleResult<impl Class> {
        let js_code = match visualization {
            VisualizationIdentifier::Embedded(identifier) => {
                let result     = self.embedded_visualizations.get(identifier);
                let identifier = visualization.clone();
                let error      = || VisualizationError::NotFound{identifier}.into();
                let result : FallibleResult<String> = result.cloned().ok_or_else(error);
                result?
            },
            VisualizationIdentifier::File(path) =>
                self.language_server_rpc.read_file(&path).await?.contents
        };
        JsSourceClass::from_js_source_raw(&js_code).map_err(|_| {
            let identifier = visualization.clone();
            VisualizationError::CompileError{identifier}.into()
        })
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
        let path    = Path{root_id,segments:vec!["visualization".into()]};

        let path0 = Path::new(root_id,&["visualization","histogram.js"]);
        let path1 = Path::new(root_id,&["visualization","graph.js"]);

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
        let mut embedded_visualizations = EmbeddedVisualizations::default();
        let embedded_content            = "<extremely fast point cloud code>".to_string();
        embedded_visualizations.insert("PointCloud".to_string(),embedded_content.clone());
        let vis_controller           = Handle::new(language_server,embedded_visualizations);

        let visualizations = result(vis_controller.list_visualizations());
        let visualizations = visualizations.expect("Couldn't list visualizations.");

        assert_eq!(visualizations[0],VisualizationIdentifier::Embedded("PointCloud".to_string()));
        assert_eq!(visualizations[1],VisualizationIdentifier::File(path0));
        assert_eq!(visualizations[2],VisualizationIdentifier::File(path1));

        let content = vec![embedded_content,file_content0,file_content1];
        let zipped  = visualizations.iter().zip(content.iter());
        for (visualization,content) in zipped {
            let loaded_content = result(vis_controller.load_visualization(&visualization));
            let loaded_content = loaded_content.expect("Couldn't load visualization's content.");
            assert_eq!(*content,loaded_content);
        }
    }
}
