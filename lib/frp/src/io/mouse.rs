use crate::prelude::*;
use crate::nodes::*;


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




// ================
// === Position ===
// ================

#[derive(Clone,Copy,Debug,Default)]
pub struct Position {
    pub x:i32,
    pub y:i32,
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



// =============
// === Mouse ===
// =============

pub struct Mouse {
    pub up       : Dynamic<()>,
    pub down     : Dynamic<()>,
    pub is_down  : Dynamic<bool>,
    pub position : Dynamic<Position>,
}

impl Default for Mouse {
    fn default() -> Self {
        frp_def! { mouse.up        = source() }
        frp_def! { mouse.down      = source() }
        frp_def! { mouse.position  = source() }
        frp_def! { mouse.down_bool = down.constant(true) }
        frp_def! { mouse.up_bool   = up.constant(false) }
        frp_def! { mouse.is_down   = down_bool.merge(&up_bool) }
        Self {up,down,is_down,position}
    }
}

impl Mouse {
    pub fn new() -> Self {
        default()
    }
}