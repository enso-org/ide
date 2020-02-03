#![allow(missing_docs)]

use crate::prelude::*;
pub use enso_frp::*;

mod tests {
    use super::*;

    use crate::system::web;
    use crate::control::io::mouse2;
    use crate::control::io::mouse2::MouseManager;







    // ============
    // === Test ===
    // ============



    #[allow(unused_variables)]
    pub fn test (callback: Box<dyn Fn(f32,f32)>) -> MouseManager {

        let document        = web::document().unwrap();
        let mouse_manager   = MouseManager::new(&document);



        println!("\n\n\n--- FRP ---\n");


        let mouse = Mouse::new();

        let mouse_down_position    = mouse.position.sample("mouse_down_position",&mouse.down);
        let mouse_position_if_down = mouse.position.gate("mouse_position_if_down",&mouse.is_down);

        let final_position_ref_i  = Recursive::<EventData<Position>>::new_named("final_position_ref");
        let final_position_ref    = Dynamic::from(&final_position_ref_i);

        let pos_diff_on_down   = mouse_down_position.map2("pos_diff_on_down", &final_position_ref, |m,f| {m - f});
        let final_position  = mouse_position_if_down.map2("final_position", &pos_diff_on_down, |m,f| {m - f});
        let debug              = final_position.sample("debug", &mouse.position);



        final_position_ref_i.initialize(&final_position);

        final_position_ref.event.set_display_id(final_position.event.display_id());
        final_position_ref.behavior.set_display_id(final_position.event.display_id());



        trace("X" , &debug.event);


        final_position.map("foo",move|p| {callback(p.x as f32,-p.y as f32)});

//        final_position.behavior.display_graphviz();

        let target = mouse.position.event.clone_ref();
        let handle = mouse_manager.on_move.add(move |event:&mouse2::event::OnMove| {
            target.emit(&EventData(Position::new(event.client_x(),event.client_y())));
        });
        handle.forget();

        let target = mouse.down.event.clone_ref();
        let handle = mouse_manager.on_down.add(move |event:&mouse2::event::OnDown| {
            target.emit(&EventData(()));
        });
        handle.forget();

        let target = mouse.up.event.clone_ref();
        let handle = mouse_manager.on_up.add(move |event:&mouse2::event::OnUp| {
            target.emit(&EventData(()));
        });
        handle.forget();

        mouse_manager

    }
}
pub use tests::*;
