#![allow(missing_docs)]

//! NOTE
//! This file is under a heavy development. It contains commented lines of code and some code may
//! be of poor quality. Expect drastic changes.

#![feature(associated_type_defaults)]
#![feature(clamp)]
#![feature(drain_filter)]
#![feature(entry_insert)]
#![feature(fn_traits)]
#![feature(overlapping_marker_traits)]
#![feature(option_result_contains)]
#![feature(specialization)]
#![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![feature(unboxed_closures)]
#![feature(vec_remove_item)]
#![feature(weak_into_raw)]

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_import_braces)]
#![warn(unused_qualifications)]

#![recursion_limit="1024"]

#[warn(missing_docs)]
pub mod component;

pub mod builtin;
pub mod data;

use crate::component::node;
use crate::component::visualization;
use crate::component::visualization::MockDataGenerator3D;
use crate::component::type_coloring;

use enso_args::ARGS;
use enso_frp as frp;
use ensogl::DEPRECATED_Animation;
use ensogl::DEPRECATED_Tween;
use ensogl::application::Application;
use ensogl::application::shortcut;
use ensogl::application;
use ensogl::data::color;
use ensogl::display::Scene;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::object::Id;
use ensogl::display::shape::StyleWatch;
use ensogl::display;
use ensogl::gui::cursor;
use ensogl::prelude::*;
use ensogl::system::web;
use ensogl_theme as theme;



// ===============
// === Prelude ===
// ===============

/// Commonly used utilities.
pub mod prelude {
    pub use ensogl::application::command::View;
    pub use ensogl::prelude::*;
}



// =================
// === Constants ===
// =================

const SNAP_DISTANCE_THRESHOLD          : f32 = 10.0;
const VIZ_PREVIEW_MODE_TOGGLE_TIME_MS  : f32 = 300.0;
const MACOS_TRAFFIC_LIGHTS_CONTENT     : f32 = 52.0;
const MACOS_TRAFFIC_LIGHTS_SIDE_OFFSET : f32 = 13.0;
const MACOS_TRAFFIC_LIGHTS_WIDTH       : f32 =
    MACOS_TRAFFIC_LIGHTS_CONTENT + 2.0 * MACOS_TRAFFIC_LIGHTS_SIDE_OFFSET;



// =================
// === SharedVec ===
// =================

#[derive(Clone,CloneRef,Debug,Derivative)]
#[derivative(Default(bound=""))]
pub struct SharedVec<T> {
    pub raw : Rc<RefCell<Vec<T>>>
}

impl<T> SharedVec<T> {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }

    /// Append an element to the back of a collection.
    pub fn push(&self, t:T) {
        self.raw.borrow_mut().push(t);
    }

    /// Remove the first instance of `item` from the vector if the item exists.
    pub fn remove_item(&self, t:&T) where T:PartialEq {
        self.raw.borrow_mut().remove_item(t);
    }

    /// Return `true` if the slice contains an element with the given value.
    pub fn contains(&self, t:&T) -> bool where T:PartialEq {
        self.raw.borrow().contains(t)
    }

    /// Return clone of the first element of the slice, or `None` if it is empty.
    pub fn first_cloned(&self) -> Option<T> where T:Clone {
        self.raw.borrow().first().cloned()
    }

    /// Return clone of the last element of the slice, or `None` if it is empty.
    pub fn last_cloned(&self) -> Option<T> where T:Clone {
        self.raw.borrow().last().cloned()
    }

    /// Replace the collection with the default value, and return the previous value.
    pub fn mem_take(&self) -> Vec<T> {
        mem::take(&mut self.raw.borrow_mut())
    }
}

impl<T:Clone> SharedVec<T> {
    /// Return a vector of all items stored in the collection in order.
    pub fn items(&self) -> Vec<T> {
        self.raw.borrow().clone()
    }
}



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

    pub fn contains(&self, value:&T) -> bool {
        self.raw.borrow().contains(value)
    }
}

impl<T,S> SharedHashSet<T,S> {
    pub fn is_empty(&self) -> bool {
        self.raw.borrow().is_empty()
    }

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

    pub fn get_copied(&self, k:&K) -> Option<V>
    where V:Copy {
        self.raw.borrow().get(k).copied()
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



// =================
// === FrpInputs ===
// =================

ensogl::define_endpoints! {
    Input {
        // === General ===
        /// Cancel the operation being currently performed. Often mapped to the escape key.
        cancel(),


        // === Node Selection ===

        /// Node press event
        node_press(),
        /// Node press event
        node_release(),
        /// Enable nodes multi selection mode. It works like inverse mode for single node selection
        /// and like merge mode for multi node selection mode.
        enable_node_multi_select(),
        /// Disable nodes multi selection mode. It works like inverse mode for single node selection
        /// and like merge mode for multi node selection mode.
        disable_node_multi_select(),
        /// Toggle nodes multi selection mode. It works like inverse mode for single node selection
        /// and like merge mode for multi node selection mode.
        toggle_node_multi_select(),

        /// Enable nodes merge selection mode.
        enable_node_merge_select(),
        /// Disable nodes merge selection mode.
        disable_node_merge_select(),
        /// Toggles nodes merge selection mode.
        toggle_node_merge_select(),

        /// Enable nodes subtract selection mode.
        enable_node_subtract_select(),
        /// Disable nodes subtract selection mode.
        disable_node_subtract_select(),
        /// Toggle nodes subtract selection mode.
        toggle_node_subtract_select(),

        /// Enable nodes inverse selection mode.
        enable_node_inverse_select(),
        /// Disable nodes inverse selection mode.
        disable_node_inverse_select(),
        /// Toggle nodes inverse selection mode.
        toggle_node_inverse_select(),


        // === Navigation ===

        /// Enter the last selected node.
        enter_selected_node(),
        /// Enter the node currently under the cursor.
        enter_hovered_node(),
        /// Steps out of the current node, popping the topmost stack frame from the crumb list.
        exit_node(),


        // === Node Editing ===

        /// Add a new node and place it in the origin of the workspace.
        add_node(),
        /// Add a new node and place it at the mouse cursor position.
        add_node_at_cursor(),
        /// Remove all selected nodes from the graph.
        remove_selected_nodes(),
        /// Remove all nodes from the graph.
        remove_all_nodes(),
        /// Enable mode in which the pressed node will be edited.
        edit_mode_on(),
        /// Disable mode in which the pressed node will be edited.
        edit_mode_off(),
        /// Stop node editing, whatever node is currently edited.
        stop_editing(),
        /// Remove all nodes from the graph.
        collapse_selected_nodes(),
        /// Indicate whether this node had an error or not.
        set_node_error_status(NodeId,Option<node::error::Error>),


        // === Visualization ===

        /// Simulates a visualization open press event. In case the event will be shortly followed by `release_visualization_visibility`, the visualization will be shown permanently. In other case, it will be disabled as soon as the `release_visualization_visibility` is emitted.
        press_visualization_visibility(),
        /// Simulates a visualization open double press event. This event toggles the visualization fullscreen mode.
        double_press_visualization_visibility(),
        /// Simulates a visualization open release event. See `press_visualization_visibility` to learn more.
        release_visualization_visibility(),
        /// Cycle the visualization for the selected nodes.
        cycle_visualization_for_selected_node(),


        // === Scene Navigation ===

        /// Stop the scene camera from moving around, locking the scene in place.
        /// Can be used, e.g., if there is a fullscreen visualisation active, or navigation should
        ///only work for a selected visualisation.
        set_navigator_disabled(bool),


        // === Debug ===

        /// Push a hardcoded breadcrumb without notifying the controller.
        debug_push_breadcrumb(),
        /// Pop a breadcrumb without notifying the controller.
        debug_pop_breadcrumb(),
        /// Set a test visualization data for the selected nodes. Useful for testing visualizations during their development.
        debug_set_test_visualization_data_for_selected_node(),


        set_detached_edge_targets    (EdgeEndpoint),
        set_detached_edge_sources    (EdgeEndpoint),
        set_edge_source              ((EdgeId,EdgeEndpoint)),
        set_edge_target              ((EdgeId,EdgeEndpoint)),
        unset_edge_source            (EdgeId),
        unset_edge_target            (EdgeId),
        connect_nodes                ((EdgeEndpoint,EdgeEndpoint)),
        deselect_all_nodes           (),
        press_node_input             (EdgeEndpoint),
        press_node_output            (EdgeEndpoint),
        remove_all_node_edges        (NodeId),
        remove_all_node_input_edges  (NodeId),
        remove_all_node_output_edges (NodeId),
        remove_edge                  (EdgeId),
        select_node                  (NodeId),
        remove_node                  (NodeId),
        edit_node                    (NodeId),
        collapse_nodes               ((Vec<NodeId>,NodeId)),
        set_node_expression          ((NodeId,node::Expression)),
        set_node_position            ((NodeId,Vector2)),
        set_expression_usage_type    ((NodeId,ast::Id,Option<Type>)),
        set_method_pointer           ((ast::Id,Option<MethodPointer>)),
        cycle_visualization          (NodeId),
        set_visualization            ((NodeId,Option<visualization::Path>)),
        register_visualization       (Option<visualization::Definition>),
        set_visualization_data       ((NodeId,visualization::Data)),
        enable_visualization         (NodeId),
    }

    Output {

        // === Edge ===

        on_edge_add                            (EdgeId),
        on_edge_drop                           (EdgeId),
        on_edge_source_set                     ((EdgeId,EdgeEndpoint)),
        on_edge_source_set_with_target_not_set ((EdgeId,EdgeEndpoint)),
        on_edge_target_set_with_source_not_set ((EdgeId,EdgeEndpoint)),
        on_edge_target_set                     ((EdgeId,EdgeEndpoint)),
        on_edge_source_unset                   ((EdgeId,EdgeEndpoint)),
        on_edge_target_unset                   ((EdgeId,EdgeEndpoint)),

        /// Fires always when there is a new edge with source set but target not set. This could
        /// happen after the target was disconnected or the edge was created and its source was
        /// connected.
        on_edge_only_target_not_set (EdgeId),

        /// Fires always when there is a new edge with target set but source not set. This could
        /// happen after the source was disconnected or the edge was created and its target was
        /// connected.
        on_edge_only_source_not_set (EdgeId),

        on_edge_endpoint_unset      ((EdgeId,EdgeEndpoint)),
        on_edge_endpoint_set        ((EdgeId,EdgeEndpoint)),
        on_edge_endpoints_set       (EdgeId),
        on_some_edges_targets_unset (),
        on_some_edges_sources_unset (),
        on_all_edges_targets_set    (),
        on_all_edges_sources_set    (),
        on_all_edges_endpoints_set  (),
        some_edge_targets_unset     (bool),
        some_edge_sources_unset     (bool),
        some_edge_endpoints_unset   (bool),

        hover_node_input            (Option<EdgeEndpoint>),
        hover_node_output           (Option<EdgeEndpoint>),


        // === Other ===
        // FIXME: To be refactored

        node_added                (NodeId),
        node_removed              (NodeId),
        nodes_collapsed           ((Vec<NodeId>,NodeId)),
        node_hovered              (Option<Switch<NodeId>>),
        node_selected             (NodeId),
        node_deselected           (NodeId),
        node_position_set         ((NodeId,Vector2)),
        node_position_set_batched ((NodeId,Vector2)),
        node_expression_set       ((NodeId,String)),
        node_entered              (NodeId),
        node_exited               (),
        node_editing_started      (NodeId),
        node_editing_finished     (NodeId),
        node_action_freeze        ((NodeId,bool)),
        node_action_skip          ((NodeId,bool)),
        node_edit_mode            (bool),
        nodes_labels_visible      (bool),


        visualization_enabled           (NodeId),
        visualization_disabled          (NodeId),
        visualization_enable_fullscreen (NodeId),
        visualization_set_preprocessor  ((NodeId,data::EnsoCode)),

        on_visualization_select     (Switch<NodeId>),
        some_visualisation_selected (bool),

        node_being_edited (Option<NodeId>),
        node_editing (bool),

        navigator_active (bool),
    }
}


impl application::command::FrpNetworkProvider for GraphEditor {
    fn network(&self) -> &frp::Network {
        &self.model.network
    }
}



// ============
// === Node ===
// ============

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Node {
    #[shrinkwrap(main_field)]
    pub view      : component::Node,
    pub in_edges  : SharedHashSet<EdgeId>,
    pub out_edges : SharedHashSet<EdgeId>,
}

#[derive(Clone,CloneRef,Copy,Debug,Default,Eq,From,Hash,Into,PartialEq)]
pub struct NodeId(pub Id);

impl Node {
    pub fn new(view:component::Node) -> Self {
        let in_edges  = default();
        let out_edges = default();
        Self {view,in_edges,out_edges}
    }

    pub fn id(&self) -> NodeId {
        self.view.id().into()
    }

    /// Return all edges connected to this node. Ingoing and outgoing both.
    pub fn all_edges(self) -> Vec<EdgeId> {
        self.in_edges.keys().extended(self.out_edges.keys())
    }

}

impl display::Object for Node {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object()
    }
}

impl Display for NodeId {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0,f)
    }
}



