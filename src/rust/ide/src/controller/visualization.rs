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
use graph_editor::component::visualization::class;
use graph_editor::component::visualization::JsSourceClass;



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

/// This enum is used to identify a visualization both in the project folder or natively embedded in
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

/// Embedded visualizations mapped from name to source code.
#[derive(Shrinkwrap,Debug,Clone,CloneRef,Default)]
#[shrinkwrap(mutable)]
pub struct EmbeddedVisualizations {
    #[allow(missing_docs)]
    pub map : Rc<HashMap<EmbeddedVisualizationName,Rc<class::Handle>>>
}




// ==============
// === Handle ===
// ==============

const VISUALISATION_FOLDER : &str = "visualization";

/// Visualization Controller's state.
#[derive(Debug,Clone,CloneRef)]
pub struct Handle {
    language_server_rpc     : Rc<language_server::Connection>,
    embedded_visualizations : EmbeddedVisualizations,
    graph_editor            : Rc<RefCell<Option<GraphEditor>>>
}

impl Handle {
    /// Creates a new visualization controller.
    pub fn new
    ( language_server_rpc     : Rc<language_server::Connection>
    , embedded_visualizations : EmbeddedVisualizations) -> Self {
        let graph_editor = Rc::new(RefCell::new(None));
        Self {language_server_rpc,embedded_visualizations,graph_editor}
    }

    /// Sets the GraphEditor to report about visualizations availability.
    pub async fn set_graph_editor(&self, graph_editor:Option<GraphEditor>) -> FallibleResult<()> {
        let identifiers = self.list_visualizations().await;
        let identifiers = identifiers.unwrap_or_default();
        for identifier in identifiers {
            let visualization = self.load_visualization(&identifier).await;
            let visualization = visualization.map(|visualization| {
                if let Some(graph_editor) = graph_editor.as_ref() {
                    let class_handle = &Some(visualization);
                    graph_editor.frp.register_visualization_class.emit_event(class_handle);
                }
            });
            visualization?;
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
    (&self, visualization:&VisualizationIdentifier) -> FallibleResult<Rc<class::Handle>> {
        match visualization {
            VisualizationIdentifier::Embedded(identifier) => {
                let result     = self.embedded_visualizations.get(identifier);
                let identifier = visualization.clone();
                let error      = || VisualizationError::NotFound{identifier}.into();
                result.cloned().ok_or_else(error)
            },
            VisualizationIdentifier::File(path) => {
                let js_code    = self.language_server_rpc.read_file(&path).await?.contents;
                let identifier = visualization.clone();
                let error      = |_| VisualizationError::CompileError{identifier}.into();
                let js_class   = JsSourceClass::from_js_source_raw(&js_code).map_err(error);
                js_class.map(|js_class| Rc::new(class::Handle::new(js_class)))
            }
        }
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    use enso_protocol::language_server::FileSystemObject;
    use enso_protocol::language_server::Path;
    use graph_editor::component::visualization::{NativeConstructorClass, Signature, Visualization};
    use graph_editor::component::visualization::renderer::example::native::BubbleChart;
    use ensogl::display::Scene;
    use json_rpc::expect_call;

    use wasm_bindgen_test::wasm_bindgen_test_configure;
    use wasm_bindgen_test::wasm_bindgen_test;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test(async)]
    async fn list_and_load() {
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
        expect_call!(mock_client.file_list(path=path) => Ok(file_list_result));

        let file_content0 = r#"
            class Vis0 {
                static inputTypes = ["Float"]
                onDataReceived(root,data) {}
                setSize(root,size) {}
            }
            return Vis0
        "#.to_string();
        let file_content1 = r#"
            class Vis1 {
                static inputTypes = ["Float"]
                onDataReceived(root,data) {}
                setSize(root,size) {}
            }
            return Vis1
        "#.to_string();
        let result0 = language_server::response::Read{contents:file_content0.clone()};
        let result1 = language_server::response::Read{contents:file_content1.clone()};
        expect_call!(mock_client.read_file(path=path0.clone()) => Ok(result0));
        expect_call!(mock_client.read_file(path=path1.clone()) => Ok(result1));

        let language_server             = language_server::Connection::new_mock_rc(mock_client);
        let mut embedded_visualizations = EmbeddedVisualizations::default();
        let embedded_visualization = Rc::new(class::Handle::new(NativeConstructorClass::new(
            Signature {
                name        : "Bubble Visualization (native)".to_string(),
                input_types : vec!["[[Float,Float,Float]]".to_string().into()],
            },
            |scene:&Scene| Ok(Visualization::new(BubbleChart::new(scene)))
        )));
        embedded_visualizations.insert("PointCloud".to_string(),embedded_visualization.clone());
        let vis_controller           = Handle::new(language_server,embedded_visualizations);

        let visualizations = vis_controller.list_visualizations().await;
        let visualizations = visualizations.expect("Couldn't list visualizations.");

        assert_eq!(visualizations[0],VisualizationIdentifier::Embedded("PointCloud".to_string()));
        assert_eq!(visualizations[1],VisualizationIdentifier::File(path0));
        assert_eq!(visualizations[2],VisualizationIdentifier::File(path1));

        let javascript_vis0 = JsSourceClass::from_js_source_raw(&file_content0);
        let javascript_vis1 = JsSourceClass::from_js_source_raw(&file_content1);
        let javascript_vis0 = javascript_vis0.expect("Couldn't create visualization class.");
        let javascript_vis1 = javascript_vis1.expect("Couldn't create visualization class.");
        let javascript_vis0 = Rc::new(class::Handle::new(javascript_vis0));
        let javascript_vis1 = Rc::new(class::Handle::new(javascript_vis1));

        let content = vec![embedded_visualization,javascript_vis0,javascript_vis1];
        let zipped  = visualizations.iter().zip(content.iter());
        for (visualization,content) in zipped {
            let loaded_content   = vis_controller.load_visualization(&visualization).await;
            let loaded_content   = loaded_content.expect("Couldn't load visualization's content.");
            let loaded_signature = loaded_content.class();
            let loaded_signature = loaded_signature.as_ref().expect("Couldn't get class.").signature();
            let signature        = content.class();
            let signature        = signature.as_ref().expect("Couldn't get class.").signature();
            assert_eq!(signature,loaded_signature);
        }
    }
}
