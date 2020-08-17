//! Utilities for dealing with `Prefix` application Ast nodes.

use crate::prelude::*;

use crate::Ast;
use crate::Id;
use crate::crumbs::Located;
use crate::crumbs::PrefixCrumb;
use crate::HasTokens;
use crate::known;
use crate::Prefix;
use crate::Shifted;
use crate::Token;
use crate::TokenConsumer;

use utils::vec::VecExt;



// ================
// === Argument ===
// ================

/// Struct representing an element of a Prefix Chain: an argument applied over the function.
#[derive(Clone,Debug)]
pub struct Argument {
    /// An argument ast with offset between it and previous arg or function.
    pub sast      : Shifted<Ast>,
    /// The id of Prefix AST of this argument application.
    pub prefix_id : Option<Id>,
}

impl HasTokens for Argument {
    fn feed_to(&self, consumer: &mut impl TokenConsumer) {
        self.sast.feed_to(consumer)
    }
}

// ====================
// === Prefix Chain ===
// ====================

/// Result of flattening a sequence of prefix applications.
#[derive(Clone,Debug)]
pub struct Chain {
    /// The function (initial application target)
    pub func : Ast,
    /// Subsequent arguments applied over the function.
    pub args : Vec<Argument>,
}

impl Chain {
    /// Construct a prefix application chain from a function and sequence of arguments.
    pub fn new(func:Ast, args:impl IntoIterator<Item=Ast>) -> Self {
        let args = args.into_iter().map(|arg| {
            Argument {
                sast      : Shifted::new(1,arg),
                prefix_id : Some(Id::new_v4())
            }
        }).collect_vec();

        Self {func,args}
    }

    /// Translates calls like `a b c` that generate nested prefix chain like
    /// App(App(a,b),c) into flat list where first element is the function and
    /// then arguments are placed: `{func:a, args:[b,c]}`.
    pub fn from_prefix(ast:&known::Prefix) -> Chain {
        fn run(ast:&known::Prefix, mut acc: &mut Vec<Argument>) -> Ast {
            let func = match known::Prefix::try_from(&ast.func) {
                Ok(lhs_app) => run(&lhs_app, &mut acc),
                _           => ast.func.clone(),
            };
            let sast      = Shifted{wrapped:ast.arg.clone(),off:ast.off};
            let prefix_id = ast.id();
            acc.push(Argument{sast,prefix_id});
            func
        }

        let mut args = Vec::new();
        let func     = run(ast,&mut args);
        Chain {func,args}
    }

    /// Like `new` but returns None if given Ast is not of a Prefix shape.
    pub fn from_ast(ast:&Ast) -> Option<Chain> {
        known::Prefix::try_from(ast).as_ref().map(Chain::from_prefix).ok()
    }

    /// As new but if the AST is not a prefix, interprets is a function with an
    /// empty arguments list.
    pub fn from_ast_non_strict(ast:&Ast) -> Chain {
        if let Ok(ref prefix) = known::Prefix::try_from(ast) {
            // Case like `a b c`
            Self::from_prefix(prefix)
        } else if let Ok(ref section) = known::SectionRight::try_from(ast) {
            // Case like `+ a b`
            let func        = section.opr.clone();
            let right_chain = Chain::from_ast_non_strict(&section.arg);
            let sast        = Shifted{wrapped:right_chain.func, off:section.off};
            let prefix_id   = section.id();
            let mut args    = vec![Argument{sast,prefix_id}];
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
            Located::new(&func_crumbs[i..],&arg.sast.wrapped)
        })
    }

    /// Replace the `func` and first argument with a new `func` being an proper Prefix ast node.
    /// Does nothing if there are no arguments.
    pub fn fold_arg(&mut self) {
        if let Some(arg) = self.args.pop_front() {
            let new_prefix = Prefix{
                arg  : arg.sast.wrapped,
                func : self.func.clone_ref(),
                off  : arg.sast.off,
            };
            self.func = Ast::new(new_prefix,arg.prefix_id);
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

impl HasTokens for Chain {
    fn feed_to(&self, consumer: &mut impl TokenConsumer) {
        self.func.feed_to(consumer);
        for arg in &self.args {
            consumer.feed(Token::Off(arg.sast.off));
            arg.sast.wrapped.feed_to(consumer);
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    use utils::test::ExpectTuple;
    use uuid::Uuid;

    #[test]
    fn prefix_chain() {
        let a = Ast::var("a");
        let b = Ast::var("b");
        let c = Ast::var("c");

        let a_b = Ast::prefix(a.clone(),b.clone()).with_id(Uuid::new_v4());
        let a_b_c = Ast::prefix(a_b.clone(),c.clone()).with_id(Uuid::new_v4());

        let chain = Chain::from_ast(&a_b_c).unwrap();
        assert_eq!(chain.func, a);
        assert_eq!(chain.args[0].sast.wrapped, b);
        assert_eq!(chain.args[1].sast.wrapped, c);
        assert_eq!(chain.args[0].prefix_id, a_b.id);
        assert_eq!(chain.args[1].prefix_id, a_b_c.id);

        let (arg1,arg2) = chain.enumerate_args().expect_tuple();
        assert_eq!(arg1.item, &b);
        assert_eq!(a_b_c.get_traversing(&arg1.crumbs).unwrap(), &b);
        assert_eq!(arg2.item, &c);
        assert_eq!(a_b_c.get_traversing(&arg2.crumbs).unwrap(), &c);
    }

    #[test]
    fn prefix_chain_construction() {
        let a     = Ast::var("a");
        let b     = Ast::var("b");
        let c     = Ast::var("c");
        let chain = Chain::new(a,vec![b,c]);
        assert_eq!(chain.into_ast().repr(), "a b c");
    }

    // TODO[ao] add tests for modifying chain.
}
