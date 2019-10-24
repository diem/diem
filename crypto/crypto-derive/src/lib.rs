// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! This crate contains three types of derive macros:
//! - the `SilentDebug` and SilentDisplay macros are meant to be used on private key types, and
//!   elide their input for confidentiality.
//! - the `Deref` macro helps derive the canonical instances on new types.
//! - the derive macros for `libra_crypto::traits`, namely `ValidKey`, `PublicKey`, `PrivateKey`,
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
//! # #[macro_use] extern crate crypto-derive;
//! use libra_crypto::{
//!     hash::HashValue,
//!     bls12381::{BLS12381PrivateKey, BLS12381PublicKey, BLS12381Signature},
//!     ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
//! };
//! use libra_crypto_derive::{
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

mod unions;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};
use unions::*;

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
