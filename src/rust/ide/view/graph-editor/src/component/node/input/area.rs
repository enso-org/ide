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
use ensogl::gui::component;
use ensogl::gui::cursor;
use ensogl_text as text;
use ensogl_text::buffer;
use ensogl_theme as theme;
use span_tree::SpanTree;
use text::Bytes;
use text::Text;

use crate::node;

use crate::Type;
use crate::component::type_coloring;
use ensogl_text::buffer::data::unit::traits::*;



// =================
// === Constants ===
// =================

/// Enable visual port debug mode.
const DEBUG : bool = true;

/// Skip creating ports on all operations. For example, in expression `foo bar`, `foo` is considered
/// an operation.
const SKIP_OPERATIONS            : bool = true;
const SHOW_PORTS_ONLY_ON_CONNECT : bool = true;
const PORT_PADDING_X             : f32  = 4.0;



/// ================
/// === SpanTree ===
/// ================

fn iterate_layers
( root         : span_tree::node::RefMut<PortModel>
, mut on_layer : impl FnMut()
, mut on_node  : impl FnMut(&mut span_tree::node::RefMut<PortModel>) -> bool
) {
    let mut layer  = vec![root];
    let mut layers = vec![];
    loop {
        match layer.pop() {
            None => {
                match layers.pop() {
                    None    => break,
                    Some(l) => {
                        on_layer();
                        layer = l
                    }
                }
            },
            Some(mut node) => {
                if on_node(&mut node) {
                    let mut children = node.children_iter().collect_vec();
                    children.reverse(); // FIXME : add reversed
                    layers.push(children);
                }
            }
        }
    }
}

fn iterate_depth
( root         : span_tree::node::RefMut<PortModel>
, mut on_node  : impl FnMut(&mut span_tree::node::RefMut<PortModel>) -> bool
) {
    let mut layer  = vec![root];
    let mut layers = vec![];
    loop {
        match layer.pop() {
            None => {
                match layers.pop() {
                    None    => break,
                    Some(l) => layer = l,
                }
            },
            Some(mut node) => {
                if on_node(&mut node) {
                    let mut children = node.children_iter().collect_vec();
                    children.reverse(); // FIXME : add reversed
                    mem::swap(&mut children,&mut layer);
                    layers.push(children);
                }
            }
        }
    }
}


fn iterate_layers_depth<D>
( root         : span_tree::node::RefMut<PortModel>
, mut data     : D
, mut on_node  : impl FnMut(&mut span_tree::node::RefMut<PortModel>, &mut D) -> (bool,D)
) {
    let mut layer  = vec![root];
    let mut layers = vec![];
    loop {
        match layer.pop() {
            None => {
                match layers.pop() {
                    None        => break,
                    Some((l,d)) => {
                        layer = l;
                        data  = d;
                    },
                }
            },
            Some(mut node) => {
                let (ok,mut sub_data) = on_node(&mut node, &mut data);
                if ok {
                    let mut children = node.children_iter().collect_vec();
                    children.reverse(); // FIXME : add reversed
                    mem::swap(&mut sub_data,&mut data);
                    mem::swap(&mut children,&mut layer);
                    layers.push((children,sub_data));
                }
            }
        }
    }
}

fn iterate_nodes
( root    : span_tree::node::RefMut<PortModel>
, on_node : impl FnMut(&mut span_tree::node::RefMut<PortModel>) -> bool
) {
    iterate_layers(root,||{},on_node)
}

fn iterate_leaves(root:span_tree::node::RefMut<PortModel>, mut f:impl FnMut(&mut span_tree::node::RefMut<PortModel>)) {
    iterate_nodes(root,|mut node| { if node.children.is_empty() { f(&mut node) }; true })
}


// ==================
// === Port Shape ===
// ==================

/// Port shape definition.
pub mod shape {
    use super::*;
    ensogl::define_shape_system! {
        (style:Style) {
            let width  : Var<Pixels> = "input_size.x".into();
            let height : Var<Pixels> = "input_size.y".into();
            let shape  = Rect((&width,&height));
            if !DEBUG {
                let color = Var::<color::Rgba>::from("srgba(1.0,1.0,1.0,0.00001)");
                shape.fill(color).into()
            } else {
                let shape = shape.corners_radius(6.px());
                let color = Var::<color::Rgba>::from("srgba(1.0,0.0,0.0,0.1)");
                shape.fill(color).into()
            }
        }
    }
}

/// Function used to hack depth sorting. To be removed when it will be implemented in core engine.
pub fn sort_hack(scene:&Scene) {
    let logger = Logger::new("hack");
    component::ShapeView::<shape::Shape>::new(&logger,scene);
}



