//! Definition of the node input port component.


use crate::prelude::*;

use enso_frp as frp;
use enso_frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::scene::Scene;
use ensogl::display::shape::*;
use ensogl::display::traits::*;
use ensogl::display;
use ensogl::gui::cursor;
use ensogl_text as text;
use ensogl_text::buffer::data::unit::traits::*;
use ensogl_theme as theme;
use text::Text;

use crate::Type;
use crate::component::type_coloring;
use crate::node::input::port;
use crate::node;

pub use port::depth_sort_hack;



// =================
// === Constants ===
// =================

/// Width of a single glyph
pub const GLYPH_WIDTH : f32 = 7.224_609_4; // FIXME hardcoded literal

/// Enable visual port debug mode and additional port creation logging.
pub const DEBUG : bool = false;

/// Visual port offset for debugging purposes. Applied hierarchically. Applied only when `DEBUG` is
/// set to `true`.
pub const DEBUG_PORT_OFFSET : f32 = 5.0;

/// Skip creating ports on all operations. For example, in expression `foo bar`, `foo` is considered
/// an operation.
const SKIP_OPERATIONS            : bool = true;
const PORT_PADDING_X             : f32  = 4.0;
// const SHOW_PORTS_ONLY_ON_CONNECT : bool = true; // TODO



// ===============
// === SpanTree ==
// ===============

pub use span_tree::Crumb;
pub use span_tree::Crumbs;

/// Specialized `SpanTree` for the input ports model.
pub type SpanTree = span_tree::SpanTree<port::Model>;

/// Mutable reference to port inside of a `SpanTree`.
pub type PortRefMut<'a> = span_tree::node::RefMut<'a,port::Model>;




// ==================
// === Expression ===
// ==================

/// Specialized version of `node::Expression`, containing input port information.
#[derive(Clone,Default)]
pub struct Expression {
    /// Visual code representation. It can contain names of missing arguments, and thus can differ
    /// from `code`.
    pub viz_code : String,
    pub code     : String,
    pub input    : SpanTree,
}

impl Deref for Expression {
    type Target = SpanTree;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl DerefMut for Expression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl Debug for Expression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Expression({})",self.code)
    }
}

// === Conversions ===

/// Helper struct used for `Expression` conversions.
#[derive(Debug,Default)]
struct ExprConversion {
    prev_tok_local_index  : usize,
    last_parent_tok_index : usize,
}

impl ExprConversion {
    fn new(last_parent_tok_index:usize) -> Self {
        let prev_tok_local_index = default();
        Self {prev_tok_local_index,last_parent_tok_index}
    }
}

/// Traverses the `SpanTree` and constructs `viz_code` based on `code` and the `SpanTree` structure.
/// It also computes `port::Model` values in the `viz_code` representation.
impl From<node::Expression> for Expression {
    fn from(t:node::Expression) -> Self {
        // The number of letters the non-connected positional arguments occupied so far.
        let mut shift    = 0;
        let mut input    = t.input_span_tree.map(|_|port::Model::default());
        let mut viz_code = String::new();
        let code         = t.code;
        input.root_ref_mut().dfs(ExprConversion::default(),|node,info| {
            let is_expected_arg       = node.is_expected_argument();
            let span                  = node.span();
            let mut size              = span.size.value;
            let mut index             = span.index.value;
            let offet_from_prev_tok   = node.offset.value - info.prev_tok_local_index;
            info.prev_tok_local_index = node.offset.value + size;
            viz_code += &" ".repeat(offet_from_prev_tok);
            if node.children.is_empty() {
                viz_code += &code[index..index+size];
            }
            index += shift;
            if is_expected_arg {
                if let Some(name) = node.name() {
                    size      = name.len();
                    index    += 1;
                    shift    += 1 + size;
                    viz_code += " ";
                    viz_code += name;
                }
            }
            let port = node.payload_mut();
            port.local_index = index - info.last_parent_tok_index;
            port.index       = index;
            port.length      = size;
            ExprConversion::new(index)
        });
        Self {code,viz_code,input}
    }
}



// =============
// === Model ===
// =============

ensogl::define_endpoints! {
    Input {
        edit_mode_ready (bool),
        edit_mode       (bool),
        set_hover       (bool),
        set_dimmed      (bool),
        set_connected   (Crumbs,bool),
        /// Set the expression USAGE type. This is not the definition type, which can be set with
        /// `set_expression` instead. In case the usage type is set to None, ports still may be
        /// colored if the definition type was present.
        set_expression_usage_type (ast::Id,Option<Type>),
        ports_active (bool),
    }

    Output {
        pointer_style (cursor::Style),
        press         (Crumbs),
        // xhover        (Option<Crumbs>),
        width         (f32),
        expression    (Text),
        editing       (bool),
        ports_visible (bool),
        port_hover    (Switch<Crumbs>),
        body_hover    (bool),
        background_press (),
    }
}

