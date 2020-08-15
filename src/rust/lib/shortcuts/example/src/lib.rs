
use enso_prelude::*;
use wasm_bindgen::prelude::*;
use ensogl_system_web as web;
use enso_automata;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shortcuts() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();

    println!("hello2");
    enso_automata::main();
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