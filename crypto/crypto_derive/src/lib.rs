// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate contains three types of derive macros:
//! - the `SilentDebug` and SilentDisplay macros are meant to be used on private key types, and
//!   elide their input for confidentiality.
//! - the `Deref` macro helps derive the canonical instances on new types.
//! - the derive macros for `crypto::traits`, namely `ValidKey`, `PublicKey`, `PrivateKey`,
//!   `VerifyingKey`, `SigningKey` and `Signature` are meant to be derived on simple unions of types
//!   implementing these traits.
//!
//! ## Unions of Signing Traits, in detail
//!
//! Those types typically come into play when you need to accept several
//! alternatives at runtime for several signature and verification schemes
//! (ex: BLS or EdDSA, see below). In this case, it is possible to declare
//! a triplet of enum types that each describe a 'sum type' (coproduct) of these
//! alternatives. This happens to be a signing scheme itself (it has
//! canonical signature, signing & verifying key types, and verifies all
//! expected properties by trivial dispatch).
//!
//! The macros below let you define this type of union trivially under two conditions:
//! - that the variant tags for the enum have the same name, i.e. if the BLS variant for the
//!   `SignatureUnion` is `SignatureUnion::BLS(BLS12381Signature)`, then the variant of the
//!   `PublicKeyUnion` for BLS must also be `PublicKeyUnion::BLS`,
//! - that you specify the associated types `PrivateKeyType`, `SignatureType` and `PublicKeyType`
//!   for each of the three unions. `PrivateKeyType` provides the value for the
//!   `VerifyingKeyMaterial` and `PublicKeyMaterial` associated types, `PublicKeyType` provides the
//!   valid for the `SigningKeyMaterial` and `PrivateKeyMaterial` associated types and
//!   `SignatureType` provides the value for the `SignatureMaterial` associated type.
//!
//! ## Example
//!
//! ```ignore
//! # #[macro_use] extern crate crypto_derive;
//! use crypto::{
//!     hash::HashValue,
//!     bls12381::{BLS12381PrivateKey, BLS12381PublicKey, BLS12381Signature},
//!     ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
//! };
//! use crypto_derive::{
//!     SilentDebug, PrivateKey, PublicKey, Signature, SigningKey, ValidKey, VerifyingKey,
//! };
//!
//! # fn main() {
//! /// Generic public key enum
//! #[derive(
//!     Debug, Clone, PartialEq, Eq, Hash, ValidKey, PublicKey, VerifyingKey,
//! )]
//! #[PrivateKeyType = "GenericPrivateKey"]
//! #[SignatureType = "GenericSignature"]
//! pub enum GenericPublicKey {
//!     /// Ed25519 public key
//!     Ed(Ed25519PublicKey),
//!     /// BLS12-381 public key
//!     BLS(BLS12381PublicKey),
//! }
//! /// Generic private key enum
//! #[derive(SilentDebug, ValidKey, PrivateKey, SigningKey)]
//! #[PublicKeyType = "GenericPublicKey"]
//! #[SignatureType = "GenericSignature"]
//! pub enum GenericPrivateKey {
//!     /// Ed25519 private key
//!     Ed(Ed25519PrivateKey),
//!     /// BLS12-381 private key
//!     BLS(BLS12381PrivateKey),
//! }
//! /// Generic signature enum
//! #[allow(clippy::large_enum_variant)]
//! #[derive(Clone, Debug, PartialEq, Eq, Hash, Signature)]
//! #[PrivateKeyType = "GenericPrivateKey"]
//! #[PublicKeyType = "GenericPublicKey"]
//! pub enum GenericSignature {
//!     /// Ed25519 signature
//!     Ed(Ed25519Signature),
//!     /// BLS12-381 signature
//!     BLS(BLS12381Signature),
//! }
//! # }
//! ```

extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Ident};

