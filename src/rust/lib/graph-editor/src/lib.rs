#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

#![feature(associated_type_defaults)]
#![feature(clamp)]
#![feature(drain_filter)]
#![feature(overlapping_marker_traits)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(weak_into_raw)]
#![feature(fn_traits)]

#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]
#![warn(unsafe_code)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#![recursion_limit="512"]


#[warn(missing_docs)]
pub mod component;

/// Common types and functions usable in all modules of this crate.
pub mod prelude {
    pub use ensogl::prelude::*;
}

use ensogl::application;
use ensogl::prelude::*;
use ensogl::traits::*;

use crate::component::cursor::Cursor;
use crate::component::node;
use crate::component::node::Node;
use crate::component::node::WeakNode;
use crate::component::connection::Connection as Edge;
use enso_frp as frp;
use enso_frp::io::keyboard;
use enso_frp::Position;
use ensogl::display::object::Id;
use ensogl::display::world::*;
use ensogl::display;
use ensogl::system::web::StyleSetter;
use ensogl::system::web;
use nalgebra::Vector2;
use ensogl::display::Scene;
use crate::component::node::port::Expression;


#[derive(Clone,CloneRef,Debug,Default)]
pub struct NodeSet {
    data : Rc<RefCell<HashMap<Id,NodeModel>>>
}

impl NodeSet {
//    pub fn borrow(&self) -> Ref<HashMap<Id,Node>> {
//        self.data.borrow()
//    }
//
//    pub fn take(&self) -> HashMap<Id,Node> {
//        mem::take(&mut *self.data.borrow_mut())
//    }
//
    pub fn insert(&self, id:Id, node_model:NodeModel) {
        self.data.borrow_mut().insert(id,node_model);
    }

    pub fn remove(&self, id:&Id) {
        self.data.borrow_mut().remove(id);
    }
//
//    pub fn contains(&self, node:&Node) -> bool {
//        self.get(node.id()).is_some()
//    }
//
    pub fn get(&self, id:&Id) -> Option<NodeModel> {
        self.data.borrow().get(id).map(|t| t.clone_ref())
    }
//
    pub fn clear(&self) {
        self.data.borrow_mut().clear();
    }
}



#[derive(Clone,CloneRef,Debug,Default)]
pub struct WeakNodeSet {
    data : Rc<RefCell<HashMap<Id,WeakNode>>>
}

impl WeakNodeSet {
    pub fn borrow(&self) -> Ref<HashMap<Id,WeakNode>> {
        self.data.borrow()
    }

    pub fn take(&self) -> HashMap<Id,WeakNode> {
        mem::take(&mut *self.data.borrow_mut())
    }

    pub fn for_each_taken<F:Fn(Node)>(&self,f:F) {
        self.take().into_iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
    }

    pub fn for_each<F:Fn(Node)>(&self,f:F) {
        self.data.borrow().iter().for_each(|(_,node)| { node.upgrade().for_each(|n| f(n)) })
    }

    pub fn insert(&self, node:&Node) {
        self.data.borrow_mut().insert(node.id(),node.downgrade());
    }

    pub fn contains(&self, node:&Node) -> bool {
        self.get(node.id()).is_some()
    }

    pub fn get(&self, id:Id) -> Option<Node> {
        self.data.borrow().get(&id).and_then(|t| t.upgrade())
    }
}


#[derive(Clone,CloneRef,Debug,Default,Shrinkwrap)]
pub struct WeakNodeSelectionSet {
    data : WeakNodeSet
}

impl WeakNodeSelectionSet {
    pub fn clear(&self) {
        self.for_each_taken(|node| node.frp.deselect.emit(()));
    }
}



#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorFrp {
    pub network : frp::Network,
    pub inputs  : FrpInputs,
    pub status  : FrpStatus,
    pub node_release : frp::Stream<Id>
}

impl Deref for GraphEditorFrp {
    type Target = FrpInputs;
    fn deref(&self) -> &FrpInputs {
        &self.inputs
    }
}


ensogl::def_status_api! { FrpStatus
    /// Checks whether this graph editor instance is active.
    is_active,
    /// Checks whether this graph editor instance is empty.
    is_empty,
}

