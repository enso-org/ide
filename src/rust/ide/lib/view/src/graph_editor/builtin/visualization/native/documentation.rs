//! Documentation view visualization generating and presenting Enso Documentation under
//! the documented node.

use crate::prelude::*;

use crate::graph_editor::component::visualization;

use enso_frp as frp;
use ensogl::display::DomSymbol;
use ensogl::display::scene::Scene;
use ensogl::display;
use ensogl::system::web;
use ensogl::system::web::StyleSetter;
use ast::prelude::FallibleResult;


// =================
// === Constants ===
// =================

pub const DOC_VIEW_WIDTH  : f32 = 300.0;
pub const DOC_VIEW_MARGIN : f32 = 15.0;

/// Content in the documentation view when there is no data available.
const PLACEHOLDER_STR : &str = "<h3>Documentation Viewer</h3><p>No documentation available</p>";
const CORNER_RADIUS   : f32  = crate::graph_editor::component::node::CORNER_RADIUS;

/// Gets documentation view stylesheet from a CSS file.
///
/// TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO TODO
///      This file is generated currently from SASS file and will be moved to rust-based
///      generator to achieve compatibility with IDE's theme manager.
///      Expect them to land with https://github.com/enso-org/ide/issues/709
pub fn doc_style() -> String {
    format!("<style>{}</style>", include_str!("documentation/style.css"))
}



/// Model of Native visualization that generates documentation for given Enso code and embeds
/// it in a HTML container.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ViewModel {
    logger : Logger,
    dom    : DomSymbol,
    size   : Rc<Cell<Vector2>>,
}

#[allow(dead_code)]
impl ViewModel {
    /// Constructor.
    fn new(scene:&Scene) -> Self {
        let logger          = Logger::new("DocumentationView");
        let div             = web::create_div();
        let dom             = DomSymbol::new(&div);
        let screen          = scene.camera().screen();
        let doc_view_height = screen.height - (DOC_VIEW_MARGIN * 2.0);
        let size_vec        = Vector2(DOC_VIEW_WIDTH,doc_view_height);
        let size            = Rc::new(Cell::new(size_vec));

        dom.dom().set_style_or_warn("white-space"     ,"normal"                        ,&logger);
        dom.dom().set_style_or_warn("overflow-y"      ,"auto"                          ,&logger);
        dom.dom().set_style_or_warn("overflow-x"      ,"auto"                          ,&logger);
        dom.dom().set_style_or_warn("background-color","rgba(255, 255, 255, 0.85)"     ,&logger);
        dom.dom().set_style_or_warn("padding"         ,"5px"                           ,&logger);
        dom.dom().set_style_or_warn("pointer-events"  ,"auto"                          ,&logger);
        dom.dom().set_style_or_warn("border-radius"   ,format!("{}px", CORNER_RADIUS)  ,&logger);
        dom.dom().set_style_or_warn("width"           ,format!("{}px", DOC_VIEW_WIDTH) ,&logger);
        dom.dom().set_style_or_warn("height"          ,format!("{}px", doc_view_height),&logger);

        scene.dom.layers.main.manage(&dom);
        ViewModel {dom,logger,size}.init()
    }

    fn init(self) -> Self {
        self.reload_style();
        self
    }

    /// Sets size of the documentation view.
    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.reload_style();
    }

    /// Generates HTML documentation from documented Enso code.
    fn gen_html_from(program:String) -> FallibleResult<String> {
        let parser = parser::DocParser::new()?;
        let output = parser.generate_html_docs(program);
        Ok(output?)
    }

    /// Generates HTML documentation from pure Enso documentation.
    fn gen_html_from_pure(doc:String) -> FallibleResult<String> {
        let parser = parser::DocParser::new()?;
        let output = parser.generate_html_doc_pure(doc);
        Ok(output?)
    }

    /// Prepares data string for Doc Parser to work with after getting deserialization.
    /// FIXME : Removes characters that are not supported by Doc Parser yet.
    ///         https://github.com/enso-org/enso/issues/1063
    fn prepare_data_string(data_inner:&visualization::Json) -> String {
        let data_str = serde_json::to_string_pretty(&**data_inner);
        let data_str = data_str.unwrap_or_else(|e| format!("<Cannot render data: {}>", e));
        let data_str = data_str.replace("\\n", "\n");
        data_str.replace("\"", "")
    }

    /// Creates a container for generated content and embeds it with stylesheet.
    fn push_to_dom(&self, content:String) {
        let data_str = format!(r#"<div class="docVis">{}{}</div>"#, doc_style(), content);
        self.dom.dom().set_inner_html(&data_str)
    }

    /// Receives data, processes and presents it in the documentation view.
    fn receive_data(&self, data:&visualization::Data) -> Result<(),visualization::DataError> {
        let data_inner = match data {
            visualization::Data::Json {content} => content,
            _                                   => todo!(),
        };

        let data_str   = ViewModel::prepare_data_string(data_inner);
        let output     = ViewModel::gen_html_from(data_str);
        let mut output = output.unwrap_or_else(|_| String::from(PLACEHOLDER_STR));
        if output     == "" { output = String::from(PLACEHOLDER_STR) }
        // FIXME : Doc Parser related idea, where stylesheet was a separate file.
        //         Will be fixed after a commit in Engine repo and in next PR.
        let import_css = r#"<link rel="stylesheet" href="style.css" />"#;
        let output     = output.replace(import_css, "");

        self.push_to_dom(output);
        Ok(())
    }

    /// Loads an HTML file into the documentation view when there is no docstring available yet.
    /// TODO : This will be replaced with a spinner in next PR.
    fn load_no_doc_screen(&self) {
        self.push_to_dom(String::from("<p>Please wait...</p>"))
    }

    fn reload_style(&self) {
        self.dom.set_size(self.size.get());
    }
}



// ============
// === View ===
// ============

/// View of the visualization that renders the given documentation as a HTML page.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct View {
    #[shrinkwrap(main_field)]
    pub model : ViewModel,
    pub frp   : visualization::instance::Frp,
    network   : frp::Network,
}

impl View {
    /// Definition of this visualization.
    pub fn definition() -> visualization::Definition {
        let path = visualization::Path::builtin("Documentation View");
        visualization::Definition::new(
            visualization::Signature::new_for_any_type(path,visualization::Format::Json),
            |scene| { Ok(Self::new(scene).into()) }
        )
    }

    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let network = default();
        let frp     = visualization::instance::Frp::new(&network);
        let model   = ViewModel::new(scene);
        model.load_no_doc_screen();
        Self {model,frp,network} . init()
    }

    fn init(self) -> Self {
        let network = &self.network;
        let model   = self.model.clone_ref();
        let frp     = self.frp.clone_ref();
        frp::extend! { network
            eval frp.set_size  ((size) model.set_size(*size));
            eval frp.send_data ([frp](data) {
                if let Err(e) = model.receive_data(data) {
                    frp.data_receive_error.emit(Some(e));
                }
             });
        }
        self
    }
}

impl From<View> for visualization::Instance {
    fn from(t: View) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance {
        &self.dom.display_object()
    }
}
