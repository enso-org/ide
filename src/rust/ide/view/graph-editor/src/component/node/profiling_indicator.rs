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



// =============
// === Shape ===
// =============

mod shape {
    use super::*;

    pub const WIDTH_OUTER       : f32 = 15.0;
    pub const WIDTH_INNER       : f32 = 10.0;
    pub const THICKNESS         : f32 = WIDTH_OUTER - WIDTH_INNER;
    pub const LABEL_GAP_PADDING : f32 = 5.0;
    pub const LABEL_GAP_HEIGHT  : f32 = 16.0;

    ensogl::define_shape_system! {
        (style:Style,color_rgba:Vector4<f32>,label_width:f32) {
            let width  = Var::<Pixels>::from("input_size.x");
            let height = Var::<Pixels>::from("input_size.y");
            let width  = width  - node::PADDING.px() * 2.0;
            let height = height - node::PADDING.px() * 2.0;
            let radius = node::RADIUS.px();

            let base    = Rect((&width,&height)).corners_radius(&radius);
            let outer   = base.grow(WIDTH_OUTER.px());
            let inner   = base.grow(WIDTH_INNER.px());
            let outline = outer - inner;

            let upper_center_y    = height / 2.0 + WIDTH_INNER.px() + THICKNESS.px() / 2.0;
            let label_gap_width   = label_width * 1.px() + LABEL_GAP_PADDING.px() * 2.0;
            let label_gap         = Rect((&label_gap_width,LABEL_GAP_HEIGHT.px()));
            let label_gap         = label_gap.corners_radius(LABEL_GAP_PADDING.px());
            let label_gap         = label_gap.translate_y(upper_center_y);

            (outline-label_gap).fill(color_rgba).into()
        }
    }
}



// ============
// === Frp  ===
// ============

ensogl::define_endpoints! {
    Input {
        set_size                (Vector2),
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
    display_object : display::object::Instance,
    shape          : shape::View,
    label          : text::Area,
    frp            : Frp
}

impl Deref for ProfilingIndicator {
    type Target = Frp;

    fn deref(&self) -> &Self::Target {
        &self.frp
    }
}


impl ProfilingIndicator {
    pub fn new(app: &Application) -> Self {
        let scene          = app.display.scene();
        let styles         = StyleWatch::new(&scene.style_sheet);
        let logger         = Logger::new("ProfilingIndicator");
        let display_object = display::object::Instance::new(&logger);

        let shape = shape::View::new(&logger);
        display_object.add_child(&shape);
        ensogl::shapes_order_dependencies! {
            app.display.scene() => {
                crate::component::edge::front::corner -> shape;
                crate::component::edge::front::line   -> shape;
            }
        }

        let label = text::Area::new(app);
        display_object.add_child(&label);
        label.remove_from_scene_layer_DEPRECATED(&scene.layers.main);
        label.add_to_scene_layer_DEPRECATED(&scene.layers.label);

        let frp     = Frp::new();
        let network = &frp.network;
        let color   = color::Animation::new(network);

        frp::extend! { network

            // === Visibility ===

            visibility <- all_with(&frp.set_editor_mode,&frp.set_execution_status,|mode,status| {
                match (mode,status) {
                    (EditorMode::Profiling,ExecutionStatus::Finished {..}) => true,
                    _                                                      => false,
                }
            });

            color.target_alpha <+ visibility.map(|&is_visible| {
                if is_visible {
                    1.0
                } else {
                    0.0
                }
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


            // === Shape ===

            eval  frp.set_size((size) shape.size.set(*size));
            eval label.width((&width) shape.label_width.set(width));
            eval color.value((&color) shape.color_rgba.set(color::Rgba::from(color).into()));


            // === Label ===

            eval frp.set_size([label](size) {
                let height         = size.y - node::PADDING * 2.0;
                let upper_center_y = height / 2.0 + shape::WIDTH_INNER + shape::THICKNESS / 2.0;
                label.set_position_y(upper_center_y + text::area::LINE_HEIGHT / 2.0)
            });
            eval label.width((&width) label.set_position_x(-width/2.0));
            label.set_content <+ frp.set_execution_status.map(|&status|
                match status {
                    ExecutionStatus::Running             => "Running".to_string(),
                    ExecutionStatus::Finished {duration} => format!("{} ms", duration)
                });
            label.set_default_color <+ color.value.map(|c| c.into());
            label.set_color_all     <+ color.value.map(|c| c.into());
        }

        ProfilingIndicator {display_object,shape,label,frp}
    }
}


impl display::Object for ProfilingIndicator {
    fn display_object(&self) -> &display::object::Instance {
        &self.display_object
    }
}