// ============
// === Edge ===
// ============

#[derive(Clone,CloneRef,Debug,Shrinkwrap)]
pub struct Edge {
    #[shrinkwrap(main_field)]
    pub view : component::Edge,
    source   : Rc<RefCell<Option<EdgeEndpoint>>>,
    target   : Rc<RefCell<Option<EdgeEndpoint>>>,
}

#[derive(Clone,CloneRef,Copy,Debug,Default,Eq,From,Hash,Into,PartialEq)]
pub struct EdgeId(pub Id);

impl Edge {
    pub fn new(view:component::Edge) -> Self {
        let source = default();
        let target = default();
        Self {view,source,target}
    }

    pub fn id(&self) -> EdgeId {
        self.view.id().into()
    }

    pub fn target(&self) -> Option<EdgeEndpoint> {
        self.target.borrow().as_ref().map(|t| t.clone_ref())
    }

    pub fn source(&self) -> Option<EdgeEndpoint> {
        self.source.borrow().as_ref().map(|t| t.clone_ref())
    }

    pub fn has_source(&self) -> bool {
        self.source.borrow().is_some()
    }

    pub fn has_target(&self) -> bool {
        self.target.borrow().is_some()
    }

    pub fn set_source(&self, source:EdgeEndpoint) {
        *self.source.borrow_mut() = Some(source)
    }

    pub fn set_target(&self, target:EdgeEndpoint) {
        *self.target.borrow_mut() = Some(target)
    }

    pub fn take_source(&self) -> Option<EdgeEndpoint> {
        mem::take(&mut *self.source.borrow_mut())
    }

    pub fn take_target(&self) -> Option<EdgeEndpoint> {
        mem::take(&mut *self.target.borrow_mut())
    }

}

impl display::Object for Edge {
    fn display_object(&self) -> &display::object::Instance {
        &self.view.display_object()
    }
}

impl Display for EdgeId {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0,f)
    }
}



// ============
// === Type ===
// ============

/// Typename information that may be associated with the given Port.
///
/// `None` means that type for the port is unknown.
#[derive(Clone,Debug,Default,Hash)]
pub struct Type(pub ImString);

impl Deref for Type {
    type Target = ImString;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Type {
    /// Check whether this is any type, the most generic type in Enso. The empty string is
    /// considered to be an empty type as well.
    pub fn is_any(&self) -> bool {
        self.as_str() == "Any" || self.is_empty()
    }
}

impl From<String> for Type {
    fn from(s:String) -> Self {
        Type(s.into())
    }
}

impl Display for Type {
    fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"{}",self.0)
    }
}



// =============================
// === OptionalMethodPointer ===
// =============================

/// Information about target definition for node entering.
// TODO [mwu]
//  As currently there is no good place to wrap Rc into a newtype that can be easily depended on
//  both by `ide-view` and `ide` crates, we put this as-is. Refactoring should be considered in the
//  future, once code organization and emerging patterns are more clear.
#[derive(Clone,Debug,Shrinkwrap,PartialEq,Eq)]
pub struct MethodPointer(pub Rc<enso_protocol::language_server::MethodPointer>);

impl From<enso_protocol::language_server::MethodPointer> for MethodPointer {
    fn from(method_pointer:enso_protocol::language_server::MethodPointer) -> Self {
        Self(Rc::new(method_pointer))
    }
}



// =================
// === LocalCall ===
// =================

/// A specific function call occurring within another function's definition body.
/// It's closely related to the `LocalCall` type defined in `Language Server` types, but uses the
/// new type `MethodPointer` defined in `GraphEditor`.
#[derive(Clone,Debug,Eq,PartialEq)]
pub struct LocalCall {
    /// An expression being a call to a method.
    pub call : enso_protocol::language_server::ExpressionId,
    /// A pointer to the called method.
    pub definition : MethodPointer,
}



// ==================
// === EdgeEndpoint ===
// ==================

#[derive(Clone,CloneRef,Debug,Default)]
pub struct EdgeEndpoint {
    pub node_id : NodeId,
    pub port    : span_tree::Crumbs,
}

impl EdgeEndpoint {
    pub fn new(node_id:impl Into<NodeId>, port:span_tree::Crumbs) -> Self {
        let node_id = node_id.into();
        Self {node_id,port}
    }

    pub fn is_connected_to(&self, node_id:NodeId) -> bool {
        self.node_id == node_id
    }
}



// ============
// === Grid ===
// ============

/// Defines a snapping grid for nodes. The grid implementation is currently very simple. For each
/// node, the grid records its position and allows querying for positions close to the recorded
/// ones.
#[derive(Debug,Clone,Default)]
pub struct Grid {
    sorted_xs : Vec<f32>,
    sorted_ys : Vec<f32>,
}

impl Grid {
    /// Query the grid for a close position to the provided using the provided threshold distance.
    pub fn close_to(&self, position:Vector2<f32>, threshold:f32) -> Vector2<Option<f32>> {
        let x = Self::axis_close_to(&self.sorted_xs,position.x,threshold);
        let y = Self::axis_close_to(&self.sorted_ys,position.y,threshold);
        Vector2(x,y)
    }

    fn axis_close_to(axis:&[f32], pos:f32, threshold:f32) -> Option<f32> {
        match axis.binary_search_by(|t| t.partial_cmp(&pos).unwrap()) {
            Ok (ix) => Some(axis[ix]),
            Err(ix) => {
                let max         = axis.len();
                let left_pos    = if ix == 0   { None } else { Some(axis[ix-1]) };
                let right_pos   = if ix == max { None } else { Some(axis[ix]) };
                let left_dist   = left_pos   . map(|t| (pos - t).abs());
                let right_dist  = right_pos  . map(|t| (pos - t).abs());
                let left_check  = left_dist  . map(|t| t < threshold).unwrap_or_default();
                let right_check = right_dist . map(|t| t < threshold).unwrap_or_default();
                match (left_check,right_check) {
                    ( false , false ) => None,
                    ( true  , false ) => left_pos,
                    ( false , true  ) => right_pos,
                    ( true  , true  ) => {
                        let left_dist  = left_dist.unwrap_or_default();
                        let right_dist = right_dist.unwrap_or_default();
                        if left_dist < right_dist { left_pos } else { right_pos }
                    }
                }
            }
        }
    }
}



// =============
// === Nodes ===
// =============

#[derive(Debug,Clone,CloneRef)]
pub struct Nodes {
    pub logger   : Logger,
    pub all      : SharedHashMap<NodeId,Node>,
    pub selected : SharedVec<NodeId>,
    pub grid     : Rc<RefCell<Grid>>,
}

impl Deref for Nodes {
    type Target = SharedHashMap<NodeId,Node>;
    fn deref(&self) -> &Self::Target {
        &self.all
    }
}

impl Nodes {
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger   = Logger::sub(logger,"nodes");
        let all      = default();
        let selected = default();
        let grid     = default();
        Self {logger,all,selected,grid}
    }

    pub fn insert(&self, node_id:NodeId, node:Node) {
        self.all.insert(node_id,node);
        self.recompute_grid(default());
    }

    fn recompute_grid(&self, blacklist:HashSet<NodeId>) {
        let mut sorted_xs = Vec::new();
        let mut sorted_ys = Vec::new();
        for (id,node) in &*self.all.raw.borrow() {
            if !blacklist.contains(id) {
                let position = node.position();
                sorted_xs.push(position.x);
                sorted_ys.push(position.y);
            }
        }
        sorted_xs.sort_unstable_by(|a,b|a.partial_cmp(b).unwrap());
        sorted_ys.sort_unstable_by(|a,b|a.partial_cmp(b).unwrap());
        *self.grid.borrow_mut() = Grid {sorted_xs,sorted_ys};
    }

    pub fn check_grid_magnet(&self, position:Vector2<f32>) -> Vector2<Option<f32>> {
        self.grid.borrow().close_to(position,SNAP_DISTANCE_THRESHOLD)
    }
}




#[derive(Debug,Clone,CloneRef)]
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
    pub fn new(logger:impl AnyLogger) -> Self {
        let logger          = Logger::sub(logger,"edges");
        let all             = default();
        let detached_source = default();
        let detached_target = default();
        Self {logger,all,detached_source,detached_target}
    }

    pub fn insert(&self, edge:Edge) {
        self.all.insert(edge.id(),edge);
    }

    pub fn detached_edges_iter(&self) -> impl Iterator<Item=EdgeId> {
        let detached_target      = self.detached_target.raw.borrow();
        let detached_source      = self.detached_source.raw.borrow();
        let mut detached         = detached_target.iter().copied().collect_vec();
        let detached_source_iter = detached_source.iter().copied();
        detached.extend(detached_source_iter);
        detached.into_iter()
    }
}



#[derive(Debug,Clone,CloneRef,Default)]
struct Visualisations {
    /// This keeps track of the currently selected visualisation. There should only ever be one
    /// visualisations selected, however due to the way that the selection is determined, it can
    /// happen that while the FRP is resolved, temporarily, we have multiple visualisation in this
    /// set. This happens because the selection status is determined bottom up from each
    /// visualisation and the reported via FRP to the graph editor. That means if the status
    /// we might see the new selection status for a visualisation getting set before we see the
    /// previously selected visualisation report its deselection. If we ever have more than one
    /// visualisation in this set after the status updates have been resolved, that is a bug.
    selected : SharedHashSet<NodeId>,
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
            down          <- source::<T> ();
            is_down       <- bool(&mouse.up_primary,&down);
            was_down      <- is_down.previous();
            mouse_up      <- mouse.up_primary.gate(&was_down);
            pos_on_down   <- mouse.position.sample(&down);
            pos_on_up     <- mouse.position.sample(&mouse_up);
            should_select <- pos_on_up.map3(&pos_on_down,&mouse.distance,Self::check);
            up            <- down.sample(&mouse_up);
            selected      <- up.gate(&should_select);
        }
        Self {down,up,is_down,selected}
    }

    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn check(end:&Vector2, start:&Vector2, diff:&f32) -> bool {
        (end-start).norm() <= diff * 2.0
    }
}

#[derive(Debug,Clone,CloneRef)]
pub struct TouchState {
    pub nodes      : TouchNetwork::<NodeId>,
    pub background : TouchNetwork::<()>,
}

