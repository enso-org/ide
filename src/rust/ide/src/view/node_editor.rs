#![allow(missing_docs)] // FIXME

use crate::prelude::*;

use crate::notification;

use ensogl::display;
use ensogl::display::object::Id;
use ensogl::display::traits::*;
use ensogl::display::world::World;
use ensogl::system::web;
use graph_editor::GraphEditor;
use graph_editor::component::node::Node;
use graph_editor::component::node::WeakNode;
use utils::channel::process_stream_with_handle;
use enso_frp::stream::EventEmitter;
use enso_frp::Position;
use wasm_bindgen::JsCast;
use weak_table::weak_value_hash_map::Entry::{Occupied, Vacant};



// ===================
// === Node Editor ===
// ===================

#[derive(Debug)]
struct GraphEditorIntegration {
    pub editor     : GraphEditor,
    pub controller : controller::Graph,
    id_to_node     : RefCell<WeakValueHashMap<ast::Id, WeakNode>>,
    node_to_id     : RefCell<WeakKeyHashMap<WeakNode, ast::Id>>,
    pub logger     : Logger,

}

impl GraphEditorIntegration {
    fn retain_ids(&self, ids:&HashSet<ast::Id>) {
        for (id,node) in self.id_to_node.borrow().iter() {
            if !ids.contains(id) {
                self.editor.remove_node(node.downgrade())
            }
        }
    }

    fn invalidate_graph(&self) -> FallibleResult<()> {
        let nodes = self.controller.nodes()?;
        let ids   = nodes.iter().map(|node| node.info.id() ).collect();
        Logger::new("DEBUG").error(|| format!("INVALIDATE {:?}", ids));
        self.retain_ids(&ids);
        for (i,node_info) in nodes.iter().enumerate() {
            let id          = node_info.info.id();
            let position    = node_info.metadata.and_then(|md| md.position);
            let default_pos = || Vector3::new(i as f32 * 100.0,0.0,0.0);
            match self.id_to_node.borrow_mut().entry(id) {
                Occupied(entry) => if let Some(pos) = position {
                    entry.get().set_position(Self::pos_to_vec3(pos));
                },
                Vacant(entry)   => {
                    let node = self.editor.add_node().upgrade().unwrap();
                    node.set_position(position.map_or_else(default_pos,Self::pos_to_vec3));
                    entry.insert(node.clone_ref());
                    self.node_to_id.borrow_mut().insert(node,id);
                }
            }
        }
        Ok(())
    }

    fn pos_to_vec3(pos:model::module::Position) -> Vector3<f32> {
        Vector3::new(pos.vector.x,pos.vector.y,0.0)
    }
}

impl GraphEditorIntegration {

    fn new(world:&World, controller:controller::Graph) -> Rc<Self> {
        let editor     = graph_editor::GraphEditor::new(world);
        let id_to_node = default();
        let node_to_id = default();
        let logger     = Logger::new("Node Editor");
        let this = Rc::new(GraphEditorIntegration {editor,controller,id_to_node,node_to_id,logger});
        Self::setup_controller_event_handling(&this);
        Self::setup_keyboard_event_handling(&this);
        Self::setup_mouse_event_handling(&this);
        this
    }

    fn setup_controller_event_handling(this:&Rc<Self>) {
        let stream  = this.controller.subscribe();
        let weak    = Rc::downgrade(this);
        let handler = process_stream_with_handle(stream,weak,move |notification,this| {
            match notification {
                notification::Graph::Invalidate => this.invalidate_graph(),
            };
            futures::future::ready(())
        });
        executor::global::spawn(handler);
    }

    fn setup_keyboard_event_handling(this:&Rc<Self>) {
        /// TODO [ao] replace with actual keybindings management.
        let weak = Rc::downgrade(this);
        let c: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |val| {
            if let Some(this) = weak.upgrade() {
                let val = val.unchecked_into::<web_sys::KeyboardEvent>();
                let key = val.key();
                if key == "Backspace" && val.ctrl_key() {
                    this.editor.selected_nodes.for_each(|node| {
                        let id = this.node_to_id.borrow().get(&node.id()).cloned();
                        if let Some(id) = id {
                            if let Err(err) = this.controller.remove_node(id) {
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
        this.editor.frp.network.map("module_update", &this.editor.frp.nodes.release, move |node| {
            let node = node.as_ref().and_then(|n| n.upgrade());
            let this = weak.upgrade();
            if let Some((node,this)) = node.and_then(|n| this.map(|t| (n,t))) {
                let id = this.node_to_id.borrow().get(&node.id()).cloned();
                if let Some(id) = id {
                    this.controller.module.with_node_metadata(id, |md| {
                        let pos = node.position();
                        md.position = Some(model::module::Position::new(pos.x, pos.y));
                    })
                }
            }
        });
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct NodeEditor {
    display_object : display::object::Instance,
    graph          : Rc<GraphEditorIntegration>,
    controller     : controller::graph::Handle,
}

impl NodeEditor {
    pub fn new(logger:&Logger, world:&World, controller:controller::graph::Handle) -> Self {
        let graph          = GraphEditorIntegration::new(world,controller.clone_ref());
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
