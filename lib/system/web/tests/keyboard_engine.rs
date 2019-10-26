use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::*;
use wasm_bindgen_test::*;
use web_sys::{ KeyEvent, KeyboardEvent, KeyboardEventInit};

use basegl_system_web::*;
use basegl_system_web::keyboard_engine::KeyboardEngine;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn capture_keypress() {
    let elem = document().unwrap().create_element("canvas").unwrap();
    let keyboard_engine = KeyboardEngine::new(&elem);
    let key_a = KeyEvent::DOM_VK_A;

    let promise = Promise::new(&mut |resolve, _| {
        let handle = keyboard_engine.capture(vec![key_a], Box::new(move || {
            web_sys::console::log_1(&"Key has been captured".into());
            resolve.call0(&JsValue::NULL).ok();
        }));
        std::mem::forget(handle);
    });

    let mut event_init = KeyboardEventInit::new();
    event_init.key_code(KeyEvent::DOM_VK_A);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap();
    elem.dispatch_event(&event).unwrap();
    JsFuture::from(promise).await.unwrap();
}

#[wasm_bindgen_test]
async fn capture_keypress_on_tag_name() {
    let document = document().unwrap();
    let elem = document.create_element("canvas").unwrap();
    document.first_child().unwrap().append_child(&elem).expect("ok");

    let keyboard_engine = KeyboardEngine::from_tag_name("canvas").unwrap();
    let key_b = KeyEvent::DOM_VK_B;
    let promise = Promise::new(&mut |resolve, _| {
        let handle = keyboard_engine.capture(vec![key_b], Box::new(move || {
            web_sys::console::log_1(&"Key has been captured".into());
            resolve.call0(&JsValue::NULL).ok();
        }));
        std::mem::forget(handle);
    });
    let mut event_init = KeyboardEventInit::new();
    event_init.key_code(KeyEvent::DOM_VK_B);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap();
    elem.dispatch_event(&event).unwrap();
    JsFuture::from(promise).await.unwrap();
}

#[wasm_bindgen_test]
async fn capture_keypress_with_two_callbacks() {
    let elem = document().unwrap().create_element("canvas").unwrap();

    let keyboard_engine = KeyboardEngine::new(&elem);
    let key_a = KeyEvent::DOM_VK_A;

    let promise = Promise::new(&mut |resolve, _| {
        let handle = keyboard_engine.capture(vec![key_a], Box::new(move || {
            web_sys::console::log_1(&"Key has been captured. callback #1".into());
            resolve.call0(&JsValue::NULL).ok();
        }));
        std::mem::forget(handle);
    });
    let promise2 = Promise::new(&mut |resolve, _| {
        let handle = keyboard_engine.capture(vec![key_a], Box::new(move || {
            web_sys::console::log_1(&"Key has been captured. callback #2".into());
            resolve.call0(&JsValue::NULL).ok();
        }));
        std::mem::forget(handle);
    });
    let mut event_init = KeyboardEventInit::new();
    event_init.key_code(KeyEvent::DOM_VK_A);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap();
    elem.dispatch_event(&event).unwrap();
    JsFuture::from(promise).await.unwrap();
    JsFuture::from(promise2).await.unwrap();
}

#[wasm_bindgen_test]
fn uncapture_keypress() {
    let elem = document().unwrap().create_element("canvas").unwrap();

    let keyboard_engine = KeyboardEngine::new(&elem);
    let key_a = KeyEvent::DOM_VK_A;
    let callback1 = Box::new(|| {
        panic!("Shouldn't be executed");
    });
    let callback2 = Box::new(|| {
        web_sys::console::log_1(&"Key has been captured. callback #2".into());
    });
    let handle1 = keyboard_engine.capture(vec![key_a], callback1);
    let handle2 = keyboard_engine.capture(vec![key_a], callback2);
    std::mem::drop(handle1);

    let mut event_init = KeyboardEventInit::new();
    event_init.key_code(KeyEvent::DOM_VK_A);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap();
    elem.dispatch_event(&event).unwrap();
}

#[wasm_bindgen_test]
fn drop_capture_keypress() {
    let elem = document().unwrap().create_element("canvas").unwrap();

    let keyboard_engine = KeyboardEngine::new(&elem);
    let key_a = KeyEvent::DOM_VK_A;
    let callback1 = Box::new(|| {
        panic!("Shouldn't be executed");
    });
    let callback2 = Box::new(|| {
        panic!("Shouldn't be executed");
    });
    let handle1 = keyboard_engine.capture(vec![key_a], callback1);
    let handle2 = keyboard_engine.capture(vec![key_a], callback2);
    keyboard_engine.drop_capture(vec![key_a]);

    let mut event_init = KeyboardEventInit::new();
    event_init.key_code(KeyEvent::DOM_VK_A);
    let event = KeyboardEvent::new_with_keyboard_event_init_dict("keydown", &event_init).unwrap();
    elem.dispatch_event(&event).unwrap();
}
