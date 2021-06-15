//! Module that contains a scrollbar component that can be used to implement scrollable components.

use crate::prelude::*;

use enso_frp as frp;
use ensogl_core::Animation;
use ensogl_core::application::Application;
use ensogl_core::application;
use ensogl_core::data::color;
use ensogl_core::display::shape::*;
use ensogl_core::display::shape::StyleWatchFrp;
use ensogl_theme as theme;

use crate::component;
use crate::selector::Bounds;
use crate::selector::bounds::absolute_value;
use crate::selector::bounds::normalise_value;
use crate::selector::bounds::should_clamp_with_overflow;
use crate::selector::model::Model;
use crate::selector;
use ensogl_core::display::object::ObjectOps;


// =================
// === Constants ===
// =================

/// Amount the scrollbar moves ona single click, relative to the viewport width.
const CLICK_JUMP_PERCENTAGE: f32 = 0.80;



// ===========
// === Frp ===
// ===========

ensogl_core::define_endpoints! {
    Input {
        resize(Vector2),

        set_track(Bounds),
        set_overall_bounds(Bounds),

        set_track_color(color::Rgba),
    }
    Output {
        track(Bounds)
    }
}

/// Returns the result of the projection of the given `vector` to the x-axis of the coordinate
/// system of the given `shape`. For example, if the vector is parallel to the x-axis of the shape
/// coordinate system, it is returned unchanged, if it is perpendicular the zero vector is returned.
fn vector_aligned_with_object(vector:&Vector2,object:impl ObjectOps) -> Vector2 {
    let object_rotation =  Rotation2::new(-object.rotation().z);
    object_rotation * vector
}

impl component::Frp<Model> for Frp {
    fn init(&self, app:&Application, model:&Model, style:&StyleWatchFrp){
        let frp                  = &self;
        let network              = &frp.network;
        let scene                = app.display.scene();
        let mouse                = &scene.mouse.frp;
        let track_position_lower = Animation::new(&network);
        let track_position_upper = Animation::new(&network);

        let base_frp = selector::Frp::new(model, style, network, frp.resize.clone().into(), mouse);

        model.use_track_handles(false);
        model.set_track_corner_round(true);
        model.show_background(false);
        model.show_left_overflow(false);
        model.show_right_overflow(false);

        let style_track_color = style.get_color(theme::component::slider::track::color);

        frp::extend! { network
            // Simple Inputs
            eval frp.set_track_color((value) model.set_track_color(*value));

            // API  - `set_track`
            track_position_lower.target <+ frp.set_track.map(|b| b.start);
            track_position_upper.target <+ frp.set_track.map(|b| b.end);
            track_position_lower.skip   <+ frp.set_track.constant(());
            track_position_upper.skip   <+ frp.set_track.constant(());

            // Normalise values for internal use.
            normalised_track_bounds <- all2(&frp.track,&frp.set_overall_bounds).map(|(track,overall)|{
                 Bounds::new(normalise_value(&(track.start,*overall)),normalise_value(&(track.end,*overall)))
            });
            normalised_track_center <- normalised_track_bounds.map(|bounds| bounds.center());
            normalised_track_width  <- normalised_track_bounds.map(|bounds| bounds.width());

            // Slider Updates
            update_slider <- all(&normalised_track_bounds,&frp.resize);
            eval update_slider(((value,size)) model.set_background_range(*value,*size));

            // Mouse IO - Clicking
            click_delta <- base_frp.background_click.map3(&normalised_track_center,&normalised_track_width,
                f!([model](click_value,current_value,track_width) {
                    let shape_aligned = vector_aligned_with_object(click_value,&model);
                    let direction = if shape_aligned.x > *current_value { 1.0 } else { -1.0 };
                    direction * track_width * CLICK_JUMP_PERCENTAGE
            }));
            click_animation <- click_delta.constant(true);

            // Mouse IO - Dragging
            drag_movement <- mouse.translation.gate(&base_frp.is_dragging_any);
            drag_delta    <- drag_movement.map2(&base_frp.track_max_width, f!([model](delta,width) {
                let shape_aligned = vector_aligned_with_object(delta,&model);
                (shape_aligned.x) / width
            }));
            drag_delta     <- drag_delta.gate(&base_frp.is_dragging_track);
            drag_animation <- drag_delta.constant(false);

            // Mouse IO - Event Evaluation
            should_animate <- any(&click_animation,&drag_animation);

            drag_center_delta <- any(&drag_delta,&click_delta);
            drag_update       <- drag_center_delta.map2(&normalised_track_bounds,|delta,Bounds{start,end}|
                Bounds::new(start+delta,end+delta)
            );

            is_in_bounds <- drag_update.map(|value| should_clamp_with_overflow(value,&None));

            new_value_absolute <- all(&frp.set_overall_bounds,&drag_update).map(|(bounds,Bounds{start,end})|
                Bounds::new(
                    absolute_value(&(*bounds,*start)),absolute_value(&(*bounds,*end))).sorted()
            ).gate(&is_in_bounds);

            new_value_lower <- new_value_absolute.map(|b| b.start);
            new_value_upper <- new_value_absolute.map(|b| b.end);

            track_position_lower.target <+ new_value_lower.gate(&is_in_bounds);
            track_position_upper.target <+ new_value_upper.gate(&is_in_bounds);

            track_position_lower.skip <+ should_animate.on_false();
            track_position_upper.skip <+ should_animate.on_false();

            track_position <- all(&track_position_lower.value,&track_position_upper.value).map(
                |(lower,upper)| Bounds::new(*lower,*upper));

            frp.source.track <+ track_position;
        }

        // Init defaults
        frp.set_overall_bounds(Bounds::new(0.0,1.0));
        frp.set_track(Bounds::new(0.25,0.55));
        frp.set_track_color(style_track_color.value());
    }
}



// =================
// === Scrollbar ===
// =================

/// Scrollbar component that can be used to implement scrollable components.
pub type Scrollbar = crate::component::Component<Model,Frp>;

impl application::View for Scrollbar {
    fn label() -> &'static str { "Scrollbar" }
    fn new(app:&Application) -> Self { Scrollbar::new(app) }
    fn app(&self) -> &Application { &self.app }
}
