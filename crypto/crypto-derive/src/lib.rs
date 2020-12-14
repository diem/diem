// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

//! # Derive macros for crypto operations
//! This crate contains four types of derive macros:
//!
//! - the `SilentDebug` and SilentDisplay macros are meant to be used on private key types, and
//!   elide their input for confidentiality.
//! - the `Deref` macro helps derive the canonical instances on new types.
//! - the derive macros for `diem_crypto::traits`, namely `ValidCryptoMaterial`, `PublicKey`, `PrivateKey`,
//!   `VerifyingKey`, `SigningKey` and `Signature` are meant to be derived on simple unions of types
//!   implementing these traits.
//! - the derive macro for `diem_crypto::hash::CryptoHasher`, which defines
//!   the domain-separation hasher structures described in `diem_crypto::hash`
//!   (look there for details). This derive macro has for sole difference that it
//!   automatically picks a unique salt for you, using the Serde name. For a container `Foo`,
//!   this is usually equivalent to:
//!   ```ignore
//!   define_hasher! {
//!    (
//!         FooHasher,
//!         FOO_HASHER,
//!         b"Foo"
//!     )
//!   }
//!   ```
//!
//! # Unions of Signing Traits, in detail
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
//! use diem_crypto::{
//!     hash::HashValue,
//!     bls12381::{BLS12381PrivateKey, BLS12381PublicKey, BLS12381Signature},
//!     ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
//! };
//! use diem_crypto_derive::{
//!     SilentDebug, PrivateKey, PublicKey, Signature, SigningKey, ValidCryptoMaterial, VerifyingKey,
//! };
//!
//! /// Generic public key enum
//! #[derive(
//!     Debug, Clone, PartialEq, Eq, Hash, ValidCryptoMaterial, PublicKey, VerifyingKey,
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
//! #[derive(SilentDebug, ValidCryptoMaterial, PrivateKey, SigningKey)]
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
//! ```

#![forbid(unsafe_code)]

extern crate proc_macro;

mod hasher;
mod unions;

use hasher::camel_to_snake;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::iter::FromIterator;
use syn::{parse_macro_input, parse_quote, Data, DeriveInput, Ident};
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

/// Deserialize from a human readable format where applicable
#[proc_macro_derive(DeserializeKey)]
pub fn deserialize_key(source: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(source).expect("Incorrect macro input");
    let name = &ast.ident;
    let name_string = name.to_string();
    let gen = quote! {
        impl<'de> ::serde::Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {
                if deserializer.is_human_readable() {
                    let encoded_key = <String>::deserialize(deserializer)?;
                    ValidCryptoMaterialStringExt::from_encoded_string(encoded_key.as_str())
                        .map_err(<D::Error as ::serde::de::Error>::custom)
                } else {
                    // In order to preserve the Serde data model and help analysis tools,
                    // make sure to wrap our value in a container with the same name
                    // as the original type.
                    #[derive(::serde::Deserialize)]
                    #[serde(rename = #name_string)]
                    struct Value<'a>(&'a [u8]);

                    let value = Value::deserialize(deserializer)?;
                    #name::try_from(value.0).map_err(|s| {
                        <D::Error as ::serde::de::Error>::custom(format!("{} with {}", s, #name_string))
                    })
                }
            }
        }
    };
    gen.into()
}

