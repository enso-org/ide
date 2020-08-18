
use enso_prelude::*;
use wasm_bindgen::prelude::*;
use ensogl_system_web as web;
use enso_automata::*;

use enso_frp as frp;

use frp::io::keyboard2::Keyboard;
use frp::io::keyboard2 as keyboard;

pub use logger;
pub use logger::*;
pub use logger::AnyLogger;
pub use logger::disabled::Logger;


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



#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shortcuts() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();

    main();

}



fn reverse_key(key:&str) -> String {
    format!("-{}",key)
}


#[derive(Debug)]
pub struct Shortcuts {
    dirty   : bool,
    nfa     : Nfa,
    dfa     : Dfa,
    states  : HashMap<String,nfa::State>,
    connections : HashSet<(nfa::State,nfa::State)>,
    always : nfa::State,

    current : dfa::State,
    pressed : HashSet<String>,
}

impl Shortcuts {
    pub fn new() -> Self {
        let mut nfa    = Nfa::default();
        let dfa    = Dfa::from(&nfa);
        let states = default();
        let connections = default();
        let always = nfa.new_pattern(nfa.start,Pattern::any().many());

        let current = Dfa::START_STATE;
        let pressed = default();
        let dirty  = true;

        Self {dirty,nfa,dfa,states,connections,always,current,pressed}
    }

    pub fn add(&mut self, expr:impl AsRef<str>) {
        self.dirty = true;

        let special_keys : HashSet<&str> = (&["ctrl","alt","meta","cmd","shift"]).iter().map(|t|*t).collect();
        let expr = expr.as_ref();

        if expr.starts_with('-') {
            let key = format!("-{}",expr[1..].trim().to_lowercase());
            let sym = Symbol::new_named(hash(&key),key);
            let pat = Pattern::symbol(&sym);
            self.nfa.new_pattern(self.always,pat);

        } else {
            let mut special = vec![];
            let mut normal = vec![];

            for chunk in expr.split('+').map(|t| t.trim()) {
                if special_keys.contains(chunk) {
                    special.push(chunk)
                } else {
                    normal.push(chunk)
                }
            }
            println!("{:?}", special);
            println!("{:?}", normal);

            let mut all = special.clone();
            all.extend(&normal);
            let out = self.add_key_permutations(self.nfa.start, &all);
        }
    }

    pub fn add_key_permutations(&mut self, source:nfa::State, keys:&[&str]) -> nfa::State {
        self.dirty = true;

        let len = keys.len();
        let mut state = source;

        for perm in keys.iter().permutations(len) {
            state = source;
            let mut path  = vec![];
            let mut repr  = String::new();
            for key in perm {
                path.push(*key);
                path.sort();
                repr = path.join(" ");
                state = match self.states.get(&repr) {
                    Some(&target) => {
                        self.bidirectional_connect(state,target,key.to_string());
                        target
                    },
                    None => {
                        let out = self.bidirectional_pattern(state,key.to_string());
                        self.states.insert(repr.clone(),out);
                        out
                    }
                }

            }
        }
        state
    }

    fn bidirectional_connect(&mut self, source:nfa::State, target:nfa::State, key:String) {
        if !self.connections.contains(&(source,target)) {
            self.connections.insert((source,target));
            let key_r = reverse_key(&key);
            let sym   = Symbol::new_named(hash(&key),key);
            let sym_r = Symbol::new_named(hash(&key_r),key_r);
            self.nfa.connect_via(source,target,&(sym.clone()..=sym));
            self.nfa.connect_via(target,source,&(sym_r.clone()..=sym_r));
        }
    }

    fn bidirectional_pattern(&mut self, source:nfa::State, key:String) -> nfa::State {
        let key_r  = reverse_key(&key);
        let sym    = Symbol::new_named(hash(&key),key);
        let sym_r  = Symbol::new_named(hash(&key_r),key_r);
        let pat    = Pattern::symbol(&sym);
        let target = self.nfa.new_pattern(source,pat);
        self.nfa.connect_via(target,source,&(sym_r.clone()..=sym_r));
        self.connections.insert((source,target));
        target
    }

