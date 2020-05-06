//! View of the node editor.

use crate::prelude::*;

use crate::notification;
use crate::controller::graph::NodeTrees;

use enso_frp as frp;
use enso_frp::stream::EventEmitter;
use ensogl::display;
use ensogl::display::traits::*;
use ensogl::application::Application;
use graph_editor::{GraphEditor, EdgeTarget};
use utils::channel::process_stream_with_handle;



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
        def condition = switch.toggle_true();
    }
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
    pub logger               : Logger,
    pub editor               : GraphEditor,
    pub controller           : controller::ExecutedGraph,
    displayed_nodes          : RefCell<Bimap<ast::Id,graph_editor::NodeId>>,
    displayed_expressions    : RefCell<HashMap<graph_editor::NodeId,String>>,
    displayed_connections    : RefCell<Bimap<controller::graph::Connection,graph_editor::EdgeId>>,
}


// === Contruction And Setup ===

impl GraphEditorIntegration {
    /// Constructor. It creates GraphEditor panel and connect it with given controller handle.
    pub fn new(logger:Logger, app:&Application, controller:controller::Graph) -> Rc<Self> {
        let editor                = app.views.new::<GraphEditor>();
        let displayed_nodes       = default();
        let displayed_connections = default();
        let displayed_expressions = default();
        let this = Rc::new(GraphEditorIntegration {editor,controller,displayed_nodes,
            displayed_expressions,displayed_connections,logger});

        let updating = Self::setup_controller_event_handling(&this);
        Self::setup_ui_event_handling(&this,updating);
        //TODO
        // if let Err(err) = this.invalidate_graph() {
        //     error!(this.logger,"Error while initializing graph display: {err}");
        // }
        this
    }

    fn setup_controller_event_handling(this:&Rc<Self>) -> enso_frp::Stream<bool> {
        let stream  = this.controller.subscribe();
        let weak    = Rc::downgrade(this);
        let network = &this.editor.network;
        enso_frp::extend! {network
            def trigger = source::<Option<notification::Graph>>();
            let (runner,updating) = fence(&network,&trigger);
            def _eval = runner.map(enclose!((weak) move |notification| {
                if let Some(this) = weak.upgrade() {
                    let result = match notification.unwrap() {
                        notification::Graph::Invalidate => this.invalidate_graph(),
                    };
                    if let Err(err) = result {
                        error!(this.logger,"Error while updating graph after receiving \
                        {notification:?} from controller: {err}");
                    }
                }
            }));
        }
        let handler = process_stream_with_handle(stream,weak,move |notification,_this| {
            trigger.emit_event(&Some(notification));
            futures::future::ready(())
        });
        executor::global::spawn(handler);
        updating
    }

    fn setup_ui_event_handling(this:&Rc<Self>, updating:enso_frp::Stream<bool>) {
        let frp     = &this.editor.frp.outputs;
        Self::bind_action_to_editor_frp(this,"node_removed", &frp.node_removed, Self::node_removed_action,&updating);
        Self::bind_action_to_editor_frp(this,"connection_added", &frp.nodes_connected, Self::connection_created_action,&updating);
        Self::bind_action_to_editor_frp(this,"connection_removed", &frp.nodes_disconnected, Self::connection_removed_action,&updating);
        Self::bind_action_to_editor_frp(this,"node_moved", &frp.node_position, Self::node_moved_action,&updating);
    }

    fn bind_action_to_editor_frp<Action,Parameter>
    ( this:&Rc<Self>
      , label:frp::node::Label
      , output:&frp::Stream<Parameter>
      , action:Action
      , updating:&enso_frp::Stream<bool>
    ) where Action    : Fn(&Self,&Parameter) -> FallibleResult<()> + 'static,
            Parameter : Clone + Debug + Default + 'static {
        let network = &this.editor.network;
        let logger  = this.logger.clone_ref();
        let weak    = Rc::downgrade(this);
        let lambda = move |parameter:&Parameter, updating:&bool| {
            if *updating {
                if let Some(this) = weak.upgrade() {
                    let result = action(&*this,parameter);
                    if result.is_err() {
                        error!(logger,"Error while performing UI action on controllers: {result:?}");
                    }
                }
            }
        };
        network.map2(label,output,updating,lambda);
    }
}


// === Invalidating Displayed Graph ===

impl GraphEditorIntegration {
    /// Reloads whole displayed content to be up to date with module state.
    pub fn invalidate_graph(&self) -> FallibleResult<()> {
        let controller::graph::Connections{trees,connections} = self.controller.graph.connections()?;
        self.invalidate_nodes(trees)?;
        self.invalidate_connections(connections);
        Ok(())
    }

