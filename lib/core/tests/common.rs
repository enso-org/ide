use basegl::display::scene::{HTMLObject};
use basegl::system::web::{document};
use web_sys::HtmlElement;

// to be implemented
// pub struct TestGroup {
//     div : HTMLObject
// }
//
// impl TestGroup {
//     pub fn new() -> Self {
//         let mut div = HTMLObject::new("div").expect("div");
//
//     }
// }

pub struct TestContainer {
    div : HTMLObject
}

impl TestContainer {
    pub fn new(name : &str, width: f32, height: f32) -> Self {
        let mut div = HTMLObject::new("div").expect("div");
        div.set_dimension(width, height + 16.0);
        div.element.style().set_property("border", "1px solid black").expect("black border");
        div.element.style().set_property("position", "relative").expect("relative");
        div.element.style().set_property("margin", "10px").expect("10px margin");

        let mut header = HTMLObject::from_html_string(&format!("<center>{}</center>", name)).expect("header");
        header.set_dimension(width, 16.0);
        header.element.style().set_property("border-bottom", "1px solid black").expect("black border");
        header.element.style().set_property("position", "relative").expect("relative");
        div.element.append_child(&header.element).expect("appended header");

        let mut container = HTMLObject::new("div").expect("container div");
        container.set_dimension(width, height);
        container.element.set_attribute("id", name).expect("set element id");
        container.element.style().set_property("position", "relative").expect("relative");
        div.element.append_child(&container.element).expect("appende container");

        document().expect("document").body().expect("body").append_child(&div.element).expect("appended div");
        Self { div }
    }

    pub fn append_child(&mut self, element : &HtmlElement) {
        self.div.element.append_child(&element).expect("appended element");
    }
}
