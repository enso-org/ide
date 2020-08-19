use crate::prelude::*;


use enso_frp as frp;
use ensogl::application;
use ensogl::application::Application;
use ensogl::display;
use ensogl_gui_list_view::entry;
use ensogl_gui_list_view::ListView;

ensogl::def_command_api!( Commands
    /// Pick the selected suggestion and add it to the current input.
    pick_suggestion,
);

ensogl_text::define_endpoints! {
    Commands { Commands }
    Input {
        resize           (Vector2<f32>),
        set_entries      (entry::AnyModelProvider),
    }
    Output {
        selected_entry  (Option<entry::Id>),
        picked_entry    (Option<entry::Id>),
        commited_entry  (Option<entry::Id>),
        size            (Vector2<f32>),
    }
}

#[derive(Clone,CloneRef,Debug)]
struct Model {
    logger         : Logger,
    display_object : display::object::Instance,
    list           : ListView,
}

impl Model {
    pub fn new(app:&Application) -> Self {
        let logger         = Logger::new("SearcherView");
        let display_object = display::object::Instance::new(&logger);
        let list           = ListView::new(app);
        display_object.add_child(&list);
        Self{logger,display_object,list}
    }
}

#[derive(Clone,CloneRef,Debug)]
pub struct View {
    model   : Model,
    pub frp : Frp,
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target { &self.frp }
}

impl View {
    pub fn new(app:&Application) -> Self {
        let model = Model::new(app);
        let frp   = Frp::new_network();
        Self{model,frp}.init()
    }

    fn init(self) -> Self {
        let network = &self.frp.network;
        let model   = &self.model;
        let frp     = &self.frp;
        let source  = &self.frp.source;

        frp::extend! {network
            eval frp.resize      ((size)    model.list.resize(size));
            eval frp.set_entries ((entries) model.list.set_entries(entries));
            source.selected_entry <+ model.list.selected_entry;
            source.commited_entry <+ model.list.chosen_entry;
            source.size           <+ model.list.size;

            is_selected         <- model.list.selected_entry.map(|e| e.is_some());
            opt_picked_entry    <- model.list.selected_entry.sample(&frp.pick_suggestion);
            source.picked_entry <+ opt_picked_entry.gate(&is_selected);
        }

        self
    }
}
impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object }
}
