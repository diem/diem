// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use quote::quote;
use syn::{DataEnum, Ident};

pub fn parse_newtype_fields(item: &syn::DeriveInput) -> (syn::Type, proc_macro2::TokenStream) {
    let fields = match item.data {
        syn::Data::Struct(ref body) => body.fields.iter().collect::<Vec<&syn::Field>>(),
        _ => vec![],
    };

    let field_ty = match fields.len() {
        1 => Some(fields[0].ty.clone()),
        _ => None,
    };
    let field_ty = field_ty.expect("#[derive(Deref)] can only be used on structs with one field.");

    let field_name = match fields[0].ident {
        Some(ref ident) => quote!(#ident),
        None => quote!(0),
    };

    match field_ty {
        syn::Type::Reference(syn::TypeReference { elem, .. }) => (*elem, quote!(self.#field_name)),
        x => (x, quote!(&self.#field_name)),
    }
}

pub fn impl_enum_tryfrom(name: &Ident, variants: &DataEnum) -> proc_macro2::TokenStream {
    // the TryFrom dispatch
    let mut try_iter = variants.variants.iter();
    let first_variant = try_iter
        .next()
        .expect("#[derive(ValidCryptoMaterial] requires a non-empty enum.");
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
            type Error = libra_crypto::CryptoMaterialError;
            fn try_from(bytes: &[u8]) -> std::result::Result<#name, Self::Error> {
                #try_chain
            }
        }
    }
}

fn match_enum_to_bytes(name: &Ident, variants: &DataEnum) -> proc_macro2::TokenStream {
    // the ValidCryptoMaterial dispatch proper
    let mut match_arms = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        match_arms.extend(quote! {
            #name::#variant_ident(key) => key.to_bytes().to_vec(),
        });
    }
    match_arms
}

pub fn impl_enum_valid_crypto_material(name: &Ident, variants: &DataEnum) -> TokenStream {
    let mut try_from = impl_enum_tryfrom(name, variants);

    let to_bytes_arms = match_enum_to_bytes(name, variants);

    try_from.extend(quote! {

        impl libra_crypto::ValidCryptoMaterial for #name {
            fn to_bytes(&self) -> Vec<u8> {
                match self {
                    #to_bytes_arms
                }
            }
        }
    });
    try_from.into()
}

pub fn get_type_from_attrs(attrs: &[syn::Attribute], attr_name: &str) -> syn::Result<syn::LitStr> {
    attrs
        .iter()
        .find(|attr| attr.path.is_ident(attr_name))
        .map_or_else(
            || {
                Err(syn::Error::new(
                    proc_macro2::Span::call_site(),
                    format!("Could not find attribute {}", attr_name),
                ))
            },
            |attr| match attr.parse_meta()? {
                syn::Meta::NameValue(meta) => {
                    if let syn::Lit::Str(lit) = &meta.lit {
                        Ok(lit.clone())
                    } else {
                        Err(syn::Error::new_spanned(
                            meta,
                            &format!("Could not parse {} attribute", attr_name)[..],
                        ))
                    }
                }
                bad => Err(syn::Error::new_spanned(
                    bad,
                    &format!("Could not parse {} attribute", attr_name)[..],
                )),
            },
        )
}

pub fn impl_enum_publickey(
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
        impl libra_crypto::PublicKey for #name {
            type PrivateKeyMaterial = #pkt;
        }
    });
    res.into()
}

pub fn impl_enum_privatekey(
    name: &Ident,
    public_key_type: syn::LitStr,
    _variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = public_key_type.parse().unwrap();
    let res = quote! {
        impl libra_crypto::PrivateKey for #name {
            type PublicKeyMaterial = #pkt;
        }
    };
    res.into()
}

pub fn impl_enum_verifyingkey(
    name: &Ident,
    private_key_type: syn::LitStr,
    signature_type: syn::LitStr,
    _variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = private_key_type.parse().unwrap();
    let st: syn::Type = signature_type.parse().unwrap();
    let res = quote! {
        impl libra_crypto::VerifyingKey for #name {
            type SigningKeyMaterial = #pkt;
            type SignatureMaterial = #st;
        }
        impl libra_crypto::private::Sealed for #name {}
    };
    res.into()
}

pub fn impl_enum_signingkey(
    name: &Ident,
    public_key_type: syn::LitStr,
    signature_type: syn::LitStr,
    variants: &DataEnum,
) -> TokenStream {
    let pkt: syn::Type = public_key_type.parse().unwrap();
    let st: syn::Type = signature_type.parse().unwrap();

    let mut match_arms = quote! {};
    let mut match_arms_arbitrary = quote! {};
    for variant in variants.variants.iter() {
        let variant_ident = &variant.ident;

        match_arms.extend(quote! {
            #name::#variant_ident(key) => Self::SignatureMaterial::#variant_ident(key.sign_message(message)),
        });
        match_arms_arbitrary.extend(quote! {
            #name::#variant_ident(key) => Self::SignatureMaterial::#variant_ident(key.sign_arbitrary_message(message)),
        });
    }
    let res = quote! {
        impl libra_crypto::SigningKey for #name {
            type VerifyingKeyMaterial = #pkt;
            type SignatureMaterial = #st;

            fn sign_message(&self, message: &libra_crypto::HashValue) -> Self::SignatureMaterial {
                match self {
                    #match_arms
                }
            }

            #[cfg(test)]
            fn sign_arbitrary_message(&self, message: &[u8]) -> Self::SignatureMaterial {
                match self {
                    #match_arms_arbitrary
                }
            }
        }
        impl libra_crypto::private::Sealed for #name {}
    };
    res.into()
}

pub fn impl_enum_signature(
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

        impl libra_crypto::Signature for #name {
            type VerifyingKeyMaterial = #pub_kt;
            type SigningKeyMaterial = #priv_kt;

            fn verify(&self, message: &HashValue, public_key: &Self::VerifyingKeyMaterial) -> ::std::result::Result<(), libra_crypto::error::Error> {
                self.verify_arbitrary_msg(message.as_ref(), public_key)
            }

            fn verify_arbitrary_msg(&self, message: &[u8], public_key: &Self::VerifyingKeyMaterial) -> std::result::Result<(), libra_crypto::error::Error> {
                match (self, public_key) {
                    #match_arms
                    _ => libra_crypto::error::bail!(
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

        impl libra_crypto::private::Sealed for #name {}
    });
    res.into()
}
