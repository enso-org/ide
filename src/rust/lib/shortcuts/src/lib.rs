#![feature(test)]
extern crate test;

use enso_prelude::*;
use wasm_bindgen::prelude::*;
use ensogl_system_web as web;
use enso_automata::*;

use enso_frp as frp;

use frp::io::keyboard2::Keyboard;
use frp::io::keyboard2 as keyboard;
use frp::io::mouse;

pub use logger;
pub use logger::*;
pub use logger::AnyLogger;
pub use logger::disabled::Logger;


pub fn hash<T:Hash>(t:&T) -> u64 {
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







fn reverse_key(key:&str) -> String {
    format!("-{}",key)
}

/// List of special keys. Special keys can be grouped together to distinguish action sequences like
/// `ctrl + a` and `a + ctrl`. Please note, that this is currently not happening.
const SIDE_KEYS : &'static [&'static str] = &["ctrl","alt","meta","cmd","shift"];


const DOUBLE_EVENT_TIME_MS : f32 = 500.0;




// ==================
// === ActionType ===
// ==================

/// The type of the action. Could be applied to keyboard, mouse, or any mix of input events.
#[derive(Clone,Copy,Debug,Eq,Hash,PartialEq)]
pub enum ActionType {
    Press, Release, DoublePress, DoubleClick
}
pub use ActionType::*;
use js_sys::Atomics::sub;




pub trait Registry<T> : Default {
    fn add        (&self, action_type: ActionType, expr: impl AsRef<str>, action: impl Into<T>);
    fn on_press   (&self, input:impl AsRef<str>) -> Vec<T>;
    fn on_release (&self, input:impl AsRef<str>) -> Vec<T>;
    fn optimize   (&self) {}
}



// =============================
// === AutomataRegistryModel ===
// =============================

#[derive(Debug)]
pub struct AutomataRegistryModel<T> {
    dirty         : bool,
    nfa           : Nfa,
    dfa           : Dfa,
    states        : HashMap<String,nfa::State>,
    connections   : HashSet<(nfa::State,nfa::State)>,
    always_state  : nfa::State,
    current       : dfa::State,
    pressed       : HashSet<String>,
    action_map    : HashMap<ActionType,HashMap<nfa::State,T>>,
    press_times   : HashMap<dfa::State,f32>,
    release_times : HashMap<dfa::State,f32>,
}


// === Getters ===

#[allow(missing_docs)]
impl<T> AutomataRegistryModel<T> {
    pub fn nfa(&self) -> &Nfa { &self.nfa }
    pub fn dfa(&self) -> &Dfa { &self.dfa }
}


// === API ===

impl<T> AutomataRegistryModel<T> {
    /// Constructor.
    pub fn new() -> Self {
        let mut nfa       = Nfa::default();
        let dfa           = Dfa::from(&nfa);
        let states        = default();
        let connections   = default();
        let always_state  = nfa.new_pattern(nfa.start,Pattern::any().many());
        let current       = Dfa::START_STATE;
        let pressed       = default();
        let dirty         = true;
        let action_map    = default();
        let press_times   = default();
        let release_times = default();
        Self {dirty,nfa,dfa,states,connections,always_state,current,pressed,action_map,press_times
            ,release_times}
    }
}

impl<T:Clone> AutomataRegistryModel<T> {
    pub fn add(&mut self, action_type:ActionType, expr:impl AsRef<str>, action:impl Into<T>) {
        self.dirty = true;

        let special_keys : HashSet<&str> = SIDE_KEYS.iter().map(|t|*t).collect();
        let expr = expr.as_ref();

        let end_state = if expr.starts_with('-') {
            let key = format!("-{}",expr[1..].trim().to_lowercase());
            let sym = Symbol::new_named(hash(&key),key);
            let pat = Pattern::symbol(&sym);
            self.nfa.new_pattern(self.always_state,pat)
        } else {
            let mut special = vec![];
            let mut normal  = vec![];

            for chunk in expr.split('+').map(|t| t.trim()) {
                // let map = if special_keys.contains(chunk) { &mut special } else { &mut normal };
                // map.push(vec![chunk.into()]);
                if special_keys.contains(chunk) {
                    special.push((chunk.into(),true))
                } else {
                    normal.push((chunk.into(),false))
                }
            }

            let mut all : Vec<(String,bool)> = special.clone();
            all.extend(normal);
            self.add_key_permutations(self.nfa.start, &all)
        };
        self.action_map.entry(action_type).or_default().insert(end_state,action.into());
    }