/// Internal model of the port area.
#[derive(Debug)]
pub struct Model {
    logger         : Logger,
    app            : Application,
    display_object : display::object::Instance,
    ports          : display::object::Instance,
    header         : display::object::Instance,
    label          : text::Area,
    expression     : RefCell<Expression>,
    id_crumbs_map  : RefCell<HashMap<ast::Id,Crumbs>>,
    styles         : StyleWatch,
}

impl Model {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let logger         = Logger::sub(&logger,"port_manager");
        let display_object = display::object::Instance::new(&logger);
        let ports          = display::object::Instance::new(&Logger::sub(&logger,"ports"));
        let header         = display::object::Instance::new(&Logger::sub(&logger,"header"));
        let app            = app.clone_ref();
        let label          = app.new_view::<text::Area>();
        let id_crumbs_map  = default();
        let expression     = default();
        let styles         = StyleWatch::new(&app.display.scene().style_sheet);
        display_object.add_child(&label);
        display_object.add_child(&ports);
        ports.add_child(&header);
        // ports.unset_parent();
        // header.unset_parent();
        Self {logger,display_object,ports,header,label,app,expression,styles,id_crumbs_map}.init()
    }

    fn init(self) -> Self {
        // FIXME[WD]: Depth sorting of labels to in front of the mouse pointer. Temporary solution.
        // It needs to be more flexible once we have proper depth management.
        let scene = self.app.display.scene();
        self.label.remove_from_view(&scene.views.main);
        self.label.add_to_view(&scene.views.label);

        let text_color = self.styles.get_color(theme::graph_editor::node::text);
        self.label.single_line(true);
        self.label.disable_command("cursor_move_up");
        self.label.disable_command("cursor_move_down");
        self.label.set_default_color(color::Rgba::from(text_color));
        self.label.mod_position(|t| t.y += 6.0);
        self.label.set_default_text_size(text::Size(12.0));
        self.label.remove_all_cursors();
        self
    }

    /// Traverse all expressions and set their colors matching the un-hovered style.
    fn init_port_coloring(&self) {
        self.set_port_hover(&default())
    }

    /// Run the provided function on the target port if exists.
    fn with_port_mut(&self, crumbs:&Crumbs, f:impl FnOnce(PortRefMut)) {
        let mut expression = self.expression.borrow_mut();
        if let Ok(node) = expression.input.root_ref_mut().get_descendant(crumbs) { f(node) }
    }

    /// Traverse all `SpanTree` leaves of the given port and emit hover style to set their colors.
    fn set_port_hover(&self, target:&Switch<Crumbs>) {
        self.with_port_mut(&target.value,|t|t.set_hover(target.is_on()))
    }

    /// Get the code color for the provided type or default code color in case the type is None.
    fn code_color(&self, tp:&Option<Type>) -> color::Lcha {
        tp.as_ref().map(|tp| type_coloring::compute(tp,&self.styles))
            .unwrap_or_else(||self.styles.get_color(theme::graph_editor::node::text))
    }

    /// Update expression type for the particular `ast::Id`.
    fn set_expression_usage_type(&self, id:ast::Id, tp:&Option<Type>) {
        if let Some(crumbs) = self.id_crumbs_map.borrow().get(&id) {
            if let Ok(port) = self.expression.borrow().input.root_ref().get_descendant(crumbs) {
                port.set_usage_type(tp)
            }
        }
    }
}



// ============
// === Area ===
// ============

#[derive(Clone,CloneRef,Debug)]
pub struct Area {
    pub frp : Frp,
    model   : Rc<Model>,
}

impl Deref for Area {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Area {
    pub fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let model   = Rc::new(Model::new(logger,app));
        let frp     = Frp::new();
        let network = &frp.network;

