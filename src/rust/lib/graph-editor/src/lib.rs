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

#![recursion_limit="1024"]


#[warn(missing_docs)]
pub mod component;

/// Common types and functions usable in all modules of this crate.
pub mod prelude {
    pub use ensogl::prelude::*;
}

use ensogl::application;
use ensogl::prelude::*;
use ensogl::traits::*;
use ensogl::application::shortcut;

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
use crate::component::visualization::Visualization;
use crate::component::visualization;

use serde_json::json;
use enso_frp::stream::ValueProvider;


fn first  <T0:Clone,T1>(t:&(T0,T1)) -> T0 { t.0.clone() }
fn second <T0,T1:Clone>(t:&(T0,T1)) -> T1 { t.1.clone() }


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

    pub fn keys(&self) -> Vec<T>
    where T:Clone {
        self.raw.borrow().iter().cloned().collect_vec()
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
    pub outputs : FrpOutputs,
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
    /// Add a new node and place it in the origin of the workspace.
    add_node,
    /// Add a new node and place it at the mouse cursor position.
    add_node_at_cursor,
    /// Remove all selected nodes from the graph.
    remove_selected_nodes,
    /// Remove all nodes from the graph.
    remove_all_nodes,
    /// Toggle the visibility of the selected visualisations
    toggle_visualization_visibility,
    /// Set the data for the selected nodes. // TODO only has dummy functionality at the moment.
    debug_set_data_for_selected_node,


    /// Enables nodes merge selection mode.
    enable_node_merge_select,
    /// Disables nodes merge selection mode.
    disable_node_merge_select,
    /// Toggles nodes merge selection mode.
    toggle_node_merge_select,


    /// Enables nodes subtract selection mode.
    enable_node_subtract_select,
    /// Disables nodes subtract selection mode.
    disable_node_subtract_select,
    /// Toggles nodes subtract selection mode.
    toggle_node_subtract_select,


    /// Enables nodes inverse selection mode.
    enable_node_inverse_select,
    /// Disables nodes inverse selection mode.
    disable_node_inverse_select,
    /// Toggles nodes inverse selection mode.
    toggle_node_inverse_select,
}

impl Commands {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def add_node                         = source();
            def add_node_at_cursor               = source();
            def remove_selected_nodes            = source();
            def remove_all_nodes                 = source();
            def toggle_visualization_visibility  = source();
            def debug_set_data_for_selected_node = source();

            def enable_node_merge_select         = source();
            def disable_node_merge_select        = source();
            def toggle_node_merge_select         = source();

            def enable_node_subtract_select      = source();
            def disable_node_subtract_select     = source();
            def toggle_node_subtract_select      = source();

            def enable_node_inverse_select       = source();
            def disable_node_inverse_select      = source();
            def toggle_node_inverse_select       = source();
        }
        Self {add_node,add_node_at_cursor,remove_selected_nodes,remove_all_nodes
             ,toggle_visualization_visibility,debug_set_data_for_selected_node
             ,enable_node_merge_select,disable_node_merge_select,toggle_node_merge_select
             ,enable_node_subtract_select,disable_node_subtract_select,toggle_node_subtract_select
             ,enable_node_inverse_select,disable_node_inverse_select,toggle_node_inverse_select}
    }
}



// =================
// === FrpInputs ===
// =================

#[derive(Debug,Clone,CloneRef,Shrinkwrap)]
pub struct FrpInputs {
    #[shrinkwrap(main_field)]
    commands                           : Commands,
    pub connect_detached_edges_to_node : frp::Source<EdgeTarget>,
    pub connect_edge_source            : frp::Source<(EdgeId,EdgeTarget)>,
    pub connect_edge_target            : frp::Source<(EdgeId,EdgeTarget)>,
    pub connect_nodes                  : frp::Source<(EdgeTarget,EdgeTarget)>,
    pub deselect_all_nodes             : frp::Source,
    pub press_node_input               : frp::Source<EdgeTarget>,
    pub remove_all_node_edges          : frp::Source<NodeId>,
    pub remove_all_node_input_edges    : frp::Source<NodeId>,
    pub remove_all_node_output_edges   : frp::Source<NodeId>,
    pub remove_edge                    : frp::Source<EdgeId>,
    pub select_node                    : frp::Source<NodeId>,
    pub set_node_expression            : frp::Source<(NodeId,Expression)>,
    pub set_node_position              : frp::Source<(NodeId,Position)>,
    pub set_visualization_data         : frp::Source<NodeId>,
    pub translate_selected_nodes       : frp::Source<Position>,
}

