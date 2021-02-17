//! Example visualisation showing the provided data as text.

use crate::prelude::*;

use crate::component::visualization::*;
use crate::component::visualization;

use enso_frp as frp;
use ensogl::data::color;
use ensogl::display::DomSymbol;
use ensogl::display::scene::Scene;
use ensogl::display::shape::primitive::StyleWatch;
use ensogl::display;
use ensogl::system::web;
use ensogl::system::web::StyleSetter;
use ensogl_theme;
use ensogl::system::web::AttributeSetter;



// =================
// === Constants ===
// =================

const PADDING_TEXT: f32 = 10.0;



// ===============
// === RawText ===
// ===============

/// Sample visualization that renders the given data as text. Useful for debugging and testing.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Error {
    model   : Model,
    frp     : visualization::instance::Frp,
    network : frp::Network,
}

impl Deref for Error {
    type Target = visualization::instance::Frp;

    fn deref(&self) -> &Self::Target { &self.frp }
}

impl Error {

    pub fn path() -> Path { Path::builtin("Error") }

    /// Definition of this visualization.
    pub fn definition() -> Definition {
        let path = Self::path();
        Definition::new(
            Signature::new_for_any_type(path,Format::Json),
            |scene| { Ok(Self::new(scene).into()) }
        )
    }

    /// Constructor.
    pub fn new(scene:&Scene) -> Self {
        let network = frp::Network::new("js_visualization_raw_text");
        let frp     = visualization::instance::Frp::new(&network);
        let model   = Model::new(scene);
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

#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Model {
    logger : Logger,
    dom    : DomSymbol,
    size   : Rc<Cell<Vector2>>,
}

impl Model {
    /// Constructor.
    fn new(scene:&Scene) -> Self {
        let logger = Logger::new("RawText");
        let div    = web::create_div();
        let dom    = DomSymbol::new(&div);
        let size   = Rc::new(Cell::new(Vector2(200.0,200.0)));

        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        let styles       = StyleWatch::new(&scene.style_sheet);
        let text_color   = styles.get_color(ensogl_theme::graph_editor::visualization::error::text);
        let text_color   = color::Rgba::from(text_color);
        let red         = text_color.red * 255.0;
        let green       = text_color.green * 255.0;
        let blue        = text_color.blue * 255.0;
        let text_color   = format!("rgba({},{},{},{})",red,green,blue,text_color.alpha);
        let padding_text = format!("{}px",PADDING_TEXT);

        dom.dom().set_attribute_or_warn("class","visualization scrollable",&logger);
        dom.dom().set_style_or_warn("overflow-y"    ,"auto"               ,&logger);
        dom.dom().set_style_or_warn("overflow-x"    ,"auto"               ,&logger);
        dom.dom().set_style_or_warn("font-family"   ,"DejaVuSansMonoBook" ,&logger);
        dom.dom().set_style_or_warn("font-size"     ,"12px"               ,&logger);
        dom.dom().set_style_or_warn("padding-left"  ,&padding_text        ,&logger);
        dom.dom().set_style_or_warn("padding-top"   ,&padding_text        ,&logger);
        dom.dom().set_style_or_warn("color"         ,text_color           ,&logger);
        dom.dom().set_style_or_warn("pointer-events","auto"               ,&logger);

        scene.dom.layers.back.manage(&dom);
        Model{dom,logger,size}.init()
    }

    fn init(self) -> Self {
        self.reload_style();
        self
    }

    fn set_size(&self, size:Vector2) {
        let x_mod = size.x - PADDING_TEXT;
        let y_mod = size.y - PADDING_TEXT;
        let size  = Vector2(x_mod,y_mod);
        self.size.set(size);
        self.reload_style();
    }

    fn receive_data(&self, data:&Data) -> Result<(),DataError> {
        if let Data::Json {content} = data {
            let conversion_error = |e:serde_json::Error|
                format!("<Cannot display error message: {}>", e);
            let message = if let serde_json::Value::String(str) = content.deref() { str.clone() }
                else { serde_json::to_string_pretty(&**content).unwrap_or_else(conversion_error) };
            self.dom.dom().set_inner_text(&message);
            Ok(())
        } else {
            Err(DataError::InvalidDataType.into())
        }
    }

    fn reload_style(&self) {
        self.dom.set_size(self.size.get());
    }
}

impl From<Error> for Instance {
    fn from(t: Error) -> Self {
        Self::new(&t,&t.frp,&t.network)
    }
}

impl display::Object for Error {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.dom.display_object()
    }
}
