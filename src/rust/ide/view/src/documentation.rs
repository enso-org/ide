//! Documentation view visualization generating and presenting Enso Documentation under
//! the documented node.

use crate::prelude::*;

use crate::graph_editor::component::visualization;

pub use visualization::container::overlay;

use ast::prelude::FallibleResult;
use enso_frp as frp;
use ensogl::data::color;
use ensogl::display;
use ensogl::display::DomSymbol;
use ensogl::display::scene::Scene;
use ensogl::display::shape::primitive::StyleWatch;
use ensogl::system::web;
use ensogl::system::web::clipboard;
use ensogl::system::web::StyleSetter;
use ensogl::system::web::AttributeSetter;
use ensogl::gui::component;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use web_sys::MouseEvent;



// =================
// === Constants ===
// =================

/// Width of Documentation panel.
pub const VIEW_WIDTH  : f32 = 300.0;
/// Height of Documentation panel.
pub const VIEW_HEIGHT : f32 = 300.0;

/// Content in the documentation view when there is no data available.
const PLACEHOLDER_STR   : &str = "<h3>Documentation Viewer</h3><p>No documentation available</p>";
const CORNER_RADIUS     : f32  = crate::graph_editor::component::node::CORNER_RADIUS;
const PADDING           : f32  = 5.0;
const CODE_BLOCK_CLASS  : &str = "CodeBlock";
const COPY_BUTTON_CLASS : &str = "copyCodeBtn";

/// Get documentation view stylesheet from a CSS file.
///
/// TODO [MM] : This file is generated currently from SASS file, and generated code should never
///             be included in a codebase, so it will be moved to rust-based generator to achieve
///             compatibility with IDE's theme manager.
///             Expect them to land with https://github.com/enso-org/ide/issues/709
fn documentation_style() -> String {
    format!("<style>{}</style>", include_str!("documentation/style.css"))
}



// =============
// === Model ===
// =============

/// The input type for documentation parser. See documentation of `View` for details.
#[derive(Clone,Copy,Debug)]
enum InputFormat {
    AST,Docstring
}

type CodeCopyClosure = Closure<dyn FnMut(MouseEvent)>;

/// Model of Native visualization that generates documentation for given Enso code and embeds
/// it in a HTML container.
#[derive(Clone,CloneRef,Debug)]
#[allow(missing_docs)]
pub struct Model {
    logger : Logger,
    dom    : DomSymbol,
    size   : Rc<Cell<Vector2>>,
    /// The purpose of this overlay is stop propagating mouse events under the documentation panel
    /// to EnsoGL shapes, and pass them to the DOM instead.
    overlay            : component::ShapeView<overlay::Shape>,
    display_object     : display::object::Instance,
    code_copy_closures : Rc<CloneCell<Vec<CodeCopyClosure>>>
}

impl Model {
    /// Constructor.
    fn new(scene:&Scene) -> Self {
        let logger         = Logger::new("DocumentationView");
        let display_object = display::object::Instance::new(&logger);
        let div            = web::create_div();
        let dom            = DomSymbol::new(&div);
        let size           = Rc::new(Cell::new(Vector2(VIEW_WIDTH,VIEW_HEIGHT)));
        let overlay        = component::ShapeView::<overlay::Shape>::new(&logger,scene);

        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        let styles   = StyleWatch::new(&scene.style_sheet);
        let bg_color = styles.get_color(ensogl_theme::graph_editor::visualization::background);
        let bg_color = color::Rgba::from(bg_color);
        let bg_hex   = format!("rgba({},{},{},{})",
            bg_color.red*255.0,bg_color.green*255.0,bg_color.blue*255.0,bg_color.alpha);

        let shadow_alpha_path = ensogl_theme::graph_editor::visualization::shadow::html::alpha;
        let shadow_alpha_size = ensogl_theme::graph_editor::visualization::shadow::html::size;
        let shadow_alpha = styles.get_number_or(shadow_alpha_path,0.16);
        let shadow_size  = styles.get_number_or(shadow_alpha_size,16.0);
        let shadow       = format!("0 0 {}px rgba(0, 0, 0, {})",shadow_size,shadow_alpha);

        dom.dom().set_attribute_or_warn("class"       ,"scrollable"                 ,&logger);
        dom.dom().set_style_or_warn("white-space"     ,"normal"                     ,&logger);
        dom.dom().set_style_or_warn("overflow-y"      ,"auto"                       ,&logger);
        dom.dom().set_style_or_warn("overflow-x"      ,"auto"                       ,&logger);
        dom.dom().set_style_or_warn("background-color",bg_hex                       ,&logger);
        dom.dom().set_style_or_warn("padding"         ,format!("{}px",PADDING)      ,&logger);
        dom.dom().set_style_or_warn("pointer-events"  ,"auto"                       ,&logger);
        dom.dom().set_style_or_warn("border-radius"   ,format!("{}px",CORNER_RADIUS),&logger);
        dom.dom().set_style_or_warn("box-shadow"      ,shadow                       ,&logger);

        overlay.shape.roundness.set(1.0);
        overlay.shape.radius.set(CORNER_RADIUS);
        display_object.add_child(&dom);
        display_object.add_child(&overlay);
        scene.dom.layers.front.manage(&dom);

        let code_copy_closures = default();
        Model {logger,dom,size,overlay,display_object,code_copy_closures}.init()
    }

