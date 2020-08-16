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


// pub fn main() {
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







    // let sym_a = Symbol::from(81);
    // let sym_b = Symbol::from(97);
    // let sym_c = Symbol::from(99);
    // // let sym_x = Symbol::from(110);
    //
    // let mut nfa : Nfa = default();
    // let end_a = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    // let end_b = nfa.new_pattern(end_a,Pattern::symbol(sym_b));
    // let end_c = nfa.new_pattern(end_b,Pattern::symbol(sym_c));
    // // let end_x = nfa.new_pattern(nfa.start,Pattern::symbol(sym_x));
    //
    // let dfa = Dfa::from(&nfa);
    //
    //
    // println!("---------");
    // print_matrix(&dfa.links);
    //
    //
    // println!("---------");
    // let after_a = dfa.next_state(Dfa::START_STATE,sym_a);
    // println!(">> {:?}",after_a);
    // let after_b = dfa.next_state(after_a,sym_b);
    // println!(">> {:?}",after_b);
    // let after_c = dfa.next_state(after_b,sym_c);
    // println!(">> {:?}",after_c);


    // println!("===================");


#[derive(Debug)]
pub struct Shortcuts {
    nfa    : Nfa,
    dfa    : Dfa,
    states : HashMap<String,nfa::State>,
}

impl Shortcuts {
    pub fn new() -> Self {
        let nfa    = Nfa::default();
        let dfa    = Dfa::from(&nfa);
        let states = default();
        Self {nfa,dfa,states}
    }

    pub fn add_key_sequence(&mut self, keys:&[&str]) -> nfa::State {
        match keys {
            [] => self.nfa.start,
            _ => todo!()
        }
    }

    pub fn add_kb_shortcut(&mut self, m:&str, key:&str) {

        let sym_mouse_0 = Symbol::from(hash(&"mouse_0".to_string()));
        let sym_mouse_1 = Symbol::from(hash(&"mouse_1".to_string()));
        let sym_mouse_2 = Symbol::from(hash(&"mouse_2".to_string()));
        let sym_mouse_3 = Symbol::from(hash(&"mouse_3".to_string()));
        let sym_mouse_4 = Symbol::from(hash(&"mouse_4".to_string()));

        let pat_mouse_0 = Pattern::symbol(sym_mouse_0);
        let pat_mouse_1 = Pattern::symbol(sym_mouse_1);
        let pat_mouse_2 = Pattern::symbol(sym_mouse_2);
        let pat_mouse_3 = Pattern::symbol(sym_mouse_3);
        let pat_mouse_4 = Pattern::symbol(sym_mouse_4);

        let pat_any_mouse = pat_mouse_0 | pat_mouse_1 | pat_mouse_2 | pat_mouse_3 | pat_mouse_4;


        let m_sym = Symbol::from(hash(&m.to_string()));
        let m_sym_r = Symbol::from(hash(&format!("release {}",m)));
        let k_sym = Symbol::from(hash(&key.to_string()));
        let k_sym_r = Symbol::from(hash(&format!("release {}",key)));

        let pat_m = Pattern::symbol(m_sym);
        let pat_k = Pattern::symbol(k_sym);
        let pat_m_r = Pattern::symbol(m_sym_r);
        let pat_k_r = Pattern::symbol(k_sym_r);

        let s0 = self.nfa.start;
        let s1 = self.nfa.new_pattern(s0,pat_any_mouse.many());
        let s2 = self.nfa.new_pattern(s1,&pat_m);
        let s3 = self.nfa.new_pattern(s2,pat_any_mouse.many());
        let s4 = self.nfa.new_pattern(s3,pat_k);

        let s3_r = self.nfa.new_pattern(s3,&pat_k_r);
        let s4_r = self.nfa.new_pattern(s4,&pat_k_r);
        self.nfa.connect(s3_r,s2);
        self.nfa.connect(s4_r,s2);


        self.dfa = (&self.nfa).into();
    }
}


