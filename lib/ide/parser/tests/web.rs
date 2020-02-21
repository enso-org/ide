use parser::Parser;
use uuid::Uuid;
use wasm_bindgen_test::{wasm_bindgen_test_configure, wasm_bindgen_test};

use parser::api::Error::ParsingError;
use parser::api::IDMap;
use parser::api::Span;
use parser::api::Index;
use parser::api::Size;

wasm_bindgen_test_configure!(run_in_browser);


#[wasm_bindgen_test]
fn web_test() {
    let mut parser = Parser::new_or_panic();

    let mut parse = |input| {
        let uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let span = Span { index: Index { value: 0 }, size: Size { value: 3 } };
        let ids  = IDMap(vec![(span, uuid)]);
        match parser.parse(String::from(input), ids) {
            Err(ParsingError(str)) => str,
            _ => panic!("Not implemented.")
        }
    };

    assert_eq!(parse(""), r#"{"shape":{"Module":{"lines":[{"elem":null,"off":0}]}},"span":0}"#);

    assert_eq!(parse("x"), r#"{"shape":{"Module":{"lines":[{"elem":{"shape":{"Var":{"name":"x"}},"span":1},"off":0}]}},"span":1}"#);

    assert_eq!(parse("x y"), r#"{"shape":{"Module":{"lines":[{"elem":{"id":"00000000-0000-0000-0000-000000000000","shape":{"Prefix":{"arg":{"shape":{"Var":{"name":"y"}},"span":1},"func":{"shape":{"Var":{"name":"x"}},"span":1},"off":1}},"span":3},"off":0}]}},"span":3}"#);
}
