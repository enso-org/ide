//! View of the node editor.

use crate::prelude::*;

use crate::notification;
use crate::controller::graph::NodeTrees;

use enso_frp as frp;
use enso_frp::stream::EventEmitter;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::application::Application;
use graph_editor::GraphEditor;
use graph_editor::EdgeTarget;
use utils::channel::process_stream_with_handle;



// ==============
// === Errors ===
// ==============

#[derive(Copy,Clone,Debug,Fail)]
enum InvalidState {
    #[fail(display="Node Editor discrepancy: displayed node {:?} is not bound to any actual \
        node", _0)]
    MissingNode(graph_editor::NodeId),
    #[fail(display="Node Editor discrepancy: Node {:?} is not displayed", _0)]
    MissingDisplayedNode(ast::Id),
    #[fail(display="Node Editor discrepancy: displayed connection {:?} is not bound to any actual \
        connection", _0)]
    MissingConnection(graph_editor::EdgeId),
}

#[derive(Copy,Clone,Debug,Fail)]
#[fail(display="Discrepancy in a GraphEditor component")]
struct GraphEditorDiscrepancy;



// =================================
// === Bidirectional Map Wrapper ===
// =================================

#[derive(Clone,Default,Shrinkwrap)]
#[shrinkwrap(mutable)]
struct Bimap<K:Clone+Eq+Hash,V:Clone+Eq+Hash>(pub bidirectional_map::Bimap<K,V>);

impl<K,V> Debug for Bimap<K,V>
where K : Clone + Debug + Eq + Hash,
      V : Clone + Debug + Eq + Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(bimap) = self;
        Debug::fmt(&bimap.iter(),f)
    }
}



// =============
// === Fence ===
// =============

fn fence<T,Out>(network:&frp::Network, trigger:T) -> (frp::Stream<Out>,frp::Stream<bool>)
where T:frp::HasOutput<Output=Out>, T:Into<frp::Stream<Out>>, Out:frp::Data {
    let trigger = trigger.into();
    frp::extend! { network
        def trigger_ = trigger.constant(());
        def runner   = source::<Out>();
        def switch   = gather();
        switch.attach(&trigger_);
        def triggered = trigger.map(enclose!((runner) move |val| runner.emit_event(val)));
        switch.attach(&triggered);
        def condition    = switch.toggle();
        //FIXME[ao] this is to work around bug #427
        def manual = source::<()>();
        switch.attach(&manual);
        def _force_cache = manual.map2(&condition, |(),val| assert_eq!(*val,true));
    }
    manual.emit(());
    let runner = runner.into();
    (runner,condition)
}



// ==============================
// === GraphEditorIntegration ===
// ==============================

/// A structure integration controller and view. All changes made by user in view are reflected
/// in controller, and all controller notifications update view accordingly.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct GraphEditorIntegration {
    pub logger            : Logger,
    pub editor            : GraphEditor,
    pub controller        : controller::ExecutedGraph,
    network               : frp::Network,
    displayed_nodes       : RefCell<Bimap<ast::Id,graph_editor::NodeId>>,
    displayed_expressions : RefCell<HashMap<graph_editor::NodeId,String>>,
    displayed_connections : RefCell<Bimap<controller::graph::Connection,graph_editor::EdgeId>>,
}


// === Construction And Setup ===

impl GraphEditorIntegration {
    /// Constructor. It creates GraphEditor panel and connect it with given controller handle.
    pub fn new(logger:Logger, app:&Application, controller:controller::ExecutedGraph) -> Rc<Self> {
        let editor                = app.views.new::<GraphEditor>();
        let displayed_nodes       = default();
        let displayed_connections = default();
        let displayed_expressions = default();
        let network               = default();
        let this = Rc::new(GraphEditorIntegration {editor,controller,network,displayed_nodes,
            displayed_expressions,displayed_connections,logger});

        if let Err(err) = this.invalidate_graph() {
            error!(this.logger,"Error while initializing graph display: {err}");
        }

        Self::setup_frp_network(&this);
        this
    }