/// Serialize into a human readable format where applicable
#[proc_macro_derive(SerializeKey)]
pub fn serialize_key(source: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(source).expect("Incorrect macro input");
    let name = &ast.ident;
    let name_string = name.to_string();
    let gen = quote! {
        impl ::serde::Serialize for #name {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                if serializer.is_human_readable() {
                    self.to_encoded_string()
                        .map_err(<S::Error as ::serde::ser::Error>::custom)
                        .and_then(|str| serializer.serialize_str(&str[..]))
                } else {
                    // See comment in deserialize_key.
                    serializer.serialize_newtype_struct(
                        #name_string,
                        serde_bytes::Bytes::new(&ValidCryptoMaterial::to_bytes(self).as_slice()),
                    )
                }
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

#[proc_macro_derive(ValidCryptoMaterial)]
pub fn derive_enum_valid_crypto_material(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    match ast.data {
        Data::Enum(ref variants) => impl_enum_valid_crypto_material(name, variants),
        Data::Struct(_) | Data::Union(_) => {
            panic!("#[derive(ValidCryptoMaterial)] is only defined for enums")
        }
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

// There is a unit test for this logic in the crypto crate, at
// diem_crypto::unit_tests::cryptohasher — you may have to modify it if you
// edit the below.
#[proc_macro_derive(CryptoHasher)]
pub fn hasher_dispatch(input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as DeriveInput);
    let hasher_name = Ident::new(
        &format!("{}Hasher", &item.ident.to_string()),
        Span::call_site(),
    );
    let snake_name = camel_to_snake(&item.ident.to_string());
    let static_seed_name = Ident::new(
        &format!("{}_SEED", snake_name.to_uppercase()),
        Span::call_site(),
    );

    let static_hasher_name = Ident::new(
        &format!("{}_HASHER", snake_name.to_uppercase()),
        Span::call_site(),
    );
    let type_name = &item.ident;
    let param = if item.generics.params.is_empty() {
        quote!()
    } else {
        let args = proc_macro2::TokenStream::from_iter(
            std::iter::repeat(quote!(())).take(item.generics.params.len()),
        );
        quote!(<#args>)
    };

    let out = quote!(
        /// Cryptographic hasher for an BCS-serializable #item
        #[derive(Clone)]
        pub struct #hasher_name(diem_crypto::hash::DefaultHasher);

        static #static_seed_name: diem_crypto::_once_cell::sync::OnceCell<[u8; 32]> = diem_crypto::_once_cell::sync::OnceCell::new();

        impl #hasher_name {
            fn new() -> Self {
                let name = diem_crypto::_serde_name::trace_name::<#type_name #param>()
                    .expect("The `CryptoHasher` macro only applies to structs and enums");
                #hasher_name(
                    diem_crypto::hash::DefaultHasher::new(&name.as_bytes()))
            }
        }

        static #static_hasher_name: diem_crypto::_once_cell::sync::Lazy<#hasher_name> =
            diem_crypto::_once_cell::sync::Lazy::new(|| #hasher_name::new());


        impl std::default::Default for #hasher_name
        {
            fn default() -> Self {
                #static_hasher_name.clone()
            }
        }

        impl diem_crypto::hash::CryptoHasher for #hasher_name {
            fn seed() -> &'static [u8; 32] {
                #static_seed_name.get_or_init(|| {
                    let name = diem_crypto::_serde_name::trace_name::<#type_name #param>()
                        .expect("The `CryptoHasher` macro only applies to structs and enums.").as_bytes();
                    diem_crypto::hash::DefaultHasher::prefixed_hash(&name)
                })
            }

            fn update(&mut self, bytes: &[u8]) {
                self.0.update(bytes);
            }

            fn finish(self) -> diem_crypto::hash::HashValue {
                self.0.finish()
            }
        }

        impl std::io::Write for #hasher_name {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                use diem_crypto::hash::CryptoHasher;

                self.0.update(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

    );
    out.into()
}

#[proc_macro_derive(BCSCryptoHash)]
pub fn bcs_crypto_hash_dispatch(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    let hasher_name = Ident::new(&format!("{}Hasher", &name.to_string()), Span::call_site());
    let error_msg = syn::LitStr::new(
        &format!("BCS serialization of {} should not fail", name.to_string()),
        Span::call_site(),
    );
    let generics = add_trait_bounds(ast.generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let out = quote!(
        impl #impl_generics diem_crypto::hash::CryptoHash for #name #ty_generics #where_clause {
            type Hasher = #hasher_name;

            fn hash(&self) -> diem_crypto::hash::HashValue {
                use diem_crypto::hash::CryptoHasher;

                let mut state = Self::Hasher::default();
                bcs::serialize_into(&mut state, &self).expect(#error_msg);
                state.finish()
            }
        }
    );
    out.into()
}

fn add_trait_bounds(mut generics: syn::Generics) -> syn::Generics {
    for param in generics.params.iter_mut() {
        if let syn::GenericParam::Type(type_param) = param {
            type_param.bounds.push(parse_quote!(Serialize));
        }
    }
    generics
}