    /// Process the press input event. See `on_event` docs to learn more.
    pub fn on_press(&mut self, input:impl AsRef<str>) -> Vec<T> {
        self.on_event(input,true)
    }

    /// Process the release input event. See `on_event` docs to learn more.
    pub fn on_release(&mut self, input:impl AsRef<str>) -> Vec<T> {
        self.on_event(input,false)
    }

    /// Get the approximate memory consumption of this shortcut registry DFA network.
    pub fn approx_dfa_memory_consumption_mb(&mut self) -> f32 {
        self.optimize();
        let elems = self.dfa.links.matrix.len() as f32;
        let bytes = elems * 8.0;
        bytes / 1000000.0
    }
}


// #[derive(Clone,Debug,Eq,Ord,PartialEq,PartialOrd)]
// struct KeyCombination(Vec<String>)

// === Private API ===

impl<T:Clone> AutomataRegistryModel<T> {
    fn add_key_permutations(&mut self, source:nfa::State, keys:&[(String,bool)]) -> nfa::State {
        // println!("===========");
        // println!("{:?}",keys);

        let len = keys.len();
        let mut state = source;

        for perm in keys.iter().permutations(len) {
            // println!("\n\n=================");
            // println!("perm: {:?}",perm);
            state = source;
            let mut path : Vec<&str> = vec![];


            for alt_keys in perm {
                // println!("\n>> {:?}",alt_keys);
                let (name,alt) = alt_keys;

                if *alt {
                    let alt_path = path.iter().chain(&[&**name]).cloned().sorted().collect_vec();
                    let alt_repr = alt_path.join(" ");
                    let out = self.states.get(&alt_repr).cloned().unwrap_or_else(||self.nfa.new_state_exported());

                    let alts = vec![format!("{}-left",name),format!("{}-right",name)];
                    // println!("out: {:?}",out);
                    for key in alts {
                        let mut local_path = path.clone();
                        local_path.push(&key);
                        local_path.sort();
                        let repr = local_path.join(" ");
                        // println!("? '{}'",repr);
                        match self.states.get(&repr) {
                            Some(&target) => {
                                // println!("bidirectional connect {} [{:?} --- {:?}]", key.to_string(), state, target);
                                self.bidirectional_connect_via(state,target,key.to_string());
                                self.bidirectional_connect(target,out);
                                self.bidirectional_connect_via(state,out,name.to_string());

                            },
                            None => {
                                let target = self.bidirectional_pattern(state,key.to_string());
                                // println!("bidirectional pattern {} [{:?} <-> {:?}]", key.to_string(),state, target);
                                // println!("+ '{}' -> {:?}",repr,target);
                                self.states.insert(repr.clone(),target);
                                self.bidirectional_connect(target,out);
                                self.bidirectional_connect_via(state,out,name.to_string());

                            }
                        };
                    }
                    state = out;
                    path = alt_path;
                    // println!("+ '{}' -> {:?}",alt_repr,out);
                    self.states.insert(alt_repr.clone(),out);
                } else {
                    let key = name;
                    path.push(&key);
                    path.sort();
                    let repr = path.join(" ");
                    // println!("? '{}'",repr);
                    state = match self.states.get(&repr) {
                        Some(&target) => {
                            // println!("bidirectional connect {} [{:?} --- {:?}]", key.to_string(), state, target);
                            self.bidirectional_connect_via(state,target,key.to_string());
                            target
                        },
                        None => {
                            let target = self.bidirectional_pattern(state,key.to_string());
                            // println!("bidirectional pattern {} [{:?} <-> {:?}]", key.to_string(),state, target);
                            // println!("+ '{}' -> {:?}",repr,target);
                            self.states.insert(repr.clone(),target);
                            target
                        }
                    };
                }
            }
        }
        state
    }