        frp::extend! { network

            trace frp.port_hover;
            trace frp.output.body_hover;
            trace frp.output.ports_visible;

            frp.output.source.body_hover <+ frp.set_hover;


            // === Cursor setup ===

            eval frp.input.edit_mode ([model](enabled) {
                model.label.set_focus(enabled);
                if *enabled { model.label.set_cursor_at_mouse_position(); }
                else        { model.label.remove_all_cursors(); }
            });


            // === Show / Hide Phantom Ports ===

            edit_mode <- all_with3
                (&frp.input.edit_mode,&frp.input.edit_mode_ready,&frp.input.ports_active,
                |edit_mode,edit_mode_ready,ports_active|
                     (*edit_mode || *edit_mode_ready) && !ports_active
                );
            port_vis  <- all_with(&frp.input.ports_active,&edit_mode,|a,b|*a&&(!b));

            frp.output.source.ports_visible <+ port_vis;
            frp.output.source.editing       <+ edit_mode.sampler();


            // === Label Hover ===

            let hovered = frp.output.body_hover.clone_ref();
            label_hovered <- all_with(&edit_mode,&hovered,|a,b|*a && *b);
            eval label_hovered ((t) model.label.set_hover(t));


            // === Port Hover ===

            eval frp.port_hover ((t) model.set_port_hover(t));

            eval frp.set_connected ([model]((t,s)) {
                model.with_port_mut(t,|n|n.set_connected(s));
                model.with_port_mut(t,|n|n.set_parent_connected(s));
            });


            // === Properties ===

            frp.output.source.width      <+ model.label.width;
            frp.output.source.expression <+ model.label.content.map(|t| t.clone_ref());


            // === Expression Type ===

            eval frp.set_expression_usage_type (((id,tp)) model.set_expression_usage_type(*id,tp));
        }

        Self {model,frp}
    }

    fn scene(&self) -> &Scene {
        self.model.app.display.scene()
    }

    pub fn port_offset(&self, crumbs:&[Crumb]) -> Option<Vector2<f32>> {
        let expr = self.model.expression.borrow();
        expr.root_ref().get_descendant(crumbs).ok().map(|node| {
            let unit  = GLYPH_WIDTH;
            let width = unit * node.payload.length as f32;
            let x     = width/2.0 + unit * node.payload.index as f32;
            Vector2::new(x + node::TEXT_OFF,node::HEIGHT/2.0)
        })
    }

    pub fn port_type(&self, crumbs:&Crumbs) -> Option<Type> {
        let expression = self.model.expression.borrow();
        expression.input.root_ref().get_descendant(crumbs).ok().and_then(|t|t.final_type.value())
    }
}



// ==========================
// === Expression Setting ===
// ==========================

/// Helper struct used to keep information about the current expression layer when building visual
/// port representation. A "layer" is a visual layer in terms of span tree. For example, given
/// expression `img.blur (foo (bar baz))`, we've got several layers, like the whole expression,
/// `img.blur`, `foo (bar baz)`, or `(bar baz)`. The layer builder keeps information passed from the
/// parent layer when building the nested one.
#[derive(Clone,Debug)]
struct PortLayerBuilder {
    parent_frp : Option<port::FrpEndpoints>,
    /// Parent port display object.
    parent : display::object::Instance,
    /// Information whether the parent port was a parensed expression.
    parent_parensed : bool,
    /// The number of glyphs the expression should be shifted. For example, consider `(foo bar)`,
    /// where expression `foo bar` does not get its own port, and thus a 1 glyph shift should be
    /// applied when considering its children.
    shift : usize,
    /// The depth at which the current expression is, where root is at depth 0.
    depth : usize,
}

impl PortLayerBuilder {
    /// Constructor.
    fn new
    ( parent          : impl display::Object
    , parent_frp      : Option<port::FrpEndpoints>
    , parent_parensed : bool
    , shift           : usize
    , depth           : usize
    ) -> Self {
        let parent = parent.display_object().clone_ref();
        Self {parent,parent_frp,parent_parensed,shift,depth}
    }

    fn empty(parent:impl display::Object) -> Self {
        Self::new(parent,default(),default(),default(),default())
    }

    /// Create a nested builder with increased depth and updated `parent_frp`.
    fn nested
    ( &self
      , parent          : display::object::Instance
      , new_parent_frp  : Option<port::FrpEndpoints>
      , parent_parensed : bool
      , shift           : usize
    ) -> Self {
        let depth      = self.depth + 1;
        let parent_frp = new_parent_frp.or_else(||self.parent_frp.clone());
        Self::new(parent,parent_frp,parent_parensed,shift,depth)
    }
}

impl Area {
    fn set_label_on_new_expression(&self, expression:&Expression) {
        self.model.label.set_cursor(&default());
        self.model.label.select_all();
        self.model.label.insert(&expression.viz_code);
        self.model.label.remove_all_cursors();
        if self.frp.editing.value() {
            self.model.label.set_cursor_at_end();
        }
    }