impl FrpInputs {
    pub fn new(network:&frp::Network) -> Self {
        frp::extend! { network
            def connect_detached_edges_to_node = source();
            def connect_edge_source            = source();
            def connect_edge_target            = source();
            def connect_nodes                  = source();
            def deselect_all_nodes             = source();
            def press_node_input               = source();
            def remove_all_node_edges          = source();
            def remove_all_node_input_edges    = source();
            def remove_all_node_output_edges   = source();
            def remove_edge                    = source();
            def select_node                    = source();
            def set_node_expression            = source();
            def set_node_position              = source();
            def set_visualization_data         = source();
            def translate_selected_nodes       = source();
        }
        let commands = Commands::new(&network);
        Self {commands,remove_edge,press_node_input,remove_all_node_edges
             ,remove_all_node_input_edges,remove_all_node_output_edges,set_visualization_data
             ,connect_detached_edges_to_node,connect_edge_source,connect_edge_target
             ,set_node_position,select_node,translate_selected_nodes,set_node_expression
             ,connect_nodes,deselect_all_nodes}
    }
}

impl application::command::FrpNetworkProvider for GraphEditor {
    fn network(&self) -> &frp::Network {
        &self.model.network
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



// ==================
// === FrpOutputs ===
// ==================

macro_rules! generate_frp_outputs {
    ($( $field:ident : $field_ty:ty ),* $(,)?) => {
        #[derive(Debug,Clone,CloneRef)]
        pub struct UnsealedFrpOutputs {
            network : frp::Network,
            $($field : frp::Merge<$field_ty>),*
        }

        #[derive(Debug,Clone,CloneRef)]
        pub struct FrpOutputs {
            network : frp::Network,
            $(pub $field : frp::Sampler<$field_ty>),*
        }

        #[allow(clippy::new_without_default)]
        impl UnsealedFrpOutputs {
            pub fn new() -> Self {
                frp::new_network! { network
                    $(def $field = gather();)*
                }
                Self {network, $($field),*}
            }

            pub fn seal(&self) -> FrpOutputs {
                let network = self.network.clone_ref();
                frp::extend! { TRACE_ALL network
                    $(def $field = self.$field.sampler();)*
                }
                FrpOutputs {network, $($field),*}
            }
        }
    };
}


generate_frp_outputs! {
    node_added        : NodeId,
    edge_added        : EdgeId,
    node_removed      : NodeId,
    edge_removed      : EdgeId,
    node_selected     : NodeId,
    node_deselected   : NodeId,
    node_position_set : (NodeId,Position),
    edge_source_set   : (EdgeId,EdgeTarget),
    edge_target_set   : (EdgeId,EdgeTarget),
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

#[derive(Clone,CloneRef,Copy,Debug,Default,Display,Eq,From,Hash,Into,PartialEq)]
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

#[derive(Clone,CloneRef,Copy,Debug,Default,Display,Eq,From,Hash,Into,PartialEq)]
pub struct EdgeId(pub Id);

impl Edge {
    pub fn new(view:EdgeView) -> Self {
        let source = default();
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
    pub node_id : NodeId,
    pub port    : Rc<span_tree::Crumbs>,
}

impl EdgeTarget {
    pub fn new(node_id:NodeId, port:span_tree::Crumbs) -> Self {
        let port = Rc::new(port);
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
}





#[derive(Debug,Clone,CloneRef,Default)]
pub struct Edges {
    pub logger          : Logger,
    pub all             : SharedHashMap<EdgeId,Edge>,
    pub detached_source : SharedHashSet<EdgeId>,
    pub detached_target : SharedHashSet<EdgeId>,
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
        let detached_source = default();
        let detached_target = default();
        Self {logger,all,detached_source,detached_target}
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



pub fn is_sub_crumb_of(src:&[span_tree::Crumb], tgt:&[span_tree::Crumb]) -> bool {
    if src.len() < tgt.len() { return false }
    for (s,t) in src.iter().zip(tgt.iter()) {
        if s != t { return false }
    }
    true
}

pub fn crumbs_overlap(src:&[span_tree::Crumb], tgt:&[span_tree::Crumb]) -> bool {
    is_sub_crumb_of(src,tgt) || is_sub_crumb_of(tgt,src)
}




// ===================================
// === GraphEditorModelWithNetwork ===
// ===================================

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorModelWithNetwork {
    pub model   : GraphEditorModel,
    pub network : frp::Network,
}

impl Deref for GraphEditorModelWithNetwork {
    type Target = GraphEditorModel;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl GraphEditorModelWithNetwork {
    pub fn new<S:Into<Scene>>(scene:S, cursor:Cursor) -> Self {
        let network = frp::Network::new();
        let model   = GraphEditorModel::new(scene,cursor,&network);
        Self {model,network}
    }

    fn new_node(&self, outputs:&UnsealedFrpOutputs) -> NodeId {
        let view = NodeView::new(&self.scene);
        let node = Node::new(view);
        let node_id = node.id();
        self.add_child(&node);


        let cursor = &self.cursor;
        let touch  = &self.touch_state;
        let model  = &self.model;

        frp::new_bridge_network! { [self.network, node.view.main_area.events.network]
            def _node_on_down_tagged = node.view.drag_area.events.mouse_down.map(f_!(
                touch.nodes.down.emit(node_id)
            ));
            def _cursor_mode = node.view.ports.frp.cursor_mode.map(f!((mode)
                cursor.frp.set_mode.emit(mode)
            ));
            def edge_id = node.view.frp.output_ports.mouse_down.map(f_!([model] {
                if let Some(node) = model.nodes.get_cloned_ref(&node_id) {
                    let view = EdgeView::new(&model.scene);
                    model.add_child(&view);
                    let edge = Edge::new(view);
                    let edge_id = edge.id();
                    model.edges.insert(edge);
                    model.edges.detached_target.insert(edge_id);
                    node.out_edges.insert(edge_id);
                    edge_id
                } else { default() }
            }));

            outputs.edge_added.attach(&edge_id);
            def new_edge_source = edge_id.map(move |id| (*id,EdgeTarget::new(node_id,default())));
            outputs.edge_source_set.attach(&new_edge_source);


            def _press_node_input = node.view.ports.frp.press.map(f!((crumbs)
                model.frp.press_node_input.emit(EdgeTarget::new(node_id,crumbs.clone()))
            ));
        }




        let dummy_content = visualization::default_content();
        let dom_layer    = model.scene.dom.layers.front.clone_ref();
        dom_layer.manage(&dummy_content);

        let vis:Visualization = dummy_content.into();
        node.view.frp.set_visualization.emit(Some(vis));


        self.nodes.insert(node_id,node);


        node_id
    }


    pub fn get_node_position(&self, node_id:NodeId) -> Option<Vector3<f32>> {
        self.nodes.get_cloned_ref(&node_id).map(|node| node.position())
    }

    // FIXME: remove
    pub fn deprecated_add_node(&self) -> WeakNodeView {
        todo!()
    }

    // FIXME: remove
    pub fn deprecated_remove_node(&self, node:WeakNodeView) {
        todo!()
    }
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
    pub fn new<S:Into<Scene>>(scene:S, cursor:Cursor, network:&frp::Network) -> Self {
        let scene          = scene.into();
        let logger         = Logger::new("GraphEditor");
        let display_object = display::object::Instance::new(logger.clone());
        let nodes          = Nodes::new(&logger);
        let edges          = default();
        let frp            = FrpInputs::new(network);
        let touch_state    = TouchState::new(network,&scene.mouse.frp);
        Self {logger,display_object,scene,cursor,nodes,edges,touch_state,frp }
    }

    fn new_edge(&self) -> EdgeId {
        let edge = Edge::new(EdgeView::new(&self.scene));
        self.add_child(&edge);
        self.edges.insert(edge.clone_ref());
        edge.id()
    }

    pub fn all_nodes(&self) -> Vec<NodeId> {
        self.nodes.all.keys()
    }

    pub fn selected_nodes(&self) -> Vec<NodeId> {
        self.nodes.selected.keys()
    }
}


// === Selection ===

impl GraphEditorModel {
    fn select_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            self.nodes.selected.insert(node_id);
            node.view.frp.select.emit(());
        }
    }

    fn deselect_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            self.nodes.selected.remove(&node_id);
            node.view.frp.deselect.emit(());
        }
    }
}


// === Remove ===

impl GraphEditorModel {
    fn remove_edge<E:Into<EdgeId>>(&self, edge_id:E) {
        let edge_id = edge_id.into();
        if let Some(edge) = self.edges.remove(&edge_id) {
            if let Some(source) = edge.take_source() {
                if let Some(source_node) = self.nodes.get_cloned_ref(&source.node_id) {
                    source_node.out_edges.remove(&edge_id);
                }
            }

            if let Some(target) = edge.take_target() {
                if let Some(target_node) = self.nodes.get_cloned_ref(&target.node_id) {
                    target_node.in_edges.remove(&edge_id);
                }
            }
        }
    }

    /// Warning! This function does not remove connected edges. It needs to be handled by the
    /// implementation.
    fn remove_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.remove(&node_id) {
            self.nodes.selected.remove(&node_id);
        }
    }

    fn node_in_edges(&self, node_id:impl Into<NodeId>) -> Vec<EdgeId> {
        let node_id = node_id.into();
        self.nodes.get_cloned_ref(&node_id).map(|node| {
            node.in_edges.keys()
        }).unwrap_or_default()
    }

    fn node_out_edges(&self, node_id:impl Into<NodeId>) -> Vec<EdgeId> {
        let node_id = node_id.into();
        self.nodes.get_cloned_ref(&node_id).map(|node| {
            node.out_edges.keys()
        }).unwrap_or_default()
    }

    fn node_in_and_out_edges(&self, node_id:impl Into<NodeId>) -> Vec<EdgeId> {
        let node_id = node_id.into();
        let mut edges = self.node_in_edges(node_id);
        edges.extend(&self.node_out_edges(node_id));
        edges
    }

    fn set_node_expression(&self, node_id:impl Into<NodeId>, expr:impl Into<Expression>) {
        let node_id = node_id.into();
        let expr    = expr.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.view.ports.set_expression(expr);
        }
    }
}


// === Connect ===

impl GraphEditorModel {
    fn connect_edge_source(&self, edge_id:EdgeId, target:&EdgeTarget) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
                node.out_edges.insert(edge_id);
                edge.set_source(target.clone());
                self.refresh_edge_position(edge_id);
            }
        }
    }

    fn connect_edge_target(&self, edge_id:EdgeId, target:&EdgeTarget) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
                node.in_edges.insert(edge_id);
                edge.set_target(target.clone());
                self.refresh_edge_position(edge_id);
            };
        }
    }

    fn overlapping_edges(&self, target:&EdgeTarget) -> Vec<EdgeId> {
        let mut overlapping = vec![];
        if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
            for edge_id in node.in_edges.raw.borrow().clone().into_iter() {
                if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
                    if let Some(edge_target) = edge.target() {
                        if crumbs_overlap(&edge_target.port,&target.port) {
                            overlapping.push(edge_id);
                        }
                    }
                }
            }
        }
        overlapping
    }
}


