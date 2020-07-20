use enso_prelude::*;

use parser::DocParser;

use uuid::Uuid;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::wasm_bindgen_test;



wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn doc_gen_test() {
    // no doc case
    let input = String::from("type Foo\n  type Bar");
    let program = std::env::args().nth(1).unwrap_or(input);
    let parser = DocParser::new_or_panic();
    let gen_code = parser.doc_parser_generate_html_source(program).unwrap();
    assert_eq!(gen_code, "{\"SuccessDoc\":{\"code\":\"\"}}");

    // simple cases
    let input = String::from("##\n  foo\ntype Foo\n");
    let program = std::env::args().nth(1).unwrap_or(input);
    let parser = DocParser::new_or_panic();
    let gen_code = parser.doc_parser_generate_html_source(program).unwrap();
    assert_eq!(gen_code, "{\"SuccessDoc\":{\"code\":\"<html><head><meta http-equiv=\\\"Content-Type\\\" content=\\\"text/html\\\" charset=\\\"UTF-8\\\" /><link rel=\\\"stylesheet\\\" href=\\\"style.css\\\" /><title>def Foo</title></head><body><div class=\\\"Documentation\\\"><div class=\\\"ASTHead\\\"><div class=\\\"DefNoBody\\\"><a href=\\\"Foo.html\\\"><div class=\\\"DefTitle\\\">Foo<div class=\\\"DefArgs\\\"></div></div></a></div></div><div><div class=\\\"Doc\\\"><div class=\\\"Synopsis\\\"><div class=\\\"Raw\\\">foo</div></div></div></div></div></body></html>\"}}");

    let input = String::from("##\n  DEPRECATED\n  Foo bar baz\ntype Foo\n  type Bar");
    let program = std::env::args().nth(1).unwrap_or(input);
    let parser = DocParser::new_or_panic();
    let gen_code = parser.doc_parser_generate_html_source(program).unwrap();
    assert_eq!(gen_code, "{\"SuccessDoc\":{\"code\":\"<html><head><meta http-equiv=\\\"Content-Type\\\" content=\\\"text/html\\\" charset=\\\"UTF-8\\\" /><link rel=\\\"stylesheet\\\" href=\\\"style.css\\\" /><title>def Foo</title></head><body><div class=\\\"Documentation\\\"><div class=\\\"Tags\\\"><div class=\\\"DEPRECATED\\\">DEPRECATED</div></div><div class=\\\"ASTHead\\\" style=\\\"text-decoration-line:line-through;\\\"><div class=\\\"Def\\\"><a href=\\\"Foo.html\\\"><div class=\\\"DefTitle\\\">Foo<div class=\\\"DefArgs\\\"></div></div></a></div></div><div class=\\\"Doc\\\"><div class=\\\"Synopsis\\\"><div class=\\\"Raw\\\">Foo bar baz</div></div></div><div class=\\\"ASTData\\\"><div class=\\\"Def\\\"><div class=\\\"DefBody\\\"><h2 class=\\\"constr\\\">Constructors</h2><div class=\\\"DefNoDoc\\\"><div class=\\\"DefNoBody\\\"><a href=\\\"Bar.html\\\"><div class=\\\"DefTitle\\\">Bar<div class=\\\"DefArgs\\\"></div></div></a></div></div><h2 class=\\\"constr\\\">Methods</h2></div></div></div></div></body></html>\"}}");
}
