//! This module contains implementation of ViewLayout with a single TextEditor temporarily
//! occupying half bottom of the screen as the default layout.

use crate::prelude::*;

use crate::view::temporary_panel::TemporaryPadding;
use crate::view::temporary_panel::TemporaryPanel;
use crate::view::text_editor::TextEditor;
use crate::view::node_editor::NodeEditor;
use crate::view::node_searcher::NodeSearcher;

use enso_frp::io::keyboard;
use ensogl::application::Application;
use ensogl::display::shape::text::glyph::font;
use ensogl::display::traits::*;
use ensogl::display::world::World;
use nalgebra::Vector2;
use nalgebra::zero;
use std::cell::RefCell;
use std::rc::Rc;
use graph_editor::GraphEditor;


// ==================
// === ViewLayout ===
// ==================

shared! { ViewLayout

/// Initial implementation of ViewLayout with a TextEditor and NodeEditor.
#[derive(Debug)]
pub struct ViewLayoutData {
    text_editor   : TextEditor,
    node_editor   : NodeEditor,
    node_searcher : NodeSearcher,
    size          : Vector2<f32>,
    logger        : Logger
}

impl {

    /// Sets ViewLayout size.
    pub fn set_size(&mut self, size:Vector2<f32>) {
        self.size = size;
        self.recalculate_layout();
    }
}}


// === Private Methods ===

impl ViewLayoutData {

    fn recalculate_layout(&mut self) {
        self.update_text_editor();
        self.update_graph_editor();
        self.update_node_searcher();
    }

    fn update_text_editor(&mut self) {
        let screen_size = self.size;
        let position    = Vector2::new(0.0,screen_size.y / 2.0);
        let size        = Vector2::new(screen_size.x,screen_size.y / 2.0);
        let padding     = TemporaryPadding {
            left   : 10.0,
            top    : 0.0,
            right  : 10.0,
            bottom : 0.0
        };
        self.text_editor.set_padding(padding);
        self.text_editor.set_size(size);
        TemporaryPanel::set_position(&mut self.text_editor,position);
    }

    fn update_graph_editor(&mut self) {
        let screen_size  = self.size;
        let position     = Vector3::new(50.0, screen_size.y * 3.0 / 4.0, 0.0);
        self.node_editor.set_position(position);
    }

    fn update_node_searcher(&mut self) {
        let screen_size = self.size;
        let position    = Vector3::new(screen_size.x*2.0/3.0, screen_size.y - 10.0, 0.0);
        self.node_searcher.set_position(position);
    }
}

impl ViewLayout {
    /// Creates a new ViewLayout with a single TextEditor.
    pub async fn new
    ( logger                   : &Logger
    , kb_actions               : &mut keyboard::Actions
    , application              : &Application
    , text_controller          : controller::Text
    , graph_controller         : controller::ExecutedGraph
    , visualization_controller : controller::Visualization
    , fonts                    : &mut font::Registry
    ) -> FallibleResult<Self> {
        let logger           = logger.sub("ViewLayout");
        let world            = &application.display;
        let text_editor      = TextEditor::new(&logger,world,text_controller,kb_actions,fonts);
        let graph            = graph_controller.graph.clone_ref();
        let node_searcher    = NodeSearcher::new(world,&logger,graph,fonts);
        let graph_controller = graph_controller.clone_ref();
        let node_editor      = NodeEditor::new
            (&logger,application,graph_controller,visualization_controller).await?;
        world.add_child(&text_editor.display_object());
        world.add_child(&node_editor);
        world.add_child(&node_searcher);
        let size = zero();
        let data = ViewLayoutData {text_editor,node_editor,node_searcher,size,logger};
        let rc   = Rc::new(RefCell::new(data));
        Ok(Self {rc}.init(world,kb_actions))
    }

    fn init_keyboard(self, _keyboard_actions:&mut keyboard::Actions) -> Self {
        // TODO[ao] add here some useful staff (quitting project for example)
        self
    }

    fn init(self, world:&World, keyboard_actions:&mut keyboard::Actions) -> Self {
        let screen = world.scene().camera().screen();
        let size   = Vector2::new(screen.width,screen.height);
        self.set_size(size);
        self.init_keyboard(keyboard_actions)
    }

    /// Get GraphEditor.
    pub fn graph_editor(&self) -> GraphEditor {
        self.rc.borrow_mut().node_editor.graph.graph_editor()
    }
}
