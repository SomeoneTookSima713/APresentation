#![feature(proc_macro_span)]
#![feature(let_chains)]

use quote::ToTokens;

extern crate proc_macro;
use proc_macro::TokenStream;

struct ForImplsArgs {
    types: syn::punctuated::Punctuated<syn::TypePath, syn::Token![,]>,
    code: proc_macro2::TokenStream
}

impl syn::parse::Parse for ForImplsArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut type_vec: Vec<syn::TypePath> = Vec::new();
        while !input.peek(syn::Token![=>]) {
            type_vec.push(input.parse()?);
            if input.peek(syn::Token![,]) || !input.peek(syn::Token![=>]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        let types = syn::punctuated::Punctuated::from_iter(type_vec.into_iter());
        input.parse::<syn::Token![=>]>()?;
        let code = input.parse()?;
        Ok(Self { types, code })
    }
}

fn replace_with_implementor(tt: proc_macro2::TokenTree, tokens: Vec<proc_macro2::TokenTree>) -> Vec<proc_macro2::TokenTree> {
    use proc_macro2::TokenTree as TT;
    match tt {
        TT::Ident(i) if i.to_string().as_str().eq("Implementor") => tokens,
        TT::Group(g) => vec![proc_macro2::Group::new(g.delimiter(), proc_macro2::TokenStream::from_iter(g.stream().into_iter().flat_map(|t| replace_with_implementor(t, tokens.clone())))).into()],
        t => vec![t]
    }
}

#[proc_macro]
pub fn for_types(item: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(item as ForImplsArgs);

    proc_macro2::TokenStream::from_iter(input.types.iter().flat_map(|im| {
        input.code.clone().into_iter().flat_map(|t| {
            replace_with_implementor(t, im.to_token_stream().into_iter().collect::<Vec<_>>())
        })
    })).into()
}