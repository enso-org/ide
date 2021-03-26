//! JavaScript bindings to the theme engine. They allow the inspection and modification of themes
//! directly from the JavaScript console.

use crate::prelude::*;
use wasm_bindgen::prelude::*;

use super::theme::Manager;

use crate::system::web;
use wasm_bindgen::prelude::Closure;
use js_sys;
use crate::data::color;



// ===========================
// === JavaScript Bindings ===
// ===========================

mod js {
    use super::*;
    #[wasm_bindgen(inline_js = "
        export function create_theme_manager_ref(choose,get) {
            return {choose,get}
        }

        export function create_theme_ref(set) {
            return {set}
        }
    ")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn create_theme_manager_ref(choose:&Closure<dyn Fn(String)>, get:&Closure<dyn Fn(String)->JsValue>) -> JsValue;

        #[allow(unsafe_code)]
        pub fn create_theme_ref(set:&Closure<dyn Fn(String)>) -> JsValue;
    }
}


// TODO[WD]
//     There is a better way than all memory leaks introduced by `mem::forget` after we update
//     wasm-bindgen. There is a function now `Closure::into_js_value` which passes its memory
//     management to JS GC. See https://github.com/enso-org/ide/issues/1028
pub fn expose_to_window(manager:&Manager) {
    let window = web::window();
    // let theme  = js_sys::Object::new();

    let owned_manager = manager.clone_ref();
    let choose : Closure<dyn Fn(String)> = Closure::new(move |name:String| {
        owned_manager.set_enabled(&[name])
    });

    let owned_manager = manager.clone_ref();
    let get : Closure<dyn Fn(String)->JsValue> = Closure::new(move |name:String| {
        let theme = owned_manager.get(&name).unwrap();

        let set : Closure<dyn Fn(String)> = Closure::new(move |name:String| {
            theme.set(name,color::Rgba::new(0.0,1.0,0.0,1.0))
        });

        let theme_ref = js::create_theme_ref(&set);
        mem::forget(set);
        theme_ref
    });

    let theme_manger_ref = js::create_theme_manager_ref(&choose,&get);

    mem::forget(choose);
    mem::forget(get);



    js_sys::Reflect::set(&window,&"theme".into(),&theme_manger_ref);


    // js_sys::Reflect::get()

}