impl TouchState {
    pub fn new(network:&frp::Network, mouse:&frp::io::Mouse) -> Self {
        let nodes      = TouchNetwork::<NodeId>::new(&network,mouse);
        let background = TouchNetwork::<()>::new(&network,mouse);
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
    pub fn new(app:&Application, cursor:cursor::Cursor, frp:&Frp) -> Self {
        let network = frp.network.clone_ref(); // FIXME make weak
        let model   = GraphEditorModel::new(app,cursor,&frp);
        Self {model,network}
    }

    #[allow(clippy::too_many_arguments)]
    fn new_node
    ( &self
    , pointer_style : &frp::Source<cursor::Style>
    , output_press  : &frp::Source<EdgeEndpoint>
    , input_press   : &frp::Source<EdgeEndpoint>
    , output        : &FrpEndpoints
    ) -> NodeId {
        let view    = component::Node::new(&self.app,self.vis_registry.clone_ref());
        let node    = Node::new(view);
        let node_id = node.id();
        self.add_child(&node);

        let touch = &self.touch_state;
        let model = &self.model;

        frp::new_bridge_network! { [self.network, node.frp.network] graph_node_bridge
            eval_ node.frp.background_press(touch.nodes.down.emit(node_id));

            hovered <- node.output.hover.map (move |t| Some(Switch::new(node_id,*t)));
            output.source.node_hovered <+ hovered;

            node.set_output_expression_visibility <+ self.frp.nodes_labels_visible;

            eval node.model.input.frp.pointer_style ((style) pointer_style.emit(style));
            eval node.model.output.frp.on_port_press ([output_press](crumbs){
                let target = EdgeEndpoint::new(node_id,crumbs.clone());
                output_press.emit(target);
            });

            eval node.model.input.frp.on_port_press ([input_press](crumbs)
                let target = EdgeEndpoint::new(node_id,crumbs.clone());
                input_press.emit(target);
            );

            eval node.model.input.frp.on_port_hover ([model](t) {
                let crumbs = t.on();
                let target = crumbs.map(|c| EdgeEndpoint::new(node_id,c.clone()));
                model.frp.source.hover_node_input.emit(target);
            });

            eval node.model.output.frp.on_port_hover ([model](hover) {
               let output = hover.on().map(|crumbs| EdgeEndpoint::new(node_id,crumbs.clone()));
               model.frp.source.hover_node_output.emit(output);
            });

            eval node.model.input.frp.on_port_type_change(((crumbs,_))
                model.with_input_edge_id(node_id,crumbs,|id| model.refresh_edge_color(id))
            );

            eval node.model.output.frp.on_port_type_change(((crumbs,_))
                model.with_output_edge_id(node_id,crumbs,|id| model.refresh_edge_color(id))
            );

            eval node.frp.expression((t) output.source.node_expression_set.emit((node_id,t.into())));


            // === Actions ===

            eval node.view.frp.freeze ((is_frozen) {
                output.source.node_action_freeze.emit((node_id,*is_frozen));
            });

            let set_node_disabled = &node.frp.set_disabled;
            eval node.view.frp.skip ([set_node_disabled,output](is_skipped) {
                output.source.node_action_skip.emit((node_id,*is_skipped));
                set_node_disabled.emit(is_skipped);
            });


            // === Visualizations ===

            let vis_changed    =  node.model.visualization.frp.visualisation.clone_ref();
            let vis_visible    =  node.model.visualization.frp.set_visibility.clone_ref();
            let vis_fullscreen =  node.model.visualization.frp.enable_fullscreen.clone_ref();

            vis_enabled  <- vis_visible.gate(&vis_visible);
            vis_disabled <- vis_visible.gate_not(&vis_visible);

            let vis_is_selected = node.model.visualization.frp.is_selected.clone_ref();

            selected    <- vis_is_selected.on_true();
            deselected  <- vis_is_selected.on_false();
            output.source.on_visualization_select <+ selected.constant(Switch::On(node_id));
            output.source.on_visualization_select <+ deselected.constant(Switch::Off(node_id));


            // Ensure the graph editor knows about internal changes to the visualisation. If the
            // visualisation changes that should indicate that the old one has been disabled and a
            // new one has been enabled.
            // TODO: Create a better API for updating the controller about visualisation changes
            // (see #896)
            output.source.visualization_disabled          <+ vis_changed.constant(node_id);
            output.source.visualization_enabled           <+ vis_changed.constant(node_id);

            output.source.visualization_enabled           <+ vis_enabled.constant(node_id);
            output.source.visualization_disabled          <+ vis_disabled.constant(node_id);
            output.source.visualization_enable_fullscreen <+ vis_fullscreen.constant(node_id);
        }

        self.nodes.insert(node_id,node);
        node_id
    }

    fn is_node_connected_at_input(&self, node_id:NodeId, crumbs:&span_tree::Crumbs) -> bool {
        if let Some(node) = self.nodes.get_cloned(&node_id) {
            for in_edge_id in node.in_edges.raw.borrow().iter() {
                if let Some(edge) = self.edges.get_cloned(in_edge_id) {
                    if let Some(target) = edge.target() {
                        if target.node_id == node_id && target.port == crumbs {
                            return true
                        }
                    }
                }
            }
        }
        false
    }

    pub fn get_node_position(&self, node_id:NodeId) -> Option<Vector3<f32>> {
        self.nodes.get_cloned_ref(&node_id).map(|node| node.position())
    }

    fn create_edge
    ( &self
    , edge_click : &frp::Source<EdgeId>
    , edge_over  : &frp::Source<EdgeId>
    , edge_out   : &frp::Source<EdgeId>
    ) -> EdgeId {
        let edge    = Edge::new(component::Edge::new(&self.app));
        let edge_id = edge.id();
        self.add_child(&edge);
        self.edges.insert(edge.clone_ref());

        let network = &self.network;

        frp::extend! { network
            eval_ edge.view.frp.shape_events.mouse_down ( edge_click.emit(edge_id));
            eval_ edge.view.frp.shape_events.mouse_over ( edge_over.emit(edge_id));
            eval_ edge.view.frp.shape_events.mouse_out ( edge_out.emit(edge_id));
        }

        edge_id
    }

    fn new_edge_from_output
    ( &self
    , edge_click:&frp::Source<EdgeId>
    , edge_over:&frp::Source<EdgeId>
    , edge_out:&frp::Source<EdgeId>
    ) -> EdgeId {
        let edge_id        = self.create_edge(edge_click,edge_over,edge_out);
        let first_detached = self.edges.detached_target.is_empty();
        self.edges.detached_target.insert(edge_id);
        if first_detached {
            self.frp.source.on_some_edges_targets_unset.emit(());
        }
        edge_id
    }

    fn new_edge_from_input
    ( &self
    , edge_click : &frp::Source<EdgeId>
    , edge_over  : &frp::Source<EdgeId>
    , edge_out   : &frp::Source<EdgeId>
    ) -> EdgeId {
        let edge_id        = self.create_edge(edge_click,edge_over,edge_out);
        let first_detached = self.edges.detached_source.is_empty();
        self.edges.detached_source.insert(edge_id);
        if first_detached {
            self.frp.source.on_some_edges_sources_unset.emit(());
        }
        edge_id
    }

}



// ========================
// === GraphEditorModel ===
// ========================

#[derive(Debug,Clone,CloneRef)]
pub struct GraphEditorModel {
    pub logger         : Logger,
    pub display_object : display::object::Instance,
    pub app            : Application,
    pub breadcrumbs    : component::Breadcrumbs,
    pub cursor         : cursor::Cursor,
    pub nodes          : Nodes,
    pub edges          : Edges,
    pub vis_registry   : visualization::Registry,
    touch_state        : TouchState,
    visualisations     : Visualisations,
    frp                : FrpEndpoints,
    navigator          : Navigator,

}


// === Public ===

impl GraphEditorModel {
    pub fn new
    ( app    : &Application
    , cursor : cursor::Cursor
    , frp    : &Frp
    ) -> Self {
        let network        = &frp.network;
        let scene          = app.display.scene();
        let logger         = Logger::new("GraphEditor");
        let display_object = display::object::Instance::new(&logger);
        let nodes          = Nodes::new(&logger);
        let edges          = Edges::new(&logger);
        let vis_registry   = visualization::Registry::with_default_visualizations();
        let visualisations = default();
        let touch_state    = TouchState::new(network,&scene.mouse.frp);
        let breadcrumbs    = component::Breadcrumbs::new(app.clone_ref());
        let app            = app.clone_ref();
        let frp            = frp.output.clone_ref();
        let navigator      = Navigator::new(&scene,&scene.camera());
        Self {
            logger,display_object,app,cursor,nodes,edges,touch_state,frp,breadcrumbs,
            vis_registry,visualisations,navigator
        }.init()
    }

    fn init(self) -> Self {
        self.add_child(&self.breadcrumbs);
        let is_macos     = ARGS.platform.map(|p|p.is_macos()) == Some(true);
        let is_frameless = ARGS.frame == Some(false);
        let x_offset     = if is_macos && is_frameless { MACOS_TRAFFIC_LIGHTS_WIDTH }
                           else                        { MACOS_TRAFFIC_LIGHTS_SIDE_OFFSET };
        self.breadcrumbs.set_position_x(x_offset);
        self.breadcrumbs.set_position_y(-5.0);
        self
    }

    pub fn all_nodes(&self) -> Vec<NodeId> {
        self.nodes.all.keys()
    }

    fn scene(&self) -> &Scene {
        self.app.display.scene()
    }
}


// === Selection ===

impl GraphEditorModel {
    fn select_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            self.nodes.selected.push(node_id);
            node.frp.select.emit(());
        }
    }

    fn deselect_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            self.nodes.selected.remove_item(&node_id);
            node.frp.deselect.emit(());
        }
    }

    pub fn selected_nodes(&self) -> Vec<NodeId> {
        self.nodes.selected.items()
    }

    pub fn last_selected_node(&self) -> Option<NodeId> {
        self.nodes.selected.last_cloned()
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
                self.set_input_connected(&target,None,false); // FIXME None
                if let Some(target_node) = self.nodes.get_cloned_ref(&target.node_id) {
                    target_node.in_edges.remove(&edge_id);
                }
            }
        }
    }

    fn set_input_connected(&self, target:&EdgeEndpoint, tp:Option<Type>, status:bool) {
        if let Some(node) = self.nodes.get_cloned(&target.node_id) {
            node.view.set_input_connected(&target.port,tp,status);
        }
    }

    fn set_edge_target_connection_status(&self, edge_id:EdgeId, status:bool) {
        self.with_edge_target(edge_id,|tgt| self.set_endpoint_connection_status(edge_id,&tgt,status));
    }

    fn set_endpoint_connection_status(&self, edge_id:EdgeId, target:&EdgeEndpoint, status:bool) {
        let tp = self.edge_source_type(edge_id);
        self.set_input_connected(target,tp,status);
    }

    // FIXME: make nicer
    fn enable_visualization(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.model.visualization.frp.set_visibility.emit(true);
        }
    }

    // FIXME: make nicer
    fn disable_visualization(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.model.visualization.frp.set_visibility.emit(false);
        }
    }

    fn enable_visualization_fullscreen(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.model.visualization.frp.enable_fullscreen.emit(());
        }
    }

    /// Warning! This function does not remove connected edges. It needs to be handled by the
    /// implementation.
    fn remove_node(&self, node_id:impl Into<NodeId>) {
        let node_id = node_id.into();
        self.nodes.remove(&node_id);
        self.nodes.selected.remove_item(&node_id);
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

    fn set_node_expression(&self, node_id:impl Into<NodeId>, expr:impl Into<node::Expression>) {
        let node_id = node_id.into();
        let expr    = expr.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.frp.set_expression.emit(expr);
        }
        for edge_id in self.node_out_edges(node_id) {
            self.refresh_edge_source_size(edge_id);
        }
    }

    fn is_connection(&self, edge_id:impl Into<EdgeId>) -> bool {
        let edge_id = edge_id.into();
        match self.edges.get_cloned_ref(&edge_id) {
            None    => false,
            Some(e) => e.has_source() && e.has_target()
        }
    }
}


// === Connect ===

