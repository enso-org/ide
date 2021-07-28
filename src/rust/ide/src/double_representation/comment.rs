use crate::prelude::*;

use ast::{known, MacroPatternMatch, Shifted};
use ast::prelude::fmt::Formatter;


#[cfg(test)]
mod tests {
    use super::*;
    use ast::Shape;
    use crate::double_representation::definition::DefinitionProvider;
    use ast::macros::DocCommentInfo;


    #[test]
    fn parse_comment() {
        let parser = parser::Parser::new_or_panic();
        let code = r#"
main =
    ## Adding number is important.
       Like really, really important.
    sum = 2+2
    sum"#;
        let ast = parser.parse_module(code,default()).unwrap();
        let main_id = double_representation::definition::Id::new_plain_name("main");
        let module_info = double_representation::module::Info {ast:ast.clone_ref()};

        let main = double_representation::module::get_definition(&ast,&main_id).unwrap();
        let lines = main.block_lines();

        for line in lines {
            if let Some(doc) = line.elem.as_ref().and_then(ast::macros::DocCommentInfo::new) {
                DEBUG!("{doc}");
            }
        }


//        assert_eq!(as_disable_comment(&ast), Some("Żar".into()));



        // match ast.shape() {
        //     Shape::Match(r#match) => {
        //         let first_segment = &r#match.segs.head;
        //         assert_eq!(ast::identifier::name(&first_segment.head), Some("#"));
        //         use ast::HasTokens;
        //         ERROR!("'{first_segment.body.repr()}'");
        //
        //         // first_segment.body.feed_to(&mut |ast:ast::Token| {
        //         //     ERROR!("{ast.repr()}");
        //         // });
        //         //ERROR!("{first_segment.body}");
        //     }
        //     _ => todo!(),
        // }
    }

}
