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

    pub fn remove(&self, t:&T) -> bool {
        self.raw.borrow_mut().remove(t)
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

    pub fn replace_with(&self, t:HashSet<T,S>) {
        *self.raw.borrow_mut() = t;
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

    pub fn get_cloned_ref(&self, k:&K) -> Option<V>
    where V:CloneRef {
        self.raw.borrow().get(k).map(|t| t.clone_ref())
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

    pub fn keys(&self) -> Vec<K>
    where K:Clone {
        self.raw.borrow().keys().cloned().collect_vec()
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
    pub connect_edge_source            : frp::Source<(EdgeId,EdgeTarget)>,
    pub connect_edge_target            : frp::Source<(EdgeId,EdgeTarget)>,
    pub connect_nodes                  : frp::Source<(EdgeTarget,EdgeTarget)>,
    pub deselect_all_nodes             : frp::Source,
    pub press_node_port                : frp::Source<(NodeId,span_tree::Crumbs)>,
    pub remove_edge                    : frp::Source<EdgeId>,
    pub select_node                    : frp::Source<NodeId>,
    pub set_node_expression            : frp::Source<(NodeId,Expression)>,
    pub set_node_position              : frp::Source<(NodeId,Position)>,
    pub translate_selected_nodes       : frp::Source<Position>,
}

impl FrpInputs {
    pub fn new() -> Self {
        frp::new_network! { network
            def add_node_at                    = source();
            def connect_detached_edges_to_node = source();
            def connect_edge_source            = source();
            def connect_edge_target            = source();
            def connect_nodes                  = source();
            def deselect_all_nodes             = source();
            def press_node_port                = source();
            def remove_edge                    = source();
            def select_node                    = source();
            def set_node_expression            = source();
            def set_node_position              = source();
            def translate_selected_nodes       = source();
        }
        let commands = Commands::new(&network);
        Self {commands,network,remove_edge,press_node_port,connect_detached_edges_to_node,connect_edge_source,connect_edge_target,add_node_at,set_node_position,select_node,translate_selected_nodes,set_node_expression,connect_nodes,deselect_all_nodes}
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

#[derive(Clone,CloneRef,Debug)]
pub struct Node {
    pub view      : NodeView,
    pub in_edges  : SharedHashSet<EdgeId>,
    pub out_edges : SharedHashSet<EdgeId>,
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

#[derive(Clone,CloneRef,Debug)]
pub struct Edge {
    pub view : EdgeView,
    source   : Rc<RefCell<Option<EdgeTarget>>>,
    target   : Rc<RefCell<Option<EdgeTarget>>>,
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
        let port   = default();
        let source = EdgeTarget::new(node_id,port);
        let source = Rc::new(RefCell::new(Some(source)));
        let target = default();
        Self {view,source,target}
    }

    pub fn id(&self) -> EdgeId {
        self.view.id().into()
    }

    pub fn target(&self) -> Option<EdgeTarget> {
        self.target.borrow().as_ref().map(|t| t.clone_ref())
    }

    pub fn source(&self) -> Option<EdgeTarget> {
        self.source.borrow().as_ref().map(|t| t.clone_ref())
    }

    pub fn set_source(&self, source:EdgeTarget) {
        *self.source.borrow_mut() = Some(source)
    }

    pub fn set_target(&self, target:EdgeTarget) {
        *self.target.borrow_mut() = Some(target)
    }

    pub fn take_source(&self) -> Option<EdgeTarget> {
        mem::take(&mut *self.source.borrow_mut())
    }

    pub fn take_target(&self) -> Option<EdgeTarget> {
        mem::take(&mut *self.target.borrow_mut())
    }
}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object()
    }
}


// ==================
// === EdgeTarget ===
// ==================

#[derive(Clone,CloneRef,Debug,Default)]
pub struct EdgeTarget {
    node_id : Rc<Cell<NodeId>>,
    port    : Rc<RefCell<span_tree::Crumbs>>,
}

impl EdgeTarget {
    pub fn new(node_id:NodeId, port:span_tree::Crumbs) -> Self {
        let node_id = Rc::new(Cell::new(node_id));
        let port    = Rc::new(RefCell::new(port));
        Self {node_id,port}
    }

    pub fn new_without_port(node_id:NodeId) -> Self {
        Self::new(node_id,default())
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id.get()
    }

    pub fn port(&self) -> span_tree::Crumbs {
        self.port.borrow().clone()
    }

    pub fn deep_clone(&self) -> Self {
        let node_id = self.node_id();
        let port    = self.port();
        Self::new(node_id,port)
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

    pub fn with<F>(&self, id:&NodeId, f:F)
    where F:Fn(Node) {
        match self.all.get_cloned_ref(id) {
            Some(t) => f(t),
            None    => warning!(self.logger, "Skipping invalid node id request ({id})."),
        }
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

    pub fn with<F>(&self, id:&EdgeId, f:F)
    where F:Fn(Edge) {
        match self.all.get_cloned_ref(id) {
            Some(t) => f(t),
            None    => warning!(self.logger, "Skipping invalid node id request ({id})."),
        }
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
    pub nodes      : TouchNetwork::<NodeId>,
    pub background : TouchNetwork::<()>,
}

impl TouchState {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        let nodes = TouchNetwork::<NodeId>::new(&network,mouse);
        let background    = TouchNetwork::<()>::new(&network,mouse);
        Self {nodes,background}
    }
}



pub fn is_sub_crumb_of(src:&span_tree::Crumbs, tgt:&span_tree::Crumbs) -> bool {
    if src.len() < tgt.len() { return false }
    for (s,t) in src.iter().zip(tgt.iter()) {
        if s != t { return false }
    }
    return true
}

pub fn crumbs_overlap(src:&span_tree::Crumbs, tgt:&span_tree::Crumbs) -> bool {
    is_sub_crumb_of(src,tgt) || is_sub_crumb_of(tgt,src)
}




// ========================
// === GraphEditorModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorModel {
    pub logger         : Logger,
    pub display_object : display::object::Instance,
    pub scene          : Scene,
    pub cursor         : Cursor,
    pub nodes          : Nodes,
    pub edges          : Edges,
    touch_state        : TouchState,
    frp                : FrpInputs,
}

// === Public ===

impl GraphEditorModel {
    pub fn new<S: Into<Scene>>(scene: S, cursor: Cursor) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("GraphEditor");
        let display_object = display::object::Instance::new(logger.clone());
        let nodes          = Nodes::new(&logger);
        let edges          = default();
        let frp            = FrpInputs::default();
        let touch_state    = TouchState::new(&frp.network,&scene.mouse.frp);
        Self {logger,display_object,scene,cursor,nodes,edges,touch_state,frp }
    }

    pub fn add_node(&self) -> NodeId {
        let view = NodeView::new(&self.scene);
        let node = Node::new(view);
        let node_id = node.id();
        self.add_child(&node);


        let cursor = &self.cursor;
        let touch  = &self.touch_state;
        let model  = self; // FIXME : cyclic dep. Remove network out of model.

        frp::new_bridge_network! { [self.frp.network, node.view.main_area.events.network]
            def _node_on_down_tagged = node.view.drag_area.events.mouse_down.map(f_!((touch) {
                touch.nodes.down.emit(node_id)
            }));
            def cursor_mode = node.view.ports.frp.cursor_mode.map(f!((cursor)(mode) {
                cursor.frp.set_mode.emit(mode);
            }));
            def _add_connection = node.view.frp.output_ports.mouse_down.map(f_!((model) {
                model.nodes.with(&node_id, |node| {
                    let view = EdgeView::new(&model.scene);
                    view.mod_position(|p| p.x = node.view.position().x + node::NODE_WIDTH/2.0);
                    view.mod_position(|p| p.y = node.view.position().y + node::NODE_HEIGHT/2.0);
                    model.add_child(&view);
                    let edge = Edge::new_with_source(view,node_id);
                    let edge_id = edge.id();
                    model.edges.insert(edge);
                    model.edges.detached.insert(edge_id);
                    node.out_edges.insert(edge_id);
                })
            }));

            def _press_node_port = node.view.ports.frp.press.map(f!((model)(crumbs){
                model.frp.press_node_port.emit((node_id,crumbs.clone()));
            }));
        }


        self.nodes.insert(node_id,node);

        node_id
    }

    pub fn add_node_at(&self, position:&Position) -> NodeId {
        let node_id = self.add_node();
        self.set_node_position(&node_id,position);
        node_id
    }

    #[deprecated(note="Use add_node instead.")]
    pub fn deprecated_add_node(&self) -> WeakNodeView {
        let node_id = self.add_node();
        let node    = self.nodes.get_cloned_ref(&node_id).unwrap();
        let weak    = node.view.downgrade();
        weak
    }

    #[deprecated(note="Use FRP remove_node instead.")]
    pub fn deprecated_remove_node(&self, node:WeakNodeView) {
        if let Some(node) = node.upgrade() {
            self.nodes.remove(&node.id().into());
        }
    }
}


// === Construction ===

impl GraphEditorModel {
    fn add_edge(&self) -> EdgeId {
        let view = EdgeView::new(&self.scene);
        let edge = Edge::new(view);
        let id   = edge.id();
        self.edges.insert(edge);
        // self.frp.register_edge.emit(id);
        id
    }
}


// === Selection ===

impl GraphEditorModel {
    fn select_node(&self, node_id:&NodeId) {
        self.deselect_all_nodes();
        if let Some(node) = self.nodes.get_cloned_ref(node_id) {
            self.nodes.selected.insert(*node_id);
            node.view.frp.select.emit(());
        }
    }

    fn deselect_all_nodes(&self) {
        for node_id in &self.nodes.selected.mem_take() {
            if let Some(node) = self.nodes.get_cloned_ref(node_id) {
                node.view.frp.deselect.emit(());
            }
        }
    }
}


// === Remove ===

impl GraphEditorModel {
    fn remove_edge(&self, edge_id:&EdgeId) {
        if let Some(edge) = self.edges.remove(edge_id) {
            if let Some(source) = edge.take_source() {
                if let Some(source_node) = self.nodes.get_cloned_ref(&source.node_id()) {
                    source_node.out_edges.remove(edge_id);
                }
            }

            if let Some(target) = edge.take_target() {
                if let Some(target_node) = self.nodes.get_cloned_ref(&target.node_id()) {
                    target_node.in_edges.remove(edge_id);
                }
            }
        }
    }

    fn remove_node(&self, node_id:&NodeId) {
        if let Some(node) = self.nodes.remove(node_id) {
            for edge_id in node.in_edges  . mem_take() { self.remove_edge(&edge_id); }
            for edge_id in node.out_edges . mem_take() { self.remove_edge(&edge_id); }
        }
    }

    fn remove_all_nodes(&self) {
        for node_id in &self.nodes.keys() {
            self.remove_node(node_id)
        }
    }

    fn remove_selected_nodes(&self) {
        for node_id in self.nodes.selected.mem_take() {
            self.remove_node(&node_id);
        }
    }
}


// === Connect ===

impl GraphEditorModel {
    fn connect_detached_edges_to_node(&self, target:&EdgeTarget) {
        if let Some(node) = self.nodes.get_cloned_ref(&target.node_id()) {
            for edge_id in self.edges.detached.mem_take() {
                self.connect_edge_target(&edge_id,target);
            }
        }
    }

    fn connect_edge_source(&self, edge_id:&EdgeId, target:&EdgeTarget) {
        if let Some(edge) = self.edges.get_cloned_ref(edge_id) {
            if let Some(old_source) = edge.take_source() {
                if let Some(node) = self.nodes.get_cloned_ref(&old_source.node_id()) {
                    node.out_edges.remove(edge_id);
                }
            }

            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id()) {
                node.out_edges.insert(*edge_id);
            }

            edge.set_source(target.deep_clone());
            self.refresh_edge_position(&edge_id);
        }
    }

    fn connect_edge_target(&self, edge_id:&EdgeId, target:&EdgeTarget) {
        if let Some(edge) = self.edges.get_cloned_ref(edge_id) {
            if let Some(old_target) = edge.take_target() {
                if let Some(node) = self.nodes.get_cloned_ref(&old_target.node_id()) {
                    node.in_edges.remove(edge_id);
                }
            }

            let target_port = target.port();
            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id()) {
                let mut overlapping = vec![];
                for edge_id in node.in_edges.raw.borrow().clone().into_iter() {
                    if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
                        if let Some(edge_target) = edge.target() {
                            if crumbs_overlap(&edge_target.port(),&target_port) {
                                overlapping.push(edge_id);
                            }
                        }
                    }
                }
                for edge_id in &overlapping {
                    self.remove_edge(edge_id)
                }
                node.in_edges.insert(*edge_id);
            };

            edge.set_target(target.deep_clone());
            self.refresh_edge_position(&edge_id);
        }
    }

    fn connect_nodes(&self, source:&EdgeTarget, target:&EdgeTarget) {
        let edge = Edge::new(EdgeView::new(&self.scene));
        self.add_child(&edge);
        self.edges.insert(edge.clone_ref());

        let edge_id = edge.id();
        self.connect_edge_source(&edge_id,source);
        self.connect_edge_target(&edge_id,target);
    }

}