    fn setup_frp_network(this:&Rc<Self>) {
        let network = &this.network;
        let editor_outs = &this.editor.frp.outputs;
        let weak    = Rc::downgrade(this);
        let node_removed        = Self::define_action(this,Self::node_removed_action);
        let connections_created = Self::define_action(this,Self::connection_created_action);
        let connections_removed = Self::define_action(this,Self::connection_removed_action);
        let node_moved          = Self::define_action(this,Self::node_moved_action);
        frp::extend! {network
            // Notifications from controller
            def handle_notification = source::<Option<notification::Graph>>();
            let (runner,is_action)  = fence(&network,&handle_notification);
            def _notification       = runner.map(f!((weak)(notification) {
                weak.upgrade().for_each(|this| this.handle_controller_notification(notification));
            }));
            // Changes in Graph Editor
            def _action = editor_outs.node_removed      .map2(&is_action,node_removed);
            def _action = editor_outs.nodes_connected   .map2(&is_action,connections_created);
            def _action = editor_outs.nodes_disconnected.map2(&is_action,connections_removed);
            def _action = this.editor.frp.node_release  .map2(&is_action,node_moved);
        }
        Self::connect_frp_to_controller_notifications(this,handle_notification);
    }

    fn connect_frp_to_controller_notifications
    (this:&Rc<Self>, frp_endpoint:frp::Source<Option<notification::Graph>>) {
        let stream  = this.controller.graph.subscribe();
        let weak    = Rc::downgrade(this);
        let handler = process_stream_with_handle(stream,weak,move |notification,_this| {
            frp_endpoint.emit_event(&Some(notification));
            futures::future::ready(())
        });
        executor::global::spawn(handler);
    }

    fn define_action<Action,Parameter>
    (this:&Rc<Self>, action:Action) -> impl Fn(&Parameter,&bool)
    where Action : Fn(&Self,&Parameter) -> FallibleResult<()> + 'static {
        let logger  = this.logger.clone_ref();
        let weak    = Rc::downgrade(this);
        move |parameter,is_action| {
            warning!(logger, "ACTION {is_action}");
            if *is_action {
                if let Some(this) = weak.upgrade() {
                    let result = action(&*this,parameter);
                    if let Err(err) = result {
                        error!(logger,"Error while performing UI action on controllers: {err}");
                    }
                }
            }
        }
    }
}


// === Invalidating Displayed Graph ===

impl GraphEditorIntegration {
    /// Handles notification received from controller.
    pub fn handle_controller_notification(&self, notification:&Option<notification::Graph>) {
        let result = match notification {
            Some(notification::Graph::Invalidate) => self.invalidate_graph(),
            other => {
                warning!(self.logger,"Handling notification {other:?} is not implemented; \
                    performing full invalidation");
                self.invalidate_graph()
            }
        };
        if let Err(err) = result {
            error!(self.logger,"Error while updating graph after receiving {notification:?} from \
                controller: {err}");
        }
    }

    /// Reloads whole displayed content to be up to date with module state.
    pub fn invalidate_graph(&self) -> FallibleResult<()> {
        use controller::graph::Connections;
        let Connections{trees,connections} = self.controller.graph.connections()?;
        self.invalidate_nodes(trees)?;
        self.invalidate_connections(connections)?;
        Ok(())
    }

    fn invalidate_nodes
    (&self, mut trees:HashMap<double_representation::node::Id,NodeTrees>) -> FallibleResult<()> {
        let nodes = self.controller.graph.nodes()?;
        let ids   = nodes.iter().map(|node| node.info.id() ).collect();
        self.retain_ids(&ids);
        for (i,node_info) in nodes.iter().enumerate() {
            let id          = node_info.info.id();
            let node_trees  = trees.remove(&id).unwrap_or_else(|| default());
            let default_pos = enso_frp::Position::new(0.0, i as f32 * 77.0);
            let displayed   = self.displayed_nodes.borrow_mut().get_fwd(&id).cloned();
            match displayed {
                Some(displayed) => self.update_displayed_node(displayed,node_info,node_trees),
                None            => self.create_displayed_node(node_info,node_trees,&default_pos),
            }
        }
        Ok(())
    }