    fn bidirectional_connect_via(&mut self, source:nfa::State, target:nfa::State, key:String) {
        if !self.connections.contains(&(source,target)) {
            self.connections.insert((source,target));
            self.connections.insert((target,source));
            let key_r = reverse_key(&key);
            let sym   = Symbol::new_named(hash(&key),key);
            let sym_r = Symbol::new_named(hash(&key_r),key_r);
            self.nfa.connect_via(source,target,&(sym.clone()..=sym));
            self.nfa.connect_via(target,source,&(sym_r.clone()..=sym_r));
        }
    }

    fn bidirectional_connect(&mut self, source:nfa::State, target:nfa::State) {
        if !self.connections.contains(&(source,target)) {
            self.connections.insert((source,target));
            self.connections.insert((target,source));
            self.nfa.connect(source,target);
            self.nfa.connect(target,source);
        }
    }

    // fn bidirectional_pattern(&mut self, source:nfa::State, target:nfa::State, key:String) {
    //     let key_r  = reverse_key(&key);
    //     let sym    = Symbol::new_named(hash(&key),key);
    //     let sym_r  = Symbol::new_named(hash(&key_r),key_r);
    //     let pat    = Pattern::symbol(&sym);
    //     self.nfa.new_pattern_to(source,target,pat);
    //     self.nfa.connect_via(target,source,&(sym_r.clone()..=sym_r));
    //     self.connections.insert((source,target));
    //     // target
    // }

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

    /// Process the `input` event. Events are strings uniquely identifying the source of the event,
    /// like "key_a", or "mouse_button_1". The `press` parameter is set to true if it was a press
    /// event, and is set to false in case of a release event.
    fn on_event(&mut self, input:impl AsRef<str>, press:bool) -> Vec<T> {
        //println!("on_event ({}) {}",press,input.as_ref());
        self.optimize();
        let action        = if press { Press }       else { Release };
        let double_action = if press { DoublePress } else { DoubleClick };
        let input         = input.as_ref().to_lowercase();
        let symbol_input  = if press { input.clone() } else { format!("-{}",input) };
        let symbol        = Symbol::from(hash(&symbol_input));
        let current_state = self.current;
        let next_state    = self.dfa.next_state(current_state,&symbol);
        let focus_state   = if press { next_state } else { current_state };
        let nfa_states    = &self.dfa.sources[focus_state.id()];
        let time : f32    = web::time_from_start() as f32;
        let last_time_map = if press { &self.press_times } else { &self.release_times };
        let last_time     = last_time_map.get(&focus_state);
        let time_diff     = last_time.map(|t| time-t);
        let is_double     = time_diff.map(|t| t < DOUBLE_EVENT_TIME_MS) == Some(true);
        let new_time      = if is_double { 0.0 } else { time };
        self.current      = next_state;
        let mut actions   = nfa_states.iter().filter_map(|t|self.get_action(action,*t)).collect_vec();
        if is_double {
            actions.extend(nfa_states.iter().filter_map(|t|self.get_action(double_action,*t)));
        }
        if press {
            self.pressed.insert(input);
            self.press_times.insert(focus_state,new_time);
        } else {
            self.pressed.remove(&input);
            self.release_times.insert(focus_state,new_time);
            if self.pressed.is_empty() {
                self.current = Dfa::START_STATE;
            }
            self.reset_to_known_state();
        }
        actions
    }

    fn reset_to_known_state(&mut self) {
        if self.current.is_invalid() {
            let path = self.pressed.iter().sorted().cloned().collect_vec();
            self.current = Dfa::START_STATE;
            for p in path {
                self.current = self.dfa.next_state(self.current, &Symbol::from(hash(&p)));
            }
        }
    }

