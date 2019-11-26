use crate::system::web::document;
use crate::system::web::dyn_into;
use crate::system::web::create_element;
use crate::system::web::get_element_by_id;
use crate::system::web::AttributeSetter;
use crate::system::web::StyleSetter;
use crate::system::web::NodeInserter;
use web_sys::HtmlElement;

/// Helper to group test containers
pub struct Group {
    pub div : HtmlElement,
}

impl Group {
    pub fn new(name : &str) -> Self {
        let div : HtmlElement = match get_element_by_id(name) {
            // If id=name exists, we use it.
            Ok(div) => dyn_into(div).expect("div should be a HtmlElement"),
            // If it doesn't exist, we create a new element.
            Err(_) => {
                let div = create_element("div")
                          .expect("TestGroup failed to create div");

                let div : HtmlElement = dyn_into(div).expect("HtmlElement");
                div.set_attribute_or_panic("id"           , name);
                div.set_property_or_panic ("display"      , "flex");
                div.set_property_or_panic ("flex-wrap"    , "wrap");
                div.set_property_or_panic ("border"       , "1px solid black");
                div.set_property_or_panic ("margin-bottom", "10px");

                let header = create_element("center")
                             .expect("TestGroup failed to create header");
                let header : HtmlElement = dyn_into(header)
                                           .expect("HtmlElement");
                header.set_inner_html(name);
                let border = "1px solid black";
                header.set_property_or_panic("border-bottom", border);
                header.set_property_or_panic("width"        , "100%");
                div.append_child_or_panic(&header);

                document()
                    .expect("Document is not present")
                    .body()
                    .expect("Body is not present")
                    .append_child_or_panic(&div);
                div
            },
        };
        Self { div }
    }
}
