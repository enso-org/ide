use enso_prelude::*;
use wasm_bindgen::prelude::*;

use ensogl_system_web as web;

use enso_shortcuts as shortcuts;
use enso_shortcuts::Registry;

use enso_frp as frp;

use frp::io::keyboard;
use frp::io::keyboard::Keyboard;

use enso_logger::AnyLogger;
use enso_logger::WarningLogger as Logger;

#[wasm_bindgen]
#[allow(dead_code)]
pub fn entry_point_shortcuts() {
    web::forward_panic_hook_to_console();
    web::set_stack_trace_limit();
    main();
}

pub fn main() {
    let shortcut_registry = shortcuts::AutomataRegistry::<String>::new();
    shortcut_registry.add(shortcuts::Press, "ctrl + a", "hello");
    shortcut_registry.add(shortcuts::Press, "ctrl + b", "hello");

    DEBUG!(shortcut_registry.nfa_as_graphviz_code());

    let logger: Logger = Logger::new("kb");
    let kb = Keyboard::new();
    let bindings = keyboard::DomBindings::new(&logger, &kb, &default());

    frp::new_network! { network
        eval kb.down ((t)shortcut_registry.on_press(t.simple_name()));
        eval kb.up   ((t)shortcut_registry.on_release(t.simple_name()));
    }
    mem::forget(network);
    mem::forget(bindings);
    mem::forget(shortcut_registry);

    let shortcut_registry = shortcuts::HashSetRegistry::<String>::new();
    shortcut_registry.add(shortcuts::Press, "ctrl a", "press ctrl a");
    shortcut_registry.add(shortcuts::Release, "ctrl a", "release ctrl a");
    shortcut_registry.add(shortcuts::Press, "a", "press a");
    shortcut_registry.add(shortcuts::Release, "a", "release a");
    DEBUG!("\n---------------");
    DEBUG!("-> " shortcut_registry.on_press("ctrl-left");?);
    DEBUG!("---");
    DEBUG!("-> " shortcut_registry.on_press("a");?);
    DEBUG!("---");
    web::simulate_sleep(1000.0);
    DEBUG!("-> " shortcut_registry.on_release("ctrl-left");?);
    DEBUG!("---");
    DEBUG!("-> " shortcut_registry.on_release("a");?);
}
