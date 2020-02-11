// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Macro helpers for working with enums.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate proc_macro;

mod num_variants;

use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Attribute, DeriveInput, Error, Ident, Meta, MetaList, NestedMeta, Result,
};

/// Derives an associated constant with the number of variants this enum has.
///
/// The default constant name is `NUM_VARIANTS`. This can be customized with `#[num_variants =
/// "FOO")]`.
#[proc_macro_derive(NumVariants, attributes(num_variants))]
pub fn derive_num_variants(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match num_variants::derive_num_variants_impl(input) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

/// Retrieves the name of an identifier specified through an attribute.
fn compute_ident(attrs: Vec<Attribute>, attribute_name: &str) -> Result<Option<Ident>> {
    let mut idents: Vec<_> = attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident(attribute_name))
        .map(|attr| {
            let err_fn = || {
                Error::new(
                    attr.span(),
                    format!("must be of the form #[{}(FOO)]", attribute_name),
                )
            };
            match attr.parse_meta() {
                Ok(Meta::List(MetaList { nested, .. })) => {
                    if nested.len() == 1 {
                        match nested.first().unwrap() {
                            NestedMeta::Meta(nested) => match nested.path().get_ident() {
                                Some(ident) => Ok((attr.span(), ident.clone())),
                                None => Err(err_fn()),
                            },
                            _ => Err(err_fn()),
                        }
                    } else {
                        Err(err_fn())
                    }
                }
                _ => Err(err_fn()),
            }
        })
        .collect::<Result<_>>()?;

    // There should only be zero or one constant names.
    match idents.len() {
        0 => Ok(None),
        1 => Ok(Some(idents.remove(0).1)),
        _ => Err(Error::new(
            idents.remove(1).0,
            format!("#[{}] used multiple times", attribute_name),
        )),
    }
}
