use crate::prelude::*;

use crate::Mode as EditorMode;
use crate::component::node;
use node::ExecutionStatus;

use enso_frp as frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display;
use ensogl::gui::text;



// =================
// === Constants ===
// =================

const LABEL_OFFSET_Y: f32 = 35.0;



// ============
// === Frp  ===
// ============

ensogl::define_endpoints! {
    Input {
        set_execution_status    (ExecutionStatus),
        set_min_global_duration (f32),
        set_max_global_duration (f32),
        set_editor_mode         (EditorMode),
    }
}



// ===========================
// === Profiling Indicator ===
// ===========================

#[derive(Clone,CloneRef,Debug)]
pub struct ProfilingIndicator {
    root  : display::object::Instance,
    label : text::Area,
    frp   : Frp,
}

impl Deref for ProfilingIndicator {
    type Target = Frp;

    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}


impl ProfilingIndicator {
    pub fn new(app: &Application) -> Self {
        let scene  = app.display.scene();
        let styles = StyleWatch::new(&scene.style_sheet);
        let root   = display::object::Instance::new(Logger::new("ProfilingIndicator"));

        let label = text::Area::new(app);
        root.add_child(&label);
        label.set_position_y(LABEL_OFFSET_Y);
        label.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label.add_to_scene_layer_DEPRECATED(&scene.layers.label);

        let frp     = Frp::new();
        let network = &frp.network;
        let color   = color::Animation::new(network);

        frp::extend! { network

            // === Visibility ===

            visibility <- all_with(&frp.set_editor_mode,&frp.set_execution_status,|mode,status| {
                matches!((mode,status),(EditorMode::Profiling,ExecutionStatus::Finished {..}))
            });

            color.target_alpha <+ visibility.map(|&is_visible| {
                if is_visible { 1.0 } else { 0.0 }
            });


            // === Color ===

            color.target_color <+ all_with3
                (&frp.set_execution_status,&frp.set_min_global_duration,
                &frp.set_max_global_duration,f!([styles](&status,&min,&max) {
                    let theme         = ensogl_theme::graph_editor::node::profiling::HERE.path();
                    let lightness     = styles.get_number(theme.sub("lightness"));
                    let chroma        = styles.get_number(theme.sub("chroma"));
                    let min_hue       = styles.get_number(theme.sub("min_hue"));
                    let max_hue       = styles.get_number(theme.sub("max_hue"));
                    let hue_delta     = max_hue - min_hue;
                    let running_color = color::Lch::new(lightness,chroma,max_hue);
                    match status {
                        ExecutionStatus::Running             => running_color,
                        ExecutionStatus::Finished {duration} => {
                            let relative_duration = (duration - min) / (max - min);
                            let hue               = min_hue + relative_duration * hue_delta;
                            color::Lch::new(lightness,chroma,hue)
                        }
                    }
                })
            );
            label.set_default_color <+ color.value.map(|c| c.into());
            label.set_color_all     <+ color.value.map(|c| c.into());


            // === Position ===

            eval label.width((&width) label.set_position_x(-width/2.0));


            // === Content ===

            label.set_content <+ frp.set_execution_status.map(|&status|
                match status {
                    ExecutionStatus::Running             => "".to_string(),
                    ExecutionStatus::Finished {duration} => format!("{} ms", duration)
                });
        }

        ProfilingIndicator {root,label,frp}
    }
}


impl display::Object for ProfilingIndicator {
    fn display_object(&self) -> &display::object::Instance {
        &self.root
    }
}
