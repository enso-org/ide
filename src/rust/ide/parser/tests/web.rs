use enso_prelude::*;

use ast::Ast;
use ast::HasRepr;
use ast::IdMap;
use data::text::*;
use parser::Parser;
use parser::api::SourceFile;

use uuid::Uuid;
use wasm_bindgen_test::wasm_bindgen_test_configure;
use wasm_bindgen_test::wasm_bindgen_test;



wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn web_test() {
    let uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();

    let parser = Parser::new_or_panic();

    let parse = |input| parser.parse_with_metadata(input).unwrap();
    let file  = |term|
        SourceFile{metadata:serde_json::json!({}), ast:ast::known::KnownAst::new_no_id(term)};


    let line = |term| {
        ast::Module {lines: vec![ast::BlockLine {elem:term,off:0}]}
    };

    let app = ast::Prefix{func:Ast::var("x"), off:3, arg:Ast::var("y")};
    let var = ast::Var{name:"x".into()};

    let ast = file(line(None));
    assert_eq!(parse(String::try_from(&ast).unwrap()), ast);

    let ast = file(line(Some(Ast::new(var,Some(uuid)))));
    assert_eq!(parse(String::try_from(&ast).unwrap()), ast);

    let ast = file(line(Some(Ast::new(app,Some(uuid)))));
    assert_eq!(parse(String::try_from(&ast).unwrap()), ast);
}
