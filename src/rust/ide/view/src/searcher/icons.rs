//! All icons that are used in the searcher.

use ensogl::prelude::*;

use ensogl::application::Application;
use ensogl::data::color;
use ensogl::display::DomSymbol;
use ensogl::display::navigation::navigator::Navigator;
use ensogl::display::object::ObjectOps;
use ensogl::display::shape::*;
use ensogl::system::web;
use ensogl_theme::application::searcher::icons as theme;
use ensogl::system::web::StyleSetter;
use std::f32::consts::PI;
use wasm_bindgen::prelude::*;

/// The width and height of all icons.
const ICON_SIZE : f32 = 16.0;

// Due to a rendering error, shapes appear too big when the camera is zoomed in very closely.
// (Documented here: https://github.com/enso-org/ide/issues/1698)
// To compensate for this, we apply `.shrink(SHRINK_AMOUNT)` to all icons in this file. By default,
// `SHRINK_AMOUNT` should be set to 0.0 to make the icons appear right on the default zoom level.
// When examining the icons closely, with a strong zoom, `SHRINK_AMOUNT` should be set to 0.4 to
// make the icons apear right on that zoom level.
const SHRINK_AMOUNT : f32 = 0.0;
// const SHRINK_AMOUNT : f32 = 0.38;



// =================
// === Favorites ===
// =================

/// A five-pointed star.
mod star {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {
            // The distance from the center to the outer points.
            let radius = 7.0;
            // The ration between the radius and the distance from center to the points where spikes
            // touch.
            let ratio = 0.447;
            // The angle between neighboring spikes.
            let rotation_step = 2.0 * PI / 5.0;
            // By "pentagon", we refer to the pentagon that forms the body of the star, filling the
            // whole space in the middle, connecting the spikes.
            let pentagon_radius       = ratio * radius;
            let pentagon_inner_radius = pentagon_radius * (rotation_step / 2.0).cos();
            let pentagon_side_length  = pentagon_radius * (rotation_step / 2.0).sin() * 2.0;
            let spike_length          = radius - pentagon_inner_radius;

            // The spike pointing upwards.
            let spike = Triangle(pentagon_side_length.px(),spike_length.px());
            let spike = spike.translate_y((radius - spike_length / 2.0).px());
            // We compose the pentagon of five triangles fanning out from the center. This is one of
            // them.
            let pentagon_fragment = Triangle(pentagon_side_length.px(),pentagon_inner_radius.px());
            let pentagon_fragment = pentagon_fragment.rotate(PI.radians());
            let pentagon_fragment = pentagon_fragment.translate_y((pentagon_inner_radius/2.0).px());

            // The spike and pentagon fragment form a star fragment. We copy it five times with
            // rotations to form the whole star.
            let fragment1 = spike + pentagon_fragment;
            let fragment2 = fragment1.rotate((rotation_step * 1.0).radians());
            let fragment3 = fragment1.rotate((rotation_step * 2.0).radians());
            let fragment4 = fragment1.rotate((rotation_step * 3.0).radians());
            let fragment5 = fragment1.rotate((rotation_step * 4.0).radians());

            let shape = fragment1 + fragment2 + fragment3 + fragment4 + fragment5;
            let shape = shape.fill(style.get_color(theme::favorites));
            let shape = shape.translate_y((-0.5).px());
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}



// ==========
// === IO ===
// ==========

// A rounded rectangle with an arrow pointing in from the left.
mod data_input {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {

            // === Rectangle ===

            let rect = Rect((10.0.px(),13.0.px())).corners_radius(1.5.px()).translate_x(1.5.px());
            // Taking just an outline.
            let rect = &rect - rect.shrink(1.0.px());
            // Creating a gap for the arrow to pass through.
            let gap  = Rect((2.0.px(),4.0.px())).translate_x((-3.0).px());
            let rect = rect - gap;


            // === Arrow ===

            let arrow_line = Rect((8.0.px(),1.0.px())).translate_x((-3.0).px());
            let arrow_tip  = Triangle(5.0.px(),4.0.px()).rotate((PI / 2.0).radians());
            let arrow_tip  = arrow_tip.translate_x(2.0.px());
            let arrow      = arrow_line + arrow_tip;


            // === Shape ===

            let shape = rect + arrow;
            let shape = shape.fill(style.get_color(theme::io));
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}

// A rounded rectangle with an arrow pointing out to the right.
mod data_output {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {

