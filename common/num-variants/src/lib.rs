// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Add an associated constant to an enum describing the number of variants it has.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate proc_macro;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Attribute, Data, DeriveInput, Error, Lit, Meta,
    MetaNameValue, Result,
};

/// Derives an associated constant with the number of variants this enum has.
///
/// The default constant name is `NUM_VARIANTS`. This can be customized with `#[num_variants =
/// "FOO")]`.
#[proc_macro_derive(NumVariants, attributes(num_variants))]
pub fn derive_num_variants(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_num_variants_impl(input) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error()),
    }
}

fn derive_num_variants_impl(input: DeriveInput) -> Result<TokenStream> {
    let input_span = input.span();
    let const_name = compute_const_name(input.attrs)?;

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let num_variants = compute_num_variants(&input.data, input_span)?;

    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            /// The number of variants in this enum.
            pub const #const_name: usize = #num_variants;
        }
    };

    Ok(expanded)
}

/// Computes the number of variants for an enum.
fn compute_num_variants(data: &Data, span: Span) -> Result<usize> {
    match data {
        Data::Enum(data) => Ok(data.variants.len()),
        Data::Struct(_) | Data::Union(_) => Err(Error::new(
            span,
            "#[derive(NumVariants)] can only be used on an enum",
        )),
    }
}

/// Computes the name of the constant.
fn compute_const_name(attrs: Vec<Attribute>) -> Result<Ident> {
    let mut const_names: Vec<_> = attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident("num_variants"))
        .map(|attr| match attr.parse_meta() {
            Ok(Meta::NameValue(MetaNameValue {
                lit: Lit::Str(lit), ..
            })) => Ok((attr.span(), lit.parse::<Ident>()?)),
            _ => Err(Error::new(
                attr.span(),
                "must be of the form #[num_variants = \"FOO\"]",
            )),
        })
        .collect::<Result<_>>()?;
    // There should only be zero or one constant names.

    match const_names.len() {
        0 => Ok(Ident::new("NUM_VARIANTS", Span::call_site())),
        1 => Ok(const_names.remove(0).1),
        _ => Err(Error::new(
            const_names.remove(1).0,
            "#[num_variants] attribute used multiple times",
        )),
    }
}
