// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access::ModuleAccess,
    file_format::{
        CompiledModule, FieldDefinition, FunctionHandle, Kind, SignatureToken, StructDefinition,
        StructFieldInformation, TypeParameterIndex,
    },
};
use move_core_types::{
    account_address::AccountAddress,
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};

/// Defines normalized representations of Move types, fields, kinds, structs, functions, and
/// modules. These representations are useful in situations that require require comparing
/// functions, resources, and types across modules. This arises in linking, compatibility checks
/// (e.g., "is it safe to deploy this new module without updating its dependents and/or restarting
/// genesis?"), defining schemas for resources stored on-chain, and (possibly in the future)
/// allowing module updates transactions.

/// A normalized version of `SignatureToken`, a type expression appearing in struct or function
/// declarations. Unlike `SignatureToken`s, `normalized::Type`s from different modules can safely be
/// compared.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Struct {
        address: AccountAddress,
        module: Identifier,
        name: Identifier,
        type_arguments: Vec<Type>,
    },
    Vector(Box<Type>),
    TypeParameter(TypeParameterIndex),
    Reference(Box<Type>),
    MutableReference(Box<Type>),
}

/// Normalized version of a `FieldDefinition`. The `name` is included even though it is
/// metadata that it is ignored by the VM. The reason: names are important to clients. We would
/// want a change from `Account { bal: u64, seq: u64 }` to `Account { seq: u64, bal: u64 }` to be
/// marked as incompatible. Not safe to compare without an enclosing `Struct`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub name: Identifier,
    pub type_: Type,
}

/// Normalized version of a `StructDefinition`. Not safe to compare without an associated
/// `ModuleId` or `Module`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Struct {
    pub name: Identifier,
    pub kind: Kind,
    pub type_parameters: Vec<Kind>,
    pub fields: Vec<Field>,
}

/// Normalized version of a `FunctionDefinition`. Not safe to compare without an associated
/// `ModuleId` or `Module`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionSignature {
    pub name: Identifier,
    pub type_parameters: Vec<Kind>,
    pub formals: Vec<Type>,
    pub ret: Vec<Type>,
}

/// Normalized version of a `CompiledModule`: its address, name, struct declarations, and public
/// function declarations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Module {
    pub address: AccountAddress,
    pub name: Identifier,
    pub structs: Vec<Struct>,
    pub public_functions: Vec<FunctionSignature>,
}

impl Module {
    /// Extract a normalized module from a `CompiledModule`. The module `m` should be verified.
    /// Nothing will break here if that is not the case, but there is little point in computing a
    /// normalized representation of a module that won't verify (since it can't be published).
    pub fn new(m: &CompiledModule) -> Self {
        let structs = m.struct_defs().iter().map(|d| Struct::new(m, d)).collect();
        let public_functions = m
            .function_defs()
            .iter()
            .filter_map(|f| {
                if f.is_public {
                    Some(FunctionSignature::new(m, m.function_handle_at(f.function)))
                } else {
                    None
                }
            })
            .collect();
        Self {
            address: *m.address(),
            name: m.name().to_owned(),
            structs,
            public_functions,
        }
    }
}

impl Type {
    /// Create a normalized `Type` for `SignatureToken` `s` in module `m`.
    pub fn new(m: &CompiledModule, s: &SignatureToken) -> Self {
        use SignatureToken::*;
        match s {
            Struct(shi) => {
                let handle = m.struct_handle_at(*shi);
                assert!(handle.type_parameters.is_empty(), "A struct with N type parameters should be encoded as StructModuleInstantiation with type_arguments = [TypeParameter(1), ..., TypeParameter(N)]");
                Type::Struct {
                    address: *m.address(),
                    module: m.name().to_owned(),
                    name: m.identifier_at(handle.name).to_owned(),
                    type_arguments: Vec::new(),
                }
            }
            StructInstantiation(shi, type_actuals) => Type::Struct {
                address: *m.address(),
                module: m.name().to_owned(),
                name: m.identifier_at(m.struct_handle_at(*shi).name).to_owned(),
                type_arguments: type_actuals.iter().map(|t| Type::new(m, t)).collect(),
            },
            Bool => Type::Bool,
            U8 => Type::U8,
            U64 => Type::U64,
            U128 => Type::U128,
            Address => Type::Address,
            Signer => Type::Signer,
            Vector(t) => Type::Vector(Box::new(Type::new(m, t))),
            TypeParameter(i) => Type::TypeParameter(*i),
            Reference(t) => Type::Reference(Box::new(Type::new(m, t))),
            MutableReference(t) => Type::MutableReference(Box::new(Type::new(m, t))),
        }
    }

