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
use graph_editor::component::node::WeakNode;
use weak_table::weak_value_hash_map;
use utils::channel::process_stream_with_handle;



// ==============================
// === GraphEditorIntegration ===
// ==============================

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
struct DisplayedEndpoint {
    node : ast::Id,
    port : controller::graph::PortId,
}

impl DisplayedEndpoint {
    fn into_controller(self) -> controller::graph::Endpoint {
        controller::graph::Endpoint::new(self.node,self.port)
    }
}

#[derive(Clone,Debug,Eq,Hash,PartialEq)]
struct DisplayedConnection {
    source      : DisplayedEndpoint,
    destination : DisplayedEndpoint,
}

impl DisplayedConnection {
    fn into_controller(self) -> controller::graph::Connection {
        controller::graph::Connection {
            source      : self.source.into_controller(),
            destination : self.destination.into_controller(),
        }
    }
}


/// A structure integration controller and view. All changes made by user in view are reflected
/// in controller, and all controller notifications update view accordingly.
#[derive(Debug)]
#[allow(missing_docs)]
pub struct GraphEditorIntegration {
    pub logger            : Logger,
    pub editor            : GraphEditor,
    pub controller        : controller::ExecutedGraph,
    id_to_node            : RefCell<WeakValueHashMap<ast::Id,WeakNode>>,
    node_to_id            : RefCell<WeakKeyHashMap<WeakNode,ast::Id>>,
    displayed_span_trees  : RefCell<WeakKeyHashMap<WeakNode,NodeTrees>>,
    displayed_connections : RefCell<HashSet<DisplayedConnection>>,
}

impl GraphEditorIntegration {
    /// Constructor. It creates GraphEditor panel and connect it with given controller handle.
    pub fn new(logger:Logger, app:&Application, controller:controller::ExecutedGraph) -> Rc<Self> {
        let editor     = app.views.new::<GraphEditor>();
        let id_to_node = default();
        let node_to_id = default();
        let this = Rc::new(GraphEditorIntegration {editor,controller,id_to_node,node_to_id,logger});
        Self::setup_controller_event_handling(&this);
        Self::setup_ui_event_handling(&this);
        //TODO
        // if let Err(err) = this.invalidate_graph() {
        //     error!(this.logger,"Error while initializing graph display: {err}");
        // }
        this
    }
}


// === Invalidating Displayed Graph ===

impl GraphEditorIntegration {
    /// Reloads whole displayed content to be up to date with module state.
    pub fn invalidate_graph(&self) -> FallibleResult<()> {
        let controller::graph::Connections{trees,connections} = self.controller.graph.connections()?;
        self.invalidate_nodes()?;
        self.invalidate_span_trees(trees)?;
        self.invalidate_connections(connections);
        Ok(())
    }

    fn invalidate_nodes(&self) -> FallibleResult<()> {
        let nodes = self.controller.nodes()?;
        let ids   = nodes.iter().map(|node| node.info.id() ).collect();
        self.retain_ids(&ids);
        for (i,node_info) in nodes.iter().enumerate() {
            let id          = node_info.info.id();
            let default_pos = Vector3::new(i as f32 * 100.0,0.0,0.0);
            match self.id_to_node.borrow_mut().entry(id) {
                weak_value_hash_map::Entry::Occupied(entry) => {
                    self.update_displayed_node(entry.get(),node_info);
                }
                weak_value_hash_map::Entry::Vacant(entry)   => {
                    let node = self.editor.add_node().upgrade().unwrap();
                    node.set_position(default_pos);
                    self.update_displayed_node(&node,node_info);
                    entry.insert(node.clone_ref());
                    self.node_to_id.borrow_mut().insert(node,id);
                }
            }
        }
        Ok(())
    }

    /// Retain only given ids in displayed graph.
    fn retain_ids(&self, ids:&HashSet<ast::Id>) {
        self.id_to_node.borrow_mut().retain(|id,node| {
            let to_retain = ids.contains(id);
            if !to_retain {
                self.editor.remove_node(node.downgrade());
            }
            to_retain
        });
    }

