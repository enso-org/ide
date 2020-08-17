//! Documentation view visualization generating and presenting Enso Documentation under
//! the documented node.

use crate::prelude::*;

use crate::graph_editor::component::visualization;

use enso_frp as frp;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::camera::Camera2d;
use ensogl::display::DomSymbol;
use ensogl::display::object::ObjectOps;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::Sprite;
use ensogl::gui::component;
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

/// Get documentation view stylesheet from a CSS file.
///
/// TODO [MM] : This file is generated currently from SASS file, and generated code should never
///             be included in a codebase, so it will be moved to rust-based generator to achieve
///             compatibility with IDE's theme manager.
///             Expect them to land with https://github.com/enso-org/ide/issues/709
pub fn doc_style() -> String {
    format!("<style>{}</style>", include_str!("documentation/style.css"))
}



// ==================
// === Background ===
// ==================

mod background {
    use super::*;

    ensogl::define_shape_system! {
         () {
             let bg_color = color::Rgba::new(1.0,0.0,0.0,1.0);
             Plane().fill(bg_color).into()
         }
     }
}



// =================
// === ViewModel ===
// =================

/// Model of Native visualization that generates documentation for given Enso code and embeds
/// it in a HTML container.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct ViewModel {
    logger         : Logger,
    display_object : display::object::Instance,
    background     : component::ShapeView<background::Shape>,
    dom            : DomSymbol,
    scene          : Scene,
    camera         : Camera2d,
    size           : Rc<Cell<Vector2>>,
}

impl ViewModel {
    /// Constructor.
    fn new(scene:&Scene) -> Self {
        let logger          = Logger::new("DocumentationView");
        let display_object  = display::object::Instance::new(&logger);
        let scene           = scene.clone_ref();
        let camera          = scene.camera().clone_ref();
        let div             = web::create_div();
        let dom             = DomSymbol::new(&div);
        let screen          = camera.screen();
        let doc_view_height = screen.height - (DOC_VIEW_MARGIN * 2.0);
        let size_vec        = Vector2(DOC_VIEW_WIDTH,doc_view_height);
        let size            = Rc::new(Cell::new(size_vec));
        let background      = component::ShapeView::<background::Shape>::new(&logger,&scene);

        let shape_system = scene.shapes.shape_system(PhantomData::<background::Shape>);
        scene.views.main.remove(&shape_system.shape_system.symbol);
        scene.views.cursor.add(&shape_system.shape_system.symbol);

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

        ViewModel {logger,display_object,background,dom,scene,camera,size}.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.background);
        self.dom.unset_parent();
        self.background.add_child(&self.dom);
        self.reload_style();
        self
    }

    /// Set size of the documentation view.
    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.reload_style();
    }

    /// Generate HTML documentation from documented Enso code.
    fn gen_html_from(program:String) -> FallibleResult<String> {
        let parser = parser::DocParser::new()?;
        let output = parser.generate_html_docs(program);
        Ok(output?)
    }

    /// Prepare data string for Doc Parser to work with after deserialization.
    /// FIXME [MM]:  Removes characters that are not supported by Doc Parser yet.
    ///              https://github.com/enso-org/enso/issues/1063
    fn prepare_data_string(data_inner:&visualization::Json) -> String {
        let data_str = serde_json::to_string_pretty(&**data_inner);
        let data_str = data_str.unwrap_or_else(|e| format!("<Cannot render data: {}>", e));
        let data_str = data_str.replace("\\n", "\n");
        data_str.replace("\"", "")
    }

    /// Create a container for generated content and embed it with stylesheet.
    fn push_to_dom(&self, content:String) {
        let data_str = format!(r#"<div class="docVis">{}{}</div>"#, doc_style(), content);
        self.dom.dom().set_inner_html(&data_str)
    }

    /// Receive data, process and present it in the documentation view.
    fn receive_data(&self, data:&visualization::Data) -> Result<(),visualization::DataError> {
        let data_inner = match data {
            visualization::Data::Json {content} => content,
            _                                   => todo!(),
        };

        let data_str   = ViewModel::prepare_data_string(data_inner);
        let output     = ViewModel::gen_html_from(data_str);
        let mut output = output.unwrap_or_else(|_| String::from(PLACEHOLDER_STR));
        if output     == "" { output = String::from(PLACEHOLDER_STR) }
        // FIXME [MM] : Because of how Doc Parser was implemented in Engine repo, there is need to
        //              remove stylesheet link from generated code, that would otherwise point to
        //              non-existing file, as now stylesheet is connected by include_str! macro, and
        //              soon will be replaced by a style generator.
        //              This hack will be removed when https://github.com/enso-org/enso/issues/1063
        //              will land in Engine's repo, also fixing non-existent character bug.
        let import_css = r#"<link rel="stylesheet" href="style.css" />"#;
        let output     = output.replace(import_css, "");

        self.push_to_dom(output);
        Ok(())
    }

    /// Load an HTML file into the documentation view when user is waiting for data to be received.
    /// TODO [MM] : This will be replaced with a spinner in next PR.
    fn load_no_doc_screen(&self) {
        self.push_to_dom(String::from("<p>Please wait...</p>"))
    }

    fn reload_style(&self) {
        let position_x = (self.camera.screen().width - DOC_VIEW_WIDTH) / 2.0 - DOC_VIEW_MARGIN;
        self.dom.set_size(self.size.get());
        self.dom.set_position_x(position_x);

        self.background.shape.sprite.size.set(self.size.get());
        self.background.set_position_x(position_x);
    }
}

impl display::Object for ViewModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
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
        let model   = &self.model;
        let frp     = &self.frp;
        frp::extend! { network
            eval frp.set_size  ((size) model.set_size(*size));
            eval frp.send_data ([frp, model](data) {
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
