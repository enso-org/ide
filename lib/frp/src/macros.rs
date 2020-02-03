//! This module defines common macros for FRP netwrok definition.



/// Utility for easy definition of the FRP network. In order to keep the network easy to debug and
/// reason about, each node constructor consumes a label. Providing labels manually is time
/// consuming and error prone. This utility infers the name from the assignment shape and provides
/// it automatically to the FRP node constructor.
#[macro_export]
macro_rules! frp_def {
    ($var:ident = $fn:ident $(.$fn2:ident)* $(::<$ty:ty>)? ($($args:tt)*)) => {
        let $var = $fn $(.$fn2)* $(::<$ty>)?
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