    /// Retain only given ids in displayed graph.
    fn retain_ids(&self, ids:&HashSet<ast::Id>) {
        let to_remove = {
            let borrowed = self.displayed_nodes.borrow();
            let filtered = borrowed.iter().filter(|(id,_)| !ids.contains(id));
            filtered.map(|(k,v)| (*k,*v)).collect_vec()
        };
        for (id,displayed_id) in to_remove {
            self.editor.frp.inputs.remove_node.emit_event(&displayed_id);
            self.displayed_nodes.borrow_mut().remove_fwd(&id);
        }
    }

    fn create_displayed_node
    (&self, info:&controller::graph::Node, trees:NodeTrees, default_pos:&enso_frp::Position) {
        let id           = info.info.id();
        let displayed_id = self.editor.add_node();
        self.update_displayed_node(displayed_id,info,trees);
        // If position wasn't present in metadata, we must initialize it.
        if info.metadata.and_then(|md| md.position).is_none() {
            self.editor.frp.inputs.set_node_position.emit_event(&(displayed_id,*default_pos));
        }
        self.displayed_nodes.borrow_mut().insert(id,displayed_id);
    }

    fn update_displayed_node
    (&self, node:graph_editor::NodeId, info:&controller::graph::Node, trees:NodeTrees) {
        let position = info.metadata.and_then(|md| md.position);
        if let Some(pos) = position {
            let pos = enso_frp::Position::new(pos.vector.x,pos.vector.y);
            self.editor.frp.inputs.set_node_position.emit_event(&(node,pos));
        }
        let expression = info.info.expression().repr();
        if Some(&expression) != self.displayed_expressions.borrow().get(&node) {
            let code_and_trees = graph_editor::component::node::port::Expression {
                code             : expression.clone(),
                input_span_tree  : trees.inputs,
                output_span_tree : trees.outputs.unwrap_or_else(|| default())
            };
            self.editor.frp.inputs.set_node_expression.emit_event(&(node,code_and_trees));
            self.displayed_expressions.borrow_mut().insert(node,expression);
        }
    }

    fn invalidate_connections
    (&self, connections:Vec<controller::graph::Connection>) -> FallibleResult<()> {
        self.retain_connections(&connections);
        for con in connections {
            if !self.displayed_connections.borrow().contains_fwd(&con) {
                let graph_con = self.to_graph_editor_edge(con.clone())?;
                self.editor.frp.inputs.connect_nodes.emit_event(&(graph_con));
                let edge_id = self.editor.frp.outputs.edge_added.value();
                self.displayed_connections.borrow_mut().insert(con,edge_id);
            }
        }
        Ok(())
    }

    fn retain_connections(&self, connections:&Vec<controller::graph::Connection>) {
        let to_remove = {
            let borrowed = self.displayed_connections.borrow();
            let filtered = borrowed.iter().filter(|(con,_)| !connections.contains(con));
            filtered.map(|(_,edge_id)| *edge_id).collect_vec()
        };
        for edge_id in to_remove {
            self.editor.frp.inputs.remove_edge.emit_event(&edge_id);
            self.displayed_connections.borrow_mut().remove_rev(&edge_id);
        }
    }
}


// === Passing UI Actions To Controllers ===

impl GraphEditorIntegration {

    fn node_removed_action(&self, node:&graph_editor::NodeId) -> FallibleResult<()> {
        let id = self.get_controller_node_id(*node)?;
        self.controller.graph.remove_node(id)?;
        self.displayed_nodes.borrow_mut().remove_fwd(&id);
        Ok(())
    }

