//! This module provides utils for strings.

use enso_prelude::*;



// ===============================
// === Common Pre- and Postfix ===
// ===============================

/// Return the end index (exclusive) of the character position of the last matching character
/// of the longest common prefix of the two chars. If they are equal this will be the length of
/// the shorter string. If they are completely different this will be zero.
///
/// Example:
/// ```
/// # use utils::string::find_prefix_end_index;
/// let a = "hospital";
/// let b = "host";
/// let c = "bunny";
///
/// assert_eq!(find_prefix_end_index(a,b), 3);
/// assert_eq!(find_prefix_end_index(a,c), 0);
/// assert_eq!(find_prefix_end_index(a,a), 8);
///
/// ```
pub fn find_prefix_end_index(source_a:&str, source_b:&str) -> usize {
    let max_ix = source_a.len().min(source_b.len());
    let chars_a = source_a.chars();
    let chars_b = source_b.chars();
    let mut zipped = chars_a.zip(chars_b);
    let mismatch = zipped.find_position(|(a,b)| *a != *b);
    mismatch.map(|item| item.0).unwrap_or(max_ix)
}

/// Return the start index (exclusive) of the character position of the last matching character
/// of the longest common postfix of the two chars. Note the start position is counted backwards
/// from the end of boths strings. If they are equal this will be the length of
/// the shorter string. If they are completely different this will be zero.
///
/// Example:
/// ```
/// # use utils::string::find_postfix_start_index;
/// let a = "graveyard";
/// let b = "yard";
/// let c = "bunny";
///
/// assert_eq!(find_postfix_start_index(a,b), 4);
/// assert_eq!(find_postfix_start_index(a,c), 0);
/// assert_eq!(find_postfix_start_index(a,a), 9);
///
/// ```
pub fn find_postfix_start_index(source_a:&str, source_b:&str) -> usize {
    let max_ix = source_a.len().min(source_b.len());
    let chars_a = source_a.chars().rev();
    let chars_b = source_b.chars().rev();
    let mut zipped = chars_a.zip(chars_b);
    let mismatch = zipped.find_position(|(a,b)| *a != *b);
    mismatch.map(|item| item.0).unwrap_or(max_ix)
}
