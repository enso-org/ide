//! Provides [`Status`] to represent a node's execution status, [`Status::display_color`] to express
//! that status as a color and [`RunningTimeLabel`] to display a node's execution status.

use crate::prelude::*;

use crate::Mode as EditorMode;

use enso_frp as frp;
use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl::display;
use ensogl::gui::text;
use ensogl_theme::graph_editor::node::profiling as theme_path;



// =================
// === Constants ===
// =================

const LABEL_OFFSET_Y: f32 = 35.0;



// ==============
// === Status ===
// ==============

/// Describes whether the source code in a node is currently running or already finished. If it is
/// finished then the status contains the number of milliseconds that it took to run the code.
#[derive(Debug,Copy,Clone)]
pub enum Status {
    /// The node's code is still running.
    Running,
    /// The node finished execution.
    Finished {
        /// How many milliseconds the node took to execute.
        duration: f32,
    }
}

impl Default for Status {
    fn default() -> Self {
        Status::Running
    }
}

impl Status {
    /// Returns `true` if the node is still running.
    pub fn is_running(self) -> bool {
        matches!(self,Status::Running)
    }

    /// Returns `true` if the node finished execution.
    pub fn is_finished(self) -> bool {
        matches!(self,Status::Finished {..})
    }
}



// =============
// === Color ===
// =============

/// A theme that determines how we express the running time of a node (compared to other nodes on
/// the stage) in a color. The color's lightness and chroma in LCh color space are directly taken
/// from the theme. The chroma will be `min_time_hue` for the node with the shortest running time,
/// `max_time_hue` for the node with the longest running time and linearly interpolated in-between
/// depending on the relative running, time for all other nodes.
#[derive(Debug,Copy,Clone,Default)]
pub struct Theme {
    /// The lightness for all running times.
    pub lightness    : f32,
    /// The chroma for all running times.
    pub chroma       : f32,
    /// The hue for the minimum running time.
    pub min_time_hue : f32,
    /// The hue for the maximum running time.
    pub max_time_hue : f32,
}

impl Status {
    /// Expresses the profiling status as a color, depending on the minimum and maximum running
    /// time of any node on the stage and a [`Theme`] that allows to tweak how the colors are
    /// chosen. A node that is still running will be treated like finished node with the current
    /// maximum execution time.
    pub fn display_color
    (self, min_global_duration: f32, max_global_duration: f32, theme: Theme) -> color::Lch {
        let duration = match self {
            Status::Running => max_global_duration,
            Status::Finished {duration} => duration,
        };
        let duration_delta = max_global_duration - min_global_duration;
        let hue_delta = theme.max_time_hue - theme.min_time_hue;
        let relative_duration = if duration_delta != 0.0 {
            (duration - min_global_duration) / duration_delta
        } else {
            0.0
        };
        let relative_hue = relative_duration;
        let hue = theme.min_time_hue + relative_hue * hue_delta;
        color::Lch::new(theme.lightness, theme.chroma, hue)
    }
}



// ============
// === Frp  ===
// ============

ensogl::define_endpoints! {
    Input {
        set_status              (Status),
        set_min_global_duration (f32),
        set_max_global_duration (f32),
        set_editor_mode         (EditorMode),
    }
}



// ==========================
// === Running Time Label ===
// ==========================

/// A `display::Object` providing a label for nodes that displays the node's running time in
/// profiling mode after the node finished execution. The node's execution status has to be provided
/// through `set_status`, the diplay mode through `set_editor_mode`, the minimum and maximum running
/// time of any node on the stage through `set_min_global_duration` and `set_max_global_duration`.
/// The color of the label will reflect the status and be determined by [`Status::display_color`].
/// The necessary theme will be taken from the application's style sheet. The origin of the label,
/// as a `display::Object` should be placed on the node's center.
#[derive(Clone,CloneRef,Debug)]
pub struct RunningTimeLabel {
    root    : display::object::Instance,
    label   : text::Area,
    frp     : Frp,
    styles  : StyleWatchFrp,
}

impl Deref for RunningTimeLabel {
    type Target = Frp;

    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}

impl RunningTimeLabel {
    /// Constructs a `RunningTimeLabel` for the given application.
    pub fn new(app: &Application) -> Self {
        let scene = app.display.scene();
        let root  = display::object::Instance::new(Logger::new("ProfilingIndicator"));

        let label = text::Area::new(app);
        root.add_child(&label);
        label.set_position_y(LABEL_OFFSET_Y);
        label.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label.add_to_scene_layer_DEPRECATED(&scene.layers.label);

        let styles       = StyleWatchFrp::new(&scene.style_sheet);
        let lightness    = styles.get_number_or(theme_path::lightness,0.5);
        let chroma       = styles.get_number_or(theme_path::chroma,1.0);
        let min_time_hue = styles.get_number_or(theme_path::min_time_hue,0.4);
        let max_time_hue = styles.get_number_or(theme_path::max_time_hue,0.1);

        let frp     = Frp::new();
        let network = &frp.network;
        let color   = color::Animation::new(network);

        frp::extend! { network

            // === Visibility ===

            visibility <- all_with(&frp.set_editor_mode,&frp.set_status,|mode,status| {
                matches!((mode,status),(EditorMode::Profiling,Status::Finished {..}))
            });

            color.target_alpha <+ visibility.map(|&is_visible| {
                if is_visible { 1.0 } else { 0.0 }
            });


            // === Color ===

            theme <- all_with4(&lightness,&chroma,&min_time_hue,&max_time_hue,
                |&lightness,&chroma,&min_time_hue,&max_time_hue|
                    Theme {lightness,chroma,max_time_hue,min_time_hue});
            color.target_color <+ all_with4
                (&frp.set_status,&frp.set_min_global_duration,&frp.set_max_global_duration,&theme,
                    |&status,&min,&max,&theme| status.display_color(min,max,theme)
                );
            label.set_default_color <+ color.value.map(|c| c.into());
            label.set_color_all     <+ color.value.map(|c| c.into());


            // === Position ===

            eval label.width((&width) label.set_position_x(-width/2.0));


            // === Content ===

            label.set_content <+ frp.set_status.map(|&status|
                match status {
                    Status::Running             => "".to_string(),
                    Status::Finished {duration} => format!("{} ms", duration)
                });
        }

        RunningTimeLabel {root,label,frp,styles}
    }
}

impl display::Object for RunningTimeLabel {
    fn display_object(&self) -> &display::object::Instance {
        &self.root
    }
}
