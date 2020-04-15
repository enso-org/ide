use ast::Ast;


trait SpanTreeActions {
    fn set(&self, root:&Ast, replace_with:Ast) -> Ast;
    fn insert(&self, root:&Ast, inserted:Ast) -> Ast;
    fn erase(&self, root:&Ast) -> Ast;
}
