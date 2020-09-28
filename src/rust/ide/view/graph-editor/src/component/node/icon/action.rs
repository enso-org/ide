use crate::prelude::*;

use ensogl::data::color;
use ensogl::display::shape::*;


pub mod visibility {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color       = Var::<color::Rgba>::from(color_rgba);

            let width       = Var::<Pixels>::from("input_size.x");
            let height      = Var::<Pixels>::from("input_size.y");
            let right_angle = 90.0_f32.to_radians().radians();

            let outer_radius = &width/4.0;

            let pupil        = Circle(&width * 0.05);
            let inner_circle = Circle(&width * 0.125);


            let outer_circle = Circle(&outer_radius);

            let right_edge   = Triangle(&width/2.15,&height/3.5).rotate(right_angle).translate_x(&width/4.0);
            let left_edge    = right_edge.rotate(2.0 * right_angle);

            let eye_outer    = outer_circle + right_edge + left_edge;

            let eye          = (eye_outer - inner_circle) + pupil;
            let eye_colored  = eye.fill(fill_color);

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));

            (eye_colored+hover_area).into()
        }
    }
}


pub mod freeze {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color       = Var::<color::Rgba>::from(color_rgba);

            let width            = Var::<Pixels>::from("input_size.x");
            let height           = Var::<Pixels>::from("input_size.y");

            let right_angle  = 90.0_f32.to_radians().radians();

            let outer_circle = Circle(&width/2.0);
            let inner_circle = Circle(&width/3.0);
            let ring         = outer_circle - inner_circle;

            let vertival_bar = Rect((&width/5.0, &width/2.0)).translate_y(-&width/2.0);

            let icon = ring - &vertival_bar - &vertival_bar.rotate(right_angle) - &vertival_bar.rotate(right_angle * 2.5);

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));
            let icon         = icon.fill(fill_color);

            (icon+hover_area).into()
        }
    }
}


pub mod skip {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color       = Var::<color::Rgba>::from(color_rgba);

            let width            = Var::<Pixels>::from("input_size.x");
            let height           = Var::<Pixels>::from("input_size.y");

            let right_angle  = 90.0_f32.to_radians().radians();

            let circle = Circle(&width/2.0);

            let line_width   = &width/5.0;
            let line_height  = &width/2.0;
            let line_rounded = Rect((&line_width,&line_height)).corners_radius(&line_width);
            let line_top     = &line_rounded.rotate(right_angle/2.0).translate_y(-&line_height/4.0);
            let line_bottom  = &line_rounded.rotate(-right_angle/2.0).translate_y(&line_height/4.0);

            let icon = circle - line_top - line_bottom;

            let hover_area = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));
            let icon       = icon.fill(fill_color);

            (icon+hover_area).into()
        }
    }
}