    fn get_action(&self, action_type:ActionType, state:nfa::State) -> Option<T> {
        self.action_map.get(&action_type).and_then(|m|m.get(&state).cloned())
    }

    fn optimize(&mut self) {
        if self.dirty {
            self.dirty   = false;
            self.dfa     = (&self.nfa).into();
            self.pressed = default();
        }
    }
}

impl<T> Default for AutomataRegistryModel<T> {
    fn default() -> Self {
        Self::new()
    }
}



// ========================
// === AutomataRegistry ===
// ========================

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct AutomataRegistry<T> {
    rc : Rc<RefCell<AutomataRegistryModel<T>>>
}

impl<T> AutomataRegistry<T> {
    /// Constructor.
    pub fn new() -> Self {
        default()
    }
}

impl<T:Clone> AutomataRegistry<T> {
    pub fn nfa_as_graphviz_code(&self) -> String {
        self.rc.borrow().nfa.as_graphviz_code()
    }

    pub fn dfa_as_graphviz_code(&self) -> String {
        self.rc.borrow().dfa.as_graphviz_code()
    }

    /// Get the approximate memory consumption of this shortcut registry DFA network.
    pub fn approx_dfa_memory_consumption_mb(&mut self) -> f32 {
        self.rc.borrow_mut().approx_dfa_memory_consumption_mb()
    }
}

impl<T:Clone> Registry<T> for AutomataRegistry<T> {
    fn add(&self, action_type:ActionType, expr:impl AsRef<str>, action:impl Into<T>) {
        self.rc.borrow_mut().add(action_type,expr,action)
    }

    /// Process the press input event. See `on_event` docs to learn more.
    fn on_press(&self, input:impl AsRef<str>) -> Vec<T> {
        self.rc.borrow_mut().on_press(input)
    }

    /// Process the release input event. See `on_event` docs to learn more.
    fn on_release(&self, input:impl AsRef<str>) -> Vec<T> {
        self.rc.borrow_mut().on_release(input)
    }

    fn optimize(&self) {
        self.rc.borrow_mut().optimize();
    }
}



// ============================
// === HashSetRegistryModel ===
// ============================

#[derive(Debug)]
pub struct HashSetRegistryModel<T> {
    actions       : HashMap<ActionType,HashMap<String,T>>,
    pressed       : HashSet<String>,
    press_times   : HashMap<String,f32>,
    release_times : HashMap<String,f32>,
    side_keys     : HashMap<String,Vec<String>>
}

impl<T> HashSetRegistryModel<T> {
    pub fn new() -> Self {
        let actions       = default();
        let pressed       = default();
        let press_times   = default();
        let release_times = default();
        let side_keys     = default();
        Self {actions,pressed,press_times,release_times,side_keys} . init()
    }

    fn init(mut self) -> Self {
        for key in SIDE_KEYS {
            let alts = vec![format!("{}-left",key),format!("{}-right",key),key.to_string()];
            self.side_keys.insert(key.to_string(),alts);
        }
        self
    }

    fn current_expr(&self) -> String {
        self.pressed.iter().sorted().join(" + ")
    }
}

impl<T:Clone> HashSetRegistryModel<T> {
    pub fn add(&mut self, action_type:ActionType, input:impl AsRef<str>, action:impl Into<T>) {
        let input  = input.as_ref();
        let action = action.into();
        let exprs  = self.possible_exprs(input);
        let map    = self.actions.entry(action_type).or_default();
        for expr in exprs {
            map.insert(expr, action.clone());
        }
        // self.actions.insert(expr,action.into());
    }