impl GraphEditorModel {
    fn set_edge_source(&self, edge_id:EdgeId, target:impl Into<EdgeEndpoint>) {
        let target = target.into();
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
                node.out_edges.insert(edge_id);
                edge.set_source(target);
                edge.view.frp.source_attached.emit(true);
                // FIXME: both lines require edge to refresh. Let's make it more efficient.
                self.refresh_edge_position(edge_id);
                self.refresh_edge_source_size(edge_id);
            }

        }
    }

    fn remove_edge_source(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(source) = edge.take_source() {
                if let Some(node) = self.nodes.get_cloned_ref(&source.node_id) {
                    node.out_edges.remove(&edge_id);
                    edge.view.frp.source_attached.emit(false);
                    let first_detached = self.edges.detached_source.is_empty();
                    self.edges.detached_source.insert(edge_id);
                    // FIXME: both lines require edge to refresh. Let's make it more efficient.
                    self.refresh_edge_position(edge_id);
                    self.refresh_edge_source_size(edge_id);
                    if first_detached {
                        self.frp.source.on_some_edges_sources_unset.emit(());
                    }
                }
            }
        }
    }

    fn set_edge_target(&self, edge_id:EdgeId, target:impl Into<EdgeEndpoint>) {
        let target = target.into();
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
                node.in_edges.insert(edge_id);
                edge.set_target(target);

                self.edges.detached_target.remove(&edge_id);
                let all_attached = self.edges.detached_target.is_empty();
                if all_attached {
                    self.frp.source.on_all_edges_targets_set.emit(());
                }

                edge.view.frp.target_attached.emit(true);
                edge.view.frp.redraw.emit(());
                self.refresh_edge_position(edge_id);
            };
        }
    }

    fn remove_edge_target(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(target) = edge.take_target() {
                if let Some(node) = self.nodes.get_cloned_ref(&target.node_id) {
                    node.in_edges.remove(&edge_id);
                    let first_detached = self.edges.detached_target.is_empty();
                    self.edges.detached_target.insert(edge_id);
                    edge.view.frp.target_attached.emit(false);
                    self.refresh_edge_position(edge_id);
                    if first_detached {
                        self.frp.source.on_some_edges_targets_unset.emit(());
                    }
                };
            }
        }
    }

    fn take_edges_with_detached_targets(&self) -> HashSet<EdgeId> {
        let edges = self.edges.detached_target.mem_take();
        self.check_edge_attachment_status_and_emit_events();
        edges
    }

    fn take_edges_with_detached_sources(&self) -> HashSet<EdgeId> {
        let edges = self.edges.detached_source.mem_take();
        self.check_edge_attachment_status_and_emit_events();
        edges
    }

    fn edges_with_detached_targets(&self) -> HashSet<EdgeId> {
        self.edges.detached_target.raw.borrow().clone()
    }

    pub fn clear_all_detached_edges(&self) -> Vec<EdgeId>{
        let source_edges = self.edges.detached_source.mem_take();
        source_edges.iter().for_each(|edge| {self.edges.all.remove(edge);});
        let target_edges = self.edges.detached_target.mem_take();
        target_edges.iter().for_each(|edge| {self.edges.all.remove(edge);});
        self.check_edge_attachment_status_and_emit_events();
        source_edges.into_iter().chain(target_edges).collect()
    }

    fn check_edge_attachment_status_and_emit_events(&self) {
        let no_detached_sources = self.edges.detached_source.is_empty();
        let no_detached_targets = self.edges.detached_target.is_empty();
        if no_detached_targets {
            self.frp.source.on_all_edges_targets_set.emit(());
        }
        if no_detached_sources {
            self.frp.source.on_all_edges_sources_set.emit(());
        }
    }

    fn overlapping_edges(&self, target:&EdgeEndpoint) -> Vec<EdgeId> {
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

    fn set_edge_freeze<T:Into<EdgeId>>(&self, edge_id:T, is_frozen:bool) {
        let edge_id = edge_id.into();
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            edge.view.frp.set_disabled.emit(is_frozen);
        }
    }
}


// === Position ===

impl GraphEditorModel {
    pub fn set_node_position(&self, node_id:impl Into<NodeId>, position:Vector2) {
        let node_id  = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.mod_position(|t| {
                t.x = position.x;
                t.y = position.y;
            });
            for edge_id in self.node_in_and_out_edges(node_id) {
                self.refresh_edge_position(edge_id);
            }
        }
    }

    fn set_node_expression_usage_type(&self, node_id:impl Into<NodeId>, ast_id:ast::Id, maybe_type:Option<Type>) {
        let node_id  = node_id.into();
        if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            // TODO[ao]: we must update root output port according to the whole expression type
            //     due to a bug in engine https://github.com/enso-org/enso/issues/1038.
            if node.view.model.output.whole_expr_id().contains(&ast_id) {
                let crumbs = span_tree::Crumbs::default();
                node.view.model.output.set_expression_usage_type(crumbs,maybe_type.clone());
            }
            let crumbs = node.view.model.get_crumbs_by_id(ast_id);
            if let Some(crumbs) = crumbs {
                node.view.frp.set_expression_usage_type.emit((crumbs,maybe_type));
            }
        }
    }

    fn disable_grid_snapping_for(&self, node_ids:&[NodeId]) {
        self.nodes.recompute_grid(node_ids.iter().cloned().collect());
    }

    pub fn node_position(&self, node_id:impl Into<NodeId>) -> Vector2<f32> {
        let node_id = node_id.into();
        self.nodes.get_cloned_ref(&node_id).map(|node| node.position().xy()).unwrap_or_default()
    }

    pub fn node_pos_mod
    (&self, node_id:impl Into<NodeId>, pos_diff:Vector2) -> (NodeId,Vector2) {
        let node_id      = node_id.into();
        let new_position = if let Some(node) = self.nodes.get_cloned_ref(&node_id) {
            node.position().xy() + pos_diff
        } else {
            default()
        };
        (node_id,new_position)
    }

    pub fn refresh_edge_position(&self, edge_id:EdgeId) {
        self.refresh_edge_source_position(edge_id);
        self.refresh_edge_target_position(edge_id);
    }

    pub fn refresh_edge_source_size(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(edge_source) = edge.source() {
                if let Some(node) = self.nodes.get_cloned_ref(&edge_source.node_id) {
                    edge.view.frp.source_width.emit(node.model.width());
                    edge.view.frp.source_height.emit(node.model.height());
                    edge.view.frp.redraw.emit(());
                }
            }
        };
    }

    pub fn refresh_edge_color(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            let color = self.edge_color(edge_id);
            edge.view.frp.set_color.emit(color);
        };
    }

    pub fn refresh_edge_source_position(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(edge_source) = edge.source() {
                if let Some(node) = self.nodes.get_cloned_ref(&edge_source.node_id) {
                    edge.mod_position(|p| {
                        p.x = node.position().x + node.model.width()/2.0;
                        p.y = node.position().y;
                    });
                }
            }
        };
    }

    pub fn refresh_edge_target_position(&self, edge_id:EdgeId) {
        if let Some(edge) = self.edges.get_cloned_ref(&edge_id) {
            if let Some(edge_target) = edge.target() {
                if let Some(node) = self.nodes.get_cloned_ref(&edge_target.node_id) {
                    let offset = node.model.input.port_offset(&edge_target.port).unwrap_or_default();
                    let pos = node.position().xy() + offset;
                    edge.view.frp.target_position.emit(pos);
                    edge.view.frp.redraw.emit(());
                }
            }
        };
    }

    fn map_node<T>(&self, id:NodeId, f:impl FnOnce(Node)->T) -> Option<T> {
        self.nodes.get_cloned_ref(&id).map(f)
    }

    fn map_edge<T>(&self, id:EdgeId, f:impl FnOnce(Edge)->T) -> Option<T> {
        self.edges.get_cloned_ref(&id).map(f)
    }

    fn with_node<T>(&self, id:NodeId, f:impl FnOnce(Node)->T) -> Option<T> {
        let out = self.map_node(id,f);
        out.map_none(||warning!(&self.logger,"Trying to access nonexistent node '{id}'"))
    }

    fn with_edge<T>(&self, id:EdgeId, f:impl FnOnce(Edge)->T) -> Option<T> {
        let out = self.map_edge(id,f);
        out.map_none(||warning!(&self.logger,"Trying to access nonexistent edge '{id}'"))
    }

    fn with_edge_map_source<T>(&self, id:EdgeId, f:impl FnOnce(EdgeEndpoint)->T) -> Option<T> {
        self.with_edge(id,|edge| edge.source.borrow().clone().map(f)).flatten()
    }

    fn with_edge_map_target<T>(&self, id:EdgeId, f:impl FnOnce(EdgeEndpoint)->T) -> Option<T> {
        self.with_edge(id,|edge| edge.target.borrow().clone().map(f)).flatten()
    }

    fn edge_source(&self, id:EdgeId) -> Option<EdgeEndpoint> {
        self.with_edge_map_source(id,|endpoint|endpoint)
    }

    fn edge_target(&self, id:EdgeId) -> Option<EdgeEndpoint> {
        self.with_edge_map_target(id,|endpoint|endpoint)
    }

    // FIXME[WD]: This implementation is slow. Node should allow for easy mapping between Crumbs
    //            and edges. Should be part of https://github.com/enso-org/ide/issues/822.
    fn with_input_edge_id<T>
    (&self, id:NodeId, crumbs:&span_tree::Crumbs, f:impl FnOnce(EdgeId)->T) -> Option<T> {
        self.with_node(id,move |node| {
            let mut target_edge_id = None;
            for edge_id in node.in_edges.keys() {
                self.with_edge(edge_id,|edge| {
                    let ok = edge.target().map(|tgt| tgt.port == crumbs) == Some(true);
                    if ok { target_edge_id = Some(edge_id) }
                });
            }
            target_edge_id.map(f)
        }).flatten()
    }

    // FIXME[WD]: This implementation is slow. Node should allow for easy mapping between Crumbs
    //            and edges. Should be part of https://github.com/enso-org/ide/issues/822.
    fn with_output_edge_id<T>
    (&self, id:NodeId, crumbs:&span_tree::Crumbs, f:impl FnOnce(EdgeId)->T) -> Option<T> {
        self.with_node(id,move |node| {
            let mut target_edge_id = None;
            for edge_id in node.out_edges.keys() {
                self.with_edge(edge_id,|edge| {
                    let ok = edge.target().map(|tgt| tgt.port == crumbs) == Some(true);
                    if ok { target_edge_id = Some(edge_id) }
                });
            }
            target_edge_id.map(f)
        }).flatten()
    }

    fn with_edge_source<T>(&self, id:EdgeId, f:impl FnOnce(EdgeEndpoint)->T) -> Option<T> {
        self.with_edge(id,|edge| {
            edge.source.borrow().clone().map(f).map_none(
                || warning!(&self.logger,"Trying to access nonexistent source of the edge {id}.")
            )
        }).flatten()
    }

    fn with_edge_target<T>(&self, id:EdgeId, f:impl FnOnce(EdgeEndpoint)->T) -> Option<T> {
        self.with_edge(id,|edge| {
            edge.target.borrow().clone().map(f).map_none(
                || warning!(&self.logger,"Trying to access nonexistent target of the edge {id}.")
            )
        }).flatten()
    }

    fn with_edge_map_source_node<T>
    (&self, edge_id:EdgeId, f:impl FnOnce(Node,span_tree::Crumbs)->T) -> Option<T> {
        self.with_edge_map_source(edge_id,|t| self.map_node(t.node_id,|node|f(node,t.port))).flatten()
    }

    fn with_edge_map_target_node<T>
    (&self, edge_id:EdgeId, f:impl FnOnce(Node,span_tree::Crumbs)->T) -> Option<T> {
        self.with_edge_map_target(edge_id,|t| self.map_node(t.node_id,|node|f(node,t.port))).flatten()
    }

    fn edge_source_type(&self, edge_id:EdgeId) -> Option<Type> {
        self.with_edge_map_source_node(edge_id,|n,c|n.model.output.port_type(&c)).flatten()
    }

    fn edge_target_type(&self, edge_id:EdgeId) -> Option<Type> {
        self.with_edge_map_target_node(edge_id,|n,c|n.model.input.port_type(&c)).flatten()
    }

    fn edge_hover_type(&self) -> Option<Type> {
        let hover_tgt = self.frp.hover_node_input.value();
        hover_tgt.and_then(|tgt|
            self.with_node(tgt.node_id,|node| node.model.input.port_type(&tgt.port)).flatten()
        )
    }

    /// Return a color for the edge.
    ///
    /// The algorithm works as follow:
    /// 1. We query the type of the currently hovered port if any.
    /// 2. In case the previous point returns None, we query the edge target type, if any.
    /// 3. In case the previous point returns None, we query the edge source type, if any.
    /// 4. In case the previous point returns None, we use the generic type (gray color).
    ///
    /// This might need to be more sophisticated in the case of polymorphic types. For example,
    /// consider the edge source type to be `(a,Number)`, and target to be `(Text,a)`. These unify
    /// to `(Text,Number)`.
    fn edge_color(&self, edge_id:EdgeId) -> color::Lcha {
        // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
        let styles    = StyleWatch::new(&self.scene().style_sheet);
        let edge_type = self.edge_hover_type()
            .or_else(|| self.edge_target_type(edge_id))
            .or_else(|| self.edge_source_type(edge_id));
        let opt_color = edge_type.map(|t|type_coloring::compute(&t,&styles));
        opt_color.unwrap_or_else(|| styles.get_color(theme::code::types::any::selection))
    }

    fn first_detached_edge(&self) -> Option<EdgeId> {
        self.edges.detached_edges_iter().next()
    }

    fn first_detached_edge_source_type(&self) -> Option<Type> {
        self.first_detached_edge().and_then(|edge_id| self.edge_source_type(edge_id))
    }

    #[allow(dead_code)]
    fn first_detached_edge_target_type(&self) -> Option<Type> {
        self.first_detached_edge().and_then(|edge_id| self.edge_target_type(edge_id))
    }

    /// Return a color for the first detached edge.
    pub fn first_detached_edge_color(&self) -> Option<color::Lcha> {
        self.first_detached_edge().map(|t|self.edge_color(t))
    }

    pub fn has_edges_with_detached_targets(&self, node_id:NodeId) -> bool {
        let mut found = false;
        self.with_node(node_id,|node| {
            for edge_id in node.out_edges.keys() {
                if self.with_edge(edge_id, |edge| edge.has_target()) == Some(false) {
                    found = true;
                    break;
                }
            }
        });
        found
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
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl GraphEditor {
    /// Add a new node and returns its ID.
    pub fn add_node(&self) -> NodeId {
        self.frp.add_node.emit(());
        self.frp.output.node_added.value()
    }
}

impl application::View for GraphEditor {
    fn label() -> &'static str {
        "GraphEditor"
    }

    fn new(app:&Application) -> Self {
        new_graph_editor(app)
    }

    fn app(&self) -> &Application {
        &self.model.app
    }

    fn default_shortcuts() -> Vec<application::shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[
          // === Drag ===
            (Press   , ""              , "left-mouse-button" , "node_press")
          , (Release , ""              , "left-mouse-button" , "node_release")
          , (Press   , "!node_editing" , "backspace"         , "remove_selected_nodes")
          , (Press   , ""              , "cmd g"             , "collapse_selected_nodes")

          // === Visualization ===
          , (Press       , "!node_editing" , "space" , "press_visualization_visibility")
          // , (DoublePress , "!node_editing" , "space" , "double_press_visualization_visibility")
          , (Release     , "!node_editing" , "space" , "release_visualization_visibility")

          // === Selection ===
          , (Press   , "" , "shift"                   , "enable_node_multi_select")
          , (Press   , "" , "shift left-mouse-button" , "enable_node_multi_select")
          , (Release , "" , "shift"                   , "disable_node_multi_select")
          , (Release , "" , "shift left-mouse-button" , "disable_node_multi_select")
          , (Press   , "" , "shift ctrl"              , "toggle_node_merge_select")
          , (Release , "" , "shift ctrl"              , "toggle_node_merge_select")
          , (Press   , "" , "shift alt"               , "toggle_node_subtract_select")
          , (Release , "" , "shift alt"               , "toggle_node_subtract_select")
          , (Press   , "" , "shift ctrl alt"          , "toggle_node_inverse_select")
          , (Release , "" , "shift ctrl alt"          , "toggle_node_inverse_select")

          // === Navigation ===
          , (Press       , ""              , "ctrl space"        , "cycle_visualization_for_selected_node")
          , (DoublePress , ""              , "left-mouse-button" , "enter_hovered_node")
          , (Press       , "!node_editing" , "enter"             , "enter_selected_node")
          , (Press       , ""              , "alt enter"         , "exit_node")

          // === Node Editing ===
          , (Press   , "" , "cmd"                   , "edit_mode_on")
          , (Release , "" , "cmd"                   , "edit_mode_off")
          , (Press   , "" , "cmd enter"             , "edit_selected_node")
          , (Press   , "" , "cmd left-mouse-button" , "edit_mode_on")
          , (Release , "" , "cmd left-mouse-button" , "edit_mode_off")
          , (Release , "" , "enter"                 , "stop_editing")

          // === Debug ===
          , (Press , "debug_mode" , "ctrl d"           , "debug_set_test_visualization_data_for_selected_node")
          , (Press , "debug_mode" , "ctrl shift enter" , "debug_push_breadcrumb")
          , (Press , "debug_mode" , "ctrl shift up"    , "debug_pop_breadcrumb")
          , (Press , "debug_mode" , "ctrl n"           , "add_node_at_cursor")

        ]).iter().map(|(a,b,c,d)|Self::self_shortcut_when(*a,*c,*d,*b)).collect()

    }
}

