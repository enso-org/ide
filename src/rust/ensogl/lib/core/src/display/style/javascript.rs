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
            let interactive_mode = () => {
                let element = document.getElementById('colors-debug')
                let observer = new MutationObserver(() => {
                    let vars = element.style.cssText.split(';').map((t) => t.trim().slice(2).replaceAll('-','.').split(':'))
                    vars.pop()
                    let changes = []
                    for(let [key,val] of vars) {
                        let num = parseFloat(val)
                        if(isNaN(num)) {
                            let colorMatch = val.split('(')[1].split(')')[0].split(' ')
                            let normColor  = `rgb(${colorMatch[0]/255},${colorMatch[1]/255},${colorMatch[2]/255})`
                            changes.push([key,normColor])
                        } else {
                            changes.push([key,`${num}`])
                        }
                    }
                    for(let [key,val] of changes) {
                        set(key,val)
                    }
                })
                observer.observe(element,{attributes:true})
            }
            return {set,interactive_mode}
        }
    ")]
    extern "C" {
        #[allow(unsafe_code)]
        pub fn create_theme_manager_ref(choose:&Choose, get:&Get) -> JsValue;

        #[allow(unsafe_code)]
        pub fn create_theme_ref(set:&Set) -> JsValue;
    }

    pub type Choose = Closure<dyn Fn(String)>;
    pub type Get    = Closure<dyn Fn(String)->JsValue>;
    pub type Set    = Closure<dyn Fn(String,String)>;
}


// TODO[WD]
//     There is a better way than all memory leaks introduced by `mem::forget` after we update
//     wasm-bindgen. There is a function now `Closure::into_js_value` which passes its memory
//     management to JS GC. See https://github.com/enso-org/ide/issues/1028
/// Expose the `window.theme` variable which can be used to inspect and change the theme directly
/// from the JavaScript console.
pub fn expose_to_window(manager:&Manager) {
    let window = web::window();

    let owned_manager = manager.clone_ref();
    let choose : js::Choose = Closure::new(move |name| owned_manager.set_enabled(&[name]));

    let owned_manager = manager.clone_ref();
    let get : js::Get = Closure::new(move |name:String| {
        let theme         = owned_manager.get(&name).unwrap();
        let set : js::Set = Closure::new(move |name,value| theme.set(name,value));
        let theme_ref     = js::create_theme_ref(&set);
        mem::forget(set);
        theme_ref
    });

    let theme_manger_ref = js::create_theme_manager_ref(&choose,&get);

    mem::forget(choose);
    mem::forget(get);

    js_sys::Reflect::set(&window,&"theme".into(),&theme_manger_ref);
}