    fn update_displayed_node
    (&self, node:&graph_editor::component::node::Node, info:&controller::graph::Node) {
        let position = info.metadata.and_then(|md| md.position);
        if let Some(pos) = position {
            node.set_position(Self::pos_to_vec3(pos));
        }
    }

    fn pos_to_vec3(pos:model::module::Position) -> Vector3<f32> {
        Vector3::new(pos.vector.x,pos.vector.y,0.0)
    }

    fn invalidate_span_trees
    (&self, trees:HashMap<double_representation::node::Id,NodeTrees>) -> FallibleResult<()> {
        let nodes_with_trees = trees.into_iter().filter_map(|(id,trees)| {
            self.id_to_node.borrow().get(&id).map(|node| (node,trees))
        });
        for (node,NodeTrees{inputs,outputs}) in nodes_with_trees {
            let set_inputs_event  = Some((node.downgrade(),inputs.clone()));
            let set_outputs_event = Some((node.downgrade(),outputs.clone()));
            let frp               = &self.editor.frp;
            let emit_inputs       = || frp.set_expression_span_tree.emit_event(&set_inputs_event);
            let emit_outputs      = || frp.set_pattern_span_tree.emit_event(&set_outputs_event);
            match self.displayed_span_trees.borrow_mut().entry(node.clone_ref()) {
                weak_table::weak_key_hash_map::Entry::Occupied(mut entry) => {
                    if entry.get().inputs != inputs {
                        entry.get_mut().inputs = inputs;
                        emit_inputs();
                    }
                    if entry.get().outputs != outputs {
                        entry.get_mut().outputs = outputs;
                        emit_outputs();
                    }
                },
                weak_table::weak_key_hash_map::Entry::Vacant(entry) => {
                    entry.insert(NodeTrees{inputs,outputs});
                    emit_inputs();
                    emit_outputs();
                }
            }
        }
        Ok(())
    }

    fn invalidate_connections(&self, connections:Vec<controller::graph::Connection>) {
        let connections = connections.into_iter().map(|con| {
            let source      = DisplayedEndpoint {node:con.source.node     , port:con.source.port};
            let destination = DisplayedEndpoint {node:con.destination.node, port:con.destination.port};
            DisplayedConnection{source,destination}
        }).collect();
        self.retain_connections(&connections);
        for con in &connections {
            if !self.displayed_connections.borrow().contains(&con) {
                if let Some(graph_con) = self.convert_connection(con.clone()) {
                    self.editor.frp.add_connection.emit_event(&Some(graph_con));
                }
            }
        }
        *self.displayed_connections.borrow_mut() = connections;
    }

    fn retain_connections(&self, connections:&HashSet<DisplayedConnection>) {
        for connection in &*self.displayed_connections.borrow() {
            if !connections.contains(connection) {
                if let Some(graph_con) = self.convert_connection(connection.clone()) {
                    self.editor.frp.remove_connection.emit_event(&Some(graph_con));
                }
            }
        }
    }
}


// === Passing UI Actions To Controllers ===

impl GraphEditorIntegration {
    ///
    fn nodes_removed_action(&self, nodes:&Vec<WeakNode>) -> FallibleResult<()> {
        let nodes     = nodes.iter().filter_map(|weak| weak.upgrade());
        let nodes_ids = nodes.filter_map(|node| self.node_to_id.borrow().get(&node.id()).cloned());
        for node_id in nodes_ids {
            self.controller.remove_node(node_id)?;
        }
        Ok(())
    }

    fn node_moved_action(&self, node:&Option<WeakNode>) -> FallibleResult<()> {
        if let Some(node) = node.as_ref().and_then(|weak| weak.upgrade()) {
            if let Some(id) = self.node_to_id.borrow().get(&node.id()).cloned() {
                self.controller.module.with_node_metadata(id, |md| {
                    let pos = node.position();
                    md.position = Some(model::module::Position::new(pos.x, pos.y));
                })
            }
        }
        Ok(())
    }

