//! A Code Editor component.

use crate::prelude::*;

use crate::documentation;

use enso_frp as frp;
use enso_frp::io::keyboard::Key;
use ensogl::application;
use ensogl::application::{Application, shortcut};
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl_text as text;

pub use ensogl_gui_list_view::entry;



// =================
// === Constants ===
// =================


pub const HEIGHT : f32 = 200.0;
/// The padding between text area and scene left boundary.
pub const PADDING_LEFT : f32 = 7.0;



// ===========
// === Frp ===
// ===========

ensogl::def_command_api!( Commands
    /// Show the Code Editor.
    show,
    /// Hide the Code Editor.
    hide,
    /// Toggle Code Editor visibility.
    toggle,
);

ensogl_text::define_endpoints! {
    Commands { Commands }
    Input {
    }
    Output {
        scroll_position (f32),
        is_shown        (bool),
    }
}


// ============
// === View ===
// ============

struct View {
    model : text::Area,
    frp   : Frp,
}

impl View {
    pub fn new(app:&Application) -> Self {
        let frp     = Frp::new_network();
        let network = &frp.network;
        let model   = app.new_view::<text::Area>();
        let height  = Animation::<f32>::new(network);

        model.set_position_x(PADDING_LEFT);

        frp::extend!{ network
            let is_shown      =  frp.output.is_shown.clone_ref();
            is_hidden         <- is_shown.map(|b| !b);
            show_after_toggle <- frp.toggle.gate(&is_hidden);
            hide_after_toggle <- frp.toggle.gate(&is_shown);
            show              <- any(frp.input.show,show_after_toggle);
            hide              <- any(frp.input.hide,hide_after_toggle);

            eval_ show (() height.set_target_value(HEIGHT));
            eval_ hide (() height.set_target_value(0.0));
            eval_ show (() model.set_active_on());
            eval_ hide (() model.set_active_off());
            frp.source.is_shown <+ bool(hide,show);

            position <- all_with(height_fraction.value,f!(h)
                model.set_position_y(h)
            )
        }

        Self{model,frp}
    }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object() }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::command::CommandApi for View {
    fn command_api_docs() -> Vec<application::command::EndpointDocs> {
        Commands::command_api_docs()
    }

    fn command_api(&self) -> Vec<application::command::CommandEndpoint> {
        self.frp.input.command.command_api()
    }
}

impl application::command::Provider for View {
    fn label() -> &'static str { "CodeEditor" }
}

impl application::View for View {
    fn new(app: &Application) -> Self { Self::new(app) }
}

impl application::shortcut::DefaultShortcutProvider for View {
    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        vec!
        [ Self::self_shortcut(shortcut::Action::press   (&[Key::Control,Key::Character("`".into())], &[]) , "toggle"),
        ]
    }
}
