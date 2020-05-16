//! This module defines common macros for FRP netwrok definition.

/// Utility for an easy definition of a new FRP network. In order to keep the network easy to debug
/// and reason about, each node constructor consumes a label. Providing labels manually is time
/// consuming and error prone. This utility infers the name from the assignment shape and provides
/// it automatically to the FRP node constructor.
///
/// The syntax exposed by this macro is very similar to standard Rust syntax. There is are two new
/// keywords - `def` and `trace`. The former defines new FRP nodes while the later provides handy
/// debugging utility. Every line which does not start with the keywords is interpreted just as a
/// regular Rust code. Moreover, there is a special flag `TRACE_ALL` which you can use as the first
/// text inside of macro, which will automatically enable each definition to be traced.
///
/// A simple counter network which prints the current count on every number and prints "hello world"
/// on creation is presented below.
///
/// ```compile_fail
/// frp::new_network! { network1
///     def source = source();
///     def count  = source.count();
///     trace count;
///     println!("Hello world!");
/// }
/// ```
#[macro_export]
macro_rules! new_network {
    (TRACE_ALL $($ts:tt)*) => { $crate::_new_network! { TRACE    $($ts)* } };
    ($($ts:tt)*)           => { $crate::_new_network! { NO_TRACE $($ts)* } };
}

/// Just like `new_network` but for the dynamic FRP mode.
#[macro_export]
macro_rules! new_dynamic_network {
    (TRACE_ALL $($ts:tt)*) => { $crate::_new_dynamic_network! { TRACE    $($ts)* } };
    ($($ts:tt)*)           => { $crate::_new_dynamic_network! { NO_TRACE $($ts)* } };
}

/// Extends the provided network with new rules. See documentation of `new_network` to learn more.
#[macro_export]
macro_rules! extend {
    (TRACE_ALL $($ts:tt)*) => { $crate::_extend! { TRACE    $($ts)* } };
    ($($ts:tt)*)           => { $crate::_extend! { NO_TRACE $($ts)* } };
}



// ===================
// === Private API ===
// ===================

// === New ===

/// Internal helper for `new_network` macro.
#[macro_export]
macro_rules! _new_network {
    ($trace:ident $network:ident $($ts:tt)*) => {
        let $network = $crate::Network::new();
        $crate::_extend! { $trace $network $($ts)* }
    };
}

/// Internal helper for `new_dynamic_network` macro.
#[macro_export]
macro_rules! _new_dynamic_network {
    ($trace:ident $($ts:tt)*) => {
        let __dynamic__ = $crate::DynamicNetwork::new();
        $crate::_extend! { $trace __dynamic__ $($ts)* }
    };
}

/// Creates a new `BridgeNetwork` for the provided networks.
#[macro_export]
macro_rules! new_bridge_network {
    ([$($($path:ident).*),*] $($ts:tt)*) => {
        let _birdge_network_ = $crate::Network::new();
        $crate::extend! { _birdge_network_ $($ts)* }
        let _birdge_network_ = $crate::BridgeNetwork::from(_birdge_network_);
        $($($path).*.register_bridge_network(&_birdge_network_);)*
    };
}


