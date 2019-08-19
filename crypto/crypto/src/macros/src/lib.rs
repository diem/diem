// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(SilentDisplay)]
pub fn silent_display(source: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(source).expect("Infallible");
    let name = &ast.ident;
    let gen = quote! {
        // In order to ensure that secrets are never leaked, Display is elided
        impl ::std::fmt::Display for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "<elided secret for {}>", stringify!(#name))
            }
        }
    };
    gen.into()
}

#[proc_macro_derive(SilentDebug)]
pub fn silent_debug(source: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(source).expect("Infallible");
    let name = &ast.ident;
    let gen = quote! {
        // In order to ensure that secrets are never leaked, Debug is elided
        impl ::std::fmt::Debug for #name {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(f, "<elided secret for {}>", stringify!(#name))
            }
        }
    };
    gen.into()
}
