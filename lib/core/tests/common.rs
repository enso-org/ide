use basegl::display::rendering::HTMLObject;
use basegl::system::web::document;
use basegl::system::web::dyn_into;
use basegl::system::web::create_element;
use basegl::system::web::get_element_by_id;
use basegl::system::web::AttributeSetter;
use basegl::system::web::StyleSetter;
use web_sys::HtmlElement;

// =================
// === TestGroup ===
// =================

pub struct TestGroup {
    pub div : HtmlElement,
}

impl TestGroup {
    pub fn new() -> Self {
        let div : HtmlElement = match get_element_by_id("testgroup") {
            Ok(div) => dyn_into(div).expect("div should be a HtmlElement"),
            Err(_) => {
                let div = create_element("div").expect("TestGroup failed to create div");
                dyn_into(div).expect("HtmlElement")
            },
        };
        div.set_attribute_or_panic("id", "testgroup");
        div.set_property_or_panic("display", "flex");
        div.set_property_or_panic("flex-wrap", "wrap");
        document()
            .expect("Document is not present")
            .body()
            .expect("Body is not present")
            .append_child(&div)
            .expect("TestGroup's div should be appended to body");
        Self { div }
    }
}

// =====================
// === TestContainer ===
// =====================

pub struct TestContainer {
    div: HTMLObject,
}

impl TestContainer {
    pub fn new(name: &str, width: f32, height: f32) -> Self {
        let mut div = HTMLObject::new("div").expect("div");
        div.set_dimensions(width, height + 16.0);

        div.element.set_property_or_panic("border", "1px solid black");
        div.element.set_property_or_panic("position", "relative");
        div.element.set_property_or_panic("margin", "10px");

        let mut header = HTMLObject::from_html_string(
                            &format!("<center>{}</center>", name)
                        ).expect("TestContainer should have a header");
        header.set_dimensions(width, 16.0);
        header.element.set_property_or_panic("border-bottom", "1px solid black");
        header.element.set_property_or_panic("position", "relative");

        div
            .element
            .append_child(&header.element)
            .expect("TestContainer's appended header");

        let mut container = HTMLObject::new("div").expect("TestContainer's div not created");
        container.set_dimensions(width, height);

        container
            .element
            .set_attribute_or_panic("id", name);

        container
            .element
            .set_property_or_panic("position", "relative");

        div
            .element
            .append_child(&container.element)
            .expect("appende container");

        TestGroup::new()
            .div
            .append_child(&div.element)
            .expect("TestGroup failed to append TestContainer's div");
        Self { div }
    }

    pub fn append_child(&mut self, element: &HtmlElement) {
        self.div.element.append_child(&element).expect("TestContainer failed to append element");
    }
}
