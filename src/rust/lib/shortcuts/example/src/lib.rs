use enso_prelude::*;
use wasm_bindgen::prelude::*;

use ensogl_system_web as web;

use enso_shortcuts as shortcuts;
use enso_shortcuts::Registry;

use enso_frp as frp;

use frp::io::keyboard::Keyboard;
use frp::io::keyboard as keyboard;
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
    let mut shortcut_registry = shortcuts::AutomataRegistry::<String>::new();
    shortcut_registry.add(shortcuts::Press, "ctrl + a", "hello");
    shortcut_registry.add(shortcuts::Press, "ctrl + b", "hello");
    // shortcut_registry.add(shortcuts::Press, "ctrl + shift + meta + b", "hello");
    // shortcut_registry.add(shortcuts::Press, "ctrl + shift + meta + c", "hello");
    // shortcut_registry.add(shortcuts::Press, "ctrl + shift + meta + d", "hello");
    // shortcut_registry.add(shortcuts::Press, "ctrl + b", "hello2");
    // shortcut_registry.add(shortcuts::Press, "ctrl + meta + shift + b", "hello2");

    println!("{}",shortcut_registry.nfa_as_graphviz_code());

    let logger   = Logger::new("kb");
    let kb       = Keyboard::new();
    let bindings = keyboard::DomBindings::new(&logger,&kb);

    let shortcut_registry2 = shortcut_registry.clone_ref();
    let shortcut_registry3 = shortcut_registry.clone_ref();
    frp::new_network! { network
        on_down <- kb.down.map (move |t| shortcut_registry2.on_press(t.simple_name()));
        on_up   <- kb.up.map   (move |t| shortcut_registry3.on_release(t.simple_name()));
        trace on_down;
        trace on_up;
    }
    mem::forget(network);
    mem::forget(bindings);
    mem::forget(shortcut_registry);


    let mut shortcut_registry = shortcuts::HashSetRegistry::<String>::new();
    shortcut_registry.add(shortcuts::DoublePress, "ctrl + a", "hello");
    println!("---");
    println!("-> {:?}", shortcut_registry.on_press("ctrl"));
    println!("---");
    println!("-> {:?}", shortcut_registry.on_press("a"));
    println!("---");
    web::simulate_sleep(100.0);
    println!("-> {:?}", shortcut_registry.on_release("a"));
    println!("---");
    println!("-> {:?}", shortcut_registry.on_press("a"));
    println!("---");
    println!("-> {:?}", shortcut_registry.on_release("a"));
    println!("---");
    println!("-> {:?}", shortcut_registry.on_press("a"));
    println!("---");


}