pub fn main() {


    let sym_mouse_0 = Symbol::from(hash(&"mouse_0".to_string()));
    let sym_mouse_1 = Symbol::from(hash(&"mouse_1".to_string()));
    let sym_mouse_2 = Symbol::from(hash(&"mouse_2".to_string()));
    let sym_mouse_3 = Symbol::from(hash(&"mouse_3".to_string()));
    let sym_mouse_4 = Symbol::from(hash(&"mouse_4".to_string()));
    let sym_ctrl = Symbol::from(hash(&"ctrl".to_string()));
    let sym_a = Symbol::from(hash(&"a".to_string()));
    let sym_a_r = Symbol::from(hash(&"release a".to_string()));
    let sym_b = Symbol::from(hash(&"b".to_string()));
    let sym_b_r = Symbol::from(hash(&"release b".to_string()));
    let sym_c = Symbol::from(hash(&"c".to_string()));
    let sym_x = Symbol::from(hash(&"o".to_string()));

    let pat_ctrl    = Pattern::symbol(sym_ctrl);
    let pat_a       = Pattern::symbol(sym_a);
    let pat_a_r     = Pattern::symbol(sym_a_r);
    let pat_b       = Pattern::symbol(sym_b);
    let pat_b_r     = Pattern::symbol(sym_b_r);
    let pat_mouse_0 = Pattern::symbol(sym_mouse_0);
    let pat_mouse_1 = Pattern::symbol(sym_mouse_1);
    let pat_mouse_2 = Pattern::symbol(sym_mouse_2);
    let pat_mouse_3 = Pattern::symbol(sym_mouse_3);
    let pat_mouse_4 = Pattern::symbol(sym_mouse_4);

    let pat_any_mouse = pat_mouse_0 | pat_mouse_1 | pat_mouse_2 | pat_mouse_3 | pat_mouse_4;



    // let mut nfa : Nfa = default();
    // // let end_a = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    // // let end_a2 = nfa.new_pattern(nfa.start,Pattern::symbol(sym_a));
    // // let end_b = nfa.new_pattern(end_a2,Pattern::symbol(sym_b));
    // // let end_c = nfa.new_pattern(end_b,Pattern::symbol(sym_c));
    // // let end_x = nfa.new_pattern(nfa.start,Pattern::symbol(sym_x));
    //
    // let s0 = nfa.start;
    // let s1 = nfa.new_pattern(s0,pat_any_mouse.many());
    // let s2 = nfa.new_pattern(s1,&pat_ctrl);
    // let s3 = nfa.new_pattern(s2,pat_any_mouse.many());
    // let s4 = nfa.new_pattern(s3,pat_a);
    //
    // let s3_r = nfa.new_pattern(s3,&pat_a_r);
    // let s4_r = nfa.new_pattern(s4,&pat_a_r);
    //
    // nfa.connect(s3_r,s2);
    // nfa.connect(s4_r,s2);
    //
    //
    //
    //
    // let s1 = nfa.new_pattern(s0,pat_any_mouse.many());
    // let s2 = nfa.new_pattern(s1,&pat_ctrl);
    // let s3 = nfa.new_pattern(s2,pat_any_mouse.many());
    // let s4 = nfa.new_pattern(s3,pat_b);
    //
    // let s3_r = nfa.new_pattern(s3,&pat_b_r);
    // let s4_r = nfa.new_pattern(s4,&pat_b_r);
    //
    // nfa.connect(s3_r,s2);
    // nfa.connect(s4_r,s2);


    let mut s = Shortcuts::new();
    s.add_kb_shortcut("ctrl","a");
    s.add_kb_shortcut("ctrl","x");
    s.add_kb_shortcut("ctrl","v");


    // let dfa = Dfa::from(&nfa);

    println!("---------");
    print_matrix(&s.dfa.links);



    let after_1 = s.dfa.next_state(Dfa::START_STATE,sym_ctrl);
    let after_2 = s.dfa.next_state(after_1,sym_mouse_1);
    let after_3 = s.dfa.next_state(after_2,sym_a);
    let after_4 = s.dfa.next_state(after_3,sym_a_r);
    let after_5 = s.dfa.next_state(after_4,sym_a);
    println!("{:?}",after_1);
    println!("{:?}",after_2);
    println!("{:?}",after_3);
    println!("{:?}",after_4);
    println!("{:?}",after_5);


}


// ctrl + a

// keyboard: ctrl + a ; mouse: lmb

// ctrl + a

// +cmd +a -a +c