// ==================
// === Expression ===
// ==================

#[derive(Clone,Default)]
pub struct Expression {
    pub code             : String,
    pub input_span_tree  : SpanTree,
    pub output_span_tree : SpanTree,
}

impl Expression {
    pub fn debug_from_str(s:&str) -> Self {
        let code             = s.into();
        let input_span_tree  = default();
        let output_span_tree = default();
        Self {code,input_span_tree,output_span_tree}
    }
}

fn get_id_for_crumbs(span_tree:&PortSpanTree, crumbs:&[span_tree::Crumb]) -> Option<ast::Id> {
    if span_tree.root_ref().crumbs == crumbs {
        return span_tree.root.ast_id
    };
    let span_tree_descendant = span_tree.root_ref().get_descendant(crumbs);
    let ast_id        = span_tree_descendant.map(|node|{node.ast_id});
    ast_id.ok().flatten()
}


impl Debug for Expression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Expression({})",self.code)
    }
}

impl From<&Expression> for Expression {
    fn from(t:&Expression) -> Self {
        t.clone()
    }
}



// =======================
// === InputExpression ===
// =======================

#[derive(Clone,Default)]
pub struct InputExpression {
    pub code  : String,
    pub input : PortSpanTree,
}

impl Deref for InputExpression {
    type Target = PortSpanTree;
    fn deref(&self) -> &Self::Target {
        &self.input
    }
}

impl DerefMut for InputExpression {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.input
    }
}

impl Debug for InputExpression {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"Expression({})",self.code)
    }
}

impl From<Expression> for InputExpression {
    fn from(t:Expression) -> Self {
        let code  = t.code;
        let input = t.input_span_tree.map(|_|default());
        Self {code,input}
    }
}



// ================================
// === PortSpanTree / PortModel ===
// ================================

/// `SpanTree` parametrized with `PortModel` payload.
pub type PortSpanTree = SpanTree<PortModel>;

pub mod leaf {
    use super::*;
    ensogl::define_endpoints! {
        Input {
            set_optional (bool),
            set_disabled (bool),
            set_hover    (bool),
        }

        Output {
            color (color::Lcha)
        }
    }

    #[derive(Clone,Debug,Default)]
    pub struct PortModel {
        pub frp         : leaf::Frp,
        pub shape       : Option<component::ShapeView<shape::Shape>>,
        pub name        : Option<String>,
        pub index       : usize,
        pub local_index : usize,
        pub length      : usize,
        pub color       : color::Animation2,
    }

    impl Deref for PortModel {
        type Target = leaf::Frp;
        fn deref(&self) -> &Self::Target {
            &self.frp
        }
    }

    impl PortModel {
        pub fn new() -> Self {
            default()
        }
    }



}
pub use leaf::*;



// ===========
// === FRP ===
// ===========

