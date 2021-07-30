use crate::prelude::*;

// FIXME move to other parser-related tests

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Shape;
    use crate::double_representation::definition::DefinitionProvider;
    use ast::macros::DocCommentInfo;



    #[test]
    fn parse_multi_line_comment() {
        let parser = parser::Parser::new_or_panic();
        let code = r#"
main =
    ## First line
       Second line
    node"#;
        let ast = parser.parse_module(code,default()).unwrap();
        let main_id = double_representation::definition::Id::new_plain_name("main");
        let module_info = double_representation::module::Info {ast:ast.clone_ref()};

        let main = double_representation::module::get_definition(&ast,&main_id).unwrap();
        let lines = main.block_lines();
        assert_eq!(lines.len(),2);
        let doc = ast::macros::DocCommentInfo::new_indented(lines[0].elem.as_ref().unwrap(),4).unwrap();
        assert_eq!(doc.text(), " First line\n Second line");
    }

}
