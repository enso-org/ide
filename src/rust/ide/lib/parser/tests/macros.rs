#![feature(option_result_contains)]

use parser::prelude::*;

use utils::option::OptionExt;

use parser::Parser;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);


// #[test]
// fn recognizing_imports() {
//     use ast::*;
//
//     let parser = Parser::new_or_panic();
//
//     let ast = parser.parse_line("import Foo.Bar.Baz").unwrap();
//
//     let m = ast::known::Match::try_from(ast).unwrap();
//     assert_eq!(m.segs.tail.len(),0);
//     let seg : &MacroMatchSegment<Ast> = &m.segs.head;
//
//     // println!("{:?}\n\n",seg);
//     // println!("{:?}\n\n",seg.head);
//     // println!("{:?}\n\n",seg.body);
//     // println!("{}\n\n",seg.body.repr());
//
//     if ast::identifier::name(&seg.head).contains_if(|str| *str == "import") {
//         let target_module = seg.body.repr();
//         let segments = target_module.split(ast::opr::predefined::ACCESS).map(ToString::to_string).collect();
//         let import = ast::macros::ImportInfo {segments};
//         println!("{:?}",import);
//     }
//
// }

#[wasm_bindgen_test]
fn recognizing_lambdas() {
    let parser = Parser::new_or_panic();

    let expect_lambda = |code:&str, arg:&str, body:&str| {
        let ast = parser.parse_line(code).unwrap();
        let lambda = ast::macros::as_lambda(&ast).expect("failed to recognize lambda");
        assert_eq!(lambda.arg.repr(), arg);
        assert_eq!(lambda.body.repr(), body);
        assert_eq!(*lambda.arg, ast.get_traversing(&lambda.arg.crumbs).unwrap());
        assert_eq!(*lambda.body, ast.get_traversing(&lambda.body.crumbs).unwrap());
    };
    let expect_not_lambda = |code:&str| {
        let ast = parser.parse_line(code).unwrap();
        assert!(ast::macros::as_lambda_match(&ast).is_none(), "wrongly recognized a lambda");
    };

    expect_lambda("a->b",       "a",    "b");
    expect_lambda("foo->4+(4)", "foo",  "4+(4)");
    expect_lambda("a->b->c",    "a",    "b->c");
    // expect_lambda("(a->b)->c"); // TODO: Failing due to internal parser error: java.lang.NullPointerException

    expect_not_lambda("(a->b)");
    expect_not_lambda("a+b");
    expect_not_lambda("'a+b'");
    expect_not_lambda("497");
}
