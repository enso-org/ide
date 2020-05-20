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
use bimap::BiMap;



// ==============
// === Errors ===
// ==============

/// Error returned by various function inside GraphIntegration, when our mappings from controller
/// items (node or connections) to displayed items are missing some information.
#[derive(Copy,Clone,Debug,Fail)]
enum MissingMappingFor {
    #[fail(display="Displayed node {:?} is not bound to any controller node.",_0)]
    DisplayedNode(graph_editor::NodeId),
    #[fail(display="Controller node {:?} is not bound to any displayed node",_0)]
    ControllerNode(ast::Id),
    #[fail(display="Displayed connection {:?} is not bound to any controller connection", _0)]
    DisplayedConnection(graph_editor::EdgeId),
}

/// Error raised when reached some fatal inconsistency in data provided by GraphEditor.
#[derive(Copy,Clone,Debug,Fail)]
#[fail(display="Discrepancy in a GraphEditor component")]
struct GraphEditorDiscrepancy;



// ====================
// === FencedAction ===
// ====================

/// An utility to FRP network. It is wrapped closure in a set of FRP nodes. The closure is called
/// on `trigger`, and `is_running` contains information if we are still inside closure call. It
/// allows us to block some execution path to avoid infinite loops.
struct FencedAction<Parameter:frp::Data> {
    trigger    : frp::Source<Parameter>,
    is_running : frp::Stream<bool>,
}

impl<Parameter:frp::Data> FencedAction<Parameter> {
    /// Wrap the `action` in `FencedAction`.
    fn fence(network:&frp::Network, action:impl Fn(&Parameter) + 'static) -> Self {
        frp::extend! { network
            def trigger   = source::<Parameter>();
            def triggered = trigger.constant(());
            def switch    = gather();
            switch.attach(&triggered);
            def performed = trigger.map(move |param| action(param));
            switch.attach(&performed);
            def is_running = switch.toggle();
        }
        Self {trigger,is_running}
    }
}



// ==============================
// === GraphEditorIntegration ===
// ==============================

/// The gap between nodes in pixels on default node layout (when user did not set any position of
/// node - possible when node was added by editing text).
const DEFAULT_GAP_BETWEEN_NODES:f32 = 44.0;

/// A structure which handles integration between controller and graph_editor EnsoGl control.
/// All changes made by user in view are reflected in controller, and all controller notifications
/// update view accordingly.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct GraphEditorIntegration {
    pub logger            : Logger,
    pub editor            : GraphEditor,
    pub controller        : controller::ExecutedGraph,
    network               : frp::Network,
    displayed_nodes       : RefCell<BiMap<ast::Id,graph_editor::NodeId>>,
    displayed_expressions : RefCell<HashMap<graph_editor::NodeId,String>>,
    displayed_connections : RefCell<BiMap<controller::graph::Connection,graph_editor::EdgeId>>,
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
        frp::extend! {network
            let invalidate = FencedAction::fence(&network,f!([weak](()) {
                weak.upgrade().for_each(|this| {
                    let result = this.invalidate_graph();
                    if let Err(err) = result {
                        error!(this.logger,"Error while invalidating graph: {err}");
                    }
                });
            }));
        }
        let inv = &invalidate.trigger;
        let node_removed       = Self::define_action(this,Self::node_removed_action      ,inv);
        let connection_created = Self::define_action(this,Self::connection_created_action,inv);
        let connection_removed = Self::define_action(this,Self::connection_removed_action,inv);
        let node_moved         = Self::define_action(this,Self::node_moved_action        ,inv);
        frp::extend! {network
            // Notifications from controller
            let handle_notification = FencedAction::fence(&network,
                f!([weak](notification:&Option<notification::Graph>) {
                    if let Some(this) = weak.upgrade() {
                        this.handle_controller_notification(*notification);
                    }
                }
            ));

            // Changes in Graph Editor
            let is_handling_notification = handle_notification.is_running;
            def is_hold = is_handling_notification.zip_with(&invalidate.is_running, |l,r| *l || *r);
            def _action = editor_outs.node_removed      .map2(&is_hold,node_removed);
            def _action = editor_outs.connection_added  .map2(&is_hold,connection_created);
            def _action = editor_outs.connection_removed.map2(&is_hold,connection_removed);
            def _action = editor_outs.node_position_set .map2(&is_hold,node_moved);
        }
        Self::connect_frp_to_controller_notifications(this,handle_notification.trigger);
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
    (this:&Rc<Self>, action:Action, invalidate:&frp::Source<()>) -> impl Fn(&Parameter,&bool)
    where Action : Fn(&Self,&Parameter) -> FallibleResult<()> + 'static {
        let logger     = this.logger.clone_ref();
        let weak       = Rc::downgrade(this);
        let invalidate = invalidate.clone_ref();
        move |parameter,is_hold| {
            if !*is_hold {
                if let Some(this) = weak.upgrade() {
                    let result = action(&*this,parameter);
                    if let Err(err) = result {
                        error!(logger,"Error while performing UI action on controllers: {err}");
                        info!(logger,"Invalidating displayed graph");
                        invalidate.emit(());
                    }
                }
            }
        }
    }
}


