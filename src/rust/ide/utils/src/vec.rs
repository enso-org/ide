//! This module provides utils for the standard Vec<T>.

/// Attempts to remove `T` if its `index` is valid. If not, it returns `None`.
pub fn try_remove<T>(vec:&mut Vec<T>, index:usize) -> Option<T> {
    if index < vec.len() {
        Some(vec.remove(index))
    } else {
        None
    }
}

/// Attempts to remove the first element of `Vec<T>`, returns `None` if its length is zero.
pub fn pop_front<T>(vec:&mut Vec<T>) -> Option<T> {
    try_remove(vec, 0)
}
