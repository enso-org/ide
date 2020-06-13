
use crate::*;
pub use shapely::CloneRef;



// ================
// === CloneRef ===
// ================

/// Clone for internal-mutable structures. This trait can be implemented only if mutating one
/// structure will be reflected in all of its clones. Please note that it does not mean that all the
/// fields needs to provide internal mutability as well. For example, a structure can remember it's
/// creation time and store it as `f32`. As long as it cannot be mutated, the structure can
/// implement `CloneRef`. In order to guide the auto-deriving mechanism, it is advised to wrap all
/// immutable fields in the `Immutable` newtype.
pub trait CloneRef: Sized {
    fn clone_ref(&self) -> Self;
}

#[macro_export]
macro_rules! impl_clone_ref_as_clone {
    ([$($bounds:tt)*] $($toks:tt)*) => {
        impl <$($bounds)*> CloneRef for $($toks)* {
            fn clone_ref(&self) -> Self {
                self.clone()
            }
        }

        impl <$($bounds)*> From<&$($toks)*> for $($toks)* {
            fn from(t:&$($toks)*) -> Self {
                t.clone_ref()
            }
        }
    };

    ($($toks:tt)*) => {
        impl CloneRef for $($toks)* {
            fn clone_ref(&self) -> Self {
                self.clone()
            }
        }

        impl From<&$($toks)*> for $($toks)* {
            fn from(t:&$($toks)*) -> Self {
                t.clone_ref()
            }
        }
    };
}

#[macro_export]
macro_rules! impl_clone_ref_as_clone_no_from {
    ([$($bounds:tt)*] $($toks:tt)*) => {
        impl <$($bounds)*> CloneRef for $($toks)* {
            fn clone_ref(&self) -> Self {
                self.clone()
            }
        }
    };

    ($($toks:tt)*) => {
        impl CloneRef for $($toks)* {
            fn clone_ref(&self) -> Self {
                self.clone()
            }
        }
    };
}

impl_clone_ref_as_clone_no_from!(());
impl_clone_ref_as_clone_no_from!(f32);
impl_clone_ref_as_clone_no_from!(f64);
impl_clone_ref_as_clone_no_from!(i32);
impl_clone_ref_as_clone_no_from!(i64);
impl_clone_ref_as_clone_no_from!(usize);
impl_clone_ref_as_clone_no_from!([T] PhantomData<T>);
impl_clone_ref_as_clone_no_from!([T:?Sized] Rc<T>);
impl_clone_ref_as_clone_no_from!([T:?Sized] Weak<T>);

impl_clone_ref_as_clone_no_from!(wasm_bindgen::JsValue);
impl_clone_ref_as_clone_no_from!(web_sys::HtmlDivElement);
impl_clone_ref_as_clone_no_from!(web_sys::HtmlElement);
impl_clone_ref_as_clone_no_from!(web_sys::Performance);
impl_clone_ref_as_clone_no_from!(web_sys::WebGl2RenderingContext);
impl_clone_ref_as_clone_no_from!(web_sys::HtmlCanvasElement);
impl_clone_ref_as_clone_no_from!(web_sys::EventTarget);
