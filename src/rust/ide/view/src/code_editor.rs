//! A Code Editor component.

use crate::prelude::*;

use enso_frp as frp;
use ensogl::application;
use ensogl::application::{Application, shortcut};
use ensogl::display;
use ensogl::gui::component::Animation;
use ensogl_text as text;

pub use ensogl_gui_list_view::entry;



// =================
// === Constants ===
// =================

/// The height of code editor in project view.
pub const HEIGHT_FRACTION : f32 = 0.3;
/// The padding between text area and scene left boundary.
pub const PADDING_LEFT : f32 = 7.0;



// ===========
// === Frp ===
// ===========

ensogl::define_endpoints! {
    Input {
        /// Show the Code Editor.
        show(),
        /// Hide the Code Editor.
        hide(),
        /// Toggle Code Editor visibility.
        toggle(),
    }

    Output {
        is_shown (bool),
    }
}



// ============
// === View ===
// ============

/// The View of IDE Code Editor.
#[derive(Clone,CloneRef,Debug)]
pub struct View {
    model : text::Area,
    frp   : Frp,
}

impl Deref for View {
    type Target = Frp;
    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl View {
    /// Create Code Editor component.
    pub fn new(app:&Application) -> Self {
        let scene           = app.display.scene();
        let frp             = Frp::new_network();
        let network         = &frp.network;
        let model           = app.new_view::<text::Area>();
        let height_fraction = Animation::<f32>::new(network);

        model.set_position_x(PADDING_LEFT);
        model.set_active(true);
        model.remove_from_view(&scene.views.main);
        model.add_to_view(&scene.views.breadcrumbs);

        frp::extend!{ network
            trace frp.input.toggle;
            let is_shown      =  frp.output.is_shown.clone_ref();
            show_after_toggle <- frp.toggle.gate_not(&is_shown);
            hide_after_toggle <- frp.toggle.gate    (&is_shown);
            show              <- any(frp.input.show,show_after_toggle);
            hide              <- any(frp.input.hide,hide_after_toggle);

            eval_ show (height_fraction.set_target_value(HEIGHT_FRACTION));
            eval_ hide (height_fraction.set_target_value(0.0));
            eval_ hide (model.remove_all_cursors());

            frp.source.is_shown <+ bool(&frp.input.hide,&frp.input.show);
            frp.source.is_shown <+ frp.toggle.map2(&is_shown, |(),b| !b);

            let shape  = app.display.scene().shape();
            position <- all_with(&height_fraction.value,shape, |height_f,scene_size| {
                let height = height_f * scene_size.height;
                let x      = -scene_size.width  / 2.0 + PADDING_LEFT;
                let y      = -scene_size.height / 2.0 + height;
                Vector2(x,y)
            });
            eval position ((pos) model.set_position_xy(*pos));
        }

        Self{model,frp}
    }

    /// Return the Text Area component inside this editor.
    pub fn text_area(&self) -> &text::Area { &self.model }
}

impl display::Object for View {
    fn display_object(&self) -> &display::object::Instance { &self.model.display_object() }
}

impl application::command::FrpNetworkProvider for View {
    fn network(&self) -> &frp::Network { &self.frp.network }
}

impl application::View for View {
    fn label() -> &'static str { "CodeEditor" }

    fn new(app: &Application) -> Self { Self::new(app) }

    fn default_shortcuts() -> Vec<shortcut::Shortcut> {
        use shortcut::ActionType::*;
        (&[ (Press, "ctrl `" , "toggle")
          , (Press, "escape", "hide"  )
        ]).iter().map(|(a,b,c)|Self::self_shortcut(*a,*b,*c)).collect()
    }
}
