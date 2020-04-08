//! This module defines common macros for FRP netwrok definition.

/// Utility for an easy definition of a new FRP network. In order to keep the network easy to debug
/// and reason about, each node constructor consumes a label. Providing labels manually is time
/// consuming and error prone. This utility infers the name from the assignment shape and provides
/// it automatically to the FRP node constructor.
///
/// The syntax exposed by this macro is very similar to standard Rust syntax. There is a new
/// keyword `def` which defines new FRP nodes. Every line which does not start with this keyword
/// is interpreted just as a regular Rust code. For example, these lines define a simple counter
/// network and print "hello world" to the screen:
///
/// ```
/// frp::new_network! { network1
///     def source = source();
///     def count  = source.count();
///     def debug  = count.trace();
///     println!("Hello world!");
/// }
/// ```
#[macro_export]
macro_rules! new_network {
    ($network:ident $($ts:tt)*) => {
        let $network = $crate::Network::new();
        $crate::extend_network! { $network $($ts)* }
    };
}

/// Extends the provided network with new rules. See documentation of `new_network` to learn more.
#[macro_export]
macro_rules! extend_network {
    ($network:ident $($ts:tt)*) => {
        $crate::divide_on_terminator! { [[$crate::extend_network_lines] [$network]] $($ts)* }
    };
}

/// Internal helpers for `extend_network` macro.
#[macro_export]
macro_rules! extend_network_lines {
    ([$network:ident] [ $([$($line:tt)*])* ]) => {$(
        $crate::extend_network_line! { $network $($line)* }
    )*}
}

/// Internal helpers for `extend_network` macro.
#[macro_export]
macro_rules! divide_on_terminator {
    ($f:tt $($ts:tt)*) => { $crate::_divide_on_terminator! { $f [] [] $($ts)* } };
}

/// Internal helpers for `extend_network` macro.
#[macro_export]
macro_rules! extend_network_line {
    ($network:ident def $name:ident $(:$ty:ty)? =                                           $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!{$network,.,$name},$($arg)*)                    $($ts)* };
    ($network:ident def $name:ident $(:$ty:ty)? = $tgt1:ident                             . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!{$network,.,$name},&$tgt1,$($arg)*)             $($ts)* };
    ($network:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident               . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!{$network,.,$name},&$tgt1.$tgt2,$($arg)*)       $($ts)* };
    ($network:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident . $tgt3:ident . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!{$network,.,$name},&$tgt1.$tgt2.$tgt3,$($arg)*) $($ts)* };
    ($network:ident $($ts:tt)*) => { $($ts)* }
}

/// Internal helpers for `extend_network` macro.
#[macro_export]
macro_rules! _divide_on_terminator {
    ([[$($f:tt)*] $args:tt] $lines:tt       [])                              => { $($f)*! {$args $lines} };
    ([[$($f:tt)*] $args:tt] [$($lines:tt)*] $line:tt)                        => { MISSING_SEMICOLON };
    ($f:tt                  [$($lines:tt)*] [$($line:tt)*] ;     $($ts:tt)*) => { $crate::_divide_on_terminator! {$f               [$($lines)* [$($line)*;]] []             $($ts)*} };
    ($f:tt                  $lines:tt       [$($line:tt)*] $t:tt $($ts:tt)*) => { $crate::_divide_on_terminator! {$f               $lines                    [$($line)* $t] $($ts)*} };
}