// === Invalidating Displayed Graph ===

impl GraphEditorIntegration {
    /// Handles notification received from controller.
    pub fn handle_controller_notification(&self, notification:Option<notification::Graph>) {
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
        self.retain_nodes(&ids);
        for (i,node_info) in nodes.iter().enumerate() {
            let id          = node_info.info.id();
            let node_trees  = trees.remove(&id).unwrap_or_else(default);
            let default_pos = enso_frp::Position::new(0.0, i as f32 * -DEFAULT_GAP_BETWEEN_NODES);
            let displayed   = self.displayed_nodes.borrow_mut().get_by_left(&id).cloned();
            match displayed {
                Some(displayed) => self.update_displayed_node(displayed,node_info,node_trees),
                None            => self.create_displayed_node(node_info,node_trees,default_pos),
            }
        }
        Ok(())
    }

    /// Retain only given nodes in displayed graph.
    fn retain_nodes(&self, ids:&HashSet<ast::Id>) {
        let to_remove = {
            let borrowed = self.displayed_nodes.borrow();
            let filtered = borrowed.iter().filter(|(id,_)| !ids.contains(id));
            filtered.map(|(k,v)| (*k,*v)).collect_vec()
        };
        for (id,displayed_id) in to_remove {
            self.editor.frp.inputs.remove_node.emit_event(&displayed_id);
            self.displayed_nodes.borrow_mut().remove_by_left(&id);
        }
    }

    fn create_displayed_node
    (&self, info:&controller::graph::Node, trees:NodeTrees, default_pos:frp::Position) {
        let id           = info.info.id();
        let displayed_id = self.editor.add_node();
        self.update_displayed_node(displayed_id,info,trees);
        // If position wasn't present in metadata, we must initialize it.
        if info.metadata.and_then(|md| md.position).is_none() {
            self.editor.frp.inputs.set_node_position.emit_event(&(displayed_id,default_pos));
        }
        self.displayed_nodes.borrow_mut().insert(id,displayed_id);
    }

    fn update_displayed_node
    (&self, node:graph_editor::NodeId, info:&controller::graph::Node, trees:NodeTrees) {
        let position = info.metadata.and_then(|md| md.position);
        if let Some(pos) = position {
            let pos = frp::Position::new(pos.vector.x,pos.vector.y);
            self.editor.frp.inputs.set_node_position.emit_event(&(node,pos));
        }
        let expression = info.info.expression().repr();
        if Some(&expression) != self.displayed_expressions.borrow().get(&node) {
            let code_and_trees = graph_editor::component::node::port::Expression {
                code             : expression.clone(),
                input_span_tree  : trees.inputs,
                output_span_tree : trees.outputs.unwrap_or_else(default)
            };
            self.editor.frp.inputs.set_node_expression.emit_event(&(node,code_and_trees));
            self.displayed_expressions.borrow_mut().insert(node,expression);
        }
    }

    fn invalidate_connections
    (&self, connections:Vec<controller::graph::Connection>) -> FallibleResult<()> {
        self.retain_connections(&connections);
        for con in connections {
            if !self.displayed_connections.borrow().contains_left(&con) {
                let targets = self.edge_targets_from_controller_connection(con.clone())?;
                self.editor.frp.inputs.connect_nodes.emit_event(&targets);
                let edge_id = self.editor.frp.outputs.edge_added.value();
                self.displayed_connections.borrow_mut().insert(con,edge_id);
            }
        }
        Ok(())
    }

