//! Definition of the Port component.


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

/// Enable visual port debug mode.
pub const DEBUG : bool = false;

/// Skip creating ports on all operations. For example, in expression `foo bar`, `foo` is considered
/// an operation.
const SKIP_OPERATIONS            : bool = true;
const PORT_PADDING_X             : f32  = 4.0;
// const SHOW_PORTS_ONLY_ON_CONNECT : bool = true; // TODO



// ===============
// === SpanTree ==
// ===============

/// Specialized `SpanTree` for the input ports model.
pub type SpanTree = span_tree::SpanTree<port::Model>;



// ==================
// === Expression ===
// ==================

/// Specialized version of `node::Expression`.
#[derive(Clone,Default)]
pub struct Expression {
    pub code     : String,
    pub viz_code : String,
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
                    size   = name.len();
                    index += 1;
                    shift += 1 + size;
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



// ========================
// === PortLayerBuilder ===
// ========================

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
    fn new
    (parent:display::object::Instance, parent_frp:Option<port::FrpEndpoints>, parent_parensed:bool, shift:usize, depth:usize) -> Self {
        Self {parent,parent_frp,parent_parensed,shift,depth}
    }

    fn nested(&self, parent:display::object::Instance, parent_frp:Option<port::FrpEndpoints>, parent_parensed:bool, shift:usize) -> Self {
        let depth      = self.depth + 1;
        let parent_frp = parent_frp.or_else(||self.parent_frp.clone());
        Self::new(parent,parent_frp,parent_parensed,shift,depth)
    }

    fn empty(parent:display::object::Instance) -> Self {
        Self::new(parent,default(),default(),default(),default())
    }
}



// =============
// === Model ===
// =============

ensogl::define_endpoints! {
    Input {
        edit_mode_ready       (bool),
        edit_mode             (bool),
        hover                 (),
        unhover               (),
        set_dimmed            (bool),
        set_expression_type   (ast::Id,Option<Type>),
        set_connected (span_tree::Crumbs,bool),
    }

    Output {
        pointer_style (cursor::Style),
        press         (span_tree::Crumbs),
        hover         (Option<span_tree::Crumbs>),
        width         (f32),
        expression    (Text),
        editing       (bool),
        port_over     (span_tree::Crumbs),
        port_out      (span_tree::Crumbs),
    }
}

/// Internal model of the port manager.
#[derive(Debug)]
pub struct Model {
    logger         : Logger,
    display_object : display::object::Instance,
    ports_group    : display::object::Instance,
    header         : display::object::Instance,
    app            : Application,
    expression     : RefCell<Expression>,
    label          : text::Area,
    width          : Cell<f32>,
    styles         : StyleWatch,
    /// Used for applying type information update, which is in a form of `(ast::Id,Type)`.
    id_crumbs_map  : RefCell<HashMap<ast::Id,span_tree::Crumbs>>,
}

