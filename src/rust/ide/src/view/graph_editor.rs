use crate::prelude::*;

use crate::controller::module::Position;
use crate::executor::global::spawn;
use crate::view::temporary_panel::TemporaryPadding;

use ensogl::display;
use ensogl::display::object::ObjectOps;
use ensogl::display::Sprite;
use ensogl::display::world::World;
use std::any::TypeId;
use nalgebra::zero;
use ast::HasRepr;
use graph::Graph;
use utils::channel::process_stream_with_handle;
use js_sys::Atomics::sub;


// ====================
// === Graph Editor ===
// ====================


#[derive(Clone,Debug)]
pub struct GraphEditor {
    display_object : display::object::Node,
    graph          : Rc<RefCell<Graph>>,
    controller     : controller::graph::Handle,
    logger         : Logger,
}

impl GraphEditor {
    pub fn new(logger:&Logger, world:&World, controller:controller::graph::Handle) -> Self {
        let logger         = logger.sub("GraphEditor");
        let display_object = display::object::Node::new(&logger);
        let graph          = graph::Graph::new(world);
        display_object.add_child(&graph);
        let graph          = Rc::new(RefCell::new(graph));
        let editor         = GraphEditor{display_object,controller,logger,graph};
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
        let logger     = self.logger.clone();
        spawn(process_stream_with_handle(subscribe,weak_graph, move |notification,graph| {
            match notification {
                controller::notification::Graph::Invalidate => {
                    Self::update_graph(graph.deref(),&controller)
                }
            }
            futures::future::ready(())
        }));
    }

    fn update_graph(graph:&RefCell<Graph>, controller:&controller::graph::Handle) {
        graph.borrow_mut().clear_graph();
        if let Ok(nodes_info) = controller.nodes() {
            let nodes_with_index = nodes_info.iter().enumerate();
            let new_nodes        = nodes_with_index.map(|(i,n)| Self::map_controller_node(i,n));
            let graph_ref        = graph.borrow_mut();
            for node in new_nodes { graph_ref.add_node(node) }
        }
    }

    fn map_controller_node(index_in_graph:usize, controller_node:&controller::graph::Node)
    -> graph::Node {
        let mut node     = graph::Node::new();
        let default_pos  = Vector2::new(50.0 * index_in_graph as f32, 0.0);
        let opt_position = controller_node.metadata.and_then(|md| md.position).map(|p| p.vector);
        let position     = opt_position.unwrap_or(default_pos);
        let id           = controller_node.info.id();
        let expression   = controller_node.info.expression().repr();
        node.set_label(expression);
        node.set_position(Vector3::new(position.x,position.y,0.0));
        node
    }
}

impl<'t> From<&'t GraphEditor> for &'t display::object::Node {
    fn from(graph_editor:&'t GraphEditor) -> Self {
        &graph_editor.display_object
    }
}