    fn build_port_shapes_on_new_expression(&self, expression:&mut Expression) {
        let mut is_header = true;
        let builder       = PortLayerBuilder::empty(&self.model.ports);
        expression.root_ref_mut().dfs(builder,|mut node,builder| {
            let is_parensed = node.is_parensed();
            let skip_opr    = if SKIP_OPERATIONS {
                node.is_operation() && !is_header
            } else {
                let crumb = ast::Crumb::Infix(ast::crumbs::InfixCrumb::Operator);
                node.ast_crumbs.last().map(|t| t == &crumb) == Some(true)
            };

            let not_a_port
                =  node.is_positional_insertion_point()
                || node.is_chained()
                || node.is_root()
                || skip_opr
                || node.is_token()
                || builder.parent_parensed;

            if let Some(id) = node.ast_id {
                self.model.id_crumbs_map.borrow_mut().insert(id,node.crumbs.clone_ref());
            }

            if DEBUG {
                let indent = " ".repeat(4*builder.depth);
                let skipped = if not_a_port { "(skip)" } else { "" };
                println!("{}[{},{}] {} {:?} (tp: {:?})",indent,node.payload.index,
                         node.payload.length,skipped,node.kind.variant_name(),node.tp());
            }

            let new_parent = if not_a_port {
                builder.parent.clone_ref()
            } else {
                let port         = &mut node;
                let index        = port.payload.local_index + builder.shift;
                let size         = port.payload.length;
                let unit         = GLYPH_WIDTH;
                let width        = unit * size as f32;
                let width_padded = width + 2.0 * PORT_PADDING_X;
                let height       = 18.0;
                let padded_size  = Vector2(width_padded,height);
                let size         = Vector2(width,height);
                let logger       = &self.model.logger;
                let scene        = self.scene();
                let port_shape   = port.payload_mut().init_shape(logger,scene,size,node::HEIGHT);

                port_shape.mod_position(|t| t.x = unit * index as f32);
                if DEBUG { port_shape.mod_position(|t| t.y = DEBUG_PORT_OFFSET) }

                if is_header {
                    is_header = false;
                    self.model.header.add_child(&port_shape);
                } else {
                    builder.parent.add_child(&port_shape);
                }

                // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
                let styles             = StyleWatch::new(&self.model.app.display.scene().style_sheet);
                let missing_type_color = styles.get_color(theme::code::types::missing);
                let crumbs             = port.crumbs.clone_ref();
                let port_network       = &port.network;
                let frp                = &self.frp.output;

                frp::extend! { port_network
                    // === Aliases ===

                    let mouse_over_raw = port_shape.hover.events.mouse_over.clone_ref();
                    let mouse_out      = port_shape.hover.events.mouse_out.clone_ref();
                    let mouse_down_raw = port_shape.hover.events.mouse_down.clone_ref();


                    // === Body Hover ===
                    // This is a very important part of the FRP network. For performance reasons,
                    // the ports are shown and hidden only when mouse goes over or out of the node.
                    // This way we can show and hide ports on a single node, no matter how many
                    // nodes are on the stage. This also means, that when an edge is dragged over
                    // the node When ports are visible and ...
                    // the mouse hovers them, we want them
                    self.frp.output.source.body_hover <+ bool(&mouse_out,&mouse_over_raw);
                    mouse_over <- mouse_over_raw.gate(&frp.ports_visible);
                    mouse_down <- mouse_down_raw.gate(&frp.ports_visible);
                    bg_down    <- mouse_down_raw.gate_not(&frp.ports_visible);
                    self.frp.output.source.background_press <+ bg_down;


                    // === Press ===
                    eval_ mouse_down ([crumbs,frp] frp.source.press.emit(&crumbs));

                    // === Hover ===
                    hovered <- bool(&mouse_out,&mouse_over);
                    hover   <- hovered.map (f!([crumbs](t) Switch::new(crumbs.clone_ref(),*t)));
                    frp.source.port_hover <+ hover;



                    // === Pointer Style ===
                    let port_shape_hover = port_shape.hover.shape.clone_ref();
                    pointer_style_out   <- mouse_out.map(|_| default());
                    pointer_style_over  <- map2(&mouse_over,&port.final_type,move |_,tp| {
                        let color = tp.as_ref().map(|tp| type_coloring::compute(tp,&styles));
                        let color = color.unwrap_or(missing_type_color);
                        cursor::Style::new_highlight(&port_shape_hover,padded_size,Some(color))
                    });
                    pointer_style_hover <- any(pointer_style_over,pointer_style_out);
                    pointer_styles      <- all[pointer_style_hover,self.model.label.pointer_style];
                    pointer_style       <- pointer_styles.fold();
                    self.frp.output.source.pointer_style <+ pointer_style;
                }
                port_shape.display_object().clone_ref()
            };

            if let Some(parent_frp) = &builder.parent_frp {
                frp::extend! { port_network
                    node.frp.set_active           <+ parent_frp.set_active;
                    node.frp.set_hover            <+ parent_frp.set_hover;
                    node.frp.set_parent_connected <+ parent_frp.set_parent_connected;
                }
            }
            let new_parent_frp = Some(node.frp.output.clone_ref());
            let new_shift = if !not_a_port { 0 } else { builder.shift + node.payload.local_index };
            builder.nested(new_parent,new_parent_frp,is_parensed,new_shift)
        });
    }

