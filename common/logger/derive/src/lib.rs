// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{format_ident, quote};
use syn::{
    parse_macro_input, AngleBracketedGenericArguments, Attribute, Data, DataStruct, DeriveInput,
    Fields, FieldsNamed, GenericArgument, Meta, MetaList, NestedMeta, Path, PathArguments,
    PathSegment, Type, TypePath,
};

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree.
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => named,
        _ => panic!("derive(Schema) only supports structs with named fields"),
    };
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields: Vec<StructField> = fields
        .iter()
        .map(|f| {
            let ty = f.ty.clone();
            let value_type = extract_attr(&f.attrs);

            let inner_ty = extract_internal_type(&ty).cloned();
            StructField {
                value_type,
                ident: f.ident.clone(),
                ty: f.ty.clone(),
                inner_ty,
            }
        })
        .collect();

    let setters = fields.iter().map(|f| {
        let ident = &f.ident;

        if let Some(ty) = &f.inner_ty {
            quote! {
                pub fn #ident(mut self, #ident: #ty) -> Self {
                    self.#ident = ::std::option::Option::Some(#ident);
                    self
                }
            }
        } else {
            let ty = &f.ty;
            quote! {
                pub fn #ident(mut self, #ident: #ty) -> Self {
                    self.#ident = #ident;
                    self
                }
            }
        }
    });

    // Calls to visit_pair
    let visitor = format_ident!("visitor");
    let from_serde = quote! { ::diem_logger::Value::from_serde };
    let from_display = quote! { ::diem_logger::Value::from_display };
    let from_debug = quote! { ::diem_logger::Value::from_debug };
    let key_new = quote! { ::diem_logger::Key::new };
    let visits = fields.iter().map(|f| {
        let ident = f.ident.as_ref().unwrap();
        let ident_str = ident.to_string();

        let from_fn = match f.value_type {
            Some(ValueType::Display) => &from_display,
            Some(ValueType::Debug) => &from_debug,
            Some(ValueType::Serde) | None => &from_serde,
        };
        if f.inner_ty.is_some() {
            quote! {
                if let Some(#ident) = &self.#ident {
                    #visitor.visit_pair(#key_new(#ident_str), #from_fn(#ident));
                }
            }
        } else {
            quote! {
                #visitor.visit_pair(#key_new(#ident_str), #from_fn(&self.#ident));
            }
        }
    });

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            #(#setters)*
        }

        impl #impl_generics ::diem_logger::Schema for #name #ty_generics #where_clause {
            fn visit(&self, visitor: &mut dyn ::diem_logger::Visitor) {
                #(#visits)*
            }
        }
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

#[derive(Debug)]
enum ValueType {
    Debug,
    Display,
    Serde,
}

#[derive(Debug)]
struct StructField {
    value_type: Option<ValueType>,
    ident: Option<Ident>,
    ty: Type,
    /// Indicates that the type is wrapped by an Option type
    inner_ty: Option<Type>,
}

fn extract_internal_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(TypePath {
        qself: None,
        path: Path { segments, .. },
    }) = ty
    {
        if let Some(PathSegment { ident, arguments }) = segments.first() {
            // Extract the inner type if it is "Option"
            if ident == "Option" {
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = arguments
                {
                    if let Some(GenericArgument::Type(ty)) = args.first() {
                        return Some(ty);
                    }
                }
            }
        }
    }

    None
}

fn extract_attr(attrs: &[Attribute]) -> Option<ValueType> {
    for attr in attrs {
        if let Meta::List(MetaList { path, nested, .. }) = attr.parse_meta().unwrap() {
            for segment in path.segments {
                // Only handle schema attrs
                if segment.ident == "schema" {
                    for meta in &nested {
                        let path = if let NestedMeta::Meta(Meta::Path(path)) = meta {
                            path
                        } else {
                            panic!("unsupported schema attribute");
                        };

                        match path.segments.first().unwrap().ident.to_string().as_ref() {
                            "debug" => return Some(ValueType::Debug),
                            "display" => return Some(ValueType::Display),
                            "serde" => return Some(ValueType::Serde),
                            _ => panic!("unsupported schema attribute"),
                        }
                    }
                }
            }
        }
    }

    None
}
