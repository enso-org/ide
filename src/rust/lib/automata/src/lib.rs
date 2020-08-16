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


use std::collections::hash_map::DefaultHasher;

fn hash<T:Hash>(t:&T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}


fn print_matrix(matrix:&data::Matrix<dfa::State>) {
    println!("rows x cols = {} x {} ({})",matrix.rows, matrix.columns,matrix.matrix.len());
    for row in 0..matrix.rows {
        for column in 0..matrix.columns {
            let elem = matrix.safe_index(row,column).unwrap();
            let repr = if elem.is_invalid() { "-".into() } else { format!("{}",elem.id()) };
            print!("{} ",repr);
        }
        println!("");
    }
}


pub fn main() {
    // let mut nfa : Nfa = default();
    // let end_a = nfa.new_pattern(nfa.start,Pattern::char('a'));
    // let end_a2 = nfa.new_pattern(nfa.start,Pattern::char('a'));
    // let end_b = nfa.new_pattern(end_a2,Pattern::char('b'));
    // let end_c = nfa.new_pattern(end_b,Pattern::char('c'));
    // let end_x = nfa.new_pattern(nfa.start,Pattern::char('x'));
    //
    // let dfa = Dfa::from(&nfa);
    //
    // println!("---------");
    // let after_a = dfa.next_state(Dfa::START_STATE,Symbol::new(97));
    // let after_b = dfa.next_state(after_a,Symbol::new(98));
    // let after_c = dfa.next_state(after_b,Symbol::new(99));
    // println!("{:?}",after_a);
    // println!("{:?}",after_b);
    // println!("{:?}",after_c);







    let sym_a = Symbol::from(81);
    let sym_b = Symbol::from(97);
    let sym_c = Symbol::from(99);
    // let sym_x = Symbol::from(110);

    let mut nfa : Nfa = default();
    let end_a = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    let end_b = nfa.new_pattern(end_a,Pattern::symbol(sym_b));
    let end_c = nfa.new_pattern(end_b,Pattern::symbol(sym_c));
    // let end_x = nfa.new_pattern(nfa.start,Pattern::symbol(sym_x));

    let dfa = Dfa::from(&nfa);


    println!("---------");
    print_matrix(&dfa.links);


    println!("---------");
    let after_a = dfa.next_state(Dfa::START_STATE,sym_a);
    println!(">> {:?}",after_a);
    let after_b = dfa.next_state(after_a,sym_b);
    println!(">> {:?}",after_b);
    let after_c = dfa.next_state(after_b,sym_c);
    println!(">> {:?}",after_c);




    println!("===================");

    let sym_a = Symbol::from(hash(&"a".to_string()));
    let sym_b = Symbol::from(hash(&"b".to_string()));
    let sym_c = Symbol::from(hash(&"c".to_string()));
    let sym_x = Symbol::from(hash(&"o".to_string()));

    println!("sym_a = {:?}", sym_a);
    println!("sym_b = {:?}", sym_b);
    println!("sym_c = {:?}", sym_c);
    println!("sym_x = {:?}", sym_x);

    let mut nfa : Nfa = default();
    let end_a = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    let end_a2 = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    let end_b = nfa.new_pattern(end_a2,Pattern::symbol(sym_b));
    let end_c = nfa.new_pattern(end_b,Pattern::symbol(sym_c));
    let end_x = nfa.new_pattern(nfa.start,Pattern::symbol(sym_x));

    let dfa = Dfa::from(&nfa);

    println!("---------");

    print_matrix(&dfa.links);


    let after_a = dfa.next_state(Dfa::START_STATE,sym_a);
    let after_b = dfa.next_state(after_a,sym_b);
    let after_c = dfa.next_state(after_b,sym_c);
    println!("{:?}",after_a);
    println!("{:?}",after_b);
    println!("{:?}",after_c);

    println!("{:?}",hash(&"a".to_string()));
    println!("{:?}",hash(&"a".to_string()));
    println!("{:?}",hash(&"a".to_string()));

}