ensogl::def_command_api! { Commands
    /// Add a new node and place it at the mouse cursor position.
    add_node_at_cursor,
    /// Remove all selected nodes from the graph.
    remove_selected_nodes,
    /// Remove all nodes from the graph.
    remove_all_nodes,
}


impl Commands {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def add_node_at_cursor    = source();
            def remove_selected_nodes = source();
            def remove_all_nodes      = source();
        }
        Self {add_node_at_cursor,remove_selected_nodes,remove_all_nodes}
    }
}

#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
pub struct FrpInputs {
    #[shrinkwrap(main_field)]
    commands                     : Commands,
    register_node                : frp::Source<Option<Node>>,
    pub add_node_at              : frp::Source<Position>,
    pub set_node_position        : frp::Source<(Id,Position)>,
    pub set_node_expression      : frp::Source<(Id,Expression)>,
    pub connect_nodes            : frp::Source<(EdgeTarget,EdgeTarget)>,
    pub select_node              : frp::Source<Id>,
    pub translate_selected_nodes : frp::Source<Position>,
}

impl FrpInputs {
    pub fn new(network:&frp::Network) -> Self {
        let commands = Commands::new(network);
        frp::extend! { network
            def register_node            = source();
            def add_node_at              = source();
            def set_node_position        = source();
            def select_node              = source();
            def translate_selected_nodes = source();
            def set_node_expression      = source();
            def connect_nodes            = source();
        }
        Self {commands,register_node,add_node_at,set_node_position,select_node,translate_selected_nodes,set_node_expression,connect_nodes}
    }

    fn register_node(&self, arg:&Node) {
        self.register_node.emit(&Some(arg.clone_ref()));
    }
    pub fn add_node_at<T: AsRef<Position>>(&self, arg: T) {
        self.add_node_at.emit(arg.as_ref());
    }
    pub fn add_node_at_cursor(&self) {
        self.add_node_at_cursor.emit(());
    }
    pub fn select_node(&self, arg:Id) {
        self.select_node.emit(arg);
    }
    pub fn translate_selected_nodes<T: AsRef<Position>>(&self, arg: T) {
        self.translate_selected_nodes.emit(arg.as_ref());
    }
    pub fn remove_selected_nodes(&self) {
        self.remove_selected_nodes.emit(());
    }
    pub fn remove_all_nodes(&self) {
        self.remove_all_nodes.emit(());
    }
}

impl application::command::FrpNetworkProvider for GraphEditor {
    fn network(&self) -> &frp::Network {
        &self.frp.network
    }
}

impl application::command::CommandApi for GraphEditor {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.inputs.commands.command_api()
    }
}

impl application::command::StatusApi for GraphEditor {
    fn status_api_docs() -> Vec<application::command::EndpointDocs> {
        FrpStatus::status_api_docs()
    }

    fn status_api(&self) -> Vec<application::command::StatusEndpoint> {
        self.frp.status.status_api()
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct NodeModel {
    pub view      : Node,
    pub in_edges  : Rc<RefCell<HashSet<Id>>>,
    pub out_edges : Rc<RefCell<HashSet<Id>>>,
}

impl NodeModel {
    pub fn new(view:Node) -> Self {
        let in_edges  = default();
        let out_edges = default();
        Self {view,in_edges,out_edges}
    }
}

impl display::Object for NodeModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object()
    }
}


#[derive(Debug,Clone,CloneRef,Default)]
pub struct Nodes {
    pub set      : NodeSet,
    pub selected : WeakNodeSelectionSet,
}


#[derive(Clone,Debug,Default)]
pub struct EdgeTarget {
    node_id    : Id,
    port_crumb : span_tree::Crumbs,
}

impl EdgeTarget {
    pub fn new(node_id:Id, port_crumb:span_tree::Crumbs) -> Self {
        Self {node_id,port_crumb}
    }
}


#[derive(Debug)]
pub struct EdgeModel {
    pub view   : Edge,
    pub source : Option<EdgeTarget>,
    pub target : Option<EdgeTarget>,
}

impl EdgeModel {
    pub fn new_with_source(view:Edge, node_id:Id) -> Self {
        let port_crumb = default();
        let source     = EdgeTarget {node_id,port_crumb};
        let source     = Some(source);
        let target     = default();
        Self {view,source,target}
    }