#[proc_macro_derive(SilentDisplay)]
pub fn silent_display(source: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(source).expect("Incorrect macro input");
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
    let ast: DeriveInput = syn::parse(source).expect("Incorrect macro input");
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

fn parse_newtype_fields(item: &syn::DeriveInput) -> (syn::Type, proc_macro2::TokenStream) {
    let fields = match item.data {
        syn::Data::Struct(ref body) => body.fields.iter().collect::<Vec<&syn::Field>>(),
        _ => panic!("#[derive(Deref)] can only be used on structs"),
    };

    let field_ty = match fields.len() {
        1 => Some(fields[0].ty.clone()),
        _ => None,
    };
    let field_ty = field_ty
        .unwrap_or_else(|| panic!("#[derive(Deref)] can only be used on structs with one field."));

    let field_name = match fields[0].ident {
        Some(ref ident) => quote!(#ident),
        None => quote!(0),
    };

    match field_ty {
        syn::Type::Reference(syn::TypeReference { elem, .. }) => {
            (*elem.clone(), quote!(self.#field_name))
        }
        x => (x, quote!(&self.#field_name)),
    }
}

#[proc_macro_derive(Deref)]
pub fn derive_deref(input: TokenStream) -> TokenStream {
    let item = syn::parse(input).expect("Incorrect macro input");
    let (field_ty, field_access) = parse_newtype_fields(&item);

    let name = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    quote!(
        impl #impl_generics ::std::ops::Deref for #name #ty_generics
        #where_clause
        {
            type Target = #field_ty;

            fn deref(&self) -> &Self::Target {
                #field_access
            }
        }
    )
    .into()
}

#[proc_macro_derive(ValidKey)]
pub fn derive_enum_validkey(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    match ast.data {
        Data::Enum(ref variants) => impl_enum_validkey(name, variants),
        Data::Struct(_) | Data::Union(_) => panic!("#[derive(ValidKey)] is only defined for enums"),
    }
}

fn impl_enum_tryfrom(name: &Ident, variants: &DataEnum) -> proc_macro2::TokenStream {
    // the TryFrom dispatch
    let mut try_iter = variants.variants.iter();
    let first_variant = try_iter
        .next()
        .expect("#[derive(ValidKey] requires a non-empty enum.");
    let first_variant_ident = &first_variant.ident;
    let first_variant_arg = &first_variant
        .fields
        .iter()
        .next()
        .expect("Unrecognized enum for key types")
        .ty;

    let mut try_chain = quote! {
        #first_variant_arg::try_from(bytes).and_then(|key| Ok(#name::#first_variant_ident(key)))
    };
    for variant in try_iter {
        let variant_ident = &variant.ident;
        let variant_arg = &variant
            .fields
            .iter()
            .next()
            .expect("Unrecognized enum for key types")
            .ty;
        try_chain.extend(quote!{
            .or_else(|_err| #variant_arg::try_from(bytes).and_then(|key| Ok(#name::#variant_ident(key))))
        })
    }

    quote! {
        impl core::convert::TryFrom<&[u8]> for #name {
            type Error = crypto::CryptoMaterialError;
            fn try_from(bytes: &[u8]) -> std::result::Result<#name, Self::Error> {
                #try_chain
            }
        }
    }
}

fn match_enum_to_bytes(name: &Ident, variants: &DataEnum) -> proc_macro2::TokenStream {
    // the ValidKey dispatch proper
    let mut match_arms = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        match_arms.extend(quote! {
            #name::#variant_ident(key) => key.to_bytes().to_vec(),
        });
    }
    match_arms
}

fn impl_enum_validkey(name: &Ident, variants: &DataEnum) -> TokenStream {
    let mut try_from = impl_enum_tryfrom(name, variants);

    let to_bytes_arms = match_enum_to_bytes(name, variants);

    try_from.extend(quote! {

        impl crypto::ValidKey for #name {
            fn to_bytes(&self) -> Vec<u8> {
                match self {
                    #to_bytes_arms
                }
            }
        }
    });
    try_from.into()
}

#[proc_macro_derive(PublicKey, attributes(PrivateKeyType))]
pub fn derive_enum_publickey(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let private_key_type = get_type_from_attrs(&ast.attrs, "PrivateKeyType").unwrap();
    match ast.data {
        Data::Enum(ref variants) => impl_enum_publickey(name, private_key_type, variants),
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(PublicKey)] is only defined for enums")
        }
    }
}

fn get_type_from_attrs(attrs: &[syn::Attribute], attr_name: &str) -> syn::Result<syn::LitStr> {
    for attr in attrs {
        if !attr.path.is_ident(attr_name) {
            continue;
        }
        match attr.parse_meta()? {
            syn::Meta::NameValue(meta) => {
                if let syn::Lit::Str(lit) = &meta.lit {
                    return Ok(lit.clone());
                } else {
                    return Err(syn::Error::new_spanned(
                        meta,
                        &format!("Could not parse {} attribute", attr_name)[..],
                    ));
                }
            }
            bad => {
                return Err(syn::Error::new_spanned(
                    bad,
                    &format!("Could not parse {} attribute", attr_name)[..],
                ))
            }
        }
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        format!("Could not find attribute {}", attr_name),
    ))
}

fn impl_enum_publickey(
    name: &Ident,
    private_key_type: syn::LitStr,
    variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = private_key_type.parse().unwrap();
    let mut from_match_arms = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        from_match_arms.extend(quote! {
            #pkt::#variant_ident(key) => #name::#variant_ident(key.into()),
        });
    }
    let mut res = quote! {
        impl From<&#pkt> for #name {
            fn from(public_key: &#pkt) -> Self {
                match public_key {
                    #from_match_arms
                }
            }
        }
    };
    res.extend(quote! {
        impl crypto::PublicKey for #name {
            type PrivateKeyMaterial = #pkt;
        }
    });
    res.into()
}

#[proc_macro_derive(PrivateKey, attributes(PublicKeyType))]
pub fn derive_enum_privatekey(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let public_key_type = get_type_from_attrs(&ast.attrs, "PublicKeyType").unwrap();
    match ast.data {
        Data::Enum(ref variants) => impl_enum_privatekey(name, public_key_type, variants),
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(PrivateKey)] is only defined for enums")
        }
    }
}