            // === Rectangle ===
            let rect = Rect((9.0.px(),13.0.px())).corners_radius(1.5.px()).translate_x((-2.5).px());
            // Taking just an outline.
            let rect = &rect - rect.shrink(1.0.px());
            // Creating a gap for the arrow to pass through.
            let gap  = Rect((2.0.px(),4.0.px())).translate_x((1.5).px());
            let rect = rect - gap;


            // === Arrow ===

            let arrow_line = Rect((8.0.px(),1.0.px())).translate_x(1.0.px());
            let arrow_tip  = Triangle(5.0.px(),4.0.px()).rotate((PI / 2.0).radians());
            let arrow_tip  = arrow_tip.translate_x(6.0.px());
            let arrow      = arrow_line + arrow_tip;


            // === Shape ===

            let shape = rect + arrow;
            let shape = shape.fill(style.get_color(theme::io));
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}

// A rounded rectangle with the letter "A" and a text cursor.
mod text_input {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {

            // === Rectangle ===

            let rect = Rect((16.0.px(),11.0.px())).corners_radius(1.5.px());
            // Using just the outline.
            let rect = &rect - rect.shrink(1.0.px());
            // Creating a gap for the cursor.
            let gap = Rect((3.0.px(),13.0.px())).translate_x((3.5).px());
            let rect = rect - gap;


            // === Cursor ===

            let cursor_middle = Rect((1.0.px(),15.0.px()));
            let cursor_top    = Rect((5.0.px(),1.0.px())).translate_y(7.5.px());
            let cursor_bottom = Rect((5.0.px(),1.0.px())).translate_y((-7.5).px());
            let cursor        = (cursor_middle + cursor_top + cursor_bottom).translate_x(3.5.px());


            // === Letter ===

            // We construct the letter "A", consisting of a diagonal stroke on the left, a diagonal
            // stroke on the right and a horizontal bar in the middle.
            let left_stroke   = RoundedLineSegment(((-2.5).px(),(-5.0).px()),1.0.px());
            let right_stroke  = RoundedLineSegment((2.5.px() ,(-5.0).px()),1.0.px());
            let bar           = Rect((4.0.px(),1.0.px())).translate_y((-3.5).px());
            let letter        = left_stroke + right_stroke + bar;
            let letter        = letter.translate_x((-2.5).px()).translate_y(2.5.px());


            // === Shape ===

            let shape = rect + cursor + letter;
            let shape = shape.fill(style.get_color(theme::io));
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}

// A rounded rectangle with the number "5" and a text cursor.
mod number_input {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {

            // === Rectangle ===

            let rect = Rect((16.0.px(),11.0.px())).corners_radius(1.5.px());
            // Using just the outline.
            let rect = &rect - rect.shrink(1.0.px());
            // Creating a gap for the cursor.
            let gap  = Rect((3.0.px(),13.0.px())).translate_x((3.5).px());
            let rect = rect - gap;


            // === Cursor ===

            let cursor_middle = Rect((1.0.px(),15.0.px()));
            let cursor_top    = Rect((5.0.px(),1.0.px())).translate_y(7.5.px());
            let cursor_bottom = Rect((5.0.px(),1.0.px())).translate_y((-7.5).px());
            let cursor        = (cursor_middle + cursor_top + cursor_bottom).translate_x(3.5.px());


            // === Number ===

            // The number "5" consists of a short horizontal bar at the top, a vertical bar
            // connected to it on the left and a big arc below, connected to the vertical bar.
            let top  = Rect((3.0.px(),1.0.px()));
            let left = Rect((1.0.px(),3.0.px())).translate_x((-1.0).px()).translate_y((-1.0).px());


            // == Arc ==

