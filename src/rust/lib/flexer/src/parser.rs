use crate::automata::state::Symbol;



// ============
// == Parser ==
// ============

/// End Of File.
/// This symbol is inserted at the end of each parser input.
pub const EOF_CODE:Symbol = Symbol{val:-1};
