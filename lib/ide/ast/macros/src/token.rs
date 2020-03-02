use crate::prelude::*;

use macro_utils::path_segment_generic_args;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;
use syn::GenericArgument;
use syn::PathSegment;
use syn::Token;
use syn::punctuated::Punctuated;

/// Generates `Tokenizer` that just panics when called.
pub fn no_tokenizer
(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let target = syn::parse::<PathSegment>(input).unwrap();
    let ty_args = path_segment_generic_args(&target);
    let ret = quote!{
        impl<#(#ty_args),*> Tokenizer for #target {
            fn tokenize(&self, builder:&mut impl TokenBuilder) {
                panic!("Tokenizer not supported for Spaceless AST!")
            }
        }
    };
    ret.into()
}

/// Inner logic for `derive_tokenizer`.
pub fn derive_for_enum
(decl:&syn::DeriveInput, data:&syn::DataEnum)
 -> TokenStream  {
    let ident     = &decl.ident;
    let params    = decl.generics.params.iter().collect_vec();
    let token_arms = data.variants.iter().map(|v| {
        let con_ident = &v.ident;
        quote!( #ident::#con_ident (elem) => elem.tokenize(builder) )
    });
    let ret = quote! {
        impl<#(#params:Tokenizer),*> Tokenizer for #ident<#(#params),*> {
            fn tokenize(&self, builder:&mut impl TokenBuilder) {
                match self { #(#token_arms),* }
            }
        }
    };
    ret
}

/// Structure representing input to macros like `tokenizer!`.
///
/// Basically it consists of a typename (with optional generic arguments) and
/// sequence of expressions that yield values we use to obtain sub-tokenizer.
pub struct TokenDescription {
    pub ty      : PathSegment,
    pub ty_args : Vec<GenericArgument>,
    pub exprs   : Vec<Expr>,
}

impl syn::parse::Parse for TokenDescription {
    /// Parser user-provided input to macro into out structure.
    ///
    /// First should go a type for which implementation is to be provided,
    /// then arbitrary sequence of expressions.
    /// Panics on invalid input, which is actually fair for a macro code.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ty:PathSegment = input.parse()?;
        input.parse::<Option<syn::token::Comma>>()?;
        let exprs   = Punctuated::<Expr,Token![,]>::parse_terminated(input)?;
        let exprs   = exprs.iter().cloned().collect::<Vec<_>>();
        let ty_args = path_segment_generic_args(&ty);
        let ty_args = ty_args.into_iter().cloned().collect(); // get rid of &
        Ok(TokenDescription {ty,ty_args,exprs})
    }
}

impl TokenDescription {
    /// Fills a trait implementation template with given methods.
    pub fn make_impl
    (&self, trait_name:&str, methods:&TokenStream) -> TokenStream {
        let trait_name = syn::parse_str::<syn::TypePath>(trait_name).unwrap();
        let ty         = &self.ty;
        let ty_args    = &self.ty_args;
        quote! {
            impl<#(#ty_args:#trait_name),*> #trait_name for #ty {
                #methods
            }
        }
    }

    /// Generates `Tokenizer` instance using user-provided input.
    pub fn tokenizer(&self) -> TokenStream {
        let exprs = &self.exprs;
        self.make_impl("Tokenizer", &quote!{
            fn tokenize(&self, builder:&mut impl TokenBuilder) {
                #(#exprs.tokenize(builder);)*
            }
        })
    }
}