    fn node_moved_action
    (&self, displayed_id:&graph_editor::NodeId) -> FallibleResult<()> {
        let id   = self.get_controller_node_id(*displayed_id)?;
        let node = self.editor.nodes.get_cloned_ref(&displayed_id).ok_or(GraphEditorDiscrepancy{})?;
        let pos  = node.position();
        self.controller.graph.module.with_node_metadata(id, |md| {
            md.position = Some(model::module::Position::new(pos.x,pos.y));
        });
        Ok(())
    }

    fn connection_created_action(&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        let edge = self.editor.edges.get_cloned(&edge_id).ok_or(GraphEditorDiscrepancy)?;
        let connection = self.from_graph_editor_edge(&edge)?;
        self.controller.graph.connect(&connection)?;
        Ok(())
    }

    fn connection_removed_action(&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        let connection = self.get_controller_connection(*edge_id)?;
        self.controller.graph.disconnect(&connection)?;
        self.displayed_connections.borrow_mut().remove_fwd(&connection);
        Ok(())
    }

}


// === Utilities ===

impl GraphEditorIntegration {
    fn get_controller_node_id
    (&self, displayed_id:graph_editor::NodeId) -> Result<ast::Id,InvalidState> {
        let err = InvalidState::MissingNode(displayed_id);
        self.displayed_nodes.borrow().get_rev(&displayed_id).cloned().ok_or(err)
    }

    fn get_displayed_node_id
    (&self, node_id:ast::Id) -> Result<graph_editor::NodeId,InvalidState> {
        let err = InvalidState::MissingDisplayedNode(node_id);
        self.displayed_nodes.borrow().get_fwd(&node_id).cloned().ok_or(err)
    }

    fn get_controller_connection
    (&self, displayed_id:graph_editor::EdgeId)
    -> Result<controller::graph::Connection,InvalidState> {
        let err = InvalidState::MissingConnection(displayed_id);
        self.displayed_connections.borrow().get_rev(&displayed_id).cloned().ok_or(err)
    }

    fn to_graph_editor_edge
    (&self, connection:controller::graph::Connection) -> FallibleResult<(EdgeTarget,EdgeTarget)> {
        let src_node = self.get_displayed_node_id(connection.source.node)?;
        let dst_node = self.get_displayed_node_id(connection.destination.node)?;
        let src      = EdgeTarget::new(src_node,connection.source.port);
        let data     = EdgeTarget::new(dst_node,connection.destination.port);
        Ok((src,data))
    }

    fn from_graph_editor_edge
    (&self, connection:&graph_editor::Edge) -> FallibleResult<controller::graph::Connection> {
        let src      = connection.source().ok_or(GraphEditorDiscrepancy{})?;
        let dst      = connection.target().ok_or(GraphEditorDiscrepancy{})?;
        let src_node = self.get_controller_node_id(src.node_id())?;
        let dst_node = self.get_controller_node_id(dst.node_id())?;
        Ok(controller::graph::Connection {
            source      : controller::graph::Endpoint::new(src_node,src.port()),
            destination : controller::graph::Endpoint::new(dst_node,dst.port()),
        })
    }
}



// ==================
// === NodeEditor ===
// ==================

/// Node Editor Panel integrated with Graph Controller.
#[derive(Clone,CloneRef,Debug)]
pub struct NodeEditor {
    display_object : display::object::Instance,
    graph          : Rc<GraphEditorIntegration>,
    controller     : controller::ExecutedGraph,
}

impl NodeEditor {
    /// Create Node Editor Panel.
    pub fn new(logger:&Logger, app:&Application, controller:controller::ExecutedGraph) -> Self {
        let logger         = logger.sub("NodeEditor");
        let graph          = GraphEditorIntegration::new(logger,app,controller.clone_ref());
        let display_object = display::object::Instance::new(&graph.logger);
        display_object.add_child(&graph.editor);
        NodeEditor {display_object,graph,controller}
    }
}

impl display::Object for NodeEditor {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