fn impl_enum_privatekey(
    name: &Ident,
    public_key_type: syn::LitStr,
    _variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = public_key_type.parse().unwrap();
    let res = quote! {
        impl crypto::PrivateKey for #name {
            type PublicKeyMaterial = #pkt;
        }
    };
    res.into()
}

#[proc_macro_derive(VerifyingKey, attributes(PrivateKeyType, SignatureType))]
pub fn derive_enum_verifyingkey(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let private_key_type = get_type_from_attrs(&ast.attrs, "PrivateKeyType").unwrap();
    let signature_type = get_type_from_attrs(&ast.attrs, "SignatureType").unwrap();
    match ast.data {
        Data::Enum(ref variants) => {
            impl_enum_verifyingkey(name, private_key_type, signature_type, variants)
        }
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(PrivateKey)] is only defined for enums")
        }
    }
}

fn impl_enum_verifyingkey(
    name: &Ident,
    private_key_type: syn::LitStr,
    signature_type: syn::LitStr,
    _variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = private_key_type.parse().unwrap();
    let st: syn::Type = signature_type.parse().unwrap();
    let res = quote! {
        impl crypto::VerifyingKey for #name {
            type SigningKeyMaterial = #pkt;
            type SignatureMaterial = #st;
        }
    };
    res.into()
}

#[proc_macro_derive(SigningKey, attributes(PublicKeyType, SignatureType))]
pub fn derive_enum_signingkey(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let public_key_type = get_type_from_attrs(&ast.attrs, "PublicKeyType").unwrap();
    let signature_type = get_type_from_attrs(&ast.attrs, "SignatureType").unwrap();
    match ast.data {
        Data::Enum(ref variants) => {
            impl_enum_signingkey(name, public_key_type, signature_type, variants)
        }
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(PrivateKey)] is only defined for enums")
        }
    }
}

fn impl_enum_signingkey(
    name: &Ident,
    public_key_type: syn::LitStr,
    signature_type: syn::LitStr,
    variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = public_key_type.parse().unwrap();
    let st: syn::Type = signature_type.parse().unwrap();

    let mut match_arms = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        match_arms.extend(quote! {
            #name::#variant_ident(key) => Self::SignatureMaterial::#variant_ident(key.sign_message(message)),
        });
    }
    let res = quote! {
        impl crypto::SigningKey for #name {
            type VerifyingKeyMaterial = #pkt;
            type SignatureMaterial = #st;

            fn sign_message(&self, message: &crypto::HashValue) -> Self::SignatureMaterial {
                match self {
                    #match_arms
                }
            }
        }
    };
    res.into()
}

#[proc_macro_derive(Signature, attributes(PublicKeyType, PrivateKeyType))]
pub fn derive_enum_signature(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let public_key_type = get_type_from_attrs(&ast.attrs, "PublicKeyType").unwrap();
    let private_key_type = get_type_from_attrs(&ast.attrs, "PrivateKeyType").unwrap();
    match ast.data {
        Data::Enum(ref variants) => {
            impl_enum_signature(name, public_key_type, private_key_type, variants)
        }
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(PrivateKey)] is only defined for enums")
        }
    }
}

fn impl_enum_signature(
    name: &Ident,
    public_key_type: syn::LitStr,
    private_key_type: syn::LitStr,
    variants: &DataEnum,
) -> TokenStream {
    let priv_kt: syn::Type = private_key_type.parse().unwrap();
    let pub_kt: syn::Type = public_key_type.parse().unwrap();
    let mut res = impl_enum_tryfrom(name, variants);
    let to_bytes_arms = match_enum_to_bytes(name, variants);

    let mut match_arms = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        match_arms.extend(quote! {
            (#name::#variant_ident(sig), #pub_kt::#variant_ident(pk)) => {
                sig.verify_arbitrary_msg(message, pk)
            }
        })
    }

    res.extend(quote! {

        impl crypto::Signature for #name {
            type VerifyingKeyMaterial = #pub_kt;
            type SigningKeyMaterial = #priv_kt;

            fn verify(&self, message: &HashValue, public_key: &Self::VerifyingKeyMaterial) -> ::std::result::Result<(), ::failure::Error> {
                self.verify_arbitrary_msg(message.as_ref(), public_key)
            }

            fn verify_arbitrary_msg(&self, message: &[u8], public_key: &Self::VerifyingKeyMaterial) -> std::result::Result<(), ::failure::Error> {
                match (self, public_key) {
                    #match_arms
                    _ => ::failure::bail!(
                        "provided the wrong alternative in {:?}!",
                        (self, public_key)
                    ),
                }
            }

            fn to_bytes(&self) -> Vec<u8> {
                match self {
                    #to_bytes_arms
                }
            }
        }
    });
    res.into()
}
