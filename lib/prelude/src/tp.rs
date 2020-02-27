//! Type related utilities.

use super::std_reexports::*;

// ===================
// === TypeDisplay ===
// ===================

/// Like `Display` trait but for types. However, unlike `Display` it defaults to
/// `impl::any::type_name` if not provided with explicit implementation.
pub trait TypeDisplay {
    fn type_display() -> String;
}

impl<T> TypeDisplay for T {
    default fn type_display() -> String {
        type_name::<Self>().to_string()
    }
}

/// Formats the type for the user-facing output.
pub fn type_display<T:TypeDisplay>() -> String {
    <T as TypeDisplay>::type_display()
}