    fn init(self) -> Self {
        self.reload_style();
        self
    }

    /// Set size of the documentation view.
    fn set_size(&self, size:Vector2) {
        self.size.set(size);
        self.overlay.shape.sprite.size.set(size);
        self.reload_style();
    }

    /// Generate HTML documentation from documented Enso code.
    fn gen_html_from(string:&str, input_type: InputFormat) -> FallibleResult<String> {
        if string.is_empty() {
            Ok(PLACEHOLDER_STR.into())
        } else {
            let parser    = parser::DocParser::new()?;
            let processed = string.to_string();
            let output = match input_type {
                InputFormat::AST       => parser.generate_html_docs(processed),
                InputFormat::Docstring => parser.generate_html_doc_pure(processed),
            };
            let output = output?;
            Ok( if output.is_empty() { PLACEHOLDER_STR.into() } else { output } )
        }
    }

    /// Create a container for generated content and embed it with stylesheet.
    fn push_to_dom(&self, content:String) {
        let data_str = format!(r#"<div class="docVis">{}{}</div>"#,documentation_style(),content);
        self.dom.dom().set_inner_html(&data_str);
    }

    /// Append listeners to copy buttons in doc to enable copying examples.
    /// It is possible to do it with implemented method, because get_elements_by_class_name
    /// returns top-to-bottom sorted list of elements, as found in:
    /// https://stackoverflow.com/questions/35525811/order-of-elements-in-document-getelementsbyclassname-array
    fn attach_listeners_to_copy_buttons(&self) {
        let code_blocks  = self.dom.dom().get_elements_by_class_name(CODE_BLOCK_CLASS);
        let copy_buttons = self.dom.dom().get_elements_by_class_name(COPY_BUTTON_CLASS);
        let closures     = (0..copy_buttons.length()).map(|i| -> Result<CodeCopyClosure,u32> {
            let create_closures = || -> Option<CodeCopyClosure> {
                let copy_button = copy_buttons.get_with_index(i)?.dyn_into::<HtmlElement>().ok()?;
                let code_block  = code_blocks.get_with_index(i)?.dyn_into::<HtmlElement>().ok()?;
                let closure     = Box::new(move |_event: MouseEvent| {
                    let inner_code = code_block.inner_text();
                    clipboard::write_text(inner_code);
                });
                let closure: Closure<dyn FnMut(MouseEvent)> = Closure::wrap(closure);
                let callback = closure.as_ref().unchecked_ref();
                match copy_button.add_event_listener_with_callback("click",callback) {
                    Ok(_)  => Some(closure),
                    Err(e) => {
                        error!(&self.logger,"Unable to add event listener to copy button: {e:?}");
                        None
                    },
                }
            };
            create_closures().ok_or(i)
        });
        let (closures,errors) : (Vec<_>,Vec<_>) = closures.partition(Result::is_ok);
        let ok_closures = closures.into_iter().filter_map(|t| t.ok()).collect_vec();
        let err_indices = errors.into_iter().filter_map(|t| t.err()).collect_vec();
        if !err_indices.is_empty() {
            error!(&self.logger, "Failed to attach listeners to copy buttons with indices: {err_indices:?}.")
        }
        self.code_copy_closures.set(ok_closures)
    }

    /// Receive data, process and present it in the documentation view.
    fn receive_data(&self, data:&visualization::Data) -> Result<(),visualization::DataError> {
        let string = match data {
            visualization::Data::Json {content} => match serde_json::to_string_pretty(&**content) {
                Ok(string) => string,
                Err(err)   => {
                    error!(self.logger, "Error during documentation vis-data serialization: \
                        {err:?}");
                    return Err(visualization::DataError::InternalComputationError);
                }
            }
            _ => return Err(visualization::DataError::InvalidDataType),
        };
        self.display_doc(&string, InputFormat::Docstring);
        Ok(())
    }

    fn display_doc(&self, content:&str, content_type: InputFormat) {
        let html = match Model::gen_html_from(content,content_type) {
            Ok(html) => html,
            Err(err) => {
                error!(self.logger, "Documentation parsing error: {err:?}");
                PLACEHOLDER_STR.into()
            }
        };

        self.push_to_dom(html);
        self.attach_listeners_to_copy_buttons();
    }

    /// Load an HTML file into the documentation view when user is waiting for data to be received.
    /// TODO [MM] : This should be replaced with a EnsoGL spinner in the next PR.
    fn load_waiting_screen(&self) {
        let spinner = include_str!("documentation/spinner.html");
        self.push_to_dom(String::from(spinner))
    }

    fn reload_style(&self) {
        let size        = self.size.get();
        let real_width  = (size.x - 2.0 * PADDING).max(0.0);
        let real_height = (size.y - 2.0 * PADDING).max(0.0);
        let padding     = (size.x.min(size.y) / 2.0).min(PADDING);
        self.dom.set_size(Vector2(real_width,real_height));
        self.dom.dom().set_style_or_warn("padding",format!("{}px",padding),&self.logger);
    }
}



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
        /// Display documentation of the entity represented by given code.
        display_documentation (String),
        /// Display documentation represented by docstring.
        display_docstring (String),
    }
    Output {
        /// Indicates whether the documentation panel has been selected through clicking into
        /// it, or deselected by clicking somewhere else.
        is_selected(bool),
    }
}