    fn on_event(&mut self, input:impl AsRef<str>, press:bool) -> Vec<T> {
        let expr = if press {
            self.pressed.insert(input.as_ref().to_string());
            self.current_expr()
        } else {
            let out = self.current_expr();
            self.pressed.remove(input.as_ref());
            out
        };
        let action        = if press { Press }       else { Release };
        let double_action = if press { DoublePress } else { DoubleClick };
        let last_time_map = if press { &mut self.press_times } else { &mut self.release_times };
        let mut out       = Vec::<T>::new();
        let time : f32    = web::time_from_start() as f32;
        let last_time     = last_time_map.get(&expr);
        let time_diff     = last_time.map(|t| time-t);
        let is_double     = time_diff.map(|t| t < DOUBLE_EVENT_TIME_MS) == Some(true);
        out.extend(self.actions.get(&action).and_then(|t|t.get(&expr)).cloned());
        if is_double {
            out.extend(self.actions.get(&double_action).and_then(|t|t.get(&expr)).cloned());
            last_time_map.remove(&expr);
        } else {
            *last_time_map.entry(expr).or_default() = time;
        }
        out
    }

    pub fn on_press(&mut self, input:impl AsRef<str>) -> Vec<T> {
        self.on_event(input,true)
    }

    pub fn on_release(&mut self, input:impl AsRef<str>) -> Vec<T> {
        self.on_event(input,false)
    }

    fn possible_exprs(&self, expr:impl AsRef<str>) -> Vec<String> {
        let expr = expr.as_ref();
        let mut out : Vec<String> = vec![];
        for key in expr.split('+').map(|t| t.trim()).sorted() {
            match self.side_keys.get(key) {
                Some(alts) => {
                    if out.is_empty() {
                        out.extend(alts.iter().cloned());
                    } else {
                        let local_out = mem::take(&mut out);
                        for k in alts {
                            out.extend(local_out.iter().map(|t| format!("{} + {}", t, k)));
                        }
                    }
                },
                None => {
                    if out.is_empty() {
                        out.push(key.into());
                    } else {
                        for el in out.iter_mut() {
                            *el = format!("{} + {}", el, key);
                        }
                    }
                }
            }
        }
        out
    }
}

impl<T> Default for HashSetRegistryModel<T> {
    fn default() -> Self {
        Self::new()
    }
}





// =======================
// === HashSetRegistry ===
// =======================

#[derive(CloneRef,Debug,Derivative)]
#[derivative(Clone(bound=""))]
#[derivative(Default(bound=""))]
pub struct HashSetRegistry<T> {
    rc : Rc<RefCell<HashSetRegistryModel<T>>>
}

impl<T> HashSetRegistry<T> {
    pub fn new() -> Self {
        default()
    }
}

impl<T:Clone> Registry<T> for HashSetRegistry<T> {
    fn add(&self, action_type:ActionType, expr:impl AsRef<str>, action:impl Into<T>) {
        self.rc.borrow_mut().add(action_type,expr,action)
    }

    fn on_press(&self, input:impl AsRef<str>) -> Vec<T> {
        self.rc.borrow_mut().on_press(input)
    }

    fn on_release(&self, input:impl AsRef<str>) -> Vec<T> {
        self.rc.borrow_mut().on_release(input)
    }
}



// =============
// === Tests ===
// =============

#[cfg(test)]
mod tests {
    use super::*;

    // === Press ===

    #[test] fn automata_registry_press() { press::<AutomataRegistry<i32>>(); }
    #[test] fn hash_set_registry_press() { press::<HashSetRegistry<i32>>(); }
    fn press<T:Registry<i32>>() -> T {
        let nothing = Vec::<i32>::new();
        let mut registry : T = default();
        registry.add(Press, "ctrl + a", 0);
        assert_eq!(registry.on_press("ctrl-left"), nothing);
        for _ in 0..10 {
            assert_eq!(registry.on_press("a"), vec![0]);
            assert_eq!(registry.on_release("a"), nothing);
        }
        registry
    }


    // === Release ===

