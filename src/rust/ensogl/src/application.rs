//! Application top-level structure definition. Handles views, keyboard shortcuts and more.

pub mod command;
pub mod shortcut;
pub mod view;

pub use view::View;

use crate::prelude::*;

use crate::display;
use crate::display::world::World;
use crate::display::style::theme;
use crate::gui::cursor::Cursor;
use crate::system::web;
use ensogl_system_web::StyleSetter;



// ===================
// === Application ===
// ===================

/// A top level structure for an application. It combines a view, keyboard shortcut manager, and is
/// intended to also manage layout of visible panes.
#[derive(Debug,Clone,CloneRef)]
#[allow(missing_docs)]
pub struct Application {
    pub logger    : Logger,
    pub cursor    : Cursor,
    pub display   : World,
    pub commands  : command::Registry,
    pub shortcuts : shortcut::Registry,
    pub views     : view::Registry,
    pub themes    : theme::Manager,
}

impl Application {
    /// Constructor.
    pub fn new(dom:&web_sys::HtmlElement) -> Self {
        let logger    = Logger::new("Application");
        let display   = World::new(dom);
        let scene     = display.scene();
        let commands  = command::Registry::create(&logger);
        let shortcuts = shortcut::Registry::new(&logger,&scene.mouse.frp,&scene.keyboard.frp,&commands);
        let views     = view::Registry::create(&logger,&display,&commands,&shortcuts);
        let themes    = theme::Manager::from(&display.scene().style_sheet);
        let cursor    = Cursor::new(display.scene());
        display.add_child(&cursor);
        web::body().set_style_or_panic("cursor","none");
        Self {logger,cursor,display,commands,shortcuts,views,themes}
    }

    /// Create a new instance of a view.
    pub fn new_view<T:View>(&self) -> T {
        self.views.new_view(self)
    }
}

impl display::Object for Application {
    fn display_object(&self) -> &display::object::Instance {
        self.display.display_object()
    }
}
