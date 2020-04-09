#![allow(missing_docs)] // FIXME

use crate::prelude::*;

use crate::notification;

use ensogl::display::traits::*;
use ensogl::display;
use ensogl::display::world::World;
use graph_editor::GraphEditor;
use graph_editor::component::node::Node;
use graph_editor::component::node::WeakNode;
use utils::channel::process_stream_with_handle;
use enso_frp::stream::EventEmitter;
use enso_frp::Position;


// ====================
// === Node Editor ===
// ====================

struct GraphEditorIntegration<NodeMovedCb,NodeRemovedCb> {
    editor       : GraphEditor,
    id_to_node   : RefCell<WeakValueHashMap<ast::Id, WeakNode>>,
    node_to_id   : RefCell<WeakKeyHashMap<WeakNode, ast::Id>>,
    node_moved   : NodeMovedCb,
    node_removed : NodeRemovedCb,
}

impl<Cb1,Cb2> GraphEditorIntegration<Cb1,Cb2> {
    fn retain_ids(&self, ids:&HashSet<ast::Id>) {
        for (id,node) in self.id_to_node.borrow().iter() {
            if !ids.contains(id) {
                todo!()
            }
        }
    }

    fn add_node(&self, id:ast::Id, position:&Position) {
        self.editor.frp.add_node_at.emit_event(&position);
        let node = default(); // FIXME;
        self.id_to_node.borrow_mut().insert(id,node);
        self.node_to_id.borrow_mut().insert(node,id);
    }
}

impl <NodeMovedCb,NodeRemovedCb> GraphEditorIntegration<NodeMovedCb,NodeRemovedCb>
where NodeMovedCb   : Fn(ast::Id,Position),
      NodeRemovedCb : Fn(ast::Id) {

    fn new(world:&World, node_moved:NodeMovedCb, node_removed:NodeRemovedCb) -> Self {
        let editor     = graph_editor::GraphEditor::new(world);
        let id_to_node = default();
        let node_to_id = default();
        GraphEditorIntegration {editor,id_to_node,node_to_id,node_moved,node_removed}
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct NodeEditor {
    display_object : display::object::Node,
    graph          : Rc<GraphEditor>,
    controller     : controller::graph::Handle,
    logger         : Logger,
}

impl NodeEditor {
    pub fn new(logger:&Logger, world:&World, controller:controller::graph::Handle) -> Self {
        let logger         = logger.sub("GraphEditor");
        let display_object = display::object::Node::new(&logger);
        let graph          = Rc::new(graph_editor::GraphEditor::new(world));
        display_object.add_child(graph.deref());
        let editor         = NodeEditor {display_object,graph,controller,logger};
        editor.initialize()
    }

    fn initialize(self) -> Self {
        Self::update_graph(self.graph.deref(),&self.controller);
        self.setup_controller_notifications();
        self
    }

    fn setup_controller_notifications(&self) {
        let subscribe  = self.controller.subscribe();
        let weak_graph = Rc::downgrade(&self.graph);
        let controller = self.controller.clone();
        executor::global::spawn(process_stream_with_handle(subscribe,weak_graph, move |notification,graph| {
            match notification {
                notification::Graph::Invalidate => {
                    Self::update_graph(graph.deref(),&controller)
                }
            }
            futures::future::ready(())
        }));
    }

    fn update_graph(graph:&GraphEditor, controller:&controller::Graph) {
        graph.frp.clear_graph.emit(());
        if let Ok(nodes_info) = controller.nodes() {
            let nodes_with_index = nodes_info.iter().enumerate();
            let nodes_positions  = nodes_with_index.map(|(i,n)| n.metadata.and_then(|m| m.position).map(|p| enso_frp::Position::new(p.vector.x, p.vector.y)).unwrap_or_else(|| enso_frp::Position::new(i as f32 * 100.0,0.0)));
            for pos in nodes_positions { graph.frp.add_node_at.emit(pos) }
        }
    }
}

impl<'t> From<&'t NodeEditor> for &'t display::object::Node {
    fn from(graph_editor:&'t NodeEditor) -> Self {
        &graph_editor.display_object
    }
}
