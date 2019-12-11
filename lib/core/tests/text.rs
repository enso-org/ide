//! Test suite for the Web and headless browsers.
#![cfg(target_arch = "wasm32")]


#[cfg(test)]
mod tests {
    use web_test::*;
    use web_sys::{HtmlElement,HtmlCanvasElement};
    use wasm_bindgen_test::*;
    use basegl_system_web::{get_element_by_id, create_element, dyn_into, NodeInserter, AttributeSetter};

    web_configure!(run_in_browser);

    #[web_test]
    fn scrolling_horizontal() {
        let root = get_element_by_id("scrolling_horizontal").unwrap();
        let canvas_element : HtmlElement = dyn_into(create_element("canvas").unwrap()).unwrap();
        canvas_element.set_id("workspace");
        canvas_element.set_attribute_or_panic("display", "block");
        canvas_element.set_attribute_or_panic("width", "100px");
        canvas_element.set_attribute_or_panic("height", "100px");
        root.append_or_panic(&canvas_element);
    }
}