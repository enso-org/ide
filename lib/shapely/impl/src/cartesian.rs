/// Computes a cartesian product of the provided input.
///
/// For the following expression:
/// ```compile_fail
/// cartesian!(f [g] [a b c] [x y z]);
/// ```
///
/// It expands to:
/// ```compile_fail
/// f! { [g] [ [a x] [a y] [a z] [b x] [b y] [b z] [c x] [c y] [c z] ] }
/// ```
///
/// If you provide underscore as second argument, it is skipped in the ouput macro:
///
/// ```compile_fail
/// cartesian!(f _ [a b c] [x y z]);
/// ```
///
/// Expands to:
/// ```compile_fail
/// f! { [ [a x] [a y] [a z] [b x] [b y] [b z] [c x] [c y] [c z] ] }
/// ```
#[macro_export]
macro_rules! cartesian {
    ($f:ident $args:tt [$($a:tt)*] [$($b:tt)*]) => {
        $crate::_cartesian_impl!{ $f $args [] [$($a)*] [$($b)*] [$($b)*] }
    };
}

/// Internal helper for `cartesian` macro.
#[macro_export]
macro_rules! _cartesian_impl {
    ($f:ident _ $out:tt [] $b:tt $init_b:tt) => {
        $f!{ $out }
    };
    ($f:ident $args:tt $out:tt [] $b:tt $init_b:tt) => {
        $f!{ $args $out }
    };
    ($f:ident $args:tt $out:tt [$a:ident $($at:tt)*] [] $init_b:tt) => {
        $crate::_cartesian_impl!{ $f $args $out [$($at)*] $init_b $init_b }
    };
    ($f:ident $args:tt [$($out:tt)*] [$a:ident $($at:tt)*] [$b:ident $($bt:tt)*] $init_b:tt) => {
        $crate::_cartesian_impl!{ $f $args [$($out)* [$a $b]] [$a $($at)*] [$($bt)*] $init_b }
    };
}
