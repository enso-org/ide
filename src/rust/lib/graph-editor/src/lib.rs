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
use crate::component::node::Node as NodeView;
use crate::component::node::WeakNode as WeakNodeView;
use crate::component::connection::Connection as EdgeView;
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




// =====================
// === SharedHashSet ===
// =====================

#[derive(Derivative,CloneRef)]
#[derivative(Debug(bound="T:Eq+Hash+Debug, S:std::hash::BuildHasher"))]
pub struct SharedHashSet<T,S=std::collections::hash_map::RandomState> {
    pub raw : Rc<RefCell<HashSet<T,S>>>
}

impl<T,S> Clone for SharedHashSet<T,S> {
    fn clone(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<T,S> Default for SharedHashSet<T,S>
    where T:Eq+Hash, S:Default+std::hash::BuildHasher {
    fn default() -> Self {
        let raw = default();
        Self {raw}
    }
}

impl<T,S> SharedHashSet<T,S>
    where T:Eq+Hash, S:Default+std::hash::BuildHasher {
    pub fn new() -> Self {
        default()
    }

    pub fn mem_take(&self) -> HashSet<T,S> {
        mem::take(&mut *self.raw.borrow_mut())
    }
}

impl<T,S> SharedHashSet<T,S>
    where T:Eq+Hash, S:std::hash::BuildHasher {
    pub fn insert(&self, t:T) -> bool {
        self.raw.borrow_mut().insert(t)
    }
}

impl<T,S> SharedHashSet<T,S> {
    pub fn clear(&self) {
        self.raw.borrow_mut().clear()
    }

    pub fn for_each<F>(&self, f:F)
        where F:FnMut(&T) {
        self.raw.borrow_mut().iter().for_each(f)
    }
}



// =====================
// === SharedHashMap ===
// =====================

#[derive(Derivative,CloneRef)]
#[derivative(Debug(bound="K:Eq+Hash+Debug, V:Debug, S:std::hash::BuildHasher"))]
pub struct SharedHashMap<K,V,S=std::collections::hash_map::RandomState> {
    pub raw : Rc<RefCell<HashMap<K,V,S>>>
}

impl<K,V,S> Clone for SharedHashMap<K,V,S> {
    fn clone(&self) -> Self {
        let raw = self.raw.clone();
        Self {raw}
    }
}

impl<K,V,S> Default for SharedHashMap<K,V,S>
where K:Eq+Hash, S:Default+std::hash::BuildHasher {
    fn default() -> Self {
        let raw = default();
        Self {raw}
    }
}

impl<K,V,S> SharedHashMap<K,V,S>
where K:Eq+Hash, S:Default+std::hash::BuildHasher {
    pub fn new() -> Self {
        default()
    }

    pub fn mem_take(&self) -> HashMap<K,V,S> {
        mem::take(&mut *self.raw.borrow_mut())
    }
}

impl<K,V,S> SharedHashMap<K,V,S>
where K:Eq+Hash, S:std::hash::BuildHasher {
    pub fn insert(&self, k:K, v:V) -> Option<V> {
        self.raw.borrow_mut().insert(k,v)
    }

    pub fn get_cloned(&self, k:&K) -> Option<V>
    where V:Clone {
        self.raw.borrow().get(k).cloned()
    }

    pub fn remove(&self, k:&K) -> Option<V> {
        self.raw.borrow_mut().remove(k)
    }
}

impl<K,V,S> SharedHashMap<K,V,S> {
    pub fn clear(&self) {
        self.raw.borrow_mut().clear()
    }

    pub fn for_each<F>(&self, f:F)
    where F:FnMut((&K,&V)) {
        self.raw.borrow_mut().iter().for_each(f)
    }
}








#[derive(Debug,Clone,CloneRef)]
pub struct Frp {
    pub inputs  : FrpInputs,
    pub status  : FrpStatus,
    pub node_release : frp::Stream<NodeId>
}

impl Deref for Frp {
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


// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
pub struct FrpInputs {
    #[shrinkwrap(main_field)]
    commands                           : Commands,
    pub network                        : frp::Network,

    // === Public ===
    pub add_node_at                    : frp::Source<Position>,
    pub connect_detached_edges_to_node : frp::Source<EdgeTarget>,
    pub connect_nodes                  : frp::Source<(EdgeTarget,EdgeTarget)>,
    pub deselect_all_nodes             : frp::Source,
    pub select_node                    : frp::Source<NodeId>,
    pub set_node_expression            : frp::Source<(NodeId,Expression)>,
    pub set_node_position              : frp::Source<(NodeId,Position)>,
    pub translate_selected_nodes       : frp::Source<Position>,

    // === Private ===
    register_node : frp::Source<NodeId>,
}

impl FrpInputs {
    pub fn new() -> Self {
        frp::new_network! { network
            def add_node_at                    = source();
            def connect_detached_edges_to_node = source();
            def connect_nodes                  = source();
            def deselect_all_nodes             = source();
            def register_node                  = source();
            def select_node                    = source();
            def set_node_expression            = source();
            def set_node_position              = source();
            def translate_selected_nodes       = source();
        }
        let commands = Commands::new(&network);
        Self {commands,network,connect_detached_edges_to_node,register_node,add_node_at,set_node_position,select_node,translate_selected_nodes,set_node_expression,connect_nodes,deselect_all_nodes}
    }

//    fn register_node(&self, arg:&Node) {
//        self.register_node.emit(&Some(arg.clone_ref()));
//    }
    pub fn add_node_at<T: AsRef<Position>>(&self, arg: T) {
        self.add_node_at.emit(arg.as_ref());
    }
    pub fn add_node_at_cursor(&self) {
        self.add_node_at_cursor.emit(());
    }
    pub fn select_node(&self, arg:NodeId) {
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

impl Default for FrpInputs {
    fn default() -> Self {
        Self::new()
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



// ============
// === Node ===
// ============

#[derive(Clone,Debug)]
pub struct Node {
    pub view      : NodeView,
    pub in_edges  : HashSet<EdgeId>,
    pub out_edges : HashSet<EdgeId>,
}

#[derive(Clone,Copy,Debug,Default,Display,Eq,From,Hash,Into,PartialEq)]
pub struct NodeId(pub Id);

impl Node {
    pub fn new(view:NodeView) -> Self {
        let in_edges  = default();
        let out_edges = default();
        Self {view,in_edges,out_edges}
    }

    pub fn id(&self) -> NodeId {
        self.view.id().into()
    }
}

impl display::Object for Node {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object()
    }
}



// ============
// === Edge ===
// ============

#[derive(Clone,Debug)]
pub struct Edge {
    pub view   : EdgeView,
    pub source : Option<EdgeTarget>,
    pub target : Option<EdgeTarget>,
}

#[derive(Clone,Copy,Debug,Default,Display,Eq,From,Hash,Into,PartialEq)]
pub struct EdgeId(pub Id);

impl Edge {
    pub fn new(view:EdgeView) -> Self {
        let source = default();
        let target = default();
        Self {view,source,target}
    }

    pub fn new_with_source(view:EdgeView, node_id:NodeId) -> Self {
        let port = default();
        let source     = EdgeTarget {node_id,port};
        let source     = Some(source);
        let target     = default();
        Self {view,source,target}
    }

    pub fn id(&self) -> EdgeId {
        self.view.id().into()
    }
}



// ==================
// === EdgeTarget ===
// ==================

#[derive(Clone,Debug,Default)]
pub struct EdgeTarget {
    node_id : NodeId,
    port    : span_tree::Crumbs,
}

impl EdgeTarget {
    pub fn new(node_id:NodeId, port:span_tree::Crumbs) -> Self {
        Self {node_id,port}
    }
}







// =============
// === Nodes ===
// =============

#[derive(Debug,Clone,CloneRef)]
pub struct Nodes {
    pub logger   : Logger,
    pub all      : SharedHashMap<NodeId,Node>,
    pub selected : SharedHashSet<NodeId>,
}

impl Deref for Nodes {
    type Target = SharedHashMap<NodeId,Node>;
    fn deref(&self) -> &Self::Target {
        &self.all
    }
}

impl Nodes {
    pub fn new(logger:&Logger) -> Self {
        let logger   = logger.sub("nodes");
        let all      = default();
        let selected = default();
        Self {logger,all,selected}
    }

    pub fn with_borrow_mut<F,T>(&self, id:&NodeId, mut f:F) -> Option<T>
    where F:FnMut(&mut Node)->T {
        match self.all.raw.borrow_mut().get_mut(id) {
            Some(t) => Some(f(t)),
            None    => {
                warning!(self.logger, "Skipping invalid node id request ({id}).");
                None
            }
        }
    }

    pub fn with_borrow<F,T>(&self, id:&NodeId, f:F) -> Option<T>
        where F:Fn(&Node)->T {
        match self.all.raw.borrow().get(id) {
            Some(t) => Some(f(t)),
            None    => {
                warning!(self.logger, "Skipping invalid node id request ({id}).");
                None
            }
        }
    }

    pub fn get_view(&self, id:&NodeId) -> Option<NodeView> {
        self.with_borrow(id, |t| t.view.clone_ref())
    }
}





#[derive(Debug,Clone,CloneRef,Default)]
pub struct Edges {
    pub logger   : Logger,
    pub all      : SharedHashMap<EdgeId,Edge>,
    pub detached : SharedHashSet<EdgeId>,
}

impl Deref for Edges {
    type Target = SharedHashMap<EdgeId,Edge>;
    fn deref(&self) -> &Self::Target {
        &self.all
    }
}

impl Edges {
    pub fn new(logger:&Logger) -> Self {
        let logger   = logger.sub("edges");
        let all      = default();
        let detached = default();
        Self {logger,all,detached}
    }

    pub fn with_borrow_mut<F,T>(&self, id:&EdgeId, mut f:F) -> Option<T>
        where F:FnMut(&mut Edge)->T {
        match self.all.raw.borrow_mut().get_mut(id) {
            Some(t) => Some(f(t)),
            None    => {
                warning!(self.logger, "Skipping invalid edge id request ({id}).");
                None
            }
        }
    }

    pub fn with_borrow<F,T>(&self, id:&EdgeId, f:F) -> Option<T>
        where F:Fn(&Edge)->T {
        match self.all.raw.borrow().get(id) {
            Some(t) => Some(f(t)),
            None    => {
                warning!(self.logger, "Skipping invalid edge id request ({id}).");
                None
            }
        }
    }

    pub fn get_view(&self, id:&EdgeId) -> Option<EdgeView> {
        self.with_borrow(id, |t| t.view.clone_ref())
    }

    pub fn insert(&self, edge:Edge) {
        self.all.insert(edge.id(),edge);
    }
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
    pub nodes : TouchNetwork::<NodeId>,
    pub background    : TouchNetwork::<()>,
}

impl TouchState {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        let nodes = TouchNetwork::<NodeId>::new(&network,mouse);
        let background    = TouchNetwork::<()>::new(&network,mouse);
        Self {nodes,background}
    }
}




// =========================
// === GraphEditorModel ===
// =========================

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorModel {
    pub logger         : Logger,
    pub display_object : display::object::Instance,
    pub scene          : Scene,
    pub cursor         : Cursor,
    pub nodes          : Nodes,
    pub edges          : Edges,
    frp                : FrpInputs,
}

impl GraphEditorModel {
    pub fn new<S:Into<Scene>>(scene:S, cursor:Cursor) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("GraphEditor");
        let display_object = display::object::Instance::new(logger.clone());
        let nodes          = Nodes::new(&logger);
        let edges          = default();
        let frp            = default();
        Self {logger,display_object,scene,cursor,nodes,edges,frp}
    }

    pub fn add_node(&self) -> NodeId {
        let view = NodeView::new(&self.scene);
        let node = Node::new(view);
        let id   = node.id();
        self.nodes.insert(id,node);
        self.frp.register_node.emit(id);
        id
    }

//    pub fn get_node(&self, id:&NodeId) -> Option<Node> {
//        self.nodes.get(id)
//    }

    pub fn refresh_edge_target_position(&self, edge_id:&EdgeId) {
        let info = self.edges.raw.borrow().get(edge_id).and_then(|edge| {
            edge.target.as_ref().and_then(|edge_target| {
                self.nodes.with_borrow(&edge_target.node_id, |node| {
                    let offset = node.view.ports.get_port_offset(&edge_target.port).unwrap_or(Vector2::new(0.0,0.0));
                    let node_position = node.view.position();
                    let pos = frp::Position::new(node_position.x + offset.x, node_position.y + offset.y);
                    (edge.view.clone_ref(),pos)
                })
            })
        });
        if let Some((view,pos)) = info { view.events.target_position.emit(pos) }
    }

    #[deprecated(note="Use add_node instead.")]
    pub fn deprecated_add_node(&self) -> WeakNodeView {
        let view = NodeView::new(&self.scene);
        let weak = view.downgrade();
        let node = Node::new(view);
        let id   = node.id();
        self.nodes.insert(id,node);
        self.frp.register_node.emit(id);
        weak
    }

    #[deprecated(note="Use FRP remove_node instead.")]
    pub fn deprecated_remove_node(&self, node:WeakNodeView) {
        if let Some(node) = node.upgrade() {
            self.nodes.remove(&node.id().into());
        }
    }
}

impl display::Object for GraphEditorModel {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}


// ===================
// === GraphEditor ===
// ===================

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditor {
    pub model : GraphEditorModel,
    pub frp   : Frp,
}

impl Deref for GraphEditor {
    type Target = GraphEditorModel;
    fn deref(&self) -> &Self::Target {
        &self.model
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
        let scene  = world.scene();
        let cursor = Cursor::new(world.scene());
        web::body().set_style_or_panic("cursor","none");
        world.add_child(&cursor);

        let model          = GraphEditorModel::new(scene,cursor.clone_ref());
        let display_object = &model.display_object;
        let nodes          = &model.nodes;
        let edges          = &model.edges;
        let inputs         = &model.frp;
        let mouse          = &scene.mouse.frp;
        let network        = &inputs.network;
        let touch          = TouchState::new(&network,mouse);

        frp::extend! { network


        // === Selection Target Redirection ===

        def mouse_down_target  = mouse.press.map(f_!((model) model.scene.mouse.target.get()));
        def _perform_selection = mouse_down_target.map(f!((touch,model)(target) {
            match target {
                display::scene::Target::Background  => touch.background.down.emit(()),
                display::scene::Target::Symbol {..} => {
                    if let Some(target) = model.scene.shapes.get_mouse_target(*target) {
                        target.mouse_down().emit(());
                    }
                }
            }
        }));


        // === Cursor Selection ===

        def mouse_on_down_position = mouse.position.sample(&mouse.press);
        def selection_zero         = source::<Position>();
        def selection_size_down    = mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
        def selection_size_if_down = selection_size_down.gate(&touch.background.is_down);
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


        // === Selection ===

        def deselect_all_nodes  = inputs.deselect_all_nodes.merge(&touch.background.selected);
        def _deselect_all_nodes = deselect_all_nodes.map(f_!((model) model.nodes.selected.clear()));
        def select_node         = inputs.select_node.merge(&touch.nodes.selected);
        def _select_node        = select_node.map(f!((model)(node_id) {
            model.nodes.selected.clear();
            model.nodes.selected.insert(*node_id);
            model.nodes.get_view(node_id).for_each(|view| view.frp.select.emit(()));
        }));


        // === Connect Nodes ===

        def node_port_press   = source::<(NodeId,span_tree::Crumbs)>();
        def edge_target_press = node_port_press.map(|(id,port)| EdgeTarget::new(*id,port.clone()));
        def edge_target       = edge_target_press.merge(&inputs.connect_detached_edges_to_node);
        def _connect_detached = edge_target.map(f!((model)(target) {
            model.nodes.with_borrow_mut(&target.node_id,|node| {
                for edge_id in model.edges.detached.mem_take() {
                    model.edges.with_borrow_mut(&edge_id, |edge| {
                        edge.target = Some(EdgeTarget::new(target.node_id,target.port.clone()));
                        node.in_edges.insert(edge_id); 
                    });
                }
            })
        }));

        def _foo = inputs.connect_nodes.map(f!((model)((source,target)){
            let view = EdgeView::new(&model.scene);
            if let Some(source_node) = model.nodes.all.raw.borrow().get(&source.node_id) {
                view.mod_position(|p| p.x = source_node.position().x + node::NODE_WIDTH/2.0);
                view.mod_position(|p| p.y = source_node.position().y + node::NODE_HEIGHT/2.0);
            }
            model.add_child(&view);
            let mut edge = Edge::new_with_source(view,source.node_id);
            let edge_id = edge.id().into();
            edge.target = Some(target.clone());

            model.edges.insert(edge);

            if let Some(target_node) = model.nodes.all.raw.borrow_mut().get_mut(&target.node_id) {
                target_node.in_edges.insert(edge_id);
            }
        }));


        // === Add NodeView ===

        def add_node_at_cursor_pos = inputs.add_node_at_cursor.map2(&mouse.position,|_,p|{*p});
        def add_node_at            = inputs.add_node_at.merge(&add_node_at_cursor_pos);
        def _add_new_node_at       = add_node_at.map(f!((model)(pos) {
            let node_id = model.add_node();
            model.frp.set_node_position.emit((node_id,*pos));
        }));


        def _new_node = inputs.register_node.map(f!((cursor,network,nodes,edges,touch,display_object,scene,node_port_press,model)(node_id) {
            let node_view = {
                let borrow = model.nodes.all.raw.borrow();
                let node = borrow.get(node_id).unwrap();
                display_object.add_child(node);
                node.view.clone_ref()
            };
                frp::new_bridge_network! { [network,node_view.main_area.events.network]
                    let node_id = node_id.clone(); // FIXME: why?
                    def _node_on_down_tagged = node_view.drag_area.events.mouse_down.map(f_!((touch) {
                        touch.nodes.down.emit(node_id)
                    }));
                    def cursor_mode = node_view.ports.frp.cursor_mode.map(f!((cursor)(mode) {
                        cursor.frp.set_mode.emit(mode);
                    }));
                    def _add_connection = node_view.frp.output_ports.mouse_down.map(f_!((model) {
                        let node_view = model.nodes.all.raw.borrow().get(&node_id).unwrap().view.clone_ref();
                            let view = EdgeView::new(&model.scene);
                            view.mod_position(|p| p.x = node_view.position().x + node::NODE_WIDTH/2.0);
                            view.mod_position(|p| p.y = node_view.position().y + node::NODE_HEIGHT/2.0);
                            model.add_child(&view);
                            let edge = Edge::new_with_source(view,node_id);
                            let id = edge.id();
                            model.edges.insert(edge);
                            model.edges.detached.insert(id);

                    }));

                    def _foo = node_view.ports.frp.press.map(f!((node_port_press)(crumbs){
                        node_port_press.emit((node_id,crumbs.clone()));
                    }));
                }


//            })
        }));

        // === Set Node Position ===

        def _set_node_position = inputs.set_node_position.map(f!((nodes)((node_id,position)){
            if let Some(node) = nodes.all.raw.borrow().get(node_id) {
                node.view.mod_position(|t| {
                    t.x = position.x;
                    t.y = position.y;
                })
            }
        }));


        // === Remove Node ===

        def _remove_all      = inputs.remove_all_nodes.map(f!((nodes)(()) nodes.clear()));
        def _remove_selected = inputs.remove_selected_nodes.map(f!((nodes,nodes)(_) {
            nodes.selected.mem_take().into_iter().for_each(|node_id| {nodes.remove(&node_id);})
        }));


        // === Set NodeView Expression ===

        def _set_node_expr = inputs.set_node_expression.map(f!((nodes)((node_id,expression)){
            if let Some(node) = nodes.all.raw.borrow().get(node_id) {
                node.view.ports.set_expression(expression);
            }
        }));


        // === Move Nodes ===

        def mouse_tx_if_node_pressed = mouse.translation.gate(&touch.nodes.is_down);
        def _move_node_with_mouse    = mouse_tx_if_node_pressed.map2(&touch.nodes.down,f!((model,nodes,edges)(tx,node_id) {
            if let Some(node) = nodes.get_cloned(&node_id) {
                node.view.mod_position(|p| { p.x += tx.x; p.y += tx.y; });
                for edge_id in &node.in_edges {
                    model.refresh_edge_target_position(edge_id);
                }
            }
        }));

        def _move_selected_nodes = inputs.translate_selected_nodes.map(f!((nodes)(t) {
            for node_id in &*nodes.selected.raw.borrow() {
                if let Some(node) = nodes.get_cloned(node_id) {
                    node.mod_position(|p| {
                        p.x += t.x;
                        p.y += t.y;
                    })
                }
            }
        }));


        // === Move Edges ===

        def _move_connections = cursor.frp.position.map(f!((edges)(position) {
            edges.detached.for_each(|id| {
                edges.get_view(id).for_each(|view| view.events.target_position.emit(position))
            })
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



        let inputs = inputs.clone_ref();
        let frp = Frp {inputs,status,node_release};

        Self {model,frp}
    }


}

impl display::Object for GraphEditor {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