            let arc_center = Vector2(-0.25_f32,-3.5_f32);
            // The point where the inner side of the arc connects with the vertical bar.
            let arc_connection = Vector2(-0.5_f32,-2.5_f32);
            // Offset from the arc center to the connection.
            let connection_offset = arc_connection - arc_center;
            // The inner radius of the arc.
            let radius: f32 = connection_offset.norm();
            let arc         = Circle(radius.px());
            // Take the outline.
            let stroke_width = 1.0;
            let arc          = arc.grow(stroke_width.px()) - arc;
            // The angle that we need to cut from the circle.
            let aperture_angle = 110.0_f32.to_radians();
            let outer_radius   = radius + stroke_width;
            // To create the aperture, we will subtract a triangle with height `aperture_height` and
            // width `aperture_width`.
            let aperture_height = outer_radius + 1.0;  // We add 1.0 just to be sure to cut enough.
            // The width is computed, such that we get the right angle.
            let aperture_width = (aperture_angle / 2.0).tan() * aperture_height * 2.0;
            let aperture       = Triangle(aperture_width,aperture_height);
            // We temporarily position the tip of the gap at the origin to rotate it around that
            // point.
            let aperture = aperture.translate_y((-aperture_height / 2.0).px());
            // Make the triangle's left side point downward.
            let aperture = aperture.rotate((-aperture_angle / 2.0).radians());
            // Make the triangle's left side point right.
            let aperture = aperture.rotate(-PI.radians() / 2.0);
            // Make the triangle's left side point to the connection between vertical bar and arc.
            // We have to negate the angle because positive results of `atan2` stand for counter-
            // clockwise rotations.
            let connection_direction = -connection_offset.y.atan2(connection_offset.x);
            let aperture             = aperture.rotate(connection_direction.radians());
            let arc                  = arc - aperture;
            let arc                  = arc.translate((arc_center.x.px(),arc_center.y.px()));

            let number = (top + left + arc).translate_x((-2.0).px()).translate_y(2.5.px());


            // === Shape ===

            let shape = rect + cursor + number;
            let shape = shape.fill(style.get_color(theme::io));
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}



// ===================
// === Debug Scene ===
// ===================


/// A rectangular frame to mark the edges of icons.
mod frame {
    use super::*;

    ensogl::define_shape_system! {
        (style:Style) {
            let inner = Rect((ICON_SIZE.px(),ICON_SIZE.px()));
            let outer = inner.grow(0.2.px());
            let shape = (outer - inner).fill(color::Rgba::black());
            shape.shrink(SHRINK_AMOUNT.px()).into()
        }
    }
}

/// An entry point that displays all icons on a grid.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_searcher_icons() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();

    let logger = Logger::new("Icons example");
    let app    = Application::new(&web::get_html_element_by_id("root").unwrap());
    ensogl_theme::builtin::dark::register(&app);
    ensogl_theme::builtin::light::register(&app);
    ensogl_theme::builtin::light::enable(&app);
    let world = app.display.clone();
    mem::forget(app);
    let scene = world.scene();
    mem::forget(Navigator::new(&scene,&scene.camera()));


    // === Grid ===

    let grid_div = web::create_div();
    grid_div.set_style_or_panic("width",  "1000px");
    grid_div.set_style_or_panic("height", "16px");
    grid_div.set_style_or_panic("background-size", "1.0px 1.0px");
    grid_div.set_style_or_panic("background-image",
                                "linear-gradient(to right,  grey 0.05px, transparent 0.05px),
                                 linear-gradient(to bottom, grey 0.05px, transparent 0.05px)");

    let grid = DomSymbol::new(&grid_div);
    scene.dom.layers.back.manage(&grid);
    world.add_child(&grid);
    grid.set_size(Vector2(1000.0,ICON_SIZE));
    mem::forget(grid);


    // === Frame ===

    let frame = frame::View::new(&logger);
    world.add_child(&frame);
    frame.size.set(Vector2(ICON_SIZE+20.0, ICON_SIZE+20.0));
    mem::forget(frame);


    // === Star ===

    let star = star::View::new(&logger);
    world.add_child(&star);
    star.size.set(Vector2(ICON_SIZE, ICON_SIZE));
    star.set_position_x(20.0);
    mem::forget(star);


    // === Data Input ===

    let data_input = data_input::View::new(&logger);
    world.add_child(&data_input);
    data_input.size.set(Vector2(ICON_SIZE, ICON_SIZE));
    data_input.set_position_x(40.0);
    mem::forget(data_input);


    // === Data Output ===

    let data_output = data_output::View::new(&logger);
    world.add_child(&data_output);
    data_output.size.set(Vector2(ICON_SIZE, ICON_SIZE));
    data_output.set_position_x(60.0);
    mem::forget(data_output);


    // === Text Input ===

    let text_input = text_input::View::new(&logger);
    world.add_child(&text_input);
    text_input.size.set(Vector2(ICON_SIZE, ICON_SIZE));
    text_input.set_position_x(80.0);
    mem::forget(text_input);


    // === Number Input ===

    let number_input = number_input::View::new(&logger);
    world.add_child(&number_input);
    number_input.size.set(Vector2(ICON_SIZE, ICON_SIZE));
    number_input.set_position_x(100.0);
    mem::forget(number_input);
}
