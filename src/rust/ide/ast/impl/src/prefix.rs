//! Utilities for dealing with `Prefix` application Ast nodes.

use crate::prelude::*;

use crate::Ast;
use crate::crumbs::Located;
use crate::crumbs::PrefixCrumb;
use crate::known;
use crate::Prefix;
use crate::Shifted;

use utils::vec::VecExt;

#[derive(Clone,Debug)]
/// Result of flattening a sequence of prefix applications.
pub struct Chain {
    /// The function (initial application target)
    pub func : Ast,
    /// Subsequent arguments applied over the function.
    pub args : Vec<Shifted<Ast>>,
}

impl Chain {
    /// Translates calls like `a b c` that generate nested prefix chain like
    /// App(App(a,b),c) into flat list where first element is the function and
    /// then arguments are placed: `{func:a, args:[b,c]}`.
    pub fn new(ast:&known::Prefix) -> Chain {
        fn run(ast:&known::Prefix, mut acc: &mut Vec<Shifted<Ast>>) -> Ast {
            let func = match known::Prefix::try_from(&ast.func) {
                Ok(lhs_app) => run(&lhs_app, &mut acc),
                _           => ast.func.clone(),
            };
            acc.push(Shifted{wrapped:ast.arg.clone(),off:ast.off});
            func
        }

        let mut args = Vec::new();
        let func     = run(ast,&mut args);
        Chain {func,args}
    }

    /// Like `new` but returns None if given Ast is not of a Prefix shape.
    pub fn try_new(ast:&Ast) -> Option<Chain> {
        known::Prefix::try_from(ast).as_ref().map(Chain::new).ok()
    }

    /// As new but if the AST is not a prefix, interprets is a function with an
    /// empty arguments list.
    pub fn new_non_strict(ast:&Ast) -> Chain {
        if let Ok(ref prefix) = known::Prefix::try_from(ast) {
            // Case like `a b c`
            Self::new(prefix)
        } else if let Ok(ref section) = known::SectionRight::try_from(ast) {
            // Case like `+ a b`
            let func = section.opr.clone();
            let right_chain = Chain::new_non_strict(&section.arg);
            let mut args = vec![Shifted{wrapped:right_chain.func, off:section.off}];
            args.extend(right_chain.args);
            Chain {func,args}
        } else {
            // Case like `a`
            let func = ast.clone();
            let args = Vec::new();
            Chain {func,args}
        }
    }

    /// Crumbs location for the application target (function).
    #[allow(trivial_bounds)]
    pub fn func_location(&self) -> impl Iterator<Item=PrefixCrumb> {
        // Location is always like [Func,Func,…,Func].
        std::iter::repeat(PrefixCrumb::Func).take(self.args.len())
    }

    /// Returns an application target `Ast` reference along with its location.
    pub fn located_func(&self) -> Located<&Ast> {
        Located::new(self.func_location(),&self.func)
    }

    /// Iterates over all arguments, left-to right.
    pub fn enumerate_args<'a>(&'a self) -> impl Iterator<Item = Located<&'a Ast>> + 'a {
        // Location is always like [Func,Func,…,Func,Arg].
        // We iterate beginning from the deeply nested args. So we can just create crumbs
        // location once and then just pop initial crumb when traversing arguments.
        let arg_once    = std::iter::once(PrefixCrumb::Arg);
        let func_crumbs = self.func_location().chain(arg_once).collect_vec();
        let mut i = 0;
        self.args.iter().map(move |arg| {
            i += 1;
            Located::new(&func_crumbs[i..],&arg.wrapped)
        })
    }

    /// Replace the `func` and first argument with a new `func` being an proper Prefix ast node.
    /// Does nothing if there are no arguments.
    pub fn fold_arg(&mut self) {
        if let Some(arg) = self.args.pop_front() {
            let new_prefix = Prefix{
                arg  : arg.wrapped,
                func : self.func.clone_ref(),
                off  : arg.off,
            };
            self.func = new_prefix.into();
        }
    }

    /// Convert the chain to proper AST node.
    pub fn into_ast(mut self) -> Ast {
        while !self.args.is_empty() {
            self.fold_arg()
        }
        self.func
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use utils::test::ExpectTuple;

    #[test]
    fn prefix_chain() {
        let a = Ast::var("a");
        let b = Ast::var("b");
        let c = Ast::var("c");

        let a_b = Ast::prefix(a.clone(),b.clone());
        let a_b_c = Ast::prefix(a_b.clone(),c.clone());

        let chain = Chain::try_new(&a_b_c).unwrap();
        assert_eq!(chain.func, a);
        assert_eq!(chain.args[0].wrapped, b);
        assert_eq!(chain.args[1].wrapped, c);

        let (arg1,arg2) = chain.enumerate_args().expect_tuple();
        assert_eq!(arg1.item, &b);
        assert_eq!(a_b_c.get_traversing(&arg1.crumbs).unwrap(), &b);
        assert_eq!(arg2.item, &c);
        assert_eq!(a_b_c.get_traversing(&arg2.crumbs).unwrap(), &c);
    }

    // TODO[ao] add tests for modifying chain.
}
