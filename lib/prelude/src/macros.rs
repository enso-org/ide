//! This macro defines set of common macros which are useful across different projects.


/// Allows for nicer definition of impls, similar to what Haskell or Scala does. Reduces the needed
/// boilerplate. For example, the following usage:
///
/// ```
/// struct A { name:String };
/// impls! { From<A> for String { |t| t.name.clone() } }
/// ```
///
/// compiles to:
/// ```
/// struct A { name:String };
/// impl From<A> for String {
///     fn from(t:A) -> Self {
///         t.name.clone()
///     }
/// }
/// ```
///
/// This macro is meant to support many standard traits (like From) and should grow in the future.
#[macro_export]
macro_rules! impls {
    ( From<$ty:ty> for $target:ty { |$arg:ident| $($lambda:tt)* } ) => {
        impl From <$ty> for $target {
            fn from ($arg:$ty) -> Self {
                (|$arg:$ty| $($lambda)*)($arg)
            }
        }
    }
}
