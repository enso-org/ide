use crate::prelude::*;
use wasm_bindgen::prelude::*;

use super::theme::Manager;

use crate::system::web;
use wasm_bindgen::prelude::Closure;
use js_sys;



// ===========================
// === JavaScript Bindings ===
// ===========================

mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        export function create_theme_object(refs,choose) {
            return {__refs:refs,choose}
        }
    ")]
    extern "C" {
        pub fn create_theme_object(choose:&Closure<dyn Fn(String)>) -> JsValue;
    }
}


pub fn expose_to_window(manager:&Manager) {
    let window = web::window();
    // let theme  = js_sys::Object::new();

    let manager = manager.clone_ref();
    let change_theme : Closure<dyn Fn(String)> = Closure::new(move |name:String| {
        manager.set_enabled(&[name])
    });

    let theme = js::create_theme_object(&change_theme);

    // TODO[WD]: There should be a better way than these memory leaks here after updating the
    // compiler. We might then
    mem::forget(change_theme);



    js_sys::Reflect::set(&window,&"theme".into(),&theme);


    // js_sys::Reflect::get()

}
