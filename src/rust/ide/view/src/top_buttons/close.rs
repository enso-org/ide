use crate::top_buttons::common::prelude::*;

pub use ensogl_theme::application::top_buttons::close as theme;
pub type View = crate::top_buttons::common::View<shape::DynamicShape>;

pub mod shape {
    use super::*;

    ensogl::define_shape_system! {
        (background_color:Vector4<f32>, icon_color:Vector4<f32>) {
            let radius = Var::min(Var::input_width(),Var::input_height()) / 2.0;
            let angle = Radians::from(45.0.degrees());
            let bar_length = &radius * 4.0 / 3.0;
            let bar_width = &bar_length / 6.5;
            let bar = Rect((bar_length, &bar_width)).corners_radius(bar_width);
            let cross = (bar.rotate(angle) + bar.rotate(-angle)).into();
            shape(background_color, icon_color, cross, radius)
        }
    }
}

impl ButtonShape for shape::DynamicShape {
    fn debug_name() -> &'static str {
        "CloseButton"
    }

    fn background_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::normal::background_color,
            State::Hovered => theme::hovered::background_color,
            State::Pressed => theme::pressed::background_color,
        }
    }

    fn icon_color_path(state:State) -> StaticPath {
        match state {
            State::Unconcerned => theme::normal::icon_color,
            State::Hovered => theme::hovered::icon_color,
            State::Pressed => theme::pressed::icon_color,
            _ => theme::normal::icon_color,
        }
    }

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.background_color
    }

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.icon_color
    }
}
