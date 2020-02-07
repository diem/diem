// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Macros for the VM file format and related structures.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, parse_str, Data, DataEnum, DeriveInput, Error, Field, Fields, FieldsUnnamed,
    Result, Type, Variant,
};

/// Derives the IR representation of the bytecode.
///
/// This will cause these macros to be defined:
///
/// * `make_ir_bytecode!`: Generate the IR bytecode representation.
#[proc_macro_derive(IRBytecode)]
pub fn derive_ir_bytecode(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match derive_ir_bytecode_impl(input) {
        Ok(token_stream) => token_stream.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn derive_ir_bytecode_impl(input: DeriveInput) -> Result<TokenStream> {
    let input_span = input.span();

    let data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(_) | Data::Union(_) => {
            return Err(Error::new(
                input_span,
                "#[derive(IRBytecode)] can only be used on an enum",
            ))
        }
    };

    let gen = IRBytecodeGen::new(data)?;
    gen.make_ir_enum_macro()
}

struct IRBytecodeGen<'a> {
    data: &'a DataEnum,
}

impl<'a> IRBytecodeGen<'a> {
    fn new(data: &'a DataEnum) -> Result<Self> {
        // Discriminants and named fields aren't supported.
        for variant in &data.variants {
            if variant.discriminant.is_some() {
                return Err(Error::new_spanned(
                    variant,
                    "#[derive(IRBytecode)] not supported for enums with discriminants",
                ));
            }
            if let Fields::Named(_) = variant.fields {
                return Err(Error::new_spanned(
                    variant,
                    "#[derive(IRBytecode)] not supported for named fields",
                ));
            }
        }

        Ok(Self { data })
    }

    fn make_ir_enum_macro(&self) -> Result<TokenStream> {
        let variants: Vec<_> = self
            .data
            .variants
            .iter()
            .map(|variant| {
                Ok(Variant {
                    attrs: variant.attrs.clone(),
                    ident: variant.ident.clone(),
                    fields: Self::ir_enum_fields(&variant.fields)?,
                    discriminant: None,
                })
            })
            .collect::<Result<_>>()?;
        Ok(quote! {
            #[macro_export]
            macro_rules! make_ir_bytecode {
                ($id: ident) => {
                    /// An intermediate representation of the Move VM's bytecode.
                    ///
                    /// This is isomorphic to the file format bytecode.
                    #[derive(Clone, Debug, PartialEq)]
                    pub enum $id {
                        #(#variants),*
                    }
                }
            }
        })
    }

    fn ir_enum_fields(fields: &Fields) -> Result<Fields> {
        let fields_out = match fields {
            Fields::Unnamed(fields) => Fields::Unnamed(FieldsUnnamed {
                paren_token: fields.paren_token.clone(),
                unnamed: fields
                    .unnamed
                    .iter()
                    .map(|field| Self::ir_enum_field(field))
                    .collect::<Result<_>>()?,
            }),
            Fields::Unit => Fields::Unit,
            Fields::Named(_) => unreachable!("checked in MoveIRGen constructor"),
        };
        Ok(fields_out)
    }

    fn ir_enum_field(field: &Field) -> Result<Field> {
        let mut field_out: Field = field.clone();
        field_out.ty = match &field.ty {
            Type::Path(ty) => {
                if ty.qself.is_some() {
                    return Err(Error::new_spanned(
                        field,
                        "#[derive(IRBytecode)] only supports single idents",
                    ));
                }
                match ty.path.get_ident() {
                    Some(ident) => {
                        let ident_name = ident.to_string();
                        let name_out = match ident_name.as_str() {
                            "CodeOffset" => "Label",
                            "u8" => "u8",
                            "u64" => "u64",
                            "u128" => "u128",
                            "ByteArrayPoolIndex" => "ByteArray",
                            "AddressPoolIndex" => "AccountAddress",
                            "LocalIndex" => "Var",
                            "FunctionHandleIndex" => "FunctionCall",
                            "LocalsSignatureIndex" => "Vec<Type>",
                            "FieldDefinitionIndex" => "Field",
                            "StructDefinitionIndex" => "StructName",
                            other => {
                                return Err(Error::new_spanned(
                                    field,
                                    format!("#[derive(IRBytecode)] unable to map type '{}'", other),
                                ))
                            }
                        };
                        Type::Path(parse_str(name_out)?)
                    }
                    None => {
                        return Err(Error::new_spanned(
                            field,
                            "#[derive(IRBytecode)] only supports single idents",
                        ));
                    }
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    field,
                    "#[derive(IRBytecode)] only supports single idents",
                ));
            }
        };
        Ok(field_out)
    }
}
