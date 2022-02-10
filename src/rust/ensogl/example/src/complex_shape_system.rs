//! Example scene showing simple usage of a shape system.

use ensogl_core::prelude::*;

use ensogl_core::data::color;
use ensogl_core::display::navigation::navigator::Navigator;
use ensogl_core::display::object::ObjectOps;
use ensogl_core::display::scene;
use ensogl_core::display::shape::*;
use ensogl_core::display::style::theme;
use ensogl_core::display::world::*;
use ensogl_core::system::web;
use wasm_bindgen::prelude::*;



// ==============
// === Shapes ===
// ==============

mod shape {
    use super::*;
    ensogl_core::define_shape_system! {
        (style:Style) {
            let base_color = style.get_color("base_color");
            let circle1    = Circle(50.px());
            let circle_bg  = circle1.translate_x(-(50.0.px()));
            let circle_sub = circle1.translate_y(-(50.0.px()));
            let rect       = Rect((100.0.px(),100.0.px()));
            let shape      = circle_bg + rect - circle_sub;
            let shape      = shape.fill(base_color);
            shape.into()
        }
    }
}

mod mask {
    use super::*;
    ensogl_core::define_shape_system! {
        (style:Style) {
            let shape = Circle(60.px());
            let shape = shape.fill(color::Rgb::new(1.0,0.0,0.0));
            shape.into()
        }
    }
}



// ===================
// === Entry Point ===
// ===================

/// The example entry point.
#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_complex_shape_system() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();

    let world = World::new(&web::get_html_element_by_id("root").unwrap());
    let scene = world.scene();
    let camera = scene.camera().clone_ref();
    let navigator = Navigator::new(scene, &camera);
    let logger = Logger::new("ShapeView");

    let theme_manager = theme::Manager::from(&scene.style_sheet);

    let theme1 = theme::Theme::new();
    theme1.set("base_color", color::Rgba::new(0.0, 0.0, 1.0, 1.0));
    theme1.set("animation.duration", 0.5);
    theme1.set("graph.node.shadow.color", 5.0);
    theme1.set("graph.node.shadow.size", 5.0);
    theme1.set("mouse.pointer.color", color::Rgba::new(0.3, 0.3, 0.3, 1.0));

    let theme2 = theme::Theme::new();
    theme2.set("base_color", color::Rgba::new(0.0, 1.0, 0.0, 1.0));
    theme2.set("animation.duration", 0.7);
    theme2.set("graph.node.shadow.color", 5.0);
    theme2.set("graph.node.shadow.size", 5.0);
    theme2.set("mouse.pointer.color", color::Rgba::new(0.3, 0.3, 0.3, 1.0));

    theme_manager.register("theme1", theme1);
    theme_manager.register("theme2", theme2);

    theme_manager.set_enabled(&["theme1".to_string()]);

    let style_watch = ensogl_core::display::shape::StyleWatch::new(&scene.style_sheet);
    // style_watch.set_on_style_change(|| DEBUG!("Style changed!"));
    style_watch.get("base_color");

    let view1 = shape::View::new(&logger);
    view1.size.set(Vector2::new(300.0, 300.0));
    view1.mod_position(|t| *t = Vector3::new(50.0, 50.0, 0.0));

    let mask = mask::View::new(&logger);
    mask.size.set(Vector2::new(300.0, 300.0));
    mask.mod_position(|t| *t = Vector3::new(-50.0, 0.0, 0.0));

    let scissor_box =
        scene::layer::ScissorBox::new_with_position_and_size(default(), Vector2(600, 600));
    scene.layers.main.set_scissor_box(Some(&scissor_box));

    let view2 = shape::View::new(&logger);
    view2.size.set(Vector2::new(300.0, 300.0));
    view2.mod_position(|t| *t = Vector3::new(50.0, 0.0, 0.0));

    world.add_child(&view1);
    world.add_child(&mask);

    world.keep_alive_forever();
    let scene = world.scene().clone_ref();

    let mut frame = 0;
    world
        .on_frame(move |_time| {
            mask.set_position_x(((frame as f32) / 30.0).sin() * 100.0);

            let _keep_alive = &navigator;
            let _keep_alive = &style_watch;
            let _keep_alive = &theme_manager;
            if frame == 50 {
                // These comments are left for easy debugging in the future.
                // DEBUG!("---------------");
                // DEBUG!("{scene.layers.node_searcher:#?}");
                // DEBUG!("{scene.layers.main:#?}");
                // DEBUG!("{scene.layers.mask:#?}");
                // DEBUG!("{scene.layers.node_searcher_mask:#?}");
                // DEBUG!("{scene.layers.viz:#?}");
            }
            if frame == 100 {
                DEBUG!("Adding previously hidden element.");
                scene.add_child(&view2);
                // These comments are left for easy debugging in the future.
                // DEBUG!("---------------");
                // DEBUG!("{scene.layers.node_searcher:#?}");
                // DEBUG!("{scene.layers.main:#?}");
                // DEBUG!("{scene.layers.mask:#?}");
                // DEBUG!("{scene.layers.node_searcher_mask:#?}");
                // DEBUG!("{scene.layers.viz:#?}");
            }
            if frame == 150 {
                DEBUG!("Enabling masking.");
                // These comments are left for easy debugging in the future.
                // DEBUG!("---------------");
                // DEBUG!("{scene.layers.node_searcher:#?}");
                // DEBUG!("{scene.layers.main:#?}");
                // DEBUG!("{scene.layers.mask:#?}");
                // DEBUG!("{scene.layers.node_searcher_mask:#?}");
                // DEBUG!("{scene.layers.viz:#?}");

                scene.layers.node_searcher.add_exclusive(&view1);
                scene.layers.node_searcher.add_exclusive(&view2);
                scene.layers.node_searcher_mask.add_exclusive(&mask);
            }
            if frame == 200 {
                DEBUG!("Changing the theme.");
                theme_manager.set_enabled(&["theme2".to_string()]);
            }
            frame += 1;
        })
        .forget();
}
