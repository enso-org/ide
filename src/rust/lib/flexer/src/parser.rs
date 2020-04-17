use crate::automata::state::Symbol;



// ============
// == Parser ==
// ============

/// End Of File - This symbol is inserted at the end of each parser input.
/// We can use the maximum value of u32, because no `char` (unicode scalar) can hold this value.
pub const EOF_CODE:Symbol = Symbol{val:u32::max_value()};
