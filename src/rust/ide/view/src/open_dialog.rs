pub mod project_list;

use crate::prelude::*;

use ensogl::display;
use ensogl_gui_components::file_browser;
use ensogl_gui_components::file_browser::FileBrowser;
use ensogl::application::Application;
use ensogl::display::object::Instance;



const WIDTH:f32  = file_browser::WIDTH + project_list::WIDTH + GAP;
const HEIGHT:f32 = file_browser::HEIGHT;
const GAP:f32    = 16.0;



#[derive(Clone,CloneRef,Debug)]
pub struct OpenDialog {
    logger                      : Logger,
    pub project_list            : project_list::ProjectList,
    pub file_browser            : FileBrowser,
    display_object              : display::object::Instance,
}

impl OpenDialog {
    pub fn new(app:&Application) -> Self {
        let logger         = Logger::new("OpenDialog");
        let project_list   = project_list::ProjectList::new(app);
        let file_browser   = app.new_view::<FileBrowser>();
        let display_object = display::object::Instance::new(&logger);

        display_object.add_child(&project_list);
        project_list.set_position_x(-WIDTH / 2.0 + project_list::WIDTH / 2.0);

        display_object.add_child(&file_browser);
        file_browser.set_position_x(WIDTH / 2.0 - file_browser::WIDTH / 2.0);

        app.display.scene().layers.panel.add_exclusive(&display_object);

        Self {logger,project_list,file_browser,display_object}
    }
}

impl display::Object for OpenDialog {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}