    #[test] fn automata_registry_release() { release::<AutomataRegistry<i32>>(); }
    #[test] fn hash_set_registry_release() { release::<HashSetRegistry<i32>>(); }
    fn release<T:Registry<i32>>() -> T {
        let nothing = Vec::<i32>::new();
        let mut registry : T = default();
        registry.add(Release, "ctrl + a", 0);
        assert_eq!(registry.on_press("ctrl-left"),nothing);
        for _ in 0..10 {
            assert_eq!(registry.on_press("a"), nothing);
            assert_eq!(registry.on_release("a"), vec![0]);
        }
        registry
    }


    // === DoublePress ===

    // #[test] fn automata_registry_double_press() { double_press::<AutomataRegistry<i32>>(); }
    #[test] fn hash_set_registry_double_press() { double_press::<HashSetRegistry<i32>>(); }
    fn double_press<T:Registry<i32>>() -> T {
        let nothing = Vec::<i32>::new();
        let mut registry : T = default();
        registry.add(DoublePress, "ctrl + a", 0);
        assert_eq!(registry.on_press("ctrl-left"),nothing);
        for _ in 0..10 {
            assert_eq!(registry.on_press("a"), nothing);
            assert_eq!(registry.on_release("a"), nothing);
            web::simulate_sleep(100.0);
            assert_eq!(registry.on_press("a"), vec![0]);
            assert_eq!(registry.on_release("a"), nothing);
            web::simulate_sleep(100.0);
            assert_eq!(registry.on_press("a"), nothing);
            assert_eq!(registry.on_release("a"), nothing);
            web::simulate_sleep(1000.0);
        }
        registry
    }


    #[test]
    fn automata_registry_side_keys_handling() {
        side_keys_handling::<AutomataRegistry<i32>>();
    }

    #[test]
    fn hash_set_registry_side_keys_handling() {
        side_keys_handling::<HashSetRegistry<i32>>();
    }

    fn side_keys_handling<T:Registry<i32>>() -> T {
        let nothing = Vec::<i32>::new();
        let mut registry : T = default();
        registry.add(Press, "ctrl + meta + a", 0);
        for ctrl in &["ctrl","ctrl-left","ctrl-right"] {
            for meta in &["meta","meta-left","meta-right"] {
                assert_eq!(registry.on_press(ctrl),nothing);
                assert_eq!(registry.on_press(meta),nothing);
                assert_eq!(registry.on_press("a"),vec![0]);
                assert_eq!(registry.on_release("a"),nothing);
                assert_eq!(registry.on_release(meta),nothing);
                assert_eq!(registry.on_release(ctrl),nothing);

                assert_eq!(registry.on_press(meta),nothing);
                assert_eq!(registry.on_press(ctrl),nothing);
                assert_eq!(registry.on_press("a"),vec![0]);
                assert_eq!(registry.on_release("a"),nothing);
                assert_eq!(registry.on_release(ctrl),nothing);
                assert_eq!(registry.on_release(meta),nothing);
            }
        }
        registry
    }

    #[test]
    fn automata_registry_validate_sates() {
        validate_states::<AutomataRegistry<i32>>(true);
    }

    #[test]
    fn hash_set_registry_validate_sates() {
        validate_states::<HashSetRegistry<i32>>(false);
    }

    fn validate_states<T:Registry<i32>>(allow_broken_shortcut:bool) -> T {
        let nothing = Vec::<i32>::new();
        let mut registry : T = default();
        registry.add(Press, "ctrl + meta + a", 0);
        registry.add(Press, "ctrl + meta + b", 1);
        // First shortcut.
        assert_eq!(registry.on_press("meta-left"),nothing);
        assert_eq!(registry.on_press("ctrl-left"),nothing);
        assert_eq!(registry.on_press("a"),vec![0]);
        assert_eq!(registry.on_release("a"),nothing);
        assert_eq!(registry.on_press("a"),vec![0]);
        assert_eq!(registry.on_release("a"),nothing);
        // Second shortcut.
        assert_eq!(registry.on_press("b"),vec![1]);
        assert_eq!(registry.on_release("b"),nothing);
        // Incorrect sequence.
        assert_eq!(registry.on_press("meta-right"),nothing);
        assert_eq!(registry.on_release("meta-right"),nothing);
        if allow_broken_shortcut {
            // Broken shortcut after incorrect sequence.
            assert_eq!(registry.on_press("b"), nothing);
            assert_eq!(registry.on_release("b"), nothing);
        } else {
            assert_eq!(registry.on_press("b"), vec![1]);
            assert_eq!(registry.on_release("b"), nothing);
        }
        // Restoring shortcuts on release all keys.
        assert_eq!(registry.on_release("meta-left"),nothing);
        assert_eq!(registry.on_release("ctrl-left"),nothing);
        // Testing recovered first shortcut again.
        assert_eq!(registry.on_press("meta-left"),nothing);
        assert_eq!(registry.on_press("ctrl-left"),nothing);
        assert_eq!(registry.on_press("a"),vec![0]);
        registry
    }
}



