use crate::prelude::*;

// FIXME move to other parser-related tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::double_representation::definition::DefinitionProvider;
    use ast::macros::DocCommentInfo;
    use parser::Parser;


    /// Expect `main` method, where first line is a documentation comment.
    /// The text of this comment should match the expected one.
    fn run_case(parser:&Parser, code:&str, expected_comment_text:&str) {
        let ast = parser.parse_module(code,default()).unwrap();
        let main_id = double_representation::definition::Id::new_plain_name("main");
        let main = double_representation::module::get_definition(&ast,&main_id).unwrap();
        let lines = main.block_lines();
        let first_line = lines[0].transpose_ref().unwrap();
        let doc = DocCommentInfo::new(&first_line,main.indent()).unwrap();
        let text = doc.text();
        assert_eq!(text, expected_comment_text);

        // Now, if we convert our pretty text to code, will it be the same as original line?
        let code = DocCommentInfo::text_to_repr(&text);
        let ast2 = parser.parse_line(&code).unwrap();
        let doc2 = DocCommentInfo::new(&ast2.as_ref(),main.indent()).expect(&format!("Failed to parse `{}` as comment",code));
        assert_eq!(doc.line().repr(), doc2.line().repr())
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

        // Single line case with a single trailing space after `##`.
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
        let expected = " First line\n Second line";
        run_case(&parser, code,expected);
    }

}
