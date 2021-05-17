use crate::prelude::*;

use ensogl::data::color;
use ensogl::display::shape::*;
use ensogl_gui_components::toggle_button::ColorableShape;

ensogl::define_shape_system! {
    (color_rgba:Vector4<f32>) {
        let fill_color  = Var::<color::Rgba>::from(color_rgba);
        let width        = Var::<Pixels>::from("input_size.x");
        let height       = Var::<Pixels>::from("input_size.y");

        let unit = &width * 0.3;
        let outer_circle_radius = &unit * 1.0;
        let outer_circle_thickness = &unit * 0.33;
        let inner_circle_radius = &unit * 0.2;
        let needle_radius_inner = &unit * 0.14;
        let needle_radius_outer = &unit * 0.09;
        let needle_angle = (135.0_f32).to_radians().radians();

        let base = Circle(&outer_circle_radius);
        let circle_gap = Circle(&outer_circle_radius-&outer_circle_thickness);

        // We make the gap a little bit larger than the circle to be sure that we really cover
        // everything that we want to cut, even if there are rounding errors or similar
        // imprecisions.
        let aperture_gap_size = &outer_circle_radius * 1.1;
        let aperture_gap      = Triangle(&aperture_gap_size*2.0,aperture_gap_size.clone());
        let aperture_gap      = aperture_gap.rotate(needle_angle+180.0_f32.to_radians().radians());
        let aperture_gap      = aperture_gap.translate_x(&aperture_gap_size*2.0.sqrt()*0.25);
        let aperture_gap      = aperture_gap.translate_y(-(&aperture_gap_size*2.0.sqrt()*0.25));

        let aperture_cap_1 = Circle(&outer_circle_thickness*0.5);
        let aperture_cap_1 = aperture_cap_1.translate_x(&outer_circle_radius-&outer_circle_thickness*0.5);
        let aperture_cap_2 = Circle(&outer_circle_thickness*0.5);
        let aperture_cap_2 = aperture_cap_2.translate_y(-(&outer_circle_radius-&outer_circle_thickness*0.5));

        let outer_circle = base - circle_gap - aperture_gap + aperture_cap_1 + aperture_cap_2;

        let needle_length = &outer_circle_radius-&needle_radius_outer;
        let needle = UnevenCapsule(needle_radius_outer,needle_radius_inner,needle_length);
        let needle = needle.rotate(&needle_angle);

        let inner_circle = Circle(&inner_circle_radius);

        let shape = (outer_circle + needle + inner_circle).fill(fill_color);
        let hover_area = Rect((&width,&height)).fill(HOVER_COLOR);
        (shape + hover_area).into()
    }
}

impl ColorableShape for DynamicShape {
    fn set_color(&self, color:color::Rgba) {
        self.color_rgba.set(Vector4::new(color.red,color.green,color.blue,color.alpha));
    }
}