fn enable_disable_toggle
(network:&frp::Network, enable:&frp::Any, disable:&frp::Any, toggle:&frp::Any)
-> frp::Stream<bool> {
    // FIXME: the clone_refs bellow should not be needed.
    let enable  = enable.clone_ref();
    let disable = disable.clone_ref();
    let toggle  = toggle.clone_ref();
    frp::extend! { network
        out        <- any(...);
        on_toggle  <- toggle.map2(&out,|_,t| !t);
        on_enable  <- enable.constant(true);
        on_disable <- disable.constant(false);
        out        <+ on_toggle;
        out        <+ on_enable;
        out        <+ on_disable;
    }
    out.into()
}

#[derive(Clone,Copy,Debug,Eq,PartialEq)]
pub enum SelectionMode {
    Normal,Multi,Merge,Subtract,Inverse
}

impl SelectionMode {
    pub fn single_should_select(self, was_selected:bool) -> bool {
        match self {
            Self::Normal  => true,
            Self::Merge   => true,
            Self::Multi   => !was_selected,
            Self::Inverse => !was_selected,
            _             => false
        }
    }

    pub fn single_should_deselect(self, was_selected:bool) -> bool {
        match self {
            Self::Subtract => true,
            Self::Multi    => was_selected,
            Self::Inverse  => was_selected,
            _              => false
        }
    }
}

impl Default for SelectionMode {
    fn default() -> Self {
        Self::Normal
    }
}

