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
    let css = r#"<style>
.docVis {
  -webkit-font-smoothing: antialiased;
  font-style: normal;
  word-wrap: break-word;
  font-size: 17px;
  line-height: 1.52947;
  font-weight: 400;
  letter-spacing: -0.021em;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  color: #333333;
  margin: 0;
  padding: 0;
}

.docVis p {
  display: block;
  margin-block-start: 1em;
  margin-block-end: 1em;
  margin-inline-start: 0;
  margin-inline-end: 0;
}

.docVis a:hover {
  color: #0070c9 !important;
  text-decoration: inherit;
}

.docVis a {
  color: #333333;
  background-color: transparent;
  text-decoration: inherit;
  display: inline-block;
  transition: all 0.3s ease;
}

.docVis img {
  display: block;
}

.docVis code {
  color: #0070c9;
  background-color: transparent;
  font-size: inherit;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  line-height: inherit;
  display: inline-block;
  white-space: pre-wrap;
}

.docVis button {
  display: inline-block;
  padding: 8px 30px;
  margin: 10px 0;
  outline: none;
  background-color: transparent;
  border: 1px solid #333333;
  color: #333333;
  border-radius: 5px;
  font-size: 13px;
  vertical-align: top;
  transition: all 0.3s ease;
}

.docVis button:hover {
  background-color: #333333;
  color: #e5e5e5;
}

.docVis b {
  font-weight: 600;
}

.docVis h1 {
  font-size: 34px;
  line-height: 1.08824;
  font-weight: 500;
  letter-spacing: 0.01em;
}

.docVis h2 {
  font-size: 28px;
  line-height: 1.1073;
  font-weight: 500;
  letter-spacing: 0.012em;
}

.Body h2 {
  margin: 0.65rem 0 0;
}

.docVis li {
  padding-left: 10px;
}

.creator .Unclosed,
.creator .invalidIndent,
.creator .invalidLink {
  display: inline;
  color: orangered;
}
.creator .Tags .UNRECOGNIZED {
  border: 2px solid;
  color: orangered;
}

.Unclosed,
.invalidIndent,
.invalidLink {
  display: inline;
}

.Header {
  font-size: 19px;
  font-weight: 500;
}

.Important .Header,
.Info .Header,
.Example .Header {
  margin-bottom: 0.7em;
  font-weight: 600;
  letter-spacing: -0.021em;
  line-height: 17px;
  font-synthesis: none;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
}

.Tags {
  margin-left: auto;
  margin-right: auto;
  margin-bottom: 20px;
  padding-top: 15px;
}
.Tags .DEPRECATED,
.Tags .MODIFIED,
.Tags .ADDED,
.Tags .UPCOMING,
.Tags .REMOVED,
.Tags .UNRECOGNIZED {
  line-height: 1.5;
  font-weight: 400;
  border-radius: 4px;
  font-size: 12px;
  letter-spacing: -0.021em;
  display: inline-flex;
  padding: 5px 15px;
  margin: 2px;
  white-space: nowrap;
  background: transparent;
}
.Tags .DEPRECATED {
  border: 1px solid #d20606;
  color: #d20606;
}
.Tags .MODIFIED {
  border: 1px solid #003ec3;
  color: #003ec3;
}
.Tags .ADDED {
  border: 1px solid #79A129;
  color: #79A129;
}
.Tags .UPCOMING,
.Tags .REMOVED,
.Tags .UNRECOGNIZED {
  border: 1px solid #666666;
  color: #666666;
}

.ExtForTagDetails {
  margin: 0 3px;
  color: #999999;
}

.Raw,
.Important,
.Info,
.CodeBlock,
.Example {
  margin-top: 0;
  margin-left: auto;
  margin-right: auto;
  position: relative;
  text-decoration: inherit;
}

.Body .Raw {
  margin-bottom: 0.6rem;
  font-size: 17px;
  line-height: 1.52947;
  font-weight: 400;
  letter-spacing: -0.021em;
  font-family: "SF Pro Text", "SF Pro Icons", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  color: #333333;
  font-style: normal;
}