// ==================
// === Benchmarks ===
// ==================

#[cfg(test)]
mod benchmarks {
    use super::*;
    use test::Bencher;

    const CONS_SIMPLE  : &'static str = "ctrl";
    const CONS_COMPLEX : &'static str = "ctrl + cmd + alt + shift";

    // === Construction ===

    #[bench]
    fn automata_registry_construction_simple_without_optimization(bencher:&mut Bencher) {
        construction::<AutomataRegistry<i32>>(CONS_SIMPLE,false,bencher);
    }

    #[bench]
    fn automata_registry_construction_complex_without_optimization(bencher:&mut Bencher) {
        construction::<AutomataRegistry<i32>>(CONS_COMPLEX,false,bencher);
    }

    #[bench]
    fn automata_registry_construction_simple_with_optimization(bencher:&mut Bencher) {
        construction::<AutomataRegistry<i32>>(CONS_SIMPLE,true,bencher);
    }

    #[bench]
    fn automata_registry_construction_complex_with_optimization(bencher:&mut Bencher) {
        construction::<AutomataRegistry<i32>>(CONS_COMPLEX,true,bencher);
    }

    #[bench]
    fn hashset_registry_construction_simple(bencher:&mut Bencher) {
        construction::<HashSetRegistry<i32>>(CONS_SIMPLE,true,bencher);
    }

    #[bench]
    fn hashset_registry_construction_complex(bencher:&mut Bencher) {
        construction::<HashSetRegistry<i32>>(CONS_COMPLEX,true,bencher);
    }

    fn construction<T:Registry<i32>>(input:&str, optimize:bool, bencher:&mut Bencher) -> T {
        bencher.iter(|| {
            let mut registry : T = default();
            let max_count        = test::black_box(10);
            for i in 0..max_count {
                registry.add(Press,format!("{} + a{}",i,input),i);
            }
            if optimize {
                registry.optimize();
            }
        });
        default()
    }


    // === Lookup ===

    #[bench]
    fn automata_registry_lookup(bencher:&mut Bencher) {
        lookup::<AutomataRegistry<i32>>(bencher);
    }

    #[bench]
    fn hashset_registry_lookup(bencher:&mut Bencher) {
        lookup::<HashSetRegistry<i32>>(bencher);
    }

    fn lookup<T:Registry<i32>>(bencher:&mut Bencher) -> T {
        let mut registry : T = default();
        let nothing          = Vec::<i32>::new();
        let max_count        = test::black_box(100);
        for i in 0..max_count {
            registry.add(Press, format!("ctrl + shift + a{}",i), i);
        }
        registry.optimize();
        bencher.iter(|| {
            for i in 0..max_count {
                let key = format!("a{}",i);
                assert_eq!(registry.on_press("ctrl-left"),nothing);
                assert_eq!(registry.on_press("shift-left"),nothing);
                assert_eq!(registry.on_press(&key),vec![i]);
                assert_eq!(registry.on_release(&key),nothing);
                assert_eq!(registry.on_release("shift-left"),nothing);
                assert_eq!(registry.on_release("ctrl-left"),nothing);
            }
        });
        registry
    }
}
