use crate::top_buttons::common::prelude::*;

pub use ensogl_theme::application::top_buttons::fullscreen as theme;
pub type View = crate::top_buttons::common::View<shape::DynamicShape>;

/// The shape for "fullscreen" button. The icon consists if two triangles ◤◢ centered around single
/// point.
pub mod shape {
    use super::*;
    ensogl::define_shape_system! {
        (background_color:Vector4<f32>, icon_color:Vector4<f32>) {
            let radius = Var::min(Var::input_width(),Var::input_height()) / 2.0;
            let rect = Rect((&radius,&radius));
            //let middle_strip = Rect(())
            let strip = Rect((&radius * 0.2,2000.0.px())).rotate(Radians::from(45.0.degrees()));
            let icon = rect - strip;

            shape(background_color, icon_color, icon.into(), radius)
        }
    }
}

impl ButtonShape for shape::DynamicShape {
    fn debug_name() -> &'static str {
        "FullscreenButton"
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
        }
    }

    fn background_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.background_color
    }

    fn icon_color(&self) -> &DynamicParam<Attribute<Vector4<f32>>> {
        &self.icon_color
    }
}
