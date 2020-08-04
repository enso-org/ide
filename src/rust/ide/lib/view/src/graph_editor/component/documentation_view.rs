//! Documentation view visualization generating and presenting Enso Documentation under
//! the documented node.

use crate::prelude::*;

use crate::graph_editor::component::visualization::*;
use crate::graph_editor::component::visualization;

use enso_frp as frp;
use ensogl::display::DomSymbol;
use ensogl::display::scene::Scene;
use ensogl::display;
use ensogl::system::web;
use ensogl::system::web::StyleSetter;

/// Generates Documentation View stylesheet.
pub fn get_doc_style() -> String {
    format!("<style>{}</style>", include_str!("documentation_view/style.css"))
}

#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct DocumentationViewModel {
    logger : Logger,
    dom    : DomSymbol,
    size   : Rc<Cell<Vector2>>,
}

impl DocumentationViewModel {
    /// Constructor.
    fn new(scene:&Scene) -> Self {
        let logger = Logger::new("DocumentationView");
        let div    = web::create_div();
        let dom    = DomSymbol::new(&div);
        let screen = scene.camera().screen();
        let size   = Rc::new(Cell::new(Vector2(290.0,screen.height - 30.0)));

        dom.dom().set_style_or_warn("white-space"     ,"normal"                             ,&logger);
        dom.dom().set_style_or_warn("overflow-y"      ,"auto"                               ,&logger);
        dom.dom().set_style_or_warn("overflow-x"      ,"auto"                               ,&logger);
        dom.dom().set_style_or_warn("background-color","rgba(255, 255, 255, 0.85)"          ,&logger);
        dom.dom().set_style_or_warn("padding"         ,"5px"                                ,&logger);
        dom.dom().set_style_or_warn("pointer-events"  ,"auto"                               ,&logger);
        dom.dom().set_style_or_warn("border-radius"   ,"14px"                               ,&logger);
        dom.dom().set_style_or_warn("width"           ,format!("{}px", 290)                 ,&logger);
        dom.dom().set_style_or_warn("height"          ,format!("{}px", screen.height - 30.0),&logger);

        scene.dom.layers.main.manage(&dom);
        DocumentationViewModel{dom,logger,size}.init()
    }

    fn init(self) -> Self {
        self.reload_style();
        self
    }

    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.reload_style();
    }

    fn placeholder_str() -> String {
        "<h3>Enso Documentation Viewer</h3><p>No documentation available</p>".to_string()
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

        let parser = parser::DocParser::new_or_panic();
        let output = parser.generate_html_doc_pure(data_str);
        let output = output.unwrap_or_else(|_| DocumentationViewModel::placeholder_str());
        // Fixes a Doc Parser related idea, where stylesheet was a separate file
        let output = output.replace(r#"<link rel="stylesheet" href="style.css" />"#, "");

        let data_str = format!(r#"<div class="docVis">{}{}</div>"#, get_doc_style(), output);
        self.dom.dom().set_inner_html(&data_str);
        Ok(())
    }

    /// Generates welcome screen HTML.
    pub fn welcome_screen(&self) {
        let placeholder = DocumentationViewModel::placeholder_str();
        let data_str    = format!(r#"<div class="docVis">{}{}</div>"#, get_doc_style(), placeholder);
        self.dom.dom().set_inner_html(&data_str)
    }

    fn reload_style(&self) {
        self.dom.set_size(self.size.get());
    }
}

// =========================
// === DocumentationView ===
// =========================

/// Visualization that renders the given documentation as a HTML page
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct DocumentationView {
    #[shrinkwrap(main_field)]
    pub model : DocumentationViewModel,
    pub frp   : visualization::instance::Frp,
    network   : frp::Network,
}

impl DocumentationView {
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
        let model = DocumentationViewModel::new(scene);
        model.welcome_screen();
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

impl From<DocumentationView> for Instance {
    fn from(t:DocumentationView) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for DocumentationView {
    fn display_object(&self) -> &display::object::Instance {
        &self.dom.display_object()
    }
}