    pub fn id(&self) -> Id {
        self.view.id()
    }
}

#[derive(Debug,Clone,CloneRef,Default)]
pub struct Edges {
    pub map      : Rc<RefCell<HashMap<Id,EdgeModel>>>,
    pub detached : Rc<RefCell<HashSet<Id>>>,
}


#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditor {
    pub logger         : Logger,
    pub display_object : display::object::Instance,
    pub nodes          : Nodes,
    pub edges          : Edges,
    pub frp            : GraphEditorFrp,
    pub scene          : Scene,
}

#[derive(Debug,CloneRef,Derivative)]
#[derivative(Clone(bound=""))]
pub struct TouchNetwork<T:frp::Data> {
    pub down     : frp::Source<T>,
    pub up       : frp::Stream<T>,
    pub is_down  : frp::Stream<bool>,
    pub selected : frp::Stream<T>
}

impl<T:frp::Data> TouchNetwork<T> {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        frp::extend! { network
            def down          = source::<T> ();
            def down_bool     = down.map(|_| true);
            def up_bool       = mouse.release.map(|_| false);
            def is_down       = down_bool.merge(&up_bool);
            def was_down      = is_down.previous();
            def mouse_up      = mouse.release.gate(&was_down);
            def pos_on_down   = mouse.position.sample(&down);
            def pos_on_up     = mouse.position.sample(&mouse_up);
            def should_select = pos_on_up.map3(&pos_on_down,&mouse.distance,Self::check);
            def up            = down.sample(&mouse_up);
            def selected      = up.gate(&should_select);
        }
        Self {down,up,is_down,selected}
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn check(end:&Position, start:&Position, diff:&f32) -> bool {
        (end-start).length() <= diff * 2.0
    }
}

#[derive(Debug,Clone,CloneRef)]
pub struct TouchState {
    pub nodes : TouchNetwork::<Id>,
    pub bg    : TouchNetwork::<()>,
}

impl TouchState {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        let nodes = TouchNetwork::<Id>::new(&network,mouse);
        let bg    = TouchNetwork::<()>::new(&network,mouse);
        Self {nodes,bg}
    }
}


impl GraphEditor {

    pub fn add_node(&self) -> WeakNode {
        let node = Node::new(&self.scene);
        self.frp.inputs.register_node(&node);
        node.downgrade()
    }

    pub fn add_node2(&self) -> Id {
        let node = Node::new(&self.scene);
        let id   = node.id();
        self.frp.inputs.register_node(&node);
        id
    }

    pub fn remove_node(&self, node:WeakNode) {
        if let Some(node) = node.upgrade() {
            self.nodes.set.remove(&node.id());
        }
    }

    pub fn get_node(&self, id:&Id) -> Option<NodeModel> {
        self.nodes.set.get(id)
    }
}

impl application::command::Provider for GraphEditor {
    fn label() -> &'static str {
        "GraphEditor"
    }
}

impl application::shortcut::DefaultShortcutProvider for GraphEditor {
    fn default_shortcuts() -> Vec<application::shortcut::Shortcut> {
        use keyboard::Key;
        vec! [ Self::self_shortcut(&[Key::Character("n".into())] , "add_node_at_cursor")
             , Self::self_shortcut(&[Key::Backspace]             , "remove_selected_nodes")
        ]
    }
}

impl application::View for GraphEditor {

