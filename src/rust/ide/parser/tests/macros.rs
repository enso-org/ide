use parser::prelude::*;

use parser::Parser;

use ast::known;

pub fn lambda_arg(ast:&known::Match) -> &ast::Ast {
    println!("{:?}",ast::macros::as_lambda(ast.ast()));
    todo!()
}

#[test]
fn recognizing_lambdas() {
    let parser = Parser::new_or_panic();

    let expect_lambda = |code:&str| {
        let ast = parser.parse_line(code).unwrap();
        let lambda = ast::macros::as_lambda_match(&ast).expect("failed to recognize lambda");
        assert_eq!(lambda.repr(),code);
        lambda_arg(&lambda);
    };
    let expect_not_lambda = |code:&str| {
        let ast = parser.parse_line(code).unwrap();
        assert!(ast::macros::as_lambda_match(&ast).is_none(), "wrongly recognized a lambda");
    };

    expect_lambda("a->b");
    expect_lambda("foo->4+(4)");
    expect_lambda("a->b->c");
    // expect_lambda("(a->b)->c"); // TODO: Failing due to internal parser error: java.lang.NullPointerException

    expect_not_lambda("(a->b)");
    expect_not_lambda("a+b");
    expect_not_lambda("'a+b'");
    expect_not_lambda("497");
}
