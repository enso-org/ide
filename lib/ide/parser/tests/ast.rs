//! Tests specific to Ast rather than parser itself but placed here because they depend on parser
//! to easily generate test input.

use parser::prelude::*;

use ast::HasRepr;
use ast::opr;
use ast::prefix;
use parser::api::IsParser;

#[test]
pub fn flatten_prefix_test() {
    fn expect_pieces(flattened:&prefix::Chain, pieces:Vec<&str>) {
        let mut piece_itr = pieces.iter();
        assert_eq!(flattened.args.len() + 1, pieces.len()); // +1 because `func` piece is separate field
        assert_eq!(&flattened.func.repr(),piece_itr.next().unwrap());
        flattened.args.iter().zip(piece_itr).for_each(|(lhs,rhs)|{
            assert_eq!(&lhs.repr(),rhs);
        })
    }

    let mut parser = parser::Parser::new_or_panic();
    let mut case = |code:&str, expected_pieces:Vec<&str>| {
        let ast = parser.parse(code.into(),default()).unwrap();
        let ast = ast::test_utils::expect_single_line(&ast);
        let flattened = prefix::Chain::new_non_strict(&ast);
        expect_pieces(&flattened,expected_pieces);
    };

    case("a", vec!["a"]);
    case("a b c d", vec!["a","b","c","d"]);
    case("a b + c d", vec!["a b + c d"]); // nothing to flatten, this is infix, not prefix
}

#[test]
pub fn flatten_infix_test() {
    fn expect_pieces(flattened:&opr::Chain, target:&str, pieces:Vec<&str>) {
        assert_eq!(&flattened.target.repr(),target);

        let piece_itr = pieces.iter();
        assert_eq!(flattened.args.len(), pieces.len());
        flattened.args.iter().zip(piece_itr).for_each(|(lhs,rhs)|{
            assert_eq!(&lhs.1.repr(),rhs);
        })
    }

    let mut parser = parser::Parser::new_or_panic();
    let mut case = |code:&str, target:&str, expected_pieces:Vec<&str>| {
        let ast = parser.parse(code.into(),default()).unwrap();
        let ast = ast::test_utils::expect_single_line(&ast);
        let flattened = opr::Chain::try_new(&ast).unwrap();
        expect_pieces(&flattened,target,expected_pieces);
    };

    case("a+b+c",  "a",vec!["b","c"]);
    case("a,b,c",  "c",vec!["b","a"]);
    case("a+b*c+d","a",vec!["b*c","d"]);
}
