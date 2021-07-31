use crate::prelude::*;

// FIXME move to other parser-related tests

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Shape;
    use crate::double_representation::definition::DefinitionProvider;
    use ast::macros::DocCommentInfo;
    use parser::Parser;


    /// Expect `main` method, where first line is a documentation comment.
    /// The text of this comment should match the expected one.
    fn run_case(parser:&Parser, code:&str, expected_comment_text:&str) {
        let ast = parser.parse_module(code,default()).unwrap();
        let main_id = double_representation::definition::Id::new_plain_name("main");
        let module_info = double_representation::module::Info {ast:ast.clone_ref()};

        let main = double_representation::module::get_definition(&ast,&main_id).unwrap();
        let lines = main.block_lines();
        let first_line_ast = lines[0].elem.as_ref().unwrap();
        let doc = DocCommentInfo::new_indented(first_line_ast,4).unwrap();
        assert_eq!(doc.text(), expected_comment_text);
    }

    #[test]
    fn parse_single_line_comment() {
        let parser = parser::Parser::new_or_panic();

        // Typical single line case.
        let code = r#"
main =
    ## Single line
    node"#;
        let expected = " Single line";
        run_case(&parser, code,expected);

        // Single line case without space after `##`.
        let code = r#"
main =
    ##Single line
    node"#;
        let expected = "Single line";
        run_case(&parser, code,expected);

        // Single line case with a single space after `##`.
        let code = r#"
main =
    ## 
    node"#;
        let expected = " ";
        run_case(&parser, code,expected);

        // Single line case without content.
        let code = r#"
main =

    ##
    node"#;
        let expected = "";
        run_case(&parser, code,expected);

    }

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