    fn edge_targets_from_controller_connection
    (&self, connection:controller::graph::Connection) -> FallibleResult<(EdgeTarget,EdgeTarget)> {
        let src_node = self.get_displayed_node_id(connection.source.node)?;
        let dst_node = self.get_displayed_node_id(connection.destination.node)?;
        let src      = EdgeTarget::new(src_node,connection.source.port);
        let data     = EdgeTarget::new(dst_node,connection.destination.port);
        Ok((src,data))
    }

    /// Retain only given connections in displayed graph.
    fn retain_connections(&self, connections:&[controller::graph::Connection]) {
        let to_remove = {
            let borrowed = self.displayed_connections.borrow();
            let filtered = borrowed.iter().filter(|(con,_)| !connections.contains(con));
            filtered.map(|(_,edge_id)| *edge_id).collect_vec()
        };
        for edge_id in to_remove {
            self.editor.frp.inputs.remove_edge.emit_event(&edge_id);
            self.displayed_connections.borrow_mut().remove_by_right(&edge_id);
        }
    }
}


// === Passing UI Actions To Controllers ===

// These functions are called with FRP event values as arguments. The FRP values are always provided
// by reference, even those "trivally-copy" types, To keep code cleaner we take all parameters
// by reference as well.
#[allow(clippy::trivially_copy_pass_by_ref)]
impl GraphEditorIntegration {
    fn node_removed_action(&self, node:&graph_editor::NodeId) -> FallibleResult<()> {
        let id = self.get_controller_node_id(*node)?;
        self.displayed_nodes.borrow_mut().remove_by_left(&id);
        self.controller.graph.remove_node(id)?;
        Ok(())
    }

    fn node_moved_action(&self, param:&(graph_editor::NodeId,frp::Position)) -> FallibleResult<()> {
        let (displayed_id,pos) = param;
        let id                 = self.get_controller_node_id(*displayed_id)?;
        self.controller.graph.module.with_node_metadata(id, |md| {
            md.position = Some(model::module::Position::new(pos.x,pos.y));
        });
        Ok(())
    }

    fn connection_created_action(&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        let displayed = self.editor.edges.get_cloned(&edge_id).ok_or(GraphEditorDiscrepancy)?;
        let con       = self.controller_connection_from_displayed(&displayed)?;
        let inserting = self.displayed_connections.borrow_mut().insert(con.clone(),*edge_id);
        if inserting.did_overwrite() {
            internal_warning!(self.logger,"Created connection {edge_id} overwrite some old \
                mappings in GraphEditorIntegration.")
        }
        self.controller.graph.connect(&con)?;
        Ok(())
    }

    fn connection_removed_action(&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        let connection = self.get_controller_connection(*edge_id)?;
        self.displayed_connections.borrow_mut().remove_by_left(&connection);
        self.controller.graph.disconnect(&connection)?;
        Ok(())
    }
}


// === Utilities ===

impl GraphEditorIntegration {
    fn get_controller_node_id
    (&self, displayed_id:graph_editor::NodeId) -> Result<ast::Id, MissingMappingFor> {
        let err = MissingMappingFor::DisplayedNode(displayed_id);
        self.displayed_nodes.borrow().get_by_right(&displayed_id).cloned().ok_or(err)
    }

    fn get_displayed_node_id
    (&self, node_id:ast::Id) -> Result<graph_editor::NodeId, MissingMappingFor> {
        let err = MissingMappingFor::ControllerNode(node_id);
        self.displayed_nodes.borrow().get_by_left(&node_id).cloned().ok_or(err)
    }

    fn get_controller_connection
    (&self, displayed_id:graph_editor::EdgeId)
    -> Result<controller::graph::Connection, MissingMappingFor> {
        let err = MissingMappingFor::DisplayedConnection(displayed_id);
        self.displayed_connections.borrow().get_by_right(&displayed_id).cloned().ok_or(err)
    }

    fn controller_connection_from_displayed
    (&self, connection:&graph_editor::Edge) -> FallibleResult<controller::graph::Connection> {
        let src      = connection.source().ok_or(GraphEditorDiscrepancy{})?;
        let dst      = connection.target().ok_or(GraphEditorDiscrepancy{})?;
        let src_node = self.get_controller_node_id(src.node_id)?;
        let dst_node = self.get_controller_node_id(dst.node_id)?;
        Ok(controller::graph::Connection {
            source      : controller::graph::Endpoint::new(src_node,src.port.deref().clone()),
            destination : controller::graph::Endpoint::new(dst_node,dst.port.deref().clone()),
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