    /// Initializes FRP network for every port. Please note that the networks are connected
    /// hierarchically (children get events from parents), so it is easier to init all networks
    /// this way, rather than delegate it to every port.
    fn init_port_frp_on_new_expression(&self, expression:&mut Expression) {
        let model          = &self.model;
        let selected_color = model.styles.get_color(theme::code::types::selected);
        let disabled_color = model.styles.get_color(theme::code::syntax::disabled);
        let expected_color = model.styles.get_color(theme::code::syntax::expected);

        let parent_tp : Option<frp::Stream<Option<Type>>> = None;
        expression.root_ref_mut().dfs(parent_tp,|node,parent_tp| {
            let frp          = &node.frp;
            let port_network = &frp.network;
            let is_token     = node.is_token();


            // === Type Computation ===

            frp::extend! { port_network
                def_tp <- source::<Option<Type>>();
            }
            let parent_tp = parent_tp.clone().unwrap_or_else(||{
                frp::extend! { port_network
                    empty_parent_tp <- source::<Option<Type>>();
                }
                empty_parent_tp.into()
            });
            frp::extend! { port_network
                final_tp <- all_with3(&parent_tp,&def_tp,&frp.set_usage_type,
                    move |parent_tp,def_tp,usage_tp| {
                        usage_tp.clone().or_else(||
                            if is_token {parent_tp.clone()} else {def_tp.clone()}
                        )
                    }
                );
                frp.source.final_type <+ final_tp;
            }


            // === Code Coloring ===

            if node.children.is_empty() {
                let is_expected_arg   = node.is_expected_argument();
                let text_color        = color::Animation::new(port_network);
                frp::extend! { port_network
                    base_color     <- final_tp.map(f!((t) model.code_color(t)));
                    is_selected    <- all_with(&frp.set_hover,&frp.set_parent_connected,|s,t|*s||*t);
                    select_color   <- all_with(&frp.set_hover,&base_color,|_,t|*t);
                    text_color_tgt <- all_with3(&base_color,&is_selected,&self.frp.set_dimmed,
                        move |base_color,is_selected,is_disabled| {
                            if      *is_selected    { selected_color }
                            else if *is_disabled    { disabled_color }
                            else if is_expected_arg { expected_color }
                            else                    { *base_color }
                        });
                    text_color.target              <+ text_color_tgt;
                    frp.output.source.select_color <+ select_color;
                    frp.output.source.text_color   <+ text_color.value;
                }

                let index  = node.payload.index;
                let length = node.payload.length;
                frp::extend! { port_network
                    eval frp.output.text_color ([model](color) {
                        let start_bytes = (index as i32).bytes();
                        let end_bytes   = ((index + length) as i32).bytes();
                        let range       = ensogl_text::buffer::Range::from(start_bytes..end_bytes);
                        model.label.set_color_bytes(range,color::Rgba::from(color));
                    });
                }
            }


            // === Highlight Coloring ===

            if let Some(port_shape) = &node.payload.shape {
                frp::extend! { port_network
                    viz_color <- all_with(&frp.select_color,&frp.set_connected,|color,is_connected|
                        if *is_connected {*color} else { color::Lcha::transparent() } );
                    eval viz_color ((color) port_shape.viz.shape.color.set(color::Rgba::from(color).into()));
                }
            }

            // Initialization.
            def_tp.emit(node.tp().cloned().map(|t|t.into()));
            Some(frp.final_type.clone_ref().into())
        });
    }

    pub(crate) fn set_expression(&self, expression:impl Into<node::Expression>) {
        let model          = &self.model;
        let expression     = expression.into();
        let mut expression = Expression::from(expression);
        if DEBUG { println!("\n\n=====================\nSET EXPR: {}", expression.code) }

        self.set_label_on_new_expression(&expression);
        self.build_port_shapes_on_new_expression(&mut expression);
        self.init_port_frp_on_new_expression(&mut expression);

        *model.expression.borrow_mut() = expression;
        model.init_port_coloring();
    }
}

impl display::Object for Area {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