    /// Return true if `self` is a closed type with no free type variables
    pub fn is_closed(&self) -> bool {
        use Type::*;
        match self {
            TypeParameter(_) => false,
            Bool => true,
            U8 => true,
            U64 => true,
            U128 => true,
            Address => true,
            Signer => true,
            Struct { type_arguments, .. } => type_arguments.iter().all(|t| t.is_closed()),
            Vector(t) | Reference(t) | MutableReference(t) => t.is_closed(),
        }
    }

    pub fn into_type_tag(self) -> Option<TypeTag> {
        use Type::*;
        Some(if self.is_closed() {
            match self {
                Reference(_) | MutableReference(_) => return None,
                Bool => TypeTag::Bool,
                U8 => TypeTag::U8,
                U64 => TypeTag::U64,
                U128 => TypeTag::U128,
                Address => TypeTag::Address,
                Signer => TypeTag::Signer,
                Vector(t) => TypeTag::Vector(Box::new(
                    t.into_type_tag()
                        .expect("Invariant violation: vector type argument contains reference"),
                )),
                Struct {
                    address,
                    module,
                    name,
                    type_arguments,
                } => TypeTag::Struct(StructTag {
                    address,
                    module,
                    name,
                    type_params: type_arguments
                        .into_iter()
                        .map(|t| {
                            t.into_type_tag().expect(
                                "Invariant violation: struct type argument contains reference",
                            )
                        })
                        .collect(),
                }),
                TypeParameter(_) => unreachable!(),
            }
        } else {
            return None;
        })
    }
}

impl Field {
    /// Create a `Field` for `FieldDefinition` `f` in module `m`.
    pub fn new(m: &CompiledModule, f: &FieldDefinition) -> Self {
        Field {
            name: m.identifier_at(f.name).to_owned(),
            type_: Type::new(m, &f.signature.0),
        }
    }
}

impl Struct {
    /// Create a `Struct` for `StructDefinition` `def` in module `m`. Panics if `def` is a
    /// a native struct definition.
    pub fn new(m: &CompiledModule, def: &StructDefinition) -> Self {
        let handle = m.struct_handle_at(def.struct_handle);
        let fields = match &def.field_information {
            StructFieldInformation::Native => panic!("Can't extract  for native struct"),
            StructFieldInformation::Declared(fields) => {
                fields.iter().map(|f| Field::new(m, f)).collect()
            }
        };
        Struct {
            name: m.identifier_at(handle.name).to_owned(),
            kind: if handle.is_nominal_resource {
                Kind::Resource
            } else {
                Kind::Copyable
            },
            type_parameters: handle.type_parameters.clone(),
            fields,
        }
    }
}

impl FunctionSignature {
    /// Create a `FunctionSignature` for `FunctionHandle` `f` in module `m`.
    pub fn new(m: &CompiledModule, f: &FunctionHandle) -> Self {
        FunctionSignature {
            name: m.identifier_at(f.name).to_owned(),
            type_parameters: f.type_parameters.clone(),
            formals: m
                .signature_at(f.parameters)
                .0
                .iter()
                .map(|s| Type::new(m, s))
                .collect(),
            ret: m
                .signature_at(f.return_)
                .0
                .iter()
                .map(|s| Type::new(m, s))
                .collect(),
        }
    }
}