// === Position ===

impl GraphEditorModel {
    pub fn set_node_position(&self, node_id:&NodeId, position:&Position) {
        if let Some(node) = self.nodes.get_cloned_ref(node_id) {
            node.view.mod_position(|t| {
                t.x = position.x;
                t.y = position.y;
            })
        }
    }

    pub fn refresh_edge_position(&self, edge_id:&EdgeId) {
        self.refresh_edge_source_position(edge_id);
        self.refresh_edge_target_position(edge_id);
    }

    pub fn refresh_edge_source_position(&self, edge_id:&EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(edge_id) {
            if let Some(edge_source) = edge.source() {
                self.nodes.with(&edge_source.node_id(), |node| {
                    edge.mod_position(|p| {
                        p.x = node.position().x + node::NODE_WIDTH/2.0;
                        p.y = node.position().y + node::NODE_HEIGHT/2.0;
                    });
                })
            }
        };
    }

    pub fn refresh_edge_target_position(&self, edge_id:&EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(edge_id) {
            if let Some(edge_target) = edge.target() {
                self.nodes.with(&edge_target.node_id(), |node| {
                    let offset = node.view.ports.get_port_offset(&edge_target.port.borrow()).unwrap_or(Vector2::new(0.0,0.0));
                    let node_position = node.view.position();
                    let pos = frp::Position::new(node_position.x + offset.x, node_position.y + offset.y);
                    edge.view.events.target_position.emit(pos);
                })
            }
        };
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


macro_rules! model_bind {
    ($network:ident $model:ident . $name:ident($($arg:ident),*)) => {
        frp::extend! { $network
            def _eval = $name.map(f!(($model)(($($arg),*)) $model.$name($($arg),*)));
        }
    };
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
        let touch          = &model.touch_state;

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

        def select_node        = inputs.select_node.merge(&touch.nodes.selected);
        def deselect_all_nodes = inputs.deselect_all_nodes.merge(&touch.background.selected);
        model_bind!(network model.select_node(node_id));
        model_bind!(network model.deselect_all_nodes());


        // === Connect Nodes ===

        def edge_target_press = inputs.press_node_port.map(|(id,port)| EdgeTarget::new(*id,port.clone()));
        def connect_detached_edges_to_node = edge_target_press.merge(&inputs.connect_detached_edges_to_node);
        model_bind!(network model.connect_detached_edges_to_node(target));

        let connect_edge_source = inputs.connect_edge_source.clone_ref();
        let connect_edge_target = inputs.connect_edge_target.clone_ref();
        let connect_nodes       = inputs.connect_nodes.clone_ref();
        model_bind!(network model.connect_edge_source(edge_id,target));
        model_bind!(network model.connect_edge_target(edge_id,target));
        model_bind!(network model.connect_nodes(source,target));


        // === Add NodeView ===

        def add_node_at_cursor = inputs.add_node_at_cursor.map2(&mouse.position,|_,p|{*p});
        def add_node_at        = inputs.add_node_at.merge(&add_node_at_cursor);
        model_bind!(network model.add_node_at(position));


        // === Set Node Position ===

        let set_node_position = inputs.set_node_position.clone_ref();
        model_bind!(network model.set_node_position(node_id,position));


        // === Remove Node ===

        let remove_all_nodes      = inputs.remove_all_nodes.clone_ref();
        let remove_selected_nodes = inputs.remove_selected_nodes.clone_ref();
        let remove_edge           = inputs.remove_edge.clone_ref();
        model_bind!(network model.remove_all_nodes());
        model_bind!(network model.remove_selected_nodes());
        model_bind!(network model.remove_edge(edge_id));


        // === Set NodeView Expression ===

        def _set_node_expr = inputs.set_node_expression.map(f!((nodes)((node_id,expression)){
            if let Some(node) = nodes.all.raw.borrow().get(node_id) {
                node.view.ports.set_expression(expression);
            }
        }));


        // === Move Nodes ===

        def mouse_tx_if_node_pressed = mouse.translation.gate(&touch.nodes.is_down);
        def _move_node_with_mouse    = mouse_tx_if_node_pressed.map2(&touch.nodes.down,f!((model,nodes,edges)(tx,node_id) {
            if let Some(node) = nodes.get_cloned_ref(&node_id) {
                node.view.mod_position(|p| { p.x += tx.x; p.y += tx.y; });
                for edge_id in &node.in_edges.raw.borrow().clone() {
                    model.refresh_edge_target_position(edge_id);
                }
                for edge_id in &node.out_edges.raw.borrow().clone() {
                    model.refresh_edge_position(edge_id);
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
                edges.with(id,|edge| edge.view.events.target_position.emit(position))
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

        let node_release = touch.nodes.up.clone_ref();



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