impl Model {
    /// Constructor.
    pub fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let logger         = Logger::sub(&logger,"port_manager");
        let display_object = display::object::Instance::new(&logger);
        let ports_group    = display::object::Instance::new(&Logger::sub(&logger,"ports"));
        let header         = display::object::Instance::new(&Logger::sub(&logger,"header"));
        let app            = app.clone_ref();
        let label          = app.new_view::<text::Area>();
        let id_crumbs_map  = default();
        let expression     = default();
        let width          = default();
        let styles         = StyleWatch::new(&app.display.scene().style_sheet);
        display_object.add_child(&label);
        display_object.add_child(&ports_group);
        ports_group.add_child(&header);
        header.unset_parent();
        Self {logger,display_object,ports_group,header,label,width,app,expression,styles
             ,id_crumbs_map} . init()
    }

    fn init(self) -> Self {
        self.label.single_line(true);
        self.label.disable_command("cursor_move_up");
        self.label.disable_command("cursor_move_down");

        let text_color = self.styles.get_color(theme::graph_editor::node::text::color);
        self.label.set_default_color(color::Rgba::from(text_color));

        // FIXME[WD]: Depth sorting of labels to in front of the mouse pointer. Temporary solution.
        // It needs to be more flexible once we have proper depth management.
        let scene = self.app.display.scene();
        self.label.remove_from_view(&scene.views.main);
        self.label.add_to_view(&scene.views.label);
        self.label.mod_position(|t| t.y += 6.0);
        self.label.set_default_text_size(text::Size(12.0));
        self.label.remove_all_cursors();
        self
    }

    fn with_node(&self, crumbs:&span_tree::Crumbs, f:impl FnOnce(span_tree::node::RefMut<port::Model>)) {
        let mut expression = self.expression.borrow_mut();
        if let Ok(node) = expression.input.root_ref_mut().get_descendant(crumbs) { f(node) }
    }

    /// Traverse all leaves and emit hover style to animate their colors.
    fn on_port_hover(&self, is_hovered:bool, crumbs:&span_tree::Crumbs) {
        self.with_node(crumbs,|t|t.set_hover(is_hovered))
    }

    /// Traverse all expressions and set their colors matching the un-hovered style.
    fn init_port_coloring(&self) {
        self.on_port_hover(false,&default())
    }

    fn get_base_color(&self, tp:Option<&String>) -> color::Lcha {
        tp.map(|tp| type_coloring::color_for_type(tp.clone().into(),&self.styles))
            .unwrap_or_else(||self.styles.get_color(theme::graph_editor::node::text::color))
    }
}

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

            // === Cursor setup ===

            eval frp.input.edit_mode ([model](enabled) {
                model.label.set_focus(enabled);
                if *enabled { model.label.set_cursor_at_mouse_position(); }
                else        { model.label.remove_all_cursors(); }
            });


            // === Show / Hide Phantom Ports ===

            edit_mode <- all_with(&frp.input.edit_mode,&frp.input.edit_mode_ready,|a,b|*a||*b);
            eval edit_mode ([model](edit_mode) {
                if *edit_mode {
                    model.display_object.remove_child(&model.ports_group)
                } else {
                    model.display_object.add_child(&model.ports_group);
                }
            });

            frp.output.source.hover   <+ edit_mode.gate_not(&edit_mode).constant(None);
            frp.output.source.editing <+ edit_mode.sampler();


            // === Label Hover ===

            hovered <- bool(&frp.input.unhover,&frp.input.hover);
            // The below pattern is a very special one. When mouse is over phantom port and the edit
            // mode button is pressed, the `edit_mode` event is emitted. It then emits trough the
            // network and hides phantom ports. In the next frame, the hovering is changed, so this
            // value needs to be updated.
            edit_mode_hovered <- all_with(&edit_mode,&hovered,|a,_|*a).gate(&hovered);
            label_hovered     <- all_with(&hovered,&edit_mode_hovered,|a,b|*a&&*b);
            eval label_hovered ((t) model.label.set_hover(t));


            // === Port Hover ===

            eval frp.port_over ((crumbs) model.with_node(crumbs,|n|n.set_hover(true)));
            eval frp.port_out  ((crumbs) model.with_node(crumbs,|n|n.set_hover(false)));

            eval frp.set_connected (((t,s)) model.with_node(t,|n|n.set_connected(s)));


            // === Properties ===

            frp.output.source.width      <+ model.label.width;
            frp.output.source.expression <+ model.label.content.map(|t| t.clone_ref());


            // === Color Handling ===

            // eval text_color.value ([model](color) {
            //     // FIXME[WD]: Disabled to improve colors. To be fixed before merge.
            //     // // TODO: Make const once all the components can be made const.
            //     // let all_bytes = buffer::Range::from(Bytes::from(0)..Bytes(i32::max_value()));
            //     // model.label.set_color_bytes(all_bytes,color::Rgba::from(color));
            // });

            // eval frp.set_dimmed ([text_color,style](should_dim) {
            //     let text_color_path = theme::graph_editor::node::text::color;
            //     if *should_dim {
            //         text_color.set_target(style.get_color_dim(text_color_path));
            //     } else {
            //         text_color.set_target(style.get_color(text_color_path));
            //     }
            // });

            trace frp.set_connected;
        }

        Self {model,frp}
    }

    fn scene(&self) -> &Scene {
        self.model.app.display.scene()
    }

    pub(crate) fn set_expression(&self, expression:impl Into<node::Expression>) {
        let model      = &self.model;
        let expression = expression.into();
        println!("\n\n=====================\nSET EXPR: {}",expression.code);
        println!("{:?}",expression.input_span_tree);
        let mut expression = Expression::from(expression);

        let glyph_width = 7.224_609_4; // FIXME hardcoded literal
        let width       = expression.code.len() as f32 * glyph_width;
        model.width.set(width);



        let root = model.ports_group.clone_ref();
        let mut is_header = true;

        let builder = PortLayerBuilder::empty(root);
        expression.root_ref_mut().dfs(builder,|node,builder| {
            let is_parensed = node.is_parensed();
            let skip_opr    = if SKIP_OPERATIONS {
                node.is_operation() && !is_header
            } else {
                let crumb = ast::Crumb::Infix(ast::crumbs::InfixCrumb::Operator);
                node.ast_crumbs.last().map(|t| t == &crumb) == Some(true)
            };

            let skip = node.is_positional_insertion_point()
                || node.is_chained()
                || node.is_root()
                || skip_opr
                || node.is_token()
                || builder.parent_parensed;

            if let Some(id) = node.ast_id {
                self.model.id_crumbs_map.borrow_mut().insert(id,node.crumbs.clone());
            }

            let indent = "   ".repeat(builder.depth);
            let skipped = if skip { "(skipped)" } else { "" };
            println!("{}[{},{}] {} {:?} (tp: {:?}) (parent_frp: {})",indent,node.payload.index,node.payload.length,skipped,node.kind.variant_name(),node.tp(),builder.parent_frp.is_some());
            let new_parent = if !skip {
                let index        = node.payload.local_index + builder.shift;
                let size         = node.payload.length;
                let unit         = 7.224_609_4;
                let width        = unit * size as f32;
                let width_padded = width + 2.0 * PORT_PADDING_X;
                let node_height  = 28.0;
                let height       = 18.0;
                let size         = Vector2::new(width_padded,height);

                let logger     = &model.logger;
                let scene      = self.scene();
                let port_shape = node.payload_mut().init_shapes(logger,scene).clone_ref();

                port_shape.hover.shape.sprite.size.set(Vector2::new(width_padded,node_height));
                port_shape.viz.shape.sprite.size.set(Vector2::new(width_padded,node_height));
                port_shape.hover.shape.mod_position(|t| t.x = width/2.0);
                port_shape.viz.shape.mod_position(|t| t.x = width/2.0);
                port_shape.root.mod_position(|t| t.x = unit * index as f32);
                if DEBUG {
                    port_shape.root.mod_position(|t| t.y = 5.0);
                }
                if is_header {
                    println!("ADDING HEADER");
                    is_header = false;
                    model.header.add_child(&port_shape.root);
                } else {
                    builder.parent.add_child(&port_shape.root);
                }

                // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
                let styles             = StyleWatch::new(&model.app.display.scene().style_sheet);
                let missing_type_color = styles.get_color(theme::syntax::missing::color);

                let crumbs = node.crumbs.clone_ref();
                let color = node.tp().map(
                    |tp| type_coloring::color_for_type(tp.clone().into(),&styles)
                ).unwrap_or(missing_type_color);

                let highlight = cursor::Style::new_highlight(&port_shape.hover.shape,size,Some(color));

                let leaf     = &node.frp;
                let port_network  = &leaf.network;

                let dbg = format!("OVER [{},{}] {:?} {:?}",node.payload.index,node.payload.length,node.kind.variant_name(),node.tp());

                frp::extend! { port_network
                    let mouse_over = port_shape.hover.events.mouse_over.clone_ref();
                    let mouse_out  = port_shape.hover.events.mouse_out.clone_ref();


                    // === Mouse Style ===

                    pointer_style_over  <- mouse_over.map(move |_| {
                        println!("{}",dbg);
                        highlight.clone()
                    });
                    pointer_style_out   <- mouse_out.map(|_| default());
                    pointer_style_hover <- any(pointer_style_over,pointer_style_out);
                    pointer_style       <- all
                        [ pointer_style_hover
                        , self.model.label.pointer_style
                        ].fold();
                    self.frp.output.source.pointer_style <+ pointer_style;


                    // === Port Hover ===

                    let crumbs_over  = crumbs.clone();
                    let hover_source = &self.frp.output.source.hover;
                    eval_ mouse_over (hover_source.emit(&Some(crumbs_over.clone())));
                    eval_ mouse_out  (hover_source.emit(&None));


                    // === Port Press ===

                    let crumbs_down  = crumbs.clone();
                    let press_source = &self.frp.output.source.press;
                    eval_ port_shape.hover.events.mouse_down (press_source.emit(&crumbs_down));
                }

                frp::extend! { port_network
                    self.source.port_over <+ port_shape.hover.events.mouse_over.map (f_!(crumbs.clone_ref()));
                    self.source.port_out  <+ port_shape.hover.events.mouse_out.map  (f_!(crumbs.clone_ref()));
                }
                port_shape.root.display_object().clone_ref()
            } else {
                builder.parent.clone_ref()
            };

            if let Some(parent_frp) = &builder.parent_frp {
                frp::extend! { port_network
                    node.frp.set_hover            <+ parent_frp.set_hover;
                    node.frp.set_connected        <+ parent_frp.set_connected;
                    node.frp.set_parent_connected <+ parent_frp.set_parent_connected;
                }
            }
            let new_parent_frp = Some(node.frp.output.clone_ref());
            let new_shift = if !skip { 0 } else { builder.shift + node.payload.local_index };
            builder.nested(new_parent,new_parent_frp,is_parensed,new_shift)
        });

        model.label.set_cursor(&default());
        model.label.select_all();
        model.label.insert(&expression.viz_code);
        model.label.remove_all_cursors();
        if self.frp.editing.value() {
            model.label.set_cursor_at_end();
        }


        let xx : Option<String> = None;
        expression.root_ref_mut().dfs(xx,|node,parent_tp| {
            if node.children.is_empty() {
                let leaf         = &node.frp;
                let port_network = &leaf.network;
                let is_expected_arg = node.is_expected_argument();

                let tp         = if node.is_token() { parent_tp.as_ref() } else { node.tp() };
                let base_color = model.get_base_color(tp);


                frp::extend! { port_network
                    ccc <- leaf.input.set_hover.map(f!([model](is_hovered)
                        let _model = &model; // FIXME
                        if *is_hovered { color::Lcha::from(color::Rgba::new(1.0,1.0,1.0,0.7)) }
                        else if is_expected_arg { color::Lcha::from(color::Rgba::new(1.0,1.0,1.0,0.4)) }
                        else { base_color }
                    ));
                    node.color.target <+ ccc;
                    leaf.output.source.color <+ node.color.value;
                }

                let index  = node.payload.index;
                let length = node.payload.length;
                frp::extend! { port_network
                    eval leaf.output.color ([model](color) {
                        let start_bytes = (index as i32).bytes();
                        let end_bytes   = ((index + length) as i32).bytes();
                        let range       = ensogl_text::buffer::Range::from(start_bytes..end_bytes);
                        model.label.set_color_bytes(range,color::Rgba::from(color));
                    });
                }
            }
            node.tp().cloned()
        });

        *model.expression.borrow_mut() = expression;
        model.init_port_coloring();
    }

    pub fn get_port_offset(&self, crumbs:&[span_tree::Crumb]) -> Option<Vector2<f32>> {
        let expr = self.model.expression.borrow();
        expr.root_ref().get_descendant(crumbs).ok().map(|node| {
            let unit  = 7.224_609_4;
            let width = unit * node.payload.length as f32;
            let x     = width/2.0 + unit * node.payload.index as f32;
            Vector2::new(x + node::TEXT_OFF,node::NODE_HEIGHT/2.0)
        })
    }

    pub fn get_port_color(&self, _crumbs:&[span_tree::Crumb]) -> Option<color::Lcha> {
        // let ast_id = get_id_for_crumbs(&self.model.expression.borrow().input_span_tree,&crumbs)?;
        // // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        // let styles = StyleWatch::new(&self.model.app.display.scene().style_sheet);
        // self.model.type_color_map.type_color(ast_id,&styles)
        None
    }

    pub fn width(&self) -> f32 {
        self.model.width.get()
    }

    // pub fn set_expression_type(&self, id:ast::Id, _maybe_type:Option<Type>) {
    //     if let Some(crumbs) = self.model.id_crumbs_map.borrow().get(&id) {
    //         if let Ok(_node) = self.model.expression.borrow_mut().input_span_tree.get_node(crumbs) {
    //
    //         }
    //     }
    //     // self.model.type_color_map.update_entry(id,maybe_type);
    // }
}

impl display::Object for Area {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
