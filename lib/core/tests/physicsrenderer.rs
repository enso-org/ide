// //! Test suite for the Web and headless browsers.
// #![cfg(target_arch = "wasm32")]
//
// use web_test::web_configure;
// web_configure!(run_in_browser);
//
// #[cfg(test)]
// mod tests {
//     use basegl::renderers;
//     use renderers::*;
//     use graphics::*;
//     use html::*;
//     use physics::*;
//     use basegl::system::web::StyleSetter;
//     use basegl::system::web::get_performance;
//     use web_test::*;
//
//     fn create_scene(dom_id : &str) -> HTMLScene {
//         let mut scene = HTMLScene::new(dom_id)
//                                   .expect("Failed to create HTMLScene");
//         assert_eq!(scene.len(), 0);
//
//         scene.container.dom.set_property_or_panic("background-color", "black");
//
//         // Iterate over 3 axes.
//         for axis in vec![(1, 0, 0), (0, 1, 0), (0, 0, 1)] {
//             // Creates 10 HTMLObjects per axis.
//             for i in 0 .. 10 {
//                 let mut object = HTMLObject::new("div").unwrap();
//                 object.set_dimensions(1.0, 1.0);
//
//                 // Using axis for masking.
//                 // For instance, the axis (0, 1, 0) creates:
//                 // (x, y, z) = (0, 0, 0) .. (0, 9, 0)
//                 let x = (i * axis.0) as f32;
//                 let y = (i * axis.1) as f32;
//                 let z = (i * axis.2) as f32;
//                 object.set_position(x, y, z);
//
//                 // Creates a gradient color based on the axis.
//                 let r = (x * 25.5) as u8;
//                 let g = (y * 25.5) as u8;
//                 let b = (z * 25.5) as u8;
//                 let color = format!("rgba({}, {}, {}, {})", r, g, b, 1.0);
//
//                 object.element.set_property_or_panic("background-color", color);
//                 scene.add(object);
//             }
//         }
//         assert_eq!(scene.len(), 30, "We should have 30 HTMLObjects");
//         scene
//     }
//
//     #[web_bench]
//     fn physics(b: &mut Bencher) {
//         let mut scene = create_scene("physics");
//
//         let view_dim = scene.get_dimensions();
//         assert_eq!((view_dim.x, view_dim.y), (320.0, 240.0));
//
//         let aspect_ratio = view_dim.x / view_dim.y;
//         let mut camera = Camera::perspective(45.0, aspect_ratio, 1.0, 1000.0);
//         let performance = get_performance()
//                          .expect("Couldn't get performance obj");
//
//         let physics  = PhysicsRenderer::new();
//         let renderer = HTMLRenderer::new();
//         camera.set_position(0.0, 0.0, 29.0);
//
//         let mut t0 = performance.now();
//         b.iter(move || {
//             let t1 = performance.now();
//             let dt = ((t1 - t0) / 1000.0) as f32;
//             t0 = t1;
//
//             physics.render(&mut scene, dt);
//             renderer.render(&mut camera, &scene);
//         })
//     }
// }
