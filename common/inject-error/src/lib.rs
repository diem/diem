// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Inject non-deterministic anyhow::Error on annotated functions, it requires the dependency of rand
//! and is only enabled when compiled with feature "inject-error" and **not** in cargo test.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

extern crate proc_macro;

use quote::quote;
use syn::{parse_macro_input, Expr, Lit};

/// Proc macro to specify non-deterministic error injection for functions.
/// Specify the chance with #[inject_error(probability = x)] where 0.0 <= x <= 1.0
/// When injected, the function would return anyhow!("Injected error") immediately.
/// See the example test.rs for usage.
#[proc_macro_attribute]
pub fn inject_error(
    attrs: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let expr = parse_macro_input!(attrs as syn::Expr);
    let chance: f64 = match expr {
        Expr::Assign(assign) => {
            match *assign.left {
                Expr::Path(path) => assert!(path.path.is_ident("probability"),),
                _ => panic!("Unexpected expression, use #[inject_error(probability = 0.1)]"),
            }
            match *assign.right {
                Expr::Lit(lit) => match lit.lit {
                    Lit::Float(float) => float.base10_parse().unwrap(),
                    _ => panic!("Unexpected expression, use #[inject_error(probability = 0.1)]"),
                },
                _ => panic!("Unexpected expression, use #[inject_error(probability = 0.1)]"),
            }
        }
        _ => panic!("Unexpected expression, use #[inject_error(probability = x)]"),
    };
    assert!(chance <= 1.0);
    assert!(chance >= 0.0);
    let chance = (chance * 100.0) as u64;
    let original_func = parse_macro_input!(input as syn::ItemFn);
    let mut injected_func = original_func.clone();
    let stmt0 = quote! {
        use rand::Rng;
    };
    let stmt1 = quote! {
       if rand::thread_rng().gen_range(0, 100) < #chance {
           Err(anyhow::anyhow!("Injected error"))?;
       }
    };
    injected_func
        .block
        .stmts
        .insert(0, syn::parse2(stmt0).unwrap());
    injected_func
        .block
        .stmts
        .insert(0, syn::parse2(stmt1).unwrap());
    let output = quote! {
        #[cfg(any(test, not(feature="inject-error")))]
        #original_func
        #[cfg(all(not(test), feature="inject-error"))]
        #injected_func
    };
    output.into()
}
