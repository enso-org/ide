extern crate proc_macro;

use proc_macro::TokenStream;
use syn::*;
use quote::quote;

#[proc_macro_attribute]
pub fn web_test(_args: TokenStream, input: TokenStream) -> TokenStream {
    if let Ok(mut parsed) = syn::parse::<ItemFn>(input.clone()) {
        let fn_string = format!("{}", parsed.sig.ident);
        let code = format!("Container::new(\"Tests\", \"{}\", 320.0, 240.0);",
                           fn_string);

        if let Ok(stmt) = parse_str::<Stmt>(&code) {
            parsed.block.stmts.insert(0, stmt);

            let output = quote! {
                #[wasm_bindgen_test]
                #parsed
            };
            output.into()
        } else {
            input
        }
    } else {
        input
    }
}

#[proc_macro_attribute]
pub fn web_bench(_args: TokenStream, input: TokenStream) -> TokenStream {

    if let Ok(parsed) = parse::<ItemFn>(input.clone()) {
        use proc_macro2::*;
        let input : TokenStream = input.into();

        let fn_ident = parsed.sig.ident;
        let fn_string = format!("{}", fn_ident);
        let fn_benchmark_str = format!("{}_benchmark", fn_string);
        let fn_benchmark = Ident::new(&fn_benchmark_str, Span::call_site());

        let output = quote! {
            #[wasm_bindgen_test]
            fn #fn_benchmark() {
                let container = BenchContainer::new(#fn_string, 320.0, 240.0);
                let mut b = Bencher::new(container);
                #fn_ident(&mut b);
            }
            #input
        };
        output.into()
    } else {
        input
    }
}
