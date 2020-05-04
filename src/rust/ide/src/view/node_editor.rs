//! View of the node editor.

use crate::prelude::*;

use crate::notification;
use crate::controller::graph::NodeTrees;

use ensogl::display;
use ensogl::display::traits::*;
use ensogl::system::web;
use ensogl::application::Application;
use graph_editor::GraphEditor;
use graph_editor::component::node::WeakNode;
use utils::channel::process_stream_with_handle;
use wasm_bindgen::JsCast;
use weak_table::weak_key_hash_map;
use weak_table::weak_value_hash_map;

use enso_frp::stream::EventEmitter;


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
        Self::setup_keyboard_event_handling(&this);
        Self::setup_mouse_event_handling(&this);
        if let Err(err) = this.invalidate_graph() {
            error!(this.logger,"Error while initializing graph display: {err}");
        }
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

    fn setup_keyboard_event_handling(this:&Rc<Self>) {
        // TODO [ao] replace with actual keybindings management.
        let weak = Rc::downgrade(this);
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            if let Some(this) = weak.upgrade() {
                let val = val.unchecked_into::<web_sys::KeyboardEvent>();
                let key = val.key();
                if key == "Backspace" && val.ctrl_key() {
                    this.editor.nodes.selected.for_each(|node_id| {
                        let id = this.node_to_id.borrow().get(&node_id.0).cloned(); // FIXME .0
                        if let Some(id) = id {
                            if let Err(err) = this.controller.graph.remove_node(id) {
                                this.logger.error(|| format!("ERR: {:?}", err));
                            }
                        }
                    });
                    this.editor.frp.remove_selected_nodes.emit(())
                }
            }
        }));
        web::document().add_event_listener_with_callback("keydown",c.as_ref().unchecked_ref()).unwrap();
        c.forget();
    }

    fn setup_mouse_event_handling(this:&Rc<Self>) {
        let weak = Rc::downgrade(this);
        let editor = this.editor.clone_ref();
        this.editor.network.map("module_update", &this.editor.frp.node_release, move |node_id| {
            let node_pos = editor.get_node_position(*node_id);
            let this = weak.upgrade();
            if let Some((node_pos,this)) = node_pos.and_then(|n| this.map(|t| (n,t))) {
                let id = this.node_to_id.borrow().get(&node_id.0).cloned(); // FIXME .0
                if let Some(id) = id {
                    this.controller.graph.module.with_node_metadata(id, |md| {
                        md.position = Some(model::module::Position::new(node_pos.x, node_pos.y));
                    })
                }
            }
        });
        // this.editor.frp.network.map("connection_created", &this.editor.frp.connections_added_by_command, move |connections| {
        //     if let Some(this) = weak.upgrade() {
        //         this.controller.connect()
        //     }
        // });
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