    pub fn on_press(&mut self, input:&str) {
        self.recompute();
        let input = input.to_lowercase();
        let sym = Symbol::from(hash(&input));
        let next = self.dfa.next_state(self.current,&sym);
        let next_id = next.id();
        let sfx = if next_id >= self.dfa.sources.len() { "".into() } else { format!("{:?}",self.dfa.sources[next_id]) };
        println!("on {} -> {:?} ({})",input,next,sfx);
        self.current = next;
        self.pressed.insert(input);
    }

    pub fn on_release(&mut self, input:&str) {
        self.recompute();
        let input = input.to_lowercase();
        let repr = format!("-{}",input);
        let sym = Symbol::from(hash(&repr.to_string()));
        let next = self.dfa.next_state(self.current,&sym);
        let next_id = next.id();
        let sfx = if next_id >= self.dfa.sources.len() { "".into() } else { format!("{:?}",self.dfa.sources[next_id]) };
        println!("on {} -> {:?} ({})",repr,next,sfx);
        self.current = next;
        self.pressed.remove(&input);
        if self.pressed.is_empty() {
            self.current = Dfa::START_STATE;
        }
        self.reset_to_known_state();
    }

    pub fn reset_to_known_state(&mut self) {
        if self.current.is_invalid() {
            let path = self.pressed.iter().sorted().cloned().collect_vec();
            self.current = Dfa::START_STATE;
            for p in path {
                self.current = self.dfa.next_state(self.current, &Symbol::from(hash(&p)));
            }
        }
    }

    pub fn recompute(&mut self) {
        if self.dirty {
            self.dirty   = false;
            self.dfa     = (&self.nfa).into();
            self.pressed = default();
        }
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

    let pat_ctrl    = Pattern::symbol(&sym_ctrl);
    let pat_a       = Pattern::symbol(&sym_a);
    let pat_a_r     = Pattern::symbol(&sym_a_r);
    let pat_b       = Pattern::symbol(&sym_b);
    let pat_b_r     = Pattern::symbol(&sym_b_r);
    let pat_mouse_0 = Pattern::symbol(&sym_mouse_0);
    let pat_mouse_1 = Pattern::symbol(&sym_mouse_1);
    let pat_mouse_2 = Pattern::symbol(&sym_mouse_2);
    let pat_mouse_3 = Pattern::symbol(&sym_mouse_3);
    let pat_mouse_4 = Pattern::symbol(&sym_mouse_4);

    let pat_any_mouse = pat_mouse_0 | pat_mouse_1 | pat_mouse_2 | pat_mouse_3 | pat_mouse_4;


    let mut s = Shortcuts::new();

    s.add("meta + ctrl + alt + space + a");
    s.add("- c");


    println!("---------");
    s.recompute();
    let elems = s.dfa.links.matrix.len() as f32;
    let bytes = elems * 8.0;
    let mb = bytes / 1000000.0;
    println!("!!! {}",mb);
    // print_matrix(&s.dfa.links);


    // let mut nfa = Nfa::new();
    // let p = (Pattern::any()).many();
    // nfa.new_pattern(nfa.start,p);
    // // nfa.new_pattern(nfa.start,Pattern::symbol(&Symbol::from(hash(&"p".to_string()))));
    // let dfa = Dfa::from(&nfa);
    //
    // println!("??? {:?}", dfa.next_state(Dfa::START_STATE,&Symbol::from(hash(&"p".to_string()))));
    //
    // println!("{}",nfa.visualize());
    // print_matrix(&dfa.links);



    println!("{}",s.nfa.visualize());

    println!("---------");

    println!("{}",s.dfa.visualize());

    let ss = Rc::new(RefCell::new(s));

    let logger = Logger::new("kb");
    let kb = Keyboard::new();
    let bindings = keyboard::DomBindings::new(&logger,&kb);

    let ss2 = ss.clone_ref();
    let ss3 = ss.clone_ref();
    frp::new_network! { network
        foo <- kb.down.map(move |t| ss2.borrow_mut().on_press(t.simple_name()));
        foo <- kb.up.map(move |t| ss3.borrow_mut().on_release(t.simple_name()));
    }
    mem::forget(network);
    mem::forget(bindings);
    mem::forget(ss);


}




// - shift left left
//
// any_key (?mouse)
//
// left (?mouse)
//
// key? left_down
//
// ctr -> b -> release ctrl
//
// any letter WITHOUT modifiers (typing but not cmd+a)


// +lmb           - start selection
// -lmb (ANY key) - stop selection