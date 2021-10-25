//! Tests for cases where parser currently fails. They are ignored, should be removed and placed
//! elsewhere, as the parser gets fixed.

use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn no_doc_found() {
    let input = String::from("type Foo\n  type Bar");
    let program = std::env::args().nth(1).unwrap_or(input);
    let parser = parser::DocParser::new_or_panic();
    let gen_code = parser.generate_html_docs(program).unwrap();
    assert_eq!(gen_code.len(), 22); // should be 0
}

#[wasm_bindgen_test]
fn extension_operator_methods() {
    let ast = parser::Parser::new_or_panic().parse_line_ast("Int.+").unwrap();

    use ast::*;
    if let Shape::Infix(Infix {larg:_larg,loff:_loff,opr,roff:_roff,rarg}, ..) = ast.shape() {
        if let Shape::Opr(Opr{..}) = opr.shape() {
            // TODO: should be Opr(+). https://github.com/enso-org/enso/issues/565
            if let Shape::Var(Var{..}) = rarg.shape() {
                return;
            }
        }
    }
    panic!("Should have matched into return.");
}
