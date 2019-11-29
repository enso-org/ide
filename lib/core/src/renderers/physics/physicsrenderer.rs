// use crate::prelude::*;
// use crate::renderers;
// use renderers::graphics::html::HTMLScene;
// use renderers::Scene;
// use renderers::graphics::html::HTMLObject;
//
// #[derive(Default)]
// pub struct PhysicsRenderer {}
//
// impl PhysicsRenderer {
//     pub fn new() -> Self { default() }
//
//     pub fn render(&self, mut scene : &mut HTMLScene, dt : f32) {
//         let scene : &mut Scene<HTMLObject> = &mut scene;
//         for object in &mut scene.into_iter() {
//             let position = *object.get_position();
//             object.set_position(position.x, position.y - dt, position.z);
//         }
//     }
// }
