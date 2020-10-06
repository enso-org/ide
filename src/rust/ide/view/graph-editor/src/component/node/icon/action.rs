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
            let unit        = &width/16.0;


            let outer_radius = &unit*5.0;

            let pupil        = Circle(&unit * 1.0);
            let inner_circle = Circle(&unit * 3.0);

            let outer_circle = Circle(&outer_radius);

            let right_edge   = Triangle(&unit * 7.9, &unit * 4.6)
                .rotate(right_angle)
                .translate_x(&unit * 5.3);
            let left_edge    = right_edge.rotate(2.0 * right_angle);

            let eye_outer    = outer_circle + right_edge + left_edge;

            let eye          = (eye_outer - inner_circle) + pupil;
            let eye_colored  = eye.fill(fill_color);

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));

            (eye_colored+hover_area).into()
        }
    }
}


pub mod visibility2 {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color       = Var::<color::Rgba>::from(color_rgba);

            let width            = Var::<Pixels>::from("input_size.x");
            let height           = Var::<Pixels>::from("input_size.y");

            let right_angle  = 90.0_f32.to_radians().radians();
            let unit         = &width/16.0;


            let outer_circle = Circle(&unit * 7.0);
            let inner_circle = Circle(&unit * 4.0);
            let ring         = outer_circle - inner_circle;

            let vertival_bar = Rect((&unit * 3.0, &unit * 16.0))
                                    .translate_y(-&width/2.0);

            let icon = ring - &vertival_bar - &vertival_bar.rotate(right_angle) - &vertival_bar.rotate(right_angle * 2.5);

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));
            let icon         = icon.fill(fill_color);

            (icon+hover_area).into()
        }
    }
}

fn make_ring<T:Into<Var<Pixels>>,U:Into<Var<Pixels>>>(outer_radius:T, inner_radius:U) -> AnyShape {
    let outer_circle = Circle(&outer_radius.into());
    let inner_circle = Circle(&inner_radius.into());
    let ring         = outer_circle - inner_circle;
    ring.into()
}

pub mod freeze {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color       = Var::<color::Rgba>::from(color_rgba);

            let width       = Var::<Pixels>::from("input_size.x");
            let height      = Var::<Pixels>::from("input_size.y");

            let unit         = &width/16.0;
            let right_angle  = 90.0_f32.to_radians().radians();

            let lock_body_radius = &unit * 5.0;
            let lock_body        = make_ring(&lock_body_radius,&unit*2.0);
            let lock_body        = lock_body.translate_y(-&unit * 3.0);

            let lock_top_radius = &unit * 4.0;
            let lock_top_width  = &unit * 2.0;
            let lock_top  = make_ring(&lock_top_radius,&lock_top_radius - &lock_top_width);
            let lock_top  = lock_top.intersection(HalfPlane().rotate(right_angle*2.0));
            let lock_top  = lock_top.translate_y(lock_body_radius - &unit * 3.0);

            let vertical_bar = Rect((&lock_top_width,&unit * 4.0));
            let left_bar     = vertical_bar.translate_x(&lock_top_radius-&lock_top_width/2.0);
            let right_bar    = vertical_bar.translate_x(-&lock_top_radius+&lock_top_width/2.0);

            let icon = lock_body + lock_top + left_bar + right_bar;
            let icon = icon.translate_y(unit);

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));
            let icon         = icon.fill(fill_color);

            (icon+hover_area).pixel_snap().into()
        }
    }
}


pub mod skip {
    use super::*;

    ensogl::define_shape_system! {
        (color_rgba:Vector4<f32>) {
            let fill_color   = Var::<color::Rgba>::from(color_rgba);

            let width        = Var::<Pixels>::from("input_size.x");
            let height       = Var::<Pixels>::from("input_size.y");

            let right_angle  = 90.0_f32.to_radians().radians();
            let unit         = &width/16.0;

            let circle       = Circle(&unit*7.0);

            let line_width   = &unit * 3.0;
            let line_height  = &unit * 8.0;
            let offset       = &unit * -0.22;
            let line_rounded = Rect((&line_width,&line_height)).corners_radius(&line_width);
            let line_top     = &line_rounded
                                .rotate(right_angle / 2.0)
                                .translate_y(-&line_height / 4.0 - &offset);
            let line_bottom  = &line_rounded
                                .rotate(-right_angle / 2.0)
                                .translate_y(&line_height / 4.0 + &offset);
            let skip         = line_top + line_bottom;
            let skip         = skip.translate_x(&unit * 0.5);

            let icon         = circle - skip;

            let hover_area   = Rect((width,height)).fill(color::Rgba::new(1.0,0.0,0.0,0.000_001));
            let icon         = icon.fill(fill_color);

            (icon+hover_area).into()
        }
    }
}