#[allow(unused_parens)]
fn new_graph_editor(app:&Application) -> GraphEditor {
    let world        = &app.display;
    let scene        = world.scene();
    let cursor       = &app.cursor;
    let frp          = Frp::new();
    let model        = GraphEditorModelWithNetwork::new(app,cursor.clone_ref(),&frp);
    let network      = &frp.network;
    let nodes        = &model.nodes;
    let edges        = &model.edges;
    let inputs       = &model.frp;
    let mouse        = &scene.mouse.frp;
    let touch        = &model.touch_state;
    let vis_registry = &model.vis_registry;
    let logger       = &model.logger;
    let out          = &frp.output;

    // FIXME : StyleWatch is unsuitable here, as it was designed as an internal tool for shape system (#795)
    let styles             = StyleWatch::new(&scene.style_sheet);
    let any_type_sel_color = styles.get_color(theme::code::types::any::selection);



    // ========================
    // === Scene Navigation ===
    // ========================

    frp::extend! { network
        no_vis_selected   <- out.some_visualisation_selected.on_false();
        some_vis_selected <- out.some_visualisation_selected.on_true();

        set_navigator_false  <- inputs.set_navigator_disabled.on_true();
        set_navigator_true   <- inputs.set_navigator_disabled.on_false();

        disable_navigator <- any_(&set_navigator_false,&some_vis_selected);
        enable_navigator  <- any_(&set_navigator_true,&no_vis_selected);

        eval_ disable_navigator ( model.navigator.disable() );
        eval_ enable_navigator  ( model.navigator.enable()  );

        out.source.navigator_active <+ inputs.set_navigator_disabled
                                    || out.some_visualisation_selected;
    }



    // =============================
    // === Breadcrumbs Debugging ===
    // =============================

    frp::extend! { network
        eval_ inputs.debug_push_breadcrumb(model.breadcrumbs.debug_push_breadcrumb.emit(None));
        eval_ inputs.debug_pop_breadcrumb (model.breadcrumbs.debug_pop_breadcrumb.emit(()));
    }



    // =============================
    // === Node Level Navigation ===
    // =============================

    frp::extend! { network

        target_to_enter <- inputs.enter_hovered_node.map(f_!(scene.mouse.target.get()));

        // Go level up on background click.
        enter_on_background    <= target_to_enter.map(|target| target.is_background().as_some(()));
        out.source.node_exited <+ enter_on_background;

        // Go level down on node double click.
        enter_node <= target_to_enter.map(|target| target.is_symbol().as_some(()));
        node_switch_to_enter    <- out.node_hovered.sample(&enter_node).unwrap();
        node_to_enter           <- node_switch_to_enter.map(|switch| switch.on().cloned()).unwrap();
        out.source.node_entered <+ node_to_enter;
    }



    // ============================
    // === Project Name Editing ===
    // ============================


    // === Start project name edit ===
    frp::extend! { network
        edit_mode     <- bool(&inputs.edit_mode_off,&inputs.edit_mode_on);
        eval edit_mode ((edit_mode_on) model.breadcrumbs.ide_text_edit_mode.emit(edit_mode_on));
    }


    // === Commit project name edit ===

    frp::extend! { network
        deactivate_breadcrumbs <- any3_(&touch.background.down,
                                        &out.node_editing_started,
                                        &out.node_entered);
        eval_ deactivate_breadcrumbs(model.breadcrumbs.outside_press());
    }



    // =========================
    // === User Interactions ===
    // =========================

    // === Mouse Cursor Transform ===
    frp::extend! { network
        cursor_pos_in_scene <- cursor.frp.screen_position.map(f!((position)
            scene.screen_to_scene_coordinates(*position).xy()
        ));
    }


    // === Selection Target Redirection ===

    frp::extend! { network
        mouse_down_target <- mouse.down_primary.map(f_!(model.scene().mouse.target.get()));
        mouse_up_target   <- mouse.up_primary.map(f_!(model.scene().mouse.target.get()));
        background_up     <= mouse_up_target.map(
            |t| (t==&display::scene::PointerTarget::Background).as_some(())
        );

        eval mouse_down_target([touch,model](target) {
            match target {
                display::scene::PointerTarget::Background  => touch.background.down.emit(()),
                display::scene::PointerTarget::Symbol {..} => {
                    if let Some(target) = model.scene().shapes.get_mouse_target(*target) {
                        target.mouse_down().emit(());
                    }
                }
            }
        });

        eval mouse_up_target([model](target) {
            match target {
                display::scene::PointerTarget::Background  => {} // touch.background.up.emit(()),
                display::scene::PointerTarget::Symbol {..} => {
                    if let Some(target) = model.scene().shapes.get_mouse_target(*target) {
                        target.mouse_up().emit(());
                    }
                }
            }
        });
    }


    // === Node Editing ===

    frp::extend! { network
        node_in_edit_mode     <- out.node_being_edited.map(|n| n.is_some());
        edit_mode             <- bool(&inputs.edit_mode_off,&inputs.edit_mode_on);
        node_to_edit          <- touch.nodes.down.gate(&edit_mode);
        edit_node             <- any(&node_to_edit,&inputs.edit_node);
        stop_edit_on_bg_click <- touch.background.selected.gate(&node_in_edit_mode);
        stop_edit             <- any(&stop_edit_on_bg_click,&inputs.stop_editing);
        edit_switch           <- edit_node.gate(&node_in_edit_mode);
        node_being_edited     <- out.node_being_edited.map(|n| n.unwrap_or_default());

        // The "finish" events must be emitted before "start", to properly cover the "switch" case.
        out.source.node_editing_finished <+ node_being_edited.sample(&stop_edit);
        out.source.node_editing_finished <+ node_being_edited.sample(&edit_switch);
        out.source.node_editing_started  <+ edit_node;

        out.source.node_being_edited <+ out.node_editing_started.map(|n| Some(*n));;
        out.source.node_being_edited <+ out.node_editing_finished.constant(None);
        out.source.node_editing      <+ out.node_being_edited.map(|t|t.is_some());

        out.source.node_edit_mode       <+ edit_mode;
        out.source.nodes_labels_visible <+ out.node_edit_mode || node_in_edit_mode;

        eval out.node_editing_started ([model] (id) {
            if let Some(node) = model.nodes.get_cloned_ref(&id) {
                node.model.input.frp.set_edit_mode(true);
            }
        });
        eval out.node_editing_finished ([model](id) {
            if let Some(node) = model.nodes.get_cloned_ref(&id) {
                node.model.input.set_edit_mode(false);
            }
        });
    }


    // === Cursor Selection ===
    frp::extend! { network

    mouse_on_down_position <- mouse.position.sample(&mouse.down_primary);
    selection_size_down    <- mouse.position.map2(&mouse_on_down_position,|m,n|{m-n});
    selection_size         <- selection_size_down.gate(&touch.background.is_down);

    on_press_style   <- mouse.down_primary . constant(cursor::Style::new_press());
    on_release_style <- mouse.up_primary . constant(cursor::Style::default());


    cursor_selection_start <- selection_size.map(|p| cursor::Style::new_with_all_fields_default().press().box_selection(Vector2::new(p.x,p.y)));
    cursor_selection_end   <- mouse.up_primary . constant(cursor::Style::default());
    cursor_selection       <- any (cursor_selection_start, cursor_selection_end);

    cursor_press     <- any (on_press_style, on_release_style);


    }


    // === Node Select ===

    frp::extend! { network

    deselect_all_nodes <- any_(...);

    let multi_select_flag = enable_disable_toggle
        ( network
        , &inputs.enable_node_multi_select
        , &inputs.disable_node_multi_select
        , &inputs.toggle_node_multi_select
        );

    let merge_select_flag = enable_disable_toggle
        ( network
        , &inputs.enable_node_merge_select
        , &inputs.disable_node_merge_select
        , &inputs.toggle_node_merge_select
        );

    let subtract_select_flag = enable_disable_toggle
        ( network
        , &inputs.enable_node_subtract_select
        , &inputs.disable_node_subtract_select
        , &inputs.toggle_node_subtract_select
        );

    let inverse_select_flag = enable_disable_toggle
        ( network
        , &inputs.enable_node_inverse_select
        , &inputs.disable_node_inverse_select
        , &inputs.toggle_node_inverse_select
        );


    selection_mode <- all_with4
        (&multi_select_flag,&merge_select_flag,&subtract_select_flag,&inverse_select_flag,
        |multi,merge,subtract,inverse| {
            if      *multi    { SelectionMode::Multi }
            else if *merge    { SelectionMode::Merge }
            else if *subtract { SelectionMode::Subtract }
            else if *inverse  { SelectionMode::Inverse }
            else              { SelectionMode::Normal }
        }
    );

    edit_mode               <- bool(&inputs.edit_mode_off,&inputs.edit_mode_on);
    node_to_select_non_edit <- touch.nodes.selected.gate_not(&edit_mode).gate_not(&out.some_edge_endpoints_unset);
    node_to_select_edit     <- touch.nodes.down.gate(&edit_mode);
    node_to_select          <- any(node_to_select_non_edit,node_to_select_edit,inputs.select_node);
    node_was_selected       <- node_to_select.map(f!((id) model.nodes.selected.contains(id)));

    should_select <- node_to_select.map3(&selection_mode,&node_was_selected,
        |_,mode,was_selected| mode.single_should_select(*was_selected)
    );

    should_deselect <- node_to_select.map3(&selection_mode,&node_was_selected,
        |_,mode,was_selected| mode.single_should_deselect(*was_selected)
    );

    keep_selection          <- selection_mode.map(|t| *t != SelectionMode::Normal);
    deselect_on_select      <- node_to_select.gate_not(&keep_selection);
    deselect_all_nodes      <+ deselect_on_select;
    deselect_all_nodes      <+ inputs.deselect_all_nodes;

    deselect_on_bg_press    <- touch.background.selected.gate_not(&keep_selection);
    deselect_all_nodes      <+ deselect_on_bg_press;
    all_nodes_to_deselect   <= deselect_all_nodes.map(f_!(model.nodes.selected.mem_take()));
    out.source.node_deselected <+ all_nodes_to_deselect;

    node_selected           <- node_to_select.gate(&should_select);
    node_deselected         <- node_to_select.gate(&should_deselect);
    out.source.node_selected   <+ node_selected;
    out.source.node_deselected <+ node_deselected;
    }


    // === Add Node ===
    frp::extend! { network

    node_pointer_style <- source::<cursor::Style>();

    let node_input_touch  = TouchNetwork::<EdgeEndpoint>::new(&network,&mouse);
    let node_output_touch = TouchNetwork::<EdgeEndpoint>::new(&network,&mouse);
    node_expression_set <- source();
    out.source.node_expression_set <+ node_expression_set;

    on_output_connect_drag_mode   <- node_output_touch.down.constant(true);
    on_output_connect_follow_mode <- node_output_touch.selected.constant(false);
    on_input_connect_drag_mode    <- node_input_touch.down.constant(true);
    on_input_connect_follow_mode  <- node_input_touch.selected.constant(false);

    on_connect_drag_mode   <- any(on_output_connect_drag_mode,on_input_connect_drag_mode);
    on_connect_follow_mode <- any(on_output_connect_follow_mode,on_input_connect_follow_mode);
    connect_drag_mode      <- any(on_connect_drag_mode,on_connect_follow_mode);

    on_detached_edge    <- any(&inputs.on_some_edges_targets_unset,&inputs.on_some_edges_sources_unset);
    has_detached_edge   <- bool(&out.on_all_edges_endpoints_set,&on_detached_edge);

    eval node_input_touch.down ((target)   model.frp.press_node_input.emit(target));
    eval node_output_touch.down ((target)  model.frp.press_node_output.emit(target));


    // === Edge interactions  ===

    edge_mouse_down <- source::<EdgeId>();
    edge_over       <- source::<EdgeId>();
    edge_out        <- source::<EdgeId>();
    edge_hover      <- source::<Option<EdgeId>>();

    eval  edge_over((edge_id) edge_hover.emit(Some(*edge_id)));
    eval_ edge_out(edge_hover.emit(None));

    edge_over_pos <- map2(&cursor_pos_in_scene,&edge_hover,|pos, edge_id|
        edge_id.map(|id| (id, *pos))
    ).unwrap();

    // We do not want edge hover to occur for detached edges.
    set_edge_hover <- edge_over_pos.gate_not(&has_detached_edge);

    eval set_edge_hover ([model]((edge_id,pos)) {
         if let Some(edge) = model.edges.get_cloned_ref(edge_id){
            edge.frp.hover_position.emit(Some(*pos));
            edge.frp.redraw.emit(());
        }
    });

    remove_split <- any(&edge_out,&edge_mouse_down);
    eval remove_split ([model](edge_id) {
         if let Some(edge) = model.edges.get_cloned_ref(edge_id){
            edge.frp.hover_position.emit(None);
            edge.frp.redraw.emit(());
        }
    });
    edge_click <- map2(&edge_mouse_down,&cursor_pos_in_scene,|edge_id,pos|(*edge_id,*pos));
    valid_edge_disconnect_click <- edge_click.gate_not(&has_detached_edge);

    edge_is_source_click <- valid_edge_disconnect_click.map(f!([model]((edge_id,pos)) {
        if let Some(edge) = model.edges.get_cloned_ref(edge_id){
            edge.port_to_detach_for_position(*pos) == component::edge::PortType::OutputPort
        } else {
            false
        }
    }));

    edge_source_click <- valid_edge_disconnect_click.gate(&edge_is_source_click);
    edge_target_click <- valid_edge_disconnect_click.gate_not(&edge_is_source_click);

    on_edge_source_unset <= edge_source_click.map(f!(((id,_)) model.with_edge_source(*id,|t|(*id,t))));
    on_edge_target_unset <= edge_target_click.map(f!(((id,_)) model.with_edge_target(*id,|t|(*id,t))));
    out.source.on_edge_source_unset <+ on_edge_source_unset;
    out.source.on_edge_target_unset <+ on_edge_target_unset;
    }


    // === Edge creation  ===

    frp::extend! { network

    output_down <- node_output_touch.down.constant(());
    input_down  <- node_input_touch.down.constant(());

    has_detached_edge_on_output_down <- has_detached_edge.sample(&inputs.hover_node_output);

    port_input_mouse_up  <- inputs.hover_node_input.sample(&mouse.up_primary).unwrap();
    port_output_mouse_up <- inputs.hover_node_output.sample(&mouse.up_primary).unwrap();

    attach_all_edge_inputs  <- any (port_input_mouse_up, inputs.press_node_input, inputs.set_detached_edge_targets);
    attach_all_edge_outputs <- any (port_output_mouse_up, inputs.press_node_output, inputs.set_detached_edge_sources);

    create_edge_from_output <- node_output_touch.down.gate_not(&has_detached_edge_on_output_down);
    create_edge_from_input  <- node_input_touch.down.map(|value| value.clone());


    // === Edge creation  ===

    on_new_edge    <- any(&output_down,&input_down);
    deselect_edges <- on_new_edge.gate_not(&keep_selection);
    eval_ deselect_edges ( model.clear_all_detached_edges() );

    new_output_edge <- create_edge_from_output.map(f_!([model,edge_mouse_down,edge_over,edge_out] {
        Some(model.new_edge_from_output(&edge_mouse_down,&edge_over,&edge_out))
    })).unwrap();
    new_input_edge <- create_edge_from_input.map(f!([model,edge_mouse_down,edge_over,edge_out]((target)){
        if model.is_node_connected_at_input(target.node_id,&target.port) {
            return None
        };
        Some(model.new_edge_from_input(&edge_mouse_down,&edge_over,&edge_out))
    })).unwrap();

    out.source.on_edge_add <+ new_output_edge;
    new_edge_source <- new_output_edge.map2(&node_output_touch.down, move |id,target| (*id,target.clone()));
    out.source.on_edge_source_set <+ new_edge_source;

    out.source.on_edge_add <+ new_input_edge;
    new_edge_target <- new_input_edge.map2(&node_input_touch.down, move |id,target| (*id,target.clone()));
    out.source.on_edge_target_set <+ new_edge_target;



    // ======================
    // === Node Creation  ===
    // ======================

    let add_node_at_cursor = inputs.add_node_at_cursor.clone_ref();
    add_node <- any (inputs.add_node,add_node_at_cursor);
    new_node <- add_node.map(f_!([model,node_pointer_style,out] {
        model.new_node(&node_pointer_style,&node_output_touch.down,&node_input_touch.down,&out)
    }));
    out.source.node_added <+ new_node;

    node_with_position <- add_node_at_cursor.map3(&new_node,&cursor_pos_in_scene,|_,id,pos| (*id,*pos));
    out.source.node_position_set         <+ node_with_position;
    out.source.node_position_set_batched <+ node_with_position;

    // === Event Propagation ===
    // See the docs of `Node` to learn about how the graph - nodes event propagation works.

    _eval <- all_with(&out.node_hovered,&edit_mode,f!([model](tgt,e)
        if let Some(tgt) = tgt {
            model.with_node(tgt.value,|t| t.model.input.set_edit_ready_mode(*e && tgt.is_on()));
        }
    ));
    _eval <- all_with(&out.node_hovered,&out.some_edge_targets_unset,f!([model](tgt,ok)
        if let Some(tgt) = tgt {
            let node_id        = tgt.value;
            let edge_tp        = model.first_detached_edge_source_type();
            let is_edge_source = model.has_edges_with_detached_targets(node_id);
            let is_active      = *ok && !is_edge_source && tgt.is_on();
            model.with_node(node_id,|t| t.model.input.set_ports_active(is_active,edge_tp));
        }
    ));
    }


    // === Node Actions ===

    frp::extend! { network
        freeze_edges <= out.node_action_freeze.map (f!([model]((node_id,is_frozen)) {
            let edges = model.node_in_edges(node_id);
            edges.into_iter().map(|edge_id| (edge_id,*is_frozen)).collect_vec()
        }));

        eval freeze_edges (((edge_id,is_frozen)) model.set_edge_freeze(edge_id,*is_frozen) );
    }


    // === Edge Connect ===

    frp::extend! { network

    out.source.on_edge_source_set <+ inputs.set_edge_source;
    out.source.on_edge_target_set <+ inputs.set_edge_target;

    let endpoints            = inputs.connect_nodes.clone_ref();
    edge                    <- endpoints . map(f_!(model.new_edge_from_output(&edge_mouse_down,&edge_over,&edge_out)));
    new_edge_source         <- endpoints . _0() . map2(&edge, |t,id| (*id,t.clone()));
    new_edge_target         <- endpoints . _1() . map2(&edge, |t,id| (*id,t.clone()));
    out.source.on_edge_add      <+ edge;
    out.source.on_edge_source_set <+ new_edge_source;
    out.source.on_edge_target_set <+ new_edge_target;

    detached_edges_without_targets <= attach_all_edge_inputs.map(f_!(model.take_edges_with_detached_targets()));
    detached_edges_without_sources <= attach_all_edge_outputs.map(f_!(model.take_edges_with_detached_sources()));

    new_edge_target <- detached_edges_without_targets.map2(&attach_all_edge_inputs, |id,t| (*id,t.clone()));
    out.source.on_edge_target_set <+ new_edge_target;
    new_edge_source <- detached_edges_without_sources.map2(&attach_all_edge_outputs, |id,t| (*id,t.clone()));
    out.source.on_edge_source_set <+ new_edge_source;

    on_new_edge_source <- new_edge_source.constant(());
    on_new_edge_target <- new_edge_target.constant(());

    overlapping_edges       <= out.on_edge_target_set._1().map(f!((t) model.overlapping_edges(t)));
    out.source.on_edge_drop <+ overlapping_edges;

    drop_on_bg_up  <- background_up.gate(&connect_drag_mode);
    drop_edges     <- any (drop_on_bg_up,touch.background.down);
    edge_to_drop_without_targets <= drop_edges.map(f_!(model.take_edges_with_detached_targets()));
    edge_to_drop_without_sources <= drop_edges.map(f_!(model.take_edges_with_detached_sources()));
    edge_to_drop <- any(edge_to_drop_without_targets,edge_to_drop_without_sources);
    eval edge_to_drop ((id) model.remove_edge(id));

    }

    //
    // // === Disabling self-connections ===
    //
    // frp::extend! { network
    //     node_to_disable <= out.on_edge_only_target_not_set.map(f!((id) model.with_edge_source(*id,|t|t.node_id)));
    //     eval node_to_disable ((id) model.with_node(*id,|node| node.model.input.set_ports_active(false,None)));
    //
    // }


    // === Remove Node ===
    frp::extend! { network

    all_nodes       <= inputs.remove_all_nodes      . map(f_!(model.all_nodes()));
    selected_nodes  <= inputs.remove_selected_nodes . map(f_!(model.selected_nodes()));
    nodes_to_remove <- any (all_nodes, selected_nodes);
    eval nodes_to_remove ((node_id) inputs.remove_all_node_edges.emit(node_id));

    out.source.node_removed <+ nodes_to_remove;
    }


    // === Collapse Nodes ===
    frp::extend! { network
    // TODO [mwu] https://github.com/enso-org/ide/issues/760
    //   This is currently the provisional code to enable collapse nodes refactoring. While the APIs
    //   are as-intended, their behavior isn't. Please refer to the issue for details.
    let empty_id       = NodeId::default();
    let model_clone    = model.clone_ref();
    nodes_to_collapse <- inputs.collapse_selected_nodes . map(move |_|
        (model_clone.selected_nodes(),empty_id)
    );
    out.source.nodes_collapsed <+ nodes_to_collapse;
    }


    // === Set Node Expression ===
    frp::extend! { network

    set_node_expression_string  <- inputs.set_node_expression.map(|(id,expr)| (*id,expr.code.clone()));
    out.source.node_expression_set <+ set_node_expression_string;

    }


    // === Set Node Error ===
    frp::extend! { network

    eval inputs.set_node_error_status([model]((node_id, error)) {
        if let Some(node) = model.nodes.get_cloned_ref(node_id) {
            node.set_error.emit(error)
        }
    });

    }



    // ==================
    // === Move Nodes ===
    // ==================
    frp::extend! { network

    mouse_pos <- mouse.position.map(|p| Vector2(p.x,p.y));

    // === Discovering drag targets ===

    let node_down      = touch.nodes.down.clone_ref();
    let node_is_down   = touch.nodes.is_down.clone_ref();
    node_in_edit_mode <- node_down.map2(&out.node_being_edited,|t,s| Some(*t) == *s);
    node_was_selected <- node_down.map(f!((id) model.nodes.selected.contains(id)));
    tgts_if_non_sel   <- node_down.map(|id|vec![*id]).gate_not(&node_was_selected);
    tgts_if_sel       <- node_down.map(f_!(model.nodes.selected.items())).gate(&node_was_selected);
    tgts_if_non_edit  <- any(tgts_if_non_sel,tgts_if_sel).gate_not(&node_in_edit_mode);
    tgts_if_edit      <- node_down.map(|_|default()).gate(&node_in_edit_mode);
    drag_tgts         <- any(tgts_if_non_edit,tgts_if_edit);
    any_drag_tgt      <- drag_tgts.map(|t|!t.is_empty());
    node_pos_on_down  <- node_down.map(f!((id) model.node_position(id)));
    mouse_pos_on_down <- mouse_pos.sample(&node_down);
    mouse_pos_diff    <- mouse_pos.map2(&mouse_pos_on_down,|t,s|t-s).gate(&node_is_down);
    node_pos_diff     <- mouse_pos_diff.map(f!([scene](t) t / scene.camera().zoom()));
    node_tgt_pos_rt   <- node_pos_diff.map2(&node_pos_on_down,|t,s|t+s);
    just_pressed      <- bool (&node_tgt_pos_rt,&node_pos_on_down);
    node_tgt_pos_rt   <- any  (&node_tgt_pos_rt,&node_pos_on_down);


    // === Snapping ===

    eval drag_tgts ((ids) model.disable_grid_snapping_for(ids));
    let node_tgt_pos_anim = DEPRECATED_Animation::<Vector2<f32>>::new(&network);
    let x_snap_strength   = DEPRECATED_Tween::new(&network);
    let y_snap_strength   = DEPRECATED_Tween::new(&network);
    x_snap_strength.set_duration(300.0);
    y_snap_strength.set_duration(300.0);

    _eval <- node_tgt_pos_rt.map2(&just_pressed,
        f!([model,x_snap_strength,y_snap_strength,node_tgt_pos_anim](pos,just_pressed) {
            let snapped = model.nodes.check_grid_magnet(*pos);
            let x = snapped.x.unwrap_or(pos.x);
            let y = snapped.y.unwrap_or(pos.y);
            x_snap_strength.set_target_value(if snapped.x.is_none() { 0.0 } else { 1.0 });
            y_snap_strength.set_target_value(if snapped.y.is_none() { 0.0 } else { 1.0 });
            node_tgt_pos_anim.set_target_value(Vector2::new(x,y));
            if *just_pressed {
                node_tgt_pos_anim.set_target_value(*pos);
                x_snap_strength.skip();
                y_snap_strength.skip();
                node_tgt_pos_anim.skip();
            }
    }));

    node_tgt_pos <- all_with4
        ( &node_tgt_pos_rt
        , &node_tgt_pos_anim.value
        , &x_snap_strength.value
        , &y_snap_strength.value
        , |rt,snap,xw,yw| {
            let w     = Vector2(*xw,*yw);
            let w_inv = Vector2(1.0,1.0) - w;
            rt.component_mul(&w_inv) + snap.component_mul(&w)
        });


    // === Update All Target Nodes Positions ===

    main_tgt_pos_prev <- node_tgt_pos.previous();
    main_tgt_pos_diff <- node_tgt_pos.map2(&main_tgt_pos_prev,|t,s|t-s).gate_not(&just_pressed);
    drag_tgt          <= drag_tgts.sample(&main_tgt_pos_diff);
    tgt_new_pos       <- drag_tgt.map2(&main_tgt_pos_diff,f!((id,tx) model.node_pos_mod(id,*tx)));
    out.source.node_position_set <+ tgt_new_pos;


    // === Batch Update ===

    after_drag             <- touch.nodes.up.gate_not(&just_pressed);
    tgt_after_drag         <= drag_tgts.sample(&after_drag);
    tgt_after_drag_new_pos <- tgt_after_drag.map(f!([model](id)(*id,model.node_position(id))));
    out.source.node_position_set_batched <+ tgt_after_drag_new_pos;


    // === Mouse style ===

    node_down_on_drag   <- node_down.gate(&any_drag_tgt);
    cursor_on_drag_down <- node_down_on_drag.map(|_| cursor::Style::new_with_all_fields_default().press());
    cursor_on_drag_up   <- touch.nodes.up.map(|_| cursor::Style::default());
    pointer_on_drag     <- any (&cursor_on_drag_down,&cursor_on_drag_up);


    // === Set Node Position ===

    out.source.node_position_set         <+ inputs.set_node_position;
    out.source.node_position_set_batched <+ inputs.set_node_position;
    eval out.node_position_set (((id,pos)) model.set_node_position(id,*pos));

    }


    // === Set Expression Type ===
    frp::extend! { network

    node_to_refresh <- inputs.set_expression_usage_type.map(f!([model]((node_id,ast_id,maybe_type)) {
        model.set_node_expression_usage_type(*node_id,*ast_id,maybe_type.clone());
        *node_id
    }));
    edges_to_refresh <= node_to_refresh.map(f!([nodes](node_id)
         nodes.get_cloned_ref(node_id).map(|node| node.all_edges())
    )).unwrap();
    eval edges_to_refresh ((edge) model.refresh_edge_position(*edge));

    }


    // === Move Edges ===

    frp::extend! { network

    detached_edge           <- any(&inputs.on_some_edges_targets_unset,&inputs.on_some_edges_sources_unset);
    update_edge             <- any(detached_edge,on_new_edge_source,on_new_edge_target);
    cursor_pos_on_update    <- cursor_pos_in_scene.sample(&update_edge);
    edge_refresh_cursor_pos <- any(cursor_pos_on_update,cursor_pos_in_scene);

    is_hovering_output <- inputs.hover_node_output.map(|target| target.is_some()).sampler();
    hover_node         <- inputs.hover_node_output.unwrap();

    edge_refresh_on_node_hover        <- all(edge_refresh_cursor_pos,hover_node).gate(&is_hovering_output);
    edge_refresh_cursor_pos_no_hover  <- edge_refresh_cursor_pos.gate_not(&is_hovering_output);
    edge_refresh_cursor_pos_on_hover  <- edge_refresh_on_node_hover._0();

    refresh_target      <- any(&edge_refresh_cursor_pos_on_hover,&edge_refresh_cursor_pos_no_hover);
    let refresh_source  = edge_refresh_cursor_pos_no_hover.clone_ref();
    snap_source_to_node <- edge_refresh_on_node_hover._1();

    eval refresh_target ([edges](position) {
       edges.detached_target.for_each(|id| {
            if let Some(edge) = edges.get_cloned_ref(id) {
                edge.view.frp.target_position.emit(position.xy());
                edge.view.frp.redraw.emit(());
            }
        });
    });

    eval refresh_source ([edges,model](position) {
        edges.detached_source.for_each(|edge_id| {
            if let Some(edge) = edges.get_cloned_ref(edge_id) {
                edge.view.frp.source_width.emit(cursor::DEFAULT_RADIUS);
                edge.view.frp.source_height.emit(cursor::DEFAULT_RADIUS);
                edge.view.frp.target_position.emit(-position.xy());
                edge.view.frp.redraw.emit(());
                edge.mod_position(|p| {
                    p.x = position.x;
                    p.y = position.y;
                });
                model.refresh_edge_position(*edge_id);
            }
        });
    });

    eval snap_source_to_node ([nodes,edges,model](target) {
        edges.detached_source.for_each(|edge_id| {
            if let Some(node) = nodes.get_cloned_ref(&target.node_id) {
                if let Some(edge) = edges.get_cloned_ref(edge_id) {
                    let node_width  = node.view.model.width();
                    let node_height = node.view.model.height();
                    let node_pos    = node.position();

                    edge.view.frp.source_width.emit(node_width);
                    edge.view.frp.source_height.emit(node_height);
                    edge.view.frp.target_position.emit(-node_pos.xy());
                    edge.view.frp.redraw.emit(());
                    edge.mod_position(|p| {
                        p.x = node_pos.x + node_width/2.0;
                        p.y = node_pos.y;
                    });
                    model.refresh_edge_position(*edge_id);
                }
            }
        });
    });

    }


   // === Vis Set ===
   frp::extend! { network

   def _update_vis_data = inputs.set_visualization.map(f!([logger,nodes,vis_registry]((node_id,vis_path)) {
       match (&nodes.get_cloned_ref(node_id), vis_path) {
            (Some(node), Some(vis_path)) => {
                let vis_definition = vis_registry.definition_from_path(vis_path);
                node.model.visualization.frp.set_visualization.emit(vis_definition);
            },
            (Some(node), None) => node.model.visualization.frp.set_visualization.emit(None),
             _                 => warning!(logger,"Failed to get node: {node_id:?}"),

       }
   }));


   // === Vis Selection ===
    frp::extend! { network
        eval out.on_visualization_select ([model](switch) {
            if switch.is_on() {
                model.visualisations.selected.insert(switch.value);
            } else {
                model.visualisations.selected.remove(&switch.value);
            }
        });

        out.source.some_visualisation_selected <+  out.on_visualization_select.map(f_!([model] {
            !model.visualisations.selected.is_empty()
        }));
    };


    // === Vis Update Data ===

    // TODO remove this once real data is available.
    let sample_data_generator = MockDataGenerator3D::default();
    def _set_dumy_data = inputs.debug_set_test_visualization_data_for_selected_node.map(f!([nodes,inputs](_) {
        for node_id in &*nodes.selected.raw.borrow() {
            let data    = Rc::new(sample_data_generator.generate_data()); // FIXME: why rc?
            let content = serde_json::to_value(data).unwrap();
            let data    = visualization::Data::from(content);
            inputs.set_visualization_data.emit((*node_id,data));
        }
    }));

    def _set_data = inputs.set_visualization_data.map(f!([nodes]((node_id,data)) {
         if let Some(node) = nodes.get_cloned(node_id) {
             node.model.visualization.frp.set_data.emit(data);
         }
     }));

     nodes_to_cycle <= inputs.cycle_visualization_for_selected_node.map(f_!(model.selected_nodes()));
     node_to_cycle  <- any(nodes_to_cycle,inputs.cycle_visualization);

    let cycle_count = Rc::new(Cell::new(0));
    def _cycle_visualization = node_to_cycle.map(f!([inputs,vis_registry,logger](node_id) {
        let visualizations = vis_registry.valid_sources(&"Any".into());
        cycle_count.set(cycle_count.get() % visualizations.len());
        if let Some(vis) = visualizations.get(cycle_count.get()) {
            let path = vis.signature.path.clone();
            inputs.set_visualization.emit((*node_id,Some(path)));
        } else {
            warning!(logger,"Failed to get visualization while cycling.");
        };
        cycle_count.set(cycle_count.get() + 1);
    }));


    // === Visualization toggle ===
    //
    // Algorithm:
    //     - Press key. If all selected nodes have enabled vis, disable them.
    //     - If not, enable vis on missing nodes.
    //     - Release key. If the time passed from key press was short, do nothing.
    //     - If it was long, disable vis which were disabled (preview mode).

    let viz_press_ev      = inputs.press_visualization_visibility.clone_ref();
    let viz_d_press_ev    = inputs.double_press_visualization_visibility.clone_ref();
    let viz_release       = inputs.release_visualization_visibility.clone_ref();
    viz_pressed          <- bool(&viz_release,&viz_press_ev);
    viz_was_pressed      <- viz_pressed.previous();
    viz_press            <- viz_press_ev.gate_not(&viz_was_pressed);
    viz_press_time       <- viz_press   . map(|_| web::performance().now() as f32);
    viz_release_time     <- viz_release . map(|_| web::performance().now() as f32);
    viz_press_time_diff  <- viz_release_time.map2(&viz_press_time,|t1,t0| t1-t0);
    viz_preview_mode     <- viz_press_time_diff.map(|t| *t > VIZ_PREVIEW_MODE_TOGGLE_TIME_MS);
    viz_preview_mode_end <- viz_release.gate(&viz_preview_mode);
    viz_tgt_nodes        <- viz_press.map(f_!(model.selected_nodes()));
    viz_tgt_nodes_off    <- viz_tgt_nodes.map(f!([model](node_ids) {
        node_ids.iter().cloned().filter(|node_id| {
            model.nodes.get_cloned_ref(node_id)
                .map(|node| !node.model.visualization.is_active())
                .unwrap_or_default()
        }).collect_vec()
    }));

    viz_tgt_nodes_all_on <- viz_tgt_nodes_off.map(|t| t.is_empty());
    viz_enable_by_press  <= viz_tgt_nodes.gate_not(&viz_tgt_nodes_all_on);
    viz_enable           <- any(viz_enable_by_press,inputs.enable_visualization);
    viz_disable          <= viz_tgt_nodes.gate(&viz_tgt_nodes_all_on);
    viz_preview_disable  <= viz_tgt_nodes_off.sample(&viz_preview_mode_end);
    viz_fullscreen_on    <= viz_d_press_ev.map(f_!(model.last_selected_node()));

    eval viz_enable          ((id) model.enable_visualization(id));
    eval viz_disable         ((id) model.disable_visualization(id));
    eval viz_preview_disable ((id) model.disable_visualization(id));
    eval viz_fullscreen_on   ((id) model.enable_visualization_fullscreen(id));


    // === Register Visualization ===

    def _register_visualization = inputs.register_visualization.map(f!([vis_registry](handle) {
        if let Some(handle) = handle {
            vis_registry.add(handle);
        }
    }));


    // === Entering and Exiting Nodes ===

    node_to_enter           <= inputs.enter_selected_node.map(f_!(model.last_selected_node()));
    out.source.node_entered <+ node_to_enter;
    removed_edges_on_enter  <= out.node_entered.map(f_!(model.model.clear_all_detached_edges()));
    out.source.node_exited  <+ inputs.exit_node;
    removed_edges_on_exit   <= out.node_exited.map(f_!(model.model.clear_all_detached_edges()));
    out.source.on_edge_drop <+ any(removed_edges_on_enter,removed_edges_on_exit);



    // ==================
    // === Edge Binds ===
    // ==================

    // === Source / Target ===

    eval out.on_edge_source_set   (((id,tgt)) model.set_edge_source(*id,tgt));
    eval out.on_edge_target_set   (((id,tgt)) model.set_edge_target(*id,tgt));

    eval out.on_edge_target_set   (((id,tgt)) model.set_endpoint_connection_status(*id,tgt,true));
    eval out.on_edge_target_unset (((id,tgt)) model.set_endpoint_connection_status(*id,tgt,false));

    eval out.on_edge_source_unset (((id,_)) model.remove_edge_source(*id));
    eval out.on_edge_target_unset (((id,_)) model.remove_edge_target(*id));

    is_only_tgt_not_set <-
        out.on_edge_source_set.map(f!(((id,_)) model.with_edge_map_target(*id,|_|()).is_none()));
    out.source.on_edge_source_set_with_target_not_set <+ out.on_edge_source_set.gate(&is_only_tgt_not_set);
    out.source.on_edge_only_target_not_set <+ out.on_edge_source_set_with_target_not_set._0();
    out.source.on_edge_only_target_not_set <+ out.on_edge_target_unset._0();

    is_only_src_not_set <-
        out.on_edge_target_set.map(f!(((id,_)) model.with_edge_map_source(*id,|_|()).is_none()));
    out.source.on_edge_target_set_with_source_not_set <+ out.on_edge_target_set.gate(&is_only_src_not_set);
    out.source.on_edge_only_source_not_set <+ out.on_edge_target_set_with_source_not_set._0();
    out.source.on_edge_only_source_not_set <+ out.on_edge_source_unset._0();

    eval out.on_edge_source_set   (((id,_)) model.refresh_edge_color(*id));
    eval out.on_edge_target_set   (((id,_)) model.refresh_edge_color(*id));
    eval out.on_edge_source_unset (((id,_)) model.refresh_edge_color(*id));
    eval out.on_edge_target_unset (((id,_)) model.refresh_edge_color(*id));

    edge_to_refresh_on_hover <= out.hover_node_input.map(f_!(model.edges_with_detached_targets()));
    eval edge_to_refresh_on_hover ((id) model.refresh_edge_color(*id));


    some_edge_sources_unset   <- out.on_all_edges_sources_set ?? out.on_some_edges_sources_unset;
    some_edge_targets_unset   <- out.on_all_edges_targets_set ?? out.on_some_edges_targets_unset;
    some_edge_endpoints_unset <- out.some_edge_targets_unset  || out.some_edge_sources_unset;
    out.source.some_edge_sources_unset    <+ some_edge_sources_unset;
    out.source.some_edge_targets_unset    <+ some_edge_targets_unset;
    out.source.some_edge_endpoints_unset  <+ some_edge_endpoints_unset;
    out.source.on_all_edges_endpoints_set <+ out.some_edge_endpoints_unset.on_false();


    // === Endpoints ===

    edge_source_drop <= out.on_edge_drop.map(f!((id) model.edge_source(*id).map(|t|(*id,t))));
    edge_target_drop <= out.on_edge_drop.map(f!((id) model.edge_target(*id).map(|t|(*id,t))));

    edge_endpoint_set                 <- any(out.on_edge_source_set,out.on_edge_target_set)._0();
    both_endpoints_set                <- edge_endpoint_set.map(f!((id) model.is_connection(id)));
    new_edge_with_both_endpoints_set  <- edge_endpoint_set.gate(&both_endpoints_set);
    out.source.on_edge_endpoints_set  <+ new_edge_with_both_endpoints_set;
    out.source.on_edge_endpoint_set   <+ any(out.on_edge_source_set,out.on_edge_target_set);
    out.source.on_edge_endpoint_unset <+ any(out.on_edge_source_unset,out.on_edge_target_unset);
    out.source.on_edge_endpoint_unset <+ any(edge_source_drop,edge_target_drop);


    // === Drop ===

    eval out.on_edge_drop    ((id) model.remove_edge(id));



    // ===================
    // === Other Binds ===
    // ===================

    eval out.node_selected   ((id) model.select_node(id));
    eval out.node_deselected ((id) model.deselect_node(id));
    eval out.node_removed    ((id) model.remove_node(id));

    eval inputs.set_node_expression (((id,expr)) model.set_node_expression(id,expr));
    port_to_refresh <= inputs.set_node_expression.map(f!(((id,_))model.node_in_edges(id)));
    eval port_to_refresh ((id) model.set_edge_target_connection_status(*id,true));


    // === Remove implementation ===
    out.source.node_removed <+ inputs.remove_node;
    }


    // === Remove Edge ===
    frp::extend! { network

    rm_input_edges       <- any (inputs.remove_all_node_edges, inputs.remove_all_node_input_edges);
    rm_output_edges      <- any (inputs.remove_all_node_edges, inputs.remove_all_node_output_edges);
    input_edges_to_rm    <= rm_input_edges  . map(f!((node_id) model.node_in_edges(node_id)));
    output_edges_to_rm   <= rm_output_edges . map(f!((node_id) model.node_out_edges(node_id)));
    edges_to_rm          <- any (inputs.remove_edge, input_edges_to_rm, output_edges_to_rm);
    out.source.on_edge_drop <+ edges_to_rm;
    }



    // =====================
    // === Pointer Style ===
    // =====================

    frp::extend! { network

    on_some_edges_detached <- out.some_edge_endpoints_unset.gate(&out.some_edge_endpoints_unset);
    cursor_style_edge_drag <- on_some_edges_detached.map(f_!([model]{
        if let Some(color) = model.first_detached_edge_color() {
            cursor::Style::new_color(color).press()
        } else {
            cursor::Style::new_color_no_animation(any_type_sel_color).press()
        }
    }));
    cursor_style_on_edge_drag_stop <- out.on_all_edges_endpoints_set.constant(default());
    cursor_style_edge_drag         <- any (cursor_style_edge_drag,cursor_style_on_edge_drag_stop);

    let breadcrumb_style = model.breadcrumbs.pointer_style.clone_ref();

    pointer_style <- all
        [ pointer_on_drag
        , cursor_selection
        , cursor_press
        , node_pointer_style
        , cursor_style_edge_drag
        , breadcrumb_style
        ].fold();

    eval pointer_style ((style) cursor.frp.set_style.emit(style));

    }


    // let frp = Frp::deprecated_new(network.clone(),out); // fixme clone

    GraphEditor {model,frp}
}




impl display::Object for GraphEditor {
    fn display_object(&self) -> &display::object::Instance {
        self.model.display_object()
    }
}


