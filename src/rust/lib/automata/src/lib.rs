pub mod alphabet;
pub mod data;
pub mod dfa;
pub mod nfa;
pub mod pattern;
pub mod state;
pub mod symbol;

pub use dfa::Dfa;
pub use nfa::Nfa;
pub use pattern::*;
pub use symbol::*;

use enso_prelude as prelude;


use prelude::*;

pub fn main() {
    let mut nfa : Nfa = default();
    let end_a = nfa.new_pattern(nfa.start,&Pattern::char('a'));
    let end_a2 = nfa.new_pattern(nfa.start,&Pattern::char('a'));
    let end_b = nfa.new_pattern(end_a2,&Pattern::char('b'));
    let end_c = nfa.new_pattern(end_b,&Pattern::char('c'));
    let end_x = nfa.new_pattern(nfa.start,&Pattern::char('x'));

    let dfa = Dfa::from(&nfa);

    println!("---------");
    let after_a = dfa.next_state(Dfa::START_STATE,Symbol::new(97));
    let after_b = dfa.next_state(after_a,Symbol::new(98));
    let after_c = dfa.next_state(after_b,Symbol::new(99));
    println!("{:?}",after_a);
    println!("{:?}",after_b);
    println!("{:?}",after_c);

}