.Important,
.Info,
.CodeBlock,
.Example {
  font-size: 17px;
  padding: 15px 10px 15px 20px;
  border: 0;
  border-radius: 6px;
  margin: 0.7em 0;
}

.Important {
  background-color: #FBECC2;
}

.Info {
  background-color: #D6E1CA;
}

.Example {
  background-color: #fafafa;
}

.CodeBlock {
  background-color: #fefefe;
  margin: 10px 20px;
  display: none;
}
.CodeBlock code {
  font-family: monospace;
}

.Def {
  margin: 40px auto auto;
  padding: 0 15px;
  text-decoration: inherit;
}
.Def .Synopsis,
.Def .Body,
.Def .Tags,
.Def .ASTData {
  padding-left: 0;
  text-decoration: inherit;
}
.Def .Synopsis {
  padding: 0;
  margin-bottom: 15px;
  font-size: 17px;
  font-weight: 400;
  color: #333333;
  font-style: normal;
}
.Def .constr {
  padding: 25px 0;
  margin: 0;
}
.Def .DefDoc .Body {
  display: none;
}
.Def .DefDoc .documentation {
  display: inline-flex;
  width: 100%;
  margin-bottom: 10px;
}
.Def .DefDoc .documentation .ASTHead {
  width: 30% !important;
  margin: 10px 0;
}
.Def .DefDoc .documentation .ASTHead .DefTitle,
.Def .DefDoc .documentation .ASTHead .Infix {
  padding: 0;
  font-size: 17px;
  font-weight: 400;
  font-style: normal;
  text-decoration: inherit;
}
.Def .DefDoc .documentation .ASTData {
  width: 70% !important;
}
.Def .DefDoc .documentation .Doc {
  text-decoration: inherit;
}
.Def .DefDoc .documentation .Doc .Synopsis {
  text-decoration: inherit;
  margin: 10px 0;
}
.Def .DefDoc .documentation .Tags {
  margin: 2px 0 0 auto;
  padding: 0;
}
.Def .DefNoDoc {
  padding-bottom: 10px;
}

.DefTitle {
  display: inline-flex;
  font-size: x-large;
  font-weight: 400;
  margin-bottom: 20px;
}

.DefArgs {
  margin-left: 5px;
  font-weight: 400;
  color: #0070c9;
}

.Synopsis,
.Body {
  margin: 0 auto;
  padding: 5px;
  text-align: left;
}

.Synopsis {
  margin-top: 35px;
  font-size: 20px;
}

.Documentation .ASTData,
.Documentation .ASTHead {
  text-align: left;
  line-height: 1.05;
  border-radius: 6px;
}
.Documentation .ASTData {
  width: 100%;
  background-color: #fafafa;
}
.Documentation .ASTHead {
  margin: 20px auto 5px;
  background-color: #ffffff;
}
.Documentation .ASTHead .DefTitle {
  font-size: 42px;
  margin: 0;
}
.Documentation .ASTData .ASTHead {
  background-color: #fafafa;
}
.Documentation .ASTData .ASTHead .DefTitle {
  font-size: x-large;
}
.Documentation .Documented {
  margin: 0;
  width: 100%;
  background-color: #ffffff;
}
.Documentation .DefNoBody {
  text-decoration: inherit;
}

@media (max-width: 500px) {
.Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 380px;
  }

.Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 400px;
  }

  .Def {
    padding: 5px;
  }
}
@media (min-width: 500px) {
.Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 440px;
  }

.Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 470px;
  }
}
@media (min-width: 600px) {
.Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 490px;
  }

.Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 520px;
  }
}
@media (min-width: 900px) {
.Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 680px;
  }

.Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 710px;
  }
}
@media (min-width: 1300px) {
.Synopsis,
.Body,
.Tags,
.Documentation .ASTData .Def {
    max-width: 790px;
  }

.Documentation .ASTHead,
.DefNoBody,
.DefBody {
    max-width: 820px;
  }
}

</style>
"#;
    css.to_string()
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
