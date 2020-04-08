


#[macro_export]
macro_rules! new_network {
    ($network:ident $($ts:tt)*) => {
        let $network = $crate::Network::new();
        $crate::extend_network! { $network $($ts)* }
    };
}

#[macro_export]
macro_rules! extend_network {
    ($network:ident $($ts:tt)*) => {
        $crate::divide_on_terminator! { [[$crate::extend_network_lines] [$network]] $($ts)* }
    };
}


#[macro_export]
macro_rules! extend_network_lines {
    ([$network:ident] [ $([$($line:tt)*])* ]) => {$(
        $crate::extend_network_line! { $network $($line)* }
    )*}
}

#[macro_export]
macro_rules! divide_on_terminator {
    ($f:tt $($ts:tt)*) => { $crate::_divide_on_terminator! { $f [] [] $($ts)* } };
}


#[macro_export]
macro_rules! extend_network_line {
    ($network:ident def $name:ident $(:$ty:ty)? = $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => {
        let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!($name),$($arg)*) $($ts)*
    };

    ($network:ident def $name:ident $(:$ty:ty)? = $tgt:ident . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => {
        let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!($name),&$tgt,$($arg)*) $($ts)*
    };

    ($network:ident def $name:ident $(:$ty:ty)? = $tgt:ident . $tgt2:ident . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => {
        let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!($name),&$tgt.$tgt2,$($arg)*) $($ts)*
    };

    ($network:ident def $name:ident $(:$ty:ty)? = $tgt:ident . $tgt2:ident . $tgt3:ident . $base:ident$(::<$param:ty>)?($($arg:tt)*) $($ts:tt)*) => {
        let $name $(:$ty)? = $network.$base$(::<$param>)?(stringify!($name),&$tgt.$tgt2.$tgt3,$($arg)*) $($ts)*
    };

    ($network:ident $($ts:tt)*) => { $($ts)* }
}


#[macro_export]
macro_rules! _divide_on_terminator {
    ([[$($f:tt)*] $args:tt] $lines:tt       [])                           => { $($f)*! {$args $lines} };
    ([[$($f:tt)*] $args:tt] [$($lines:tt)*] $line:tt)                     => { $crate::_divide_on_terminator! {[[$($f)*] $args] [$($lines)* $line]        []} };
    ($f:tt               [$($lines:tt)*] [$($line:tt)*] ;     $($ts:tt)*) => { $crate::_divide_on_terminator! {$f         [$($lines)* [$($line)*;]] []             $($ts)*} };
    ($f:tt               $lines:tt       [$($line:tt)*] $t:tt $($ts:tt)*) => { $crate::_divide_on_terminator! {$f         $lines                    [$($line)* $t] $($ts)*} };
}
