//! Documentation view visualization generating and presenting Enso Documentation under
//! the documented node.

use crate::prelude::*;

use crate::graph_editor::component::visualization;
use crate::graph_editor::component::visualization::Instance;
use crate::graph_editor::component::visualization::Data;
use crate::graph_editor::component::visualization::Signature;
use crate::graph_editor::component::visualization::Path;
use crate::graph_editor::component::visualization::Format;
use crate::graph_editor::component::visualization::DataError;
use crate::graph_editor::component::visualization::Definition;

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

const CORNER_RADIUS : f32 = crate::graph_editor::component::node::CORNER_RADIUS;
/// Content in the documentation view when the data is yet to be received.
const PLACEHOLDER_STR: &'static str = "<h3>Enso Documentation Viewer</h3>\
                                           <p>No documentation available</p>";


/// Generates documentation view stylesheet.
pub fn get_doc_style() -> String {
    format!("<style>{}</style>", include_str!("documentation/style.css"))
}

#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ViewModel {
    logger : Logger,
    dom    : DomSymbol,
    size   : Rc<Cell<Vector2>>,
}

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

    /// Generates HTML documentation for documented suggestion.
    fn gen_doc(doc: String) -> FallibleResult<String> {
        let parser = parser::DocParser::new()?;
        let output = parser.generate_html_doc_pure(doc);
        Ok(output?)
    }

    fn receive_data(&self, data:&Data) -> Result<(),DataError> {
        let data_inner = match data {
            Data::Json {content} => content,
            _                    => todo!(),
        };

        let data_str = serde_json::to_string_pretty(&**data_inner);
        let data_str = data_str.unwrap_or_else(|e| format!("<Cannot render data: {}>", e));
        // Fixes a Doc Parser Bug - to be removed when rewritten to rust
        let data_str = data_str.replace("\\n", "\n");
        let data_str = data_str.replace("\"", "");

        let output = ViewModel::gen_doc(data_str);
        let output = output.unwrap_or_else(|_| String::from(PLACEHOLDER_STR));
        // Fixes a Doc Parser related idea, where stylesheet was a separate file
        let output = output.replace(r#"<link rel="stylesheet" href="style.css" />"#, "");

        let data_str = format!(r#"<div class="docVis">{}{}</div>"#, get_doc_style(), output);
        self.dom.dom().set_inner_html(&data_str);
        Ok(())
    }

    /// Loads an HTML file into the documentation view when there is no docstring available.
    fn load_no_doc_screen(&self) {
        let data_str = format!(r#"<div class="docVis">{}{}</div>"#, get_doc_style(), PLACEHOLDER_STR);
        self.dom.dom().set_inner_html(&data_str)
    }

    fn reload_style(&self) {
        self.dom.set_size(self.size.get());
    }
}

// ============
// === View ===
// ============

/// Visualization that renders the given documentation as a HTML page
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
    pub fn definition() -> Definition {
        let path = Path::builtin("Documentation View Visualization (native)");
        Definition::new(
            Signature::new_for_any_type(path,Format::Json),
            |scene| { Ok(Self::new(scene).into()) }
        )
    }

    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let network = default();
        let frp   = visualization::instance::Frp::new(&network);
        let model = ViewModel::new(scene);
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

impl From<View> for Instance {
    fn from(t: View) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance {
        &self.dom.display_object()
    }
}
