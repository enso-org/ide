use basegl::display::rendering::HTMLObject;
use basegl::system::web::{document, dyn_into, create_element, get_element_by_id};
use web_sys::HtmlElement;

pub struct TestGroup {
    pub div: HtmlElement,
}

impl TestGroup {
    pub fn new() -> Self {
        let div : HtmlElement = match get_element_by_id("testgroup") {
            Ok(div) => dyn_into(div).expect("HtmlElement"),
            Err(_) => {
                let div = create_element("div").expect("div");
                dyn_into(div).expect("HtmlElement")
            },
        };
        div.set_attribute("id", "testgroup").expect("id = testgroup");
        div.style().set_property("display", "flex").expect("flexbox");
        div.style().set_property("flex-wrap", "wrap").expect("wrap");
        document()
            .expect("document")
            .body()
            .expect("body")
            .append_child(&div)
            .expect("appended div");
        Self { div }
    }
}

pub struct TestContainer {
    div: HTMLObject,
}

impl TestContainer {
    pub fn new(name: &str, width: f32, height: f32) -> Self {
        let mut div = HTMLObject::new("div").expect("div");
        div.set_dimensions(width, height + 16.0);
        div.element.style().set_property("border", "1px solid black").expect("black border");
        div.element.style().set_property("position", "relative").expect("relative");
        div.element.style().set_property("margin", "10px").expect("10px margin");

        let mut header =
            HTMLObject::from_html_string(&format!("<center>{}</center>", name)).expect("header");
        header.set_dimensions(width, 16.0);
        header
            .element
            .style()
            .set_property("border-bottom", "1px solid black")
            .expect("black border");
        header.element.style().set_property("position", "relative").expect("relative");
        div.element.append_child(&header.element).expect("appended header");

        let mut container = HTMLObject::new("div").expect("container div");
        container.set_dimensions(width, height);
        container.element.set_attribute("id", name).expect("set element id");
        container.element.style().set_property("position", "relative").expect("relative");
        div.element.append_child(&container.element).expect("appende container");

        TestGroup::new().div.append_child(&div.element).expect("appended div");
        Self { div }
    }

    pub fn append_child(&mut self, element: &HtmlElement) {
        self.div.element.append_child(&element).expect("appended element");
    }
}