    fn new(world:&World) -> Self {
        let logger = Logger::new("GraphEditor");
        let scene  = world.scene();
        let cursor = Cursor::new(world.scene());
        web::body().set_style_or_panic("cursor","none");
        world.add_child(&cursor);


        let display_object = display::object::Instance::new(logger.clone());
        let mouse          = &scene.mouse.frp;
        let network        = frp::Network::new();
        let inputs         = FrpInputs::new(&network);
        let nodes          = Nodes::default();
        let edges    = Edges::default();
        let touch          = TouchState::new(&network,mouse);


        frp::extend! { network

        // === Cursor ===

        def mouse_on_down_position = mouse.position.sample(&mouse.press);
        def selection_zero         = source::<Position>();
        def selection_size_down    = mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
        def selection_size_if_down = selection_size_down.gate(&touch.bg.is_down);
        def selection_size_on_down = selection_zero.sample(&mouse.press);
        def selection_size         = selection_size_if_down.merge(&selection_size_on_down);

        def _cursor_size = selection_size.map(f!((cursor)(p) {
            cursor.set_selection_size(Vector2::new(p.x,p.y));
        }));

        def _cursor_press = mouse.press.map(f!((cursor)(_) {
            cursor.frp.press.emit(());
        }));

        def _cursor_release = mouse.release.map(f!((cursor)(_) {
            cursor.frp.release.emit(());
        }));

//        def _cursor_position = mouse.position.map(f!((cursor)(p) {
//            cursor.set_position(Vector2::new(p.x,p.y));
//        }));


        // === Generic Selection ===

        def mouse_down_target  = mouse.press.map(f_!((scene) scene.mouse.target.get()));
        def _mouse_down_target = mouse_down_target.map(f!((touch,scene)(target) {
            match target {
                display::scene::Target::Background => {
                    touch.bg.down.emit(());
                }
                display::scene::Target::Symbol {..} => {
                    scene.shapes.get_mouse_target(*target).for_each(|target| {
                        target.mouse_down().emit(());
                    })
                }
            }
        }));


        // === Selection ===

        def _deselect_all_on_bg_press = touch.bg.selected.map(f_!((nodes) nodes.selected.clear()));
        def select_unified            = inputs.select_node.merge(&touch.nodes.selected);
        def _select_pressed           = select_unified.map(f!((nodes)(node_id) {
            if let Some(node) = nodes.set.get(node_id) {
                nodes.selected.clear();
                node.view.frp.select.emit(());
                nodes.selected.insert(&node.view);
            }
        }));

        // === Connect Nodes ===

        def node_port_press = source::<(Id,span_tree::Crumbs)>();
        def connect_nodes_on_port_press = node_port_press.map(f!((nodes,edges)((node_id,crumbs)) {
            if let Some(node) = nodes.set.get(node_id) {
                for edge_id in mem::take(&mut *edges.detached.borrow_mut()) {
                    if let Some(edge) = edges.map.borrow_mut().get_mut(&edge_id) {
                        println!("{:?}", crumbs);
                        edge.target = Some(EdgeTarget::new(*node_id,crumbs.clone()));
                        node.in_edges.borrow_mut().insert(edge_id);
                    }
                }
            }
        }));

        def _foo = inputs.connect_nodes.map(f!((scene,display_object,nodes,edges)((source,target)){
            let source_node = nodes.set.get(&source.node_id).unwrap();
            let target_node = nodes.set.get(&target.node_id).unwrap();
            let view = Edge::new(&scene);
            view.mod_position(|p| p.x = source_node.position().x + node::NODE_WIDTH/2.0);
            view.mod_position(|p| p.y = source_node.position().y + node::NODE_HEIGHT/2.0);
            display_object.add_child(&view);
            let mut edge = EdgeModel::new_with_source(view,source.node_id);
            let edge_id = edge.id();
            edge.target = Some(target.clone());
            edges.map.borrow_mut().insert(edge_id,edge);
            target_node.in_edges.borrow_mut().insert(edge_id);

//            edges.detached.borrow_mut().insert(edge_id);
        }));


        // === Add Node ===

        let scene                  = world.scene();
        def add_node_at_cursor_pos = inputs.add_node_at_cursor.map2(&mouse.position,|_,p|{*p});
        def add_node               = inputs.add_node_at.merge(&add_node_at_cursor_pos);
        def _add_new_node          = add_node.map(f!((scene,inputs)(pos) {
            let node = Node::new(&scene);
            inputs.register_node(&node);
            node.mod_position(|t| {
                t.x = pos.x;
                t.y = pos.y;
            });
        }));


        def _new_node = inputs.register_node.map(f!((cursor,network,nodes,edges,touch,display_object,scene,node_port_press)(node) {
            if let Some(node) = node {
                let node_id : Id  = node.id();
                let weak_node = node.downgrade();
                frp::new_bridge_network! { [network,node.view.events.network]
                    def _node_on_down_tagged = node.drag_view.events.mouse_down.map(f_!((weak_node,touch) {
                        touch.nodes.down.emit(node_id)
                    }));
                    def cursor_mode = node.ports.frp.cursor_mode.map(f!((cursor)(mode) {
                        cursor.frp.set_mode.emit(mode);
                    }));
                    def _add_connection = node.frp.output_ports.mouse_down.map(f_!((weak_node,edges,display_object,scene) {
                        if let Some(node) = weak_node.upgrade() {
                            let view = Edge::new(&scene);
                            view.mod_position(|p| p.x = node.position().x + node::NODE_WIDTH/2.0);
                            view.mod_position(|p| p.y = node.position().y + node::NODE_HEIGHT/2.0);
                            display_object.add_child(&view);
                            let connection = EdgeModel::new_with_source(view,node.id());
                            let id = connection.id();
                            edges.map.borrow_mut().insert(id,connection);
                            edges.detached.borrow_mut().insert(id);
                        }
                    }));

                    def _foo = node.ports.frp.press.map(f!((node_port_press)(crumbs){
                        node_port_press.emit((node_id,crumbs.clone()));
                    }));
                }
                display_object.add_child(node);
                nodes.set.insert(node.id(),NodeModel::new(node.clone_ref()));
            }
        }));

        // === Set Node Position ===

        def _set_node_position = inputs.set_node_position.map(f!((nodes)((node_id,position)){
            if let Some(node) = nodes.set.get(node_id) {
                node.view.mod_position(|t| {
                    t.x = position.x;
                    t.y = position.y;
                })
            }
        }));


        // === Remove Node ===

        def _remove_all      = inputs.remove_all_nodes.map(f!((nodes)(()) nodes.set.clear()));
        def _remove_selected = inputs.remove_selected_nodes.map(f!((nodes,nodes)(_) {
            nodes.selected.for_each_taken(|node| nodes.set.remove(&node.id()))
        }));


        // === Set Node Expression ===

        def _set_node_expr = inputs.set_node_expression.map(f!((nodes)((node_id,expression)){
            if let Some(node) = nodes.set.get(node_id) {
                node.view.ports.set_expression(expression);
            }
        }));


        // === Move Nodes ===

        def mouse_tx_if_node_pressed = mouse.translation.gate(&touch.nodes.is_down);
        def _move_node_with_mouse    = mouse_tx_if_node_pressed.map2(&touch.nodes.down,f!((nodes,edges)(tx,node_id) {
//            let node_id : Id = node.id();
            if let Some(node) = nodes.set.get(&node_id) {
                node.view.mod_position(|p| { p.x += tx.x; p.y += tx.y; });
                for edge_id in &*node.in_edges.borrow() {
                    if let Some(edge) = edges.map.borrow().get(edge_id) {
                        if let Some(edge_target) = &edge.target {
                            let offset = node.view.ports.get_port_offset(&edge_target.port_crumb).unwrap_or(Vector2::new(0.0,0.0));
                            let node_position = node.view.position();
                            let position = frp::Position::new(node_position.x + offset.x, node_position.y + offset.y);
                            edge.view.events.target_position.emit(position);
                        }
                    }
                }
            }
        }));

        def _move_selected_nodes = inputs.translate_selected_nodes.map(f!((nodes)(t) {
            nodes.selected.for_each(|node| {
                node.mod_position(|p| {
                    p.x += t.x;
                    p.y += t.y;
                })
            })
        }));


        // === Move Edges ===

        def _move_connections = cursor.frp.position.map(f!((edges)(position) {
            for id in &*edges.detached.borrow() {
                if let Some(connection) = edges.map.borrow().get(id) {
                    connection.view.events.target_position.emit(position);
                }
            }
        }));


        // === Status ===

        def is_active_src = source::<bool>();
        def is_empty_src  = source::<bool>();
        def is_active = is_active_src.sampler();
        def is_empty  = is_empty_src.sampler();

        }

        // FIXME This is a temporary solution. Should be replaced by a real thing once layout
        //       management is implemented.
        is_active_src.emit(true);

        let status = FrpStatus {is_active,is_empty};

        let node_release = touch.nodes.up;



        let frp = GraphEditorFrp {network,inputs,status,node_release};

        let scene = scene.clone_ref();
        Self {logger,frp,nodes,edges,display_object,scene}
    }


}

impl display::Object for GraphEditor {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