// === Position ===

impl GraphEditorModel {
    pub fn set_node_position(&self, node_id:impl Into<NodeId>, position:impl Into<Position>) {
        let node_id  = node_id.into();
        let position = position.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.view.mod_position(|t| {
                t.x = position.x;
                t.y = position.y;
            });
            for edge_id in self.node_in_and_out_edges(node_id) {
                self.refresh_edge_position(edge_id);
            }
        }
    }

    pub fn node_pos_mod
    (&self, node_id:impl Into<NodeId>, pos_diff:impl Into<Position>) -> (NodeId,Position) {
        let node_id      = node_id.into();
        let pos_diff     = pos_diff.into();
        let new_position = if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            let node_pos = node.position();
            let position = frp::Position::new(node_pos.x + pos_diff.x, node_pos.y + pos_diff.y);
            position
        } else {
            default()
        };
        (node_id,new_position)
    }

    pub fn refresh_edge_position(&self, edge_id:EdgeId) {
        self.refresh_edge_source_position(edge_id);
        self.refresh_edge_target_position(edge_id);
    }

    pub fn refresh_edge_source_position(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(edge_source) = edge.source() {
                if let Some(node) = self.nodes.get_cloned_ref(&edge_source.node_id) {
                    edge.mod_position(|p| {
                        p.x = node.position().x + node::NODE_WIDTH/2.0;
                        p.y = node.position().y + node::NODE_HEIGHT/2.0;
                    });
                }
            }
        };
    }

    pub fn refresh_edge_target_position(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(edge_target) = edge.target() {
                if let Some(node) = self.nodes.get_cloned_ref(&edge_target.node_id) {
                    let offset = node.view.ports.get_port_offset(&edge_target.port).unwrap_or_else(|| Vector2::new(0.0,0.0));
                    let node_position = node.view.position();
                    let pos = frp::Position::new(node_position.x + offset.x, node_position.y + offset.y);
                    edge.view.events.target_position.emit(pos);
                }
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
    pub model : GraphEditorModelWithNetwork,
    pub frp   : Frp,
}

impl Deref for GraphEditor {
    type Target = GraphEditorModelWithNetwork;
    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl GraphEditor {
    pub fn add_node(&self) -> NodeId {
        self.frp.add_node.emit(());
        self.frp.outputs.node_added.value()
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
        vec! [ Self::self_shortcut(shortcut::Action::press   (&[Key::Character("n".into())]) , "add_node_at_cursor")
             , Self::self_shortcut(shortcut::Action::press   (&[Key::Backspace])             , "remove_selected_nodes")
             , Self::self_shortcut(shortcut::Action::press   (&[Key::Character(" ".into())]) , "toggle_visualization_visibility")
             , Self::self_shortcut(shortcut::Action::press   (&[Key::Shift])                 , "toggle_node_merge_select")
             , Self::self_shortcut(shortcut::Action::release (&[Key::Shift])                 , "toggle_node_merge_select")
             , Self::self_shortcut(shortcut::Action::press   (&[Key::Control])               , "enable_node_merge_select")
//             , Self::self_shortcut(shortcut::Action::release (&[Key::Control])               , "disable_node_merge_select")
             , Self::self_shortcut(shortcut::Action::press   (&[Key::Character("d".into())]) , "debug_set_data_for_selected_node")
        ]
    }
}


macro_rules! model_bind {
    ($network:ident $model:ident . $name:ident($($arg:ident),*)) => {
        frp::extend! { $network
            def _eval = $name.map(f!([$model](($($arg),*)) $model.$name($($arg.into()),*)));
        }
    };
}

macro_rules! model_bind2 {
    ($network:ident $model:ident . $name:ident($($arg:ident),*)) => {
        frp::extend! { $network
            def _eval = $name.map(f!([$model](($($arg),*)) $model.$name($($arg),*)));
        }
    };
}


impl application::View for GraphEditor {
    fn new(world: &World) -> Self {
        new_graph_editor(world)
    }
}

#[allow(unused_parens)]
fn new_graph_editor(world:&World) -> GraphEditor {
    let scene  = world.scene();
    let cursor = Cursor::new(world.scene());
    web::body().set_style_or_panic("cursor","none");
    world.add_child(&cursor);

    let model   = GraphEditorModelWithNetwork::new(scene,cursor.clone_ref());
    let network = &model.network;
    let nodes   = &model.nodes;
    let edges   = &model.edges;
    let inputs  = &model.frp;
    let mouse   = &scene.mouse.frp;
    let touch   = &model.touch_state;

    let outputs = UnsealedFrpOutputs::new();


    frp::extend! { network

    // === Selection Target Redirection ===

    def mouse_down_target  = mouse.press.map(f_!(model.scene.mouse.target.get()));
    eval mouse_down_target([touch,model](target) {
        match target {
            display::scene::Target::Background  => touch.background.down.emit(()),
            display::scene::Target::Symbol {..} => {
                if let Some(target) = model.scene.shapes.get_mouse_target(*target) {
                    target.mouse_down().emit(());
                }
            }
        }
    });


    // === Cursor Selection ===

    def mouse_on_down_position = mouse.position.sample(&mouse.press);
    def selection_zero         = source::<Position>();
    def selection_size_down    = mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
    def selection_size_if_down = selection_size_down.gate(&touch.background.is_down);
    def selection_size_on_down = selection_zero.sample(&mouse.press);
    def selection_size         = selection_size_if_down.merge(&selection_size_on_down);

    eval selection_size ((p) cursor.set_selection_size(Vector2::new(p.x,p.y)));

    eval_ mouse.press   (cursor.frp.press.emit(()));
    eval_ mouse.release (cursor.frp.release.emit(()));


    // === Node Select ===

    def deselect_all_nodes = gather_();

    select_node             <- [inputs.select_node, touch.nodes.selected];

    merge_select         <- gather();
    toggle_merge_select  <- inputs.toggle_node_merge_select.map2(&merge_select,|_,t| !t);
    enable_merge_select  <- inputs.enable_node_merge_select.constant(true);
    disable_merge_select <- inputs.disable_node_merge_select.constant(false);
    merge_select         <+ toggle_merge_select;
    merge_select         <+ enable_merge_select;
    merge_select         <+ disable_merge_select;

    // FIXME: DEFAULT VALUE!!!
    normal_select           <- merge_select.map(|t| !t);


    deselect_on_select      <- select_node.gate(&normal_select);
    deselect_all_nodes      <+ deselect_on_select;
    outputs.node_selected   <+ select_node;
    deselect_all_nodes      <+ inputs.deselect_all_nodes;

    deselect_on_bg_press    <- touch.background.selected.gate(&normal_select);
    deselect_all_nodes      <+ deselect_on_bg_press;
    selected_nodes          <- deselect_all_nodes.map(f_!(model.nodes.selected.mem_take())).iter();
    outputs.node_deselected <+ selected_nodes;


    // === Node Connect ===

    outputs.edge_source_set <+ inputs.connect_edge_source;
    outputs.edge_target_set <+ inputs.connect_edge_target;

    let endpoints            = inputs.connect_nodes.clone_ref();
    edge                    <- endpoints . map(f_!(model.new_edge()));
    new_edge_source         <- endpoints . _0() . map2(&edge, |t,id| (*id,t.clone()));
    new_edge_target         <- endpoints . _1() . map2(&edge, |t,id| (*id,t.clone()));
    outputs.edge_added      <+ edge;
    outputs.edge_source_set <+ new_edge_source;
    outputs.edge_target_set <+ new_edge_target;

    new_node_input          <- [inputs.press_node_input, inputs.connect_detached_edges_to_node];
    detached_targets        <= new_node_input.map(f_!(model.edges.detached_target.mem_take()));
    new_edge_target         <- new_node_input.map2(&detached_targets, |t,id| (*id,t.clone()));
    outputs.edge_target_set <+ new_edge_target;

    overlapping_edges       <= outputs.edge_target_set._1().map(f!((t) model.overlapping_edges(t)));
    outputs.edge_removed    <+ overlapping_edges;


    // === Add Node ===

    let add_node_at_cursor = inputs.add_node_at_cursor.clone_ref();
    add_node           <- [inputs.add_node, add_node_at_cursor];
    new_node           <- add_node.map(f_!([model,outputs] model.new_node(&outputs)));
    outputs.node_added <+ new_node;

    node_with_position <- add_node_at_cursor.map3(&new_node,&mouse.position,|_,id,pos| (*id,*pos));
    outputs.node_position_set <+ node_with_position;


    // === Remove Node ===

    all_nodes       <= inputs.remove_all_nodes      . map(f_!(model.all_nodes()));
    selected_nodes  <= inputs.remove_selected_nodes . map(f_!(model.selected_nodes()));
    nodes_to_remove <- [all_nodes, selected_nodes];
    eval nodes_to_remove ((node_id) inputs.remove_all_node_edges.emit(node_id));

    outputs.node_removed <+ nodes_to_remove;



    // === Remove Edge ===

    rm_input_edges       <- [inputs.remove_all_node_edges, inputs.remove_all_node_input_edges];
    rm_output_edges      <- [inputs.remove_all_node_edges, inputs.remove_all_node_output_edges];
    input_edges_to_rm    <= rm_input_edges  . map(f!((node_id) model.node_in_edges(node_id)));
    output_edges_to_rm   <= rm_output_edges . map(f!((node_id) model.node_out_edges(node_id)));
    edges_to_rm          <- [inputs.remove_edge, input_edges_to_rm, output_edges_to_rm];
    outputs.edge_removed <+ edges_to_rm;




    // === Set NodeView Expression ===

    eval inputs.set_node_expression(((node_id,expr)) model.set_node_expression(node_id,expr));


    // === Move Nodes ===

    node_drag     <- mouse.translation.gate(&touch.nodes.is_down);
    node_drag_pos <- node_drag.map2(&touch.nodes.down,f!((tx,id) model.node_pos_mod(id,tx)));

    tx_sel_nodes   <- [node_drag, inputs.translate_selected_nodes];
    selected_nodes <= tx_sel_nodes.map(f_!(model.nodes.selected.keys()));
    new_node_pos   <- selected_nodes.map2(&tx_sel_nodes,f!((id,tx) model.node_pos_mod(id,tx)));

    outputs.node_position_set <+ new_node_pos;


    // === Set Node Position ===


    outputs.node_position_set <+ inputs.set_node_position;




    eval outputs.node_position_set (((id,pos)) model.set_node_position(id,pos));





    // === Move Edges ===

    def _move_connections = cursor.frp.position.map(f!([edges](position) {
        edges.detached_target.for_each(|id| {
            if let Some(edge) = edges.get_cloned_ref(id) {
                edge.view.events.target_position.emit(position)
            }
        })
    }));



    // === Vis Update Data ===

    // TODO remove this once real data is available.
    let dummy_counter = Rc::new(Cell::new(1.0_f32));
    def _update_vis_data = inputs.debug_set_data_for_selected_node.map(f_!([nodes] {
        let dc = dummy_counter.get();
        dummy_counter.set(dc + 0.1);
        let content = Rc::new(json!(format!("{}", 20.0 + 10.0 * dummy_counter.get().sin())));
        let dummy_data = Some(visualization::Data::JSON { content });
        nodes.selected.for_each(|node_id| {
            if let Some(node) = nodes.get_cloned_ref(node_id) {
                node.view.visualization_container.frp.set_data.emit(&dummy_data);
            }
        })
    }));


    // === Toggle Visualization Visibility ===

    def _toggle_selected = inputs.toggle_visualization_visibility.map(f_!(
        nodes.selected.for_each(|node_id| {
            if let Some(node) = nodes.get_cloned_ref(node_id) {
                node.view.visualization_container.toggle_visibility();
            }
        });
    ));


    // === OUTPUTS REBIND ===




    let outputs = outputs.seal();


    eval outputs.edge_source_set (((id,tgt)) model.connect_edge_source(*id,tgt));
    eval outputs.edge_target_set (((id,tgt)) model.connect_edge_target(*id,tgt));
    eval outputs.node_selected   ((id) model.select_node(id));
    eval outputs.node_deselected ((id) model.deselect_node(id));
    eval outputs.edge_removed    ((id) model.remove_edge(id));
    eval outputs.node_removed    ((id) model.remove_node(id));



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
    let frp = Frp {inputs,outputs,status,node_release};

    GraphEditor {model,frp}
}




impl display::Object for GraphEditor {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