// ============
// === View ===
// ============

/// View of the visualization that renders the given documentation as a HTML page.
///
/// The documentation can be provided in two formats: it can be code of the entity (type, method,
/// function etc) with doc comments, or the docstring only - in the latter case
/// however we're unable to summarize methods and atoms of types.
///
/// The default format is the docstring.
#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
#[allow(missing_docs)]
pub struct View {
    #[shrinkwrap(main_field)]
    pub model             : Model,
    pub visualization_frp : visualization::instance::Frp,
    pub frp               : Frp,
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
        let frp               = Frp::new();
        let visualization_frp = visualization::instance::Frp::new(&frp.network);
        let model             = Model::new(scene);
        model.load_waiting_screen();
        Self {model,visualization_frp,frp} . init(scene)
    }

    fn init(self, scene:&Scene) -> Self {
        let network       = &self.frp.network;
        let model         = &self.model;
        let visualization = &self.visualization_frp;
        let frp           = &self.frp;
        frp::extend! { network

            // === Displaying documentation ===

            eval frp.display_documentation ((cont) model.display_doc(cont,InputFormat::AST      ));
            eval frp.display_docstring     ((cont) model.display_doc(cont,InputFormat::Docstring));
            eval visualization.send_data([visualization,model](data) {
                if let Err(error) = model.receive_data(data) {
                    visualization.data_receive_error.emit(error)
                }
            });


            // === Size and position ===

            eval visualization.set_size  ((size) model.set_size(*size));


            // === Activation ===

            mouse_down_target <- scene.mouse.frp.down.map(f_!(scene.mouse.target.get()));
            selected <- mouse_down_target.map(f!([model,visualization] (target){
                if !model.overlay.shape.is_this_target(*target) {
                    visualization.deactivate.emit(());
                    false
                } else {
                    visualization.activate.emit(());
                    true
                }
            }));
            is_selected_changed <= selected.map2(&frp.output.is_selected, |&new,&old| {
                (new != old).as_some(new)
            });
            frp.source.is_selected <+ is_selected_changed;
        }
        visualization.pass_events_to_dom_if_active(scene,network);
        self
    }
}

impl From<View> for visualization::Instance {
    fn from(t: View) -> Self { Self::new(&t,&t.visualization_frp,&t.frp.network) }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}
