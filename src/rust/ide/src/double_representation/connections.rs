use crate::prelude::*;

use crate::double_representation::node::NodeInfo;
use ast::crumbs::{Crumbs, PrefixCrumb};

pub mod test_utils;

#[derive(Clone,Debug)]
pub struct Identifier { pub ast:Ast }

impl Identifier {

}

pub type LocatedIdentifier = ast::crumbs::Located<Identifier>;

#[derive(Clone,Debug)]
pub struct VariableUsageInfo {
    pub introduced : Vec<LocatedIdentifier>,
    pub used       : Vec<LocatedIdentifier>,
}

pub fn assignment_introduces_idents0(mut crumbs:Crumbs, ast:Ast) -> Vec<LocatedIdentifier> {
    match ast.shape() {
        ast::Shape::Var(_) => {
            vec![LocatedIdentifier {
                crumbs,
                item: Identifier {ast}
            }]
        }
//        ast::Shape::Prefix(prefix) => {
//            crumbs.push(PrefixCrumb::Func.into());
//            assignment_introduces_idents0(crumbs,prefix.func)
//        }
        _ => todo!(),
    }
}

pub fn assignment_introduces_idents(ast:Ast) -> Vec<LocatedIdentifier> {
    let crumbs = vec![];
    match ast.shape() {
        ast::Shape::Var(_) => {
            vec![LocatedIdentifier {
                crumbs,
                item: Identifier {ast}
            }]
        }
        _ => todo!(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ast::crumbs::Crumbs;
    use data::text::{Index, Size};
    use data::text::Span;

    use super::test_utils::*;

    fn get_span_map(ast:Ast) -> HashMap<Span,Ast> {
        let mut ret = HashMap::new();
        ast::traverse_with_index(ast, |index,ast| {
            let span = Span::new(index, Size::new(ast.len()));
            ret.insert(span,ast.clone_ref());
        });
        ret
    }

    #[test]
    fn fgfdg() {
        let code = "«sum» = »a« + »b«";
        let res  = Case::parse(code);
        let ast  = parser::Parser::new_or_panic().parse(res.code,default()).unwrap();
        ast::traverse_with_index(ast, |index,ast| {
            println!("At {} AST is {}\n\n", index.value, ast.repr());
        })

    }

}
//pub fn describe_variable_usage(node:NodeInfo)


