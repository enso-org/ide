use enso_prelude::*;
use wasm_bindgen::prelude::*;

use ensogl_system_web as web;

use enso_shortcuts as shortcuts;

use enso_frp as frp;

use frp::io::keyboard2::Keyboard;
use frp::io::keyboard2 as keyboard;
use frp::io::mouse;

use logger;
use logger::*;
use logger::AnyLogger;
use logger::disabled::Logger;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shortcuts() {
    web::forward_panic_hook_to_console();
    web::set_stdout();
    web::set_stack_trace_limit();

    main();
}

pub fn main() {

    let mut shortcut_registry = shortcuts::Registry::new();

    shortcut_registry.add(shortcuts::DoublePress, "meta + a", "hello");
    // s.add("meta + b");

    // s.add("meta + ctrl + alt + space + a1");
    // s.add("meta + ctrl + alt + space + a2");

    // s.add("- c");


    println!("---------");
    // s.recompute_on_dirty();
    // let elems = s.dfa.links.matrix.len() as f32;
    // let bytes = elems * 8.0;
    // let mb = bytes / 1000000.0;
    // println!("!!!o {}",mb);
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



    println!("{}",shortcut_registry.nfa_as_graphviz_code());

    println!("---------");

    // println!("{}",s.dfa.visualize());


    let logger = Logger::new("kb");
    let kb = Keyboard::new();
    let bindings = keyboard::DomBindings::new(&logger,&kb);

    let shortcut_registry2 = shortcut_registry.clone_ref();
    let shortcut_registry3 = shortcut_registry.clone_ref();
    frp::new_network! { network
        foo <- kb.down.map(move |t| shortcut_registry2.on_press(t.simple_name()));
        trace foo;
        foo <- kb.up.map(move |t| shortcut_registry3.on_release(t.simple_name()));
        trace foo;
    }
    mem::forget(network);
    mem::forget(bindings);
    mem::forget(shortcut_registry);
}
