// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The `FromProto` and `IntoProto` macros provide an easy way to convert Rust struct to
//! corresponding Protobuf struct, or vice versa. For example:
//! ```text
//! #[derive(FromProto, IntoProto)]
//! #[ProtoType(ProtobufStruct)]
//! struct RustStruct {
//!     field1: Field1,
//!     field2: Field2,
//!     ...
//!     fieldn: FieldN,
//! }
//! ```
//!
//! It requires that all fields (`Field1`, `Field2`, ..., `FieldN`) implement `FromProto` trait if
//! we want to derive `FromProto` for `RustStruct`. Same for `IntoProto` trait.

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, token, DeriveInput, Result,
};

/// This struct describes how the outer attribute on the struct should look like. It is just a path
/// wrapped in parentheses. For example:
/// ```text
/// #[ProtoType(crate::proto::transaction::RawTransaction)]
/// ```
struct ProtoTypeAttribute {
    #[allow(dead_code)]
    paren_token: token::Paren,
    path: syn::Path,
}

impl Parse for ProtoTypeAttribute {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let paren_token = parenthesized!(content in input);
        let path = content.call(syn::Path::parse_mod_style)?;
        Ok(ProtoTypeAttribute { paren_token, path })
    }
}

#[proc_macro_derive(FromProto, attributes(ProtoType))]
pub fn derive_fromproto_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let attr_tokens = find_prototype_attribute(&input);
    let proto_attr = parse_macro_input!(attr_tokens as ProtoTypeAttribute);

    let name = input.ident;
    let proto_type = proto_attr.path;
    let body = gen_from_proto_body(&input.data);
    let expanded = quote! {
        impl ::libra_proto_conv::FromProto for #name {
            type ProtoType = #proto_type;

            fn from_proto(mut object: #proto_type) -> ::failure::Result<Self> {
                Ok(Self {
                    #body
                })
            }
        }
    };

    expanded.into()
}

#[proc_macro_derive(IntoProto, attributes(ProtoType))]
pub fn derive_intoproto_impl(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let attr_tokens = find_prototype_attribute(&input);
    let proto_attr = parse_macro_input!(attr_tokens as ProtoTypeAttribute);

    let name = input.ident;
    let proto_type = proto_attr.path;
    let body = gen_into_proto_body(&input.data);
    let expanded = quote! {
        impl ::libra_proto_conv::IntoProto for #name {
            type ProtoType = #proto_type;

            fn into_proto(self) -> Self::ProtoType {
                let mut out = Self::ProtoType::new();
                #body
                out
            }
        }
    };

    expanded.into()
}

/// Finds an outer attribute named `ProtoType`.
fn find_prototype_attribute(rust_struct: &syn::DeriveInput) -> TokenStream {
    let mut attrs: Vec<&syn::Attribute> = rust_struct
        .attrs
        .iter()
        .filter(|attr| match attr.style {
            syn::AttrStyle::Outer => attr
                .path
                .segments
                .pairs()
                .any(|segment| segment.value().ident == "ProtoType"),
            _ => false,
        })
        .collect();
    assert_eq!(
        attrs.len(),
        1,
        "There should be exactly one ProtoType attribute.",
    );
    attrs.remove(0).clone().tokens.into()
}

/// For a struct
/// ```text
/// struct X {
///     a: TypeA,
///     b: TypeB,
/// }
/// ```
/// the function body should look like:
/// ```text
/// fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
///     Ok(Self {
///         a: TypeA::from_proto(object.take_a())?,
///         b: TypeB::from_proto(object.take_b())?,
///     })
/// }
/// ```
fn gen_from_proto_body(data: &syn::Data) -> proc_macro2::TokenStream {
    match *data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    let var_name = name.as_ref().expect("Named fields should have a name.");
                    let retrieve_value = syn::Ident::new(
                        &if is_pritimive_type(&ty) {
                            format!("get_{}", var_name)
                        } else {
                            format!("take_{}", var_name)
                        },
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        #name: <#ty as ::libra_proto_conv::FromProto>::from_proto(object.#retrieve_value())?,
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!("Only named fields are supported."),
        },
        _ => unimplemented!("Only structs are supported."),
    }
}

fn is_pritimive_type(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if type_path.qself.is_some() {
                return false;
            }

            let path = &type_path.path;
            path.is_ident("u32")
                || path.is_ident("u64")
                || path.is_ident("i32")
                || path.is_ident("i64")
                || path.is_ident("bool")
        }
        _ => unimplemented!("Other types are not supported."),
    }
}

/// For a struct
/// ```text
/// struct X {
///     a: TypeA,
///     b: TypeB,
/// }
/// ```
/// the function body should look like:
/// ```text
/// fn into(self) -> Self::ProtoType {
///     let mut out = Self::ProtoType::new();
///     out.set_a(self.a.into_proto());
///     out.set_b(self.b.into_proto());
///     out
/// }
/// ```
fn gen_into_proto_body(data: &syn::Data) -> proc_macro2::TokenStream {
    match *data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    let ty = &f.ty;
                    let set_value = syn::Ident::new(
                        &format!("set_{}", name.as_ref().unwrap()),
                        proc_macro2::Span::call_site(),
                    );
                    quote! {
                        out.#set_value(<#ty as ::libra_proto_conv::IntoProto>::into_proto(self.#name));
                    }
                });
                quote! {
                    #(#recurse)*
                }
            }
            _ => unimplemented!("Only named fields are supported."),
        },
        _ => unimplemented!("Only structs are supported."),
    }
}