    fn invalidate_nodes
    (&self, mut trees:HashMap<double_representation::node::Id,NodeTrees>) -> FallibleResult<()> {
        let nodes = self.controller.nodes()?;
        let ids   = nodes.iter().map(|node| node.info.id() ).collect();
        self.retain_ids(&ids);
        for (i,node_info) in nodes.iter().enumerate() {
            let id          = node_info.info.id();
            let node_trees  = trees.remove(&id).unwrap_or_else(|| default());
            let default_pos = enso_frp::Position::new(0.0, i as f32 * 77.0);
            let displayed   = self.displayed_nodes.borrow_mut().get_fwd(&id).cloned();
            match displayed {
                Some(displayed) => {
                    self.update_displayed_node(displayed,node_info,node_trees);
                }
                None => {
                    warning!(self.logger,"ADDING NODE {id:?}");
                    let displayed_id = self.editor.add_node();
                    self.update_displayed_node(displayed_id,node_info,node_trees);
                    if node_info.metadata.and_then(|md| md.position).is_none() {
                        // we must initialize position.
                        self.editor.frp.inputs.set_node_position.emit_event(&(displayed_id,default_pos));
                    }
                    self.displayed_nodes.borrow_mut().insert(id,displayed_id);
                }
            }
        }
        Ok(())
    }

    /// Retain only given ids in displayed graph.
    fn retain_ids(&self, ids:&HashSet<ast::Id>) {
        warning!(self.logger,"Retain IDS: {ids:?}");
        let to_remove = {
            let borrowed = self.displayed_nodes.borrow();
            let filtered = borrowed.iter().filter(|(id,_)| !ids.contains(id));
            filtered.map(|(k,v)| (*k,*v)).collect_vec()
        };
        for (id,displayed_id) in to_remove {
            self.editor.frp.inputs.remove_node.emit_event(&displayed_id);
            warning!(self.logger,"REMOVING {displayed_id:?}");
            self.displayed_nodes.borrow_mut().remove_fwd(&id);
        }
    }

    fn update_displayed_node
    (&self, node:graph_editor::NodeId, info:&controller::graph::Node, trees:NodeTrees) {
        let position = info.metadata.and_then(|md| md.position);
        if let Some(pos) = position {
            let pos = Self::convert_position(pos);
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

    fn convert_position(pos:model::module::Position) -> enso_frp::Position {
        enso_frp::Position::new(pos.vector.x,pos.vector.y)
    }

    fn invalidate_connections(&self, connections:Vec<controller::graph::Connection>) {
        self.retain_connections(&connections);
        for con in connections {
            if !self.displayed_connections.borrow().contains_fwd(&con) {
                if let Some(graph_con) = self.to_graph_editor_edge(con.clone()) {
                    self.editor.frp.inputs.connect_nodes.emit_event(&(graph_con));
                    let edge_id = self.editor.frp.outputs.edge_added.value();
                    self.displayed_connections.borrow_mut().insert(con,edge_id);
                }
            }
        }
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
        let id = self.displayed_nodes.borrow().get_rev(&node).cloned();
        if let Some(id) = id {
            self.controller.remove_node(id)?;
            self.displayed_nodes.borrow_mut().remove_fwd(&id);
        }
        Ok(())
    }

    fn node_moved_action
    (&self, node_and_position:&(graph_editor::NodeId,enso_frp::Position)) -> FallibleResult<()> {
        let (node,position) = node_and_position;
        if let Some(id) = self.displayed_nodes.borrow().get_rev(&node).cloned() {
            self.controller.module.with_node_metadata(id, |md| {
                md.position = Some(model::module::Position::new(position.x,position.y));
            });
        }
        Ok(())
    }

    fn connection_created_action
    (&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        if let Some(edge) = self.editor.edges.get_cloned(&edge_id) {
            if let Some(connection) = self.from_graph_editor_edge(&edge) {
                self.controller.connect(&connection)?;
            }
        }
        Ok(())
    }

    fn connection_removed_action
    (&self, edge_id:&graph_editor::EdgeId) -> FallibleResult<()> {
        let connection = self.displayed_connections.borrow().get_rev(edge_id).cloned();
        if let Some(connection) = connection {
            self.controller.disconnect(&connection)?;
            self.displayed_connections.borrow_mut().remove_fwd(&connection);
        }
        Ok(())
    }

}


// === Utilities ===

impl GraphEditorIntegration {
    fn to_graph_editor_edge
    (&self, connection:controller::graph::Connection) -> Option<(EdgeTarget,EdgeTarget)> {
        let src_node = *self.displayed_nodes.borrow().get_fwd(&connection.source.node)?;
        let dst_node = *self.displayed_nodes.borrow().get_fwd(&connection.destination.node)?;
        let src      = EdgeTarget::new(src_node,connection.source.port);
        let data     = EdgeTarget::new(dst_node,connection.destination.port);
        Some((src,data))
    }

    fn from_graph_editor_edge
    (&self, connection:&graph_editor::Edge) -> Option<controller::graph::Connection> {
        let src      = connection.source()?;
        let dst      = connection.target()?;
        let src_node = *self.displayed_nodes.borrow().get_rev(&src.node_id())?;
        let dst_node = *self.displayed_nodes.borrow().get_rev(&dst.node_id())?;
        Some(controller::graph::Connection {
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