    fn connections_created_action
    (&self, connections:&Vec<graph_editor::Connection>) -> FallibleResult<()> {
        let connections = connections.iter().filter_map(|con| self.convert_connection2(con.clone()));
        for connection in connections {
            self.displayed_connections.borrow_mut().insert(connection.clone());
            let dr_connection = connection.into_controller();
            self.controller.connect(&dr_connection)?;
        }
        Ok(())
    }

    fn connections_removed_action
    (&self, connections:&Vec<graph_editor::Connection>) -> FallibleResult<()> {
        let connections = connections.iter().filter_map(|con| self.convert_connection2(con.clone()));
        for connection in connections {
            self.displayed_connections.borrow_mut().remove(&connection);
            let dr_connection = connection.into_controller();
            self.controller.disconnect(&dr_connection)?;
        }
        Ok(())
    }

}


// === Setup ===

impl GraphEditorIntegration {
    fn setup_controller_event_handling(this:&Rc<Self>) {
        let stream  = this.controller.graph.subscribe();
        let weak    = Rc::downgrade(this);
        let handler = process_stream_with_handle(stream,weak,move |notification,this| {
            let result = match notification {
                notification::Graph::Invalidate => this.invalidate_graph(),
            };
            if let Err(err) = result {
                error!(this.logger,"Error while updating graph after receiving {notification:?} \
                    from controller: {err}");
            }
            futures::future::ready(())
        });
        executor::global::spawn(handler);
    }

    fn setup_ui_event_handling(this:&Rc<Self>) {
        let frp     = &this.editor.frp;
        Self::bind_action_to_editor_frp(this,"nodes_removed", &frp.nodes_removed_by_command, Self::nodes_removed_action);
        Self::bind_action_to_editor_frp(this,"connections_added", &frp.connections_added_by_command, Self::connections_created_action);
        Self::bind_action_to_editor_frp(this,"connections_removed", &frp.connections_removed_by_command, Self::connections_removed_action);
        Self::bind_action_to_editor_frp(this,"node_moved", &frp.node_release, Self::node_moved_action);
    }

    fn bind_action_to_editor_frp<Action,Parameter>
    ( this:&Rc<Self>
    , label:frp::node::Label
    , output:&frp::Stream<Parameter>
    , action:Action
    ) where Action    : Fn(&Self,&Parameter) -> FallibleResult<()> + 'static,
            Parameter : Clone + Debug + Default + 'static {
        let network = &this.editor.frp.network;
        let logger  = this.logger.clone_ref();
        let weak    = Rc::downgrade(this);
        let lambda = move |parameter:&Parameter| {
            if let Some(this) = weak.upgrade() {
                let result = action(&*this,parameter);
                if result.is_err() {
                    error!(logger,"Error while performing UI action on controllers: {result:?}");
                }
            }
        };
        network.map(label,output,lambda);
    }

    fn convert_connection
    (&self, connection:DisplayedConnection) -> Option<graph_editor::Connection> {
        let src_node = self.id_to_node.borrow().get(&connection.source.node)?.downgrade();
        let dst_node = self.id_to_node.borrow().get(&connection.destination.node)?.downgrade();
        Some(graph_editor::Connection {
            source      : graph_editor::Endpoint {node:src_node, port:connection.source.port},
            destination : graph_editor::Endpoint {node:dst_node, port:connection.destination.port}
        })
    }

    fn convert_connection2
    (&self, connection:graph_editor::Connection) -> Option<DisplayedConnection> {

        let src_node = connection.source.node.upgrade().and_then(|node|self.node_to_id.borrow().get(&node.id()).cloned())?;
        let dst_node = connection.destination.node.upgrade().and_then(|node|self.node_to_id.borrow().get(&node.id()).cloned())?;
        Some(DisplayedConnection {
            source      : DisplayedEndpoint {node:src_node, port:connection.source.port},
            destination : DisplayedEndpoint {node:dst_node, port:connection.destination.port}
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
