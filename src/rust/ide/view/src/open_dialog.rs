use crate::prelude::*;

use enso_frp as frp;
use ensogl::display;
use ensogl_gui_components::list_view;
use ensogl_gui_components::list_view::ListView;
use ensogl_gui_components::file_browser;
use ensogl_gui_components::file_browser::FileBrowser;
use ensogl::application::Application;



const PROJECT_LIST_WIDTH:f32 = 202.0;
const WIDTH:f32              = file_browser::WIDTH + PROJECT_LIST_WIDTH;
const HEIGHT:f32             = file_browser::HEIGHT;


#[derive(Clone,CloneRef,Debug)]
pub struct OpenDialog {
    logger           : Logger,
    pub project_list : ListView,
    pub file_browser : FileBrowser,
    display_object   : display::object::Instance,
}

impl OpenDialog {
    pub fn new(app:&Application) -> Self {
        let logger       = Logger::new("OpenDialog");
        let project_list = app.new_view::<ListView>();
        let file_browser = app.new_view::<FileBrowser>();
        let display_object = display::object::Instance::new(&logger);

        display_object.add_child(&project_list);
        project_list.resize(Vector2(PROJECT_LIST_WIDTH,HEIGHT));
        project_list.set_position_y(-WIDTH / 2.0 + PROJECT_LIST_WIDTH / 2.0);

        display_object.add_child(&file_browser);
        file_browser.set_position_y(WIDTH / 2.0 - file_browser::WIDTH / 2.0);

        app.display.scene().layers.panel.add_exclusive(&display_object);

        Self {logger,project_list,file_browser,display_object}
    }
}

impl display::Object for OpenDialog {
    fn display_object(&self) -> &display::object::Instance { &self.display_object }
}