// === Extend ===

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! _extend {
    ($trace:ident $network:ident $($ts:tt)*) => {
        $crate::divide_on_terminator2! { [[$crate::extend_lines] [$trace $network]] $($ts)* }
    };
}

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! extend_lines {
    ([$trace:ident $network:ident] [ $([$($line:tt)*])* ]) => {$(
        $crate::extend_line1! { $trace $network $($line)* }
    )*}
}

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! extend_line1 {
    (TRACE $network:ident def $name:ident $($toks:tt)*) => {
        $crate::extend_line2! { [] $network def $name $($toks)* }
        $crate::extend_line2! { [] $network trace $name }
    };
    ($trace:ident $($toks:tt)*) => {
        $crate::extend_line2! { [] $($toks)* }
    };
}

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! extend_line2 {
    ([$($lines:tt)*] $net:ident def $name:ident = $name2:ident) => { $($lines)* };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? =                                                                       $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),$($arg)*)                                ;] $net def $name = $name $($ts)* } };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? = $tgt1:ident                                                         . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),&$tgt1,$($arg)*)                         ;] $net def $name = $name $($ts)* } };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident                                           . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),&$tgt1.$tgt2,$($arg)*)                   ;] $net def $name = $name $($ts)* } };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident . $tgt3:ident                             . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),&$tgt1.$tgt2.$tgt3,$($arg)*)             ;] $net def $name = $name $($ts)* } };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident               . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),&$tgt1.$tgt2.$tgt3.$tgt4,$($arg)*)       ;] $net def $name = $name $($ts)* } };
    ([$($lines:tt)*] $net:ident def $name:ident $(:$ty:ty)? = $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident . $tgt5:ident . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [$($lines)* let $name $(:$ty)? = $net.$base$(::<$param>)?(concat!(stringify!($net),".",stringify!($name)),&$tgt1.$tgt2.$tgt3.$tgt4.$tgt5,$($arg)*) ;] $net def $name = $name $($ts)* } };

    ([] $net:ident $name:ident <- [ $($arg1:ident).+ ] )                                                          => { let $name = $($arg1).+.clone_ref(); };
    ([] $net:ident $name:ident <- [ $($arg1:ident).+ , $($arg2:ident).+ ] )                                       => {$crate::extend_line2! { [] $net def $name = merge2(&$($arg1).+,&$($arg2).+) } };
    ([] $net:ident $name:ident <- [ $($arg1:ident).+ , $($arg2:ident).+ , $($arg3:ident).+ ] )                    => {$crate::extend_line2! { [] $net def $name = merge3(&$($arg1).+,&$($arg2).+,&$($arg3).+) } };
    ([] $net:ident $name:ident <- [ $($arg1:ident).+ , $($arg2:ident).+ , $($arg3:ident).+ , $($arg4:ident).+ ] ) => {$crate::extend_line2! { [] $net def $name = merge4(&$($arg1).+,&$($arg2).+,&$($arg3).+,&$($arg4).+) } };

    ([] $net:ident $name:ident <_ [ $($arg1:ident).+ ] )                                                          => { let $name = $($arg1).+.constant(()); };
    ([] $net:ident $name:ident <_ [ $($arg1:ident).+ , $($arg2:ident).+ ] )                                       => {$crate::extend_line2! { [] $net def $name = merge2_(&$($arg1).+,&$($arg2).+) } };
    ([] $net:ident $name:ident <_ [ $($arg1:ident).+ , $($arg2:ident).+ , $($arg3:ident).+ ] )                    => {$crate::extend_line2! { [] $net def $name = merge3_(&$($arg1).+,&$($arg2).+,&$($arg3).+) } };
    ([] $net:ident $name:ident <_ [ $($arg1:ident).+ , $($arg2:ident).+ , $($arg3:ident).+ , $($arg4:ident).+ ] ) => {$crate::extend_line2! { [] $net def $name = merge4_(&$($arg1).+,&$($arg2).+,&$($arg3).+,&$($arg4).+) } };

    ([] $net:ident $name:ident <= $($toks:tt)*) => {$crate::extend_line2! { [] $net def $name = $($toks)* . iter()} };
    ([] $net:ident $name:ident <- $($toks:tt)*) => {$crate::extend_line2! { [] $net def $name = $($toks)* } };
    ([] $net:ident $($tgt:ident).+ <+ $($src:ident).+) => { $($tgt).+.attach(&$($src).+); };

    ([] $net:ident eval $tgt1:ident                                                         ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1                                 . map (f!($($args)*)) $($ts)* } };
    ([] $net:ident eval $tgt1:ident . $tgt2:ident                                           ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2                         . map (f!($($args)*)) $($ts)* } };
    ([] $net:ident eval $tgt1:ident . $tgt2:ident . $tgt3:ident                             ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3                 . map (f!($($args)*)) $($ts)* } };
    ([] $net:ident eval $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident               ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3 . $tgt4         . map (f!($($args)*)) $($ts)* } };
    ([] $net:ident eval $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident . $tgt5:ident ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3 . $tgt4 . $tgt5 . map (f!($($args)*)) $($ts)* } };

    ([] $net:ident eval_ $tgt1:ident                                                         ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1                                 . map (f_!($($args)*)) $($ts)* } };
    ([] $net:ident eval_ $tgt1:ident . $tgt2:ident                                           ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2                         . map (f_!($($args)*)) $($ts)* } };
    ([] $net:ident eval_ $tgt1:ident . $tgt2:ident . $tgt3:ident                             ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3                 . map (f_!($($args)*)) $($ts)* } };
    ([] $net:ident eval_ $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident               ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3 . $tgt4         . map (f_!($($args)*)) $($ts)* } };
    ([] $net:ident eval_ $tgt1:ident . $tgt2:ident . $tgt3:ident . $tgt4:ident . $tgt5:ident ($($args:tt)*) $($ts:tt)*) => { $crate::extend_line2! { [] $net def _eval = $tgt1 . $tgt2 . $tgt3 . $tgt4 . $tgt5 . map (f_!($($args)*)) $($ts)* } };

    ([] $net:ident trace $($path:ident).*) => { $net.trace(stringify!($($path).*),&$($path).*); };
    ([] $net:ident $($ts:tt)*) => { $($ts)*; }
}


// === Utils ===

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! divide_on_terminator2 {
    ($f:tt $($ts:tt)*) => { $crate::_divide_on_terminator! { $f [] [] $($ts)* } };
}

/// Internal helper for `extend` macro.
#[macro_export]
macro_rules! _divide_on_terminator {
    ([[$($f:tt)*] $args:tt] $lines:tt       [])                              => { $($f)*! {$args $lines} };
    ([[$($f:tt)*] $args:tt] [$($lines:tt)*] $line:tt)                        => { MISSING_SEMICOLON };
    ($f:tt                  [$($lines:tt)*] [$($line:tt)*] ;     $($ts:tt)*) => { $crate::_divide_on_terminator! {$f               [$($lines)* [$($line)*]] []             $($ts)*} };
    ($f:tt                  $lines:tt       [$($line:tt)*] $t:tt $($ts:tt)*) => { $crate::_divide_on_terminator! {$f               $lines                   [$($line)* $t] $($ts)*} };
}
