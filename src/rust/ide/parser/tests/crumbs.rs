use parser::prelude::*;

use parser::Parser;
use parser::api::IsParser;
//use parser::test_utils::ParserTestExts;
use ast::Ast;
//use ast::crumbs::Crumbable;
use ast::HasRepr;

#[test]
fn set_line_in_block() {
    let code = r"
main = foo
    bar
      baz
    bar";

    let ast = Parser::new_or_panic().parse(code.to_string(),default()).unwrap();

    use ast::crumbs::*;

    let crumb = Crumb::Module(ModuleCrumb{line_index:1});
    let new_ast = ast.set(&crumb, Ast::var("foo")).unwrap();
    println!("New AST =====:\n{}", new_ast.repr());
}
