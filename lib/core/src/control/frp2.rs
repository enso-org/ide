#![allow(missing_docs)]

use crate::prelude::*;
pub use enso_frp::*;

mod tests {
    use super::*;

    use crate::system::web;
    use crate::control::io::mouse2;
    use crate::control::io::mouse2::MouseManager;


    // ================
    // === Position ===
    // ================

    #[derive(Clone,Copy,Debug,Default)]
    pub struct Position {
        x:i32,
        y:i32,
    }

    impl Position {
        pub fn new(x:i32, y:i32) -> Self {
            Self {x,y}
        }
    }

    impl std::ops::Sub<&Position> for &Position {
        type Output = Position;
        fn sub(self, rhs: &Position) -> Self::Output {
            let x = self.x - rhs.x;
            let y = self.y - rhs.y;
            Position {x,y}
        }
    }


    macro_rules! frp_def {
        ($var:ident = $fn:ident $(.$fn2:ident)* $(::<$ty:ty>)? ($($args:tt)*)) => {
            let $var = Dynamic $(::<$ty>)? :: $fn $(.$fn2)*
            ( concat! {stringify!{$var}}, $($args)* );
        };

        ($scope:ident . $var:ident = $fn:ident $(::<$ty:ty>)? ($($args:tt)*)) => {
            let $var = Dynamic $(::<$ty>)? :: $fn
            ( concat! {stringify!{$scope},".",stringify!{$var}}, $($args)* );
        };

        ($scope:ident . $var:ident = $fn1:ident . $fn2:ident $(.$fn3:ident)* $(::<$ty:ty>)? ($($args:tt)*)) => {
            let $var = $fn1 . $fn2 $(.$fn3)* $(::<$ty>)?
            ( concat! {stringify!{$scope},".",stringify!{$var}}, $($args)* );
        };
    }

    // ============
    // === Test ===
    // ============

    pub struct Mouse {
        pub up       : Dynamic<()>,
        pub down     : Dynamic<()>,
        pub is_down  : Dynamic<bool>,
        pub position : Dynamic<Position>,
    }

    impl Mouse {
        pub fn new() -> Self {
            frp_def! { mouse.up        = source() }
            frp_def! { mouse.down      = source() }
            frp_def! { mouse.position  = source() }
            frp_def! { mouse.down_bool = down.constant(true) }
            frp_def! { mouse.up_bool   = up.constant(false) }
            frp_def! { mouse.is_down   = down_bool.merge(&up_bool) }
            Self {up,down,is_down,position}
        }
    }

    #[allow(unused_variables)]
    pub fn test (callback: Box<dyn Fn(f32,f32)>) -> MouseManager {

        let document        = web::document().unwrap();
        let mouse_manager   = MouseManager::new(&document);



        println!("\n\n\n--- FRP ---\n");


        let mouse = Mouse::new();

        let mouse_down_position    = mouse.position.sample("mouse_down_position",&mouse.down);
        let mouse_position_if_down = mouse.position.gate("mouse_position_if_down",&mouse.is_down);

        let final_position_ref_i  = Recursive::<EventMessage<Position>>::new_named("final_position_ref");
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
            target.emit(&EventMessage(Position::new(event.client_x(),event.client_y())));
        });
        handle.forget();

        let target = mouse.down.event.clone_ref();
        let handle = mouse_manager.on_down.add(move |event:&mouse2::event::OnDown| {
            target.emit(&EventMessage(()));
        });
        handle.forget();

        let target = mouse.up.event.clone_ref();
        let handle = mouse_manager.on_up.add(move |event:&mouse2::event::OnUp| {
            target.emit(&EventMessage(()));
        });
        handle.forget();

        mouse_manager

    }
}
pub use tests::*;