ensogl::define_endpoints! {
    Input {
        edit_mode_ready     (bool),
        edit_mode           (bool),
        hover               (),
        unhover             (),
        set_dimmed          (bool),
        set_expression_type (ast::Id,Option<Type>),
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






// =============
// === Model ===
// =============

/// Internal model of the port manager.
#[derive(Debug)]
pub struct Model {
    logger         : Logger,
    display_object : display::object::Instance,
    ports_group    : display::object::Instance,
    header         : display::object::Instance,
    app            : Application,
    expression     : RefCell<InputExpression>,
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
        Self {logger,display_object,ports_group,header,label,width,app,expression,styles
             ,id_crumbs_map} . init()
    }

    fn init(self) -> Self {
        self.label.single_line(true);
        self.label.disable_command("cursor_move_up");
        self.label.disable_command("cursor_move_down");

        let text_color = self.styles.get_color(theme::vars::graph_editor::node::text::color);
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

    fn on_port_hover(&self, is_hovered:bool, crumbs:&span_tree::Crumbs) {
        let mut expression = self.expression.borrow_mut();
        if let Ok(node) = expression.input.root_ref_mut().get_descendant(crumbs) {
            iterate_leaves(node,|node| node.payload.frp.set_hover(is_hovered))
        }
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
    (parent:display::object::Instance, parent_parensed:bool, shift:usize, depth:usize) -> Self {
        Self {parent,parent_parensed,shift,depth}
    }

    fn nested(&self, parent:display::object::Instance, parent_parensed:bool, shift:usize) -> Self {
        let depth = self.depth + 1;
        Self::new(parent,parent_parensed,shift,depth)
    }

    fn empty(parent:display::object::Instance) -> Self {
        Self::new(parent,default(),default(),default())
    }
}


// ===============
// === Manager ===
// ===============

#[derive(Clone,CloneRef,Debug)]
pub struct Manager {
    pub frp : Frp,
    model   : Rc<Model>,
}

impl Deref for Manager {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl Manager {
    pub fn new(logger:impl AnyLogger, app:&Application) -> Self {
        let model      = Rc::new(Model::new(logger,app));
        let frp        = Frp::new();
        let network    = &frp.network;
        let text_color = color::Animation::new();
        let style      = StyleWatch::new(&app.display.scene().style_sheet);


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

            eval frp.port_over ((crumbs) model.on_port_hover(true,crumbs));
            eval frp.port_out  ((crumbs) model.on_port_hover(false,crumbs));


            // === Properties ===

            frp.output.source.width      <+ model.label.width;
            frp.output.source.expression <+ model.label.content.map(|t| t.clone_ref());


            // === Color Handling ===

            eval text_color.value ([model](color) {
                // FIXME[WD]: Disabled to improve colors. To be fixed before merge.
                // // TODO: Make const once all the components can be made const.
                // let all_bytes = buffer::Range::from(Bytes::from(0)..Bytes(i32::max_value()));
                // model.label.set_color_bytes(all_bytes,color::Rgba::from(color));
            });

            eval frp.set_dimmed ([text_color,style](should_dim) {
                let text_color_path = theme::vars::graph_editor::node::text::color;
                if *should_dim {
                    text_color.set_target(style.get_color_dim(text_color_path));
                } else {
                    text_color.set_target(style.get_color(text_color_path));
                }
            });
        }

        Self {model,frp}
    }

    fn scene(&self) -> &Scene {
        self.model.app.display.scene()
    }

    pub(crate) fn set_expression(&self, expression:impl Into<Expression>) {
        let model      = &self.model;
        let expression = expression.into();
        println!("\n\n=====================\nSET EXPR: {}",expression.code);
        let mut expression = InputExpression::from(expression);

        let glyph_width = 7.224_609_4; // FIXME hardcoded literal
        let width       = expression.code.len() as f32 * glyph_width;
        model.width.set(width);



        let mut vis_expr      = expression.code.clone();



        let shift = Cell::new(0);

        iterate_layers(expression.root_ref_mut(),||{shift.set(0)},|node| {
            let is_expected_arg = node.kind.is_expected_argument();
            let span            = node.span();
            let mut size        = span.size.value;
            let mut index       = span.index.value + shift.get();
            if is_expected_arg {
                // let name = node.name().unwrap();
                // size     = name.len();
                // index   += 1;
                // shift.set(shift.get() + size + 1);
                // vis_expr.push(' ');
                // vis_expr += name;
            }
            node.payload().local_index = node.offset.value;
            node.payload().index  = index;
            node.payload().length = size;
            true
        });

        // let mut to_visit      = vec![expression.input_span_tree.root_ref_mut()];

        model.header.unset_parent();

        let root = model.ports_group.clone_ref();
        let mut is_header = true;

        let builder = PortLayerBuilder::empty(root);
        iterate_layers_depth(expression.root_ref_mut(),builder,|node,builder| {
            let is_leaf         = node.children.is_empty();
            let span            = node.span();
            let contains_root   = span.index.value == 0;
            let is_expected_arg = node.is_expected_argument();
            let is_parensed     = node.is_parensed();
            let skip_opr        = if SKIP_OPERATIONS { node.is_operation() } else {
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
            println!("{}[{},{}] {} {:?}",indent,node.payload.index,node.payload.length,skipped,node.kind.short_repr());
            let new_parent = if !skip {

                let logger_name  = format!("port({},{})",node.payload.index,node.payload.length);
                let logger       = Logger::sub(&model.logger,logger_name);
                let port         = component::ShapeView::<shape::Shape>::new(&logger,self.scene());
                let index        = node.payload.local_index + builder.shift;
                let size         = node.payload.length;
                let unit         = 7.224_609_4;
                let width        = unit * size as f32;
                let width_padded = width + 2.0 * PORT_PADDING_X;
                let node_height  = 28.0;
                let height       = 18.0;
                let size         = Vector2::new(width_padded,height);
                port.shape.sprite.size.set(Vector2::new(width_padded,node_height));
                port.shape.mod_position(|t| t.x = width/2.0);
                port.mod_position(|t| t.x = unit * index as f32);
                if DEBUG {
                    port.mod_position(|t| t.y = 5.0);
                }
                if is_header {
                    println!("ADDING HEADER");
                    is_header = false;
                    model.header.add_child(&port);
                } else {
                    builder.parent.add_child(&port);
                }

                // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
                let styles             = StyleWatch::new(&model.app.display.scene().style_sheet);
                let missing_type_color = styles.get_color(theme::vars::graph_editor::edge::_type::missing::color);

                let crumbs = node.crumbs.clone();
                // let _ast_id = get_id_for_crumbs(&expression.input_span_tree,&crumbs);
                // println!(">> {:?}",node.argument_info.clone().unwrap_or_default().typename);
                // let color  = ast_id.and_then(|id|type_map.type_color(id,styles.clone_ref()));
                // let color  = color.unwrap_or(missing_type_color);
                let color = node.tp().map(
                    |tp| type_coloring::color_for_type(tp.clone().into(),&styles)
                ).unwrap_or(missing_type_color);

                let highlight = cursor::Style::new_highlight(&port.shape,size,Some(color));

                let leaf     = &node.frp;
                let port_network  = &leaf.network;

                frp::extend! { port_network
                    let mouse_over = port.events.mouse_over.clone_ref();
                    let mouse_out  = port.events.mouse_out.clone_ref();


                    // === Mouse Style ===

                    pointer_style_over  <- mouse_over.map(move |_| highlight.clone());
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
                    eval_ port.events.mouse_down (press_source.emit(&crumbs_down));
                }

                let network = &self.frp.network;

                let index  = node.payload.index;
                let length = node.payload.length;

                frp::extend! { port_network
                    let mouse_over = port.events.mouse_over.clone_ref();
                    let mouse_out  = port.events.mouse_out.clone_ref();

                    let crumbs_over = crumbs.clone(); // fixme - passing crumbs is slow in frp
                    let crumbs_out  = crumbs.clone();

                    self.source.port_over <+ mouse_over.map (move |_| crumbs_over.clone());
                    self.source.port_out  <+ mouse_out.map  (move |_| crumbs_out.clone());
                }

                let new_parent = port.display_object().clone_ref();
                node.payload().shape = Some(port);
                new_parent
            } else {
                builder.parent.clone_ref()
            };

            let new_shift = if !skip { 0 } else { builder.shift + node.payload.local_index };
            (true,builder.nested(new_parent,is_parensed,new_shift))

                    // // FIXME: This is ugly because `children_iter` is not DoubleEndedIterator.
                    // to_visit.extend(node.children_iter().collect_vec().into_iter().rev());

            // true
        });

        // header.unset_parent();

        model.label.set_cursor(&default());
        model.label.select_all();
        model.label.insert(&vis_expr);
        model.label.remove_all_cursors();

        // === Missing args styling ===
        // FIXME[WD]: The text offset is computed in bytes as text area supports only this interface
        //            now. May break with unicode input. To be fixed.
        let arg_color   = model.styles.get_color(theme::vars::graph_editor::node::text::missing_arg_color);
        let start_bytes = (expression.code.len() as i32).bytes();
        let end_bytes   = (vis_expr.len() as i32).bytes();
        let range       = ensogl_text::buffer::Range::from(start_bytes..end_bytes);
        model.label.set_color_bytes(range,color::Rgba::from(arg_color));

        if self.frp.editing.value() {
            model.label.set_cursor_at_end();
        }


        iterate_leaves(expression.root_ref_mut(),|node| {
            let leaf         = &node.frp;
            let port_network = &leaf.network;

            frp::extend! { port_network
                ccc <- leaf.input.set_hover.map(f!([model](is_hovered)
                    if *is_hovered { color::Lcha::from(color::Rgba::new(1.0,0.0,0.0,1.0)) }
                    else { model.styles.get_color(theme::vars::graph_editor::node::text::color) }
                ));
                node.color.target <+ ccc;
                leaf.output.source.color <+ node.color.value;
            }

            let index = node.payload.index;
            let length = node.payload.length;

            frp::extend! { port_network
                eval leaf.output.color ([model](color) {
                    // println!("COLOR! Index: {}, length: {}",index,length);
                    let start_bytes = (index as i32).bytes();//(expression.code.len() as i32).bytes();
                    let end_bytes   = ((index + length) as i32).bytes();//(vis_expr.len() as i32).bytes();
                    let range       = ensogl_text::buffer::Range::from(start_bytes..end_bytes);
                    model.label.set_color_bytes(range,color::Rgba::from(color));
                });
            }
        });

        *model.expression.borrow_mut() = expression;

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

impl display::Object for Manager {
    fn display_object(&self) -> &display::object::Instance {
        &self.model.display_object
    }
}
