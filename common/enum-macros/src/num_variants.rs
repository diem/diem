// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::compute_ident;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Error, Result};

pub(crate) fn derive_num_variants_impl(input: DeriveInput) -> Result<TokenStream> {
    let input_span = input.span();
    let const_name = compute_ident(input.attrs, "num_variants")?
        .unwrap_or_else(|| Ident::new("NUM_VARIANTS", Span::call_site()));

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
