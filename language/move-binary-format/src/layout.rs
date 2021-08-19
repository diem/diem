// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access::ModuleAccess,
    file_format::{SignatureToken, StructDefinition, StructFieldInformation, StructHandleIndex},
    normalized::{Struct, Type},
    CompiledModule,
};
use anyhow::{anyhow, bail, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::{IdentStr, Identifier},
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::ModuleResolver,
    value::{MoveFieldLayout, MoveStructLayout, MoveTypeLayout},
};
use serde_reflection::{ContainerFormat, Format, Named, Registry};
use std::{
    cell::RefCell,
    collections::{btree_map::Entry, BTreeMap},
    fmt::Debug,
};

/// A persistent storage that can fetch the bytecode for a given module id
/// TODO: do we want to implement this in a way that allows clients to cache struct layouts?
pub trait GetModule {
    type Error: Debug;

    fn get_module_by_id(&self, id: &ModuleId) -> Result<Option<CompiledModule>, Self::Error>;
}

/// Simple in-memory module cache
pub struct ModuleCache<R: ModuleResolver> {
    cache: RefCell<BTreeMap<ModuleId, CompiledModule>>,
    resolver: R,
}

impl<R: ModuleResolver> ModuleCache<R> {
    pub fn new(resolver: R) -> Self {
        ModuleCache {
            cache: RefCell::new(BTreeMap::new()),
            resolver,
        }
    }

    pub fn add(&self, id: ModuleId, m: CompiledModule) {
        self.cache.borrow_mut().insert(id, m);
    }
}

impl<R: ModuleResolver> GetModule for ModuleCache<R> {
    type Error = anyhow::Error;

    fn get_module_by_id(&self, id: &ModuleId) -> Result<Option<CompiledModule>, Self::Error> {
        Ok(Some(match self.cache.borrow_mut().entry(id.clone()) {
            Entry::Vacant(entry) => {
                let module_bytes = self
                    .resolver
                    .get_module(id)
                    .map_err(|_| anyhow!("Failed to get module {:?}", id))?
                    .ok_or_else(|| anyhow!("Module {:?} doesn't exist", id))?;
                let module = CompiledModule::deserialize(&module_bytes)
                    .map_err(|_| anyhow!("Failure deserializing module {:?}", id))?;
                entry.insert(module.clone());
                module
            }
            Entry::Occupied(entry) => entry.get().clone(),
        }))
    }
}

/// Name of the Move `address` type in the serde registry
const ADDRESS: &str = "AccountAddress";

/// Name of the Move `signer` type in the serde registry
const SIGNER: &str = "Signer";

/// Type for building a registry of serde-reflection friendly struct layouts for Move types.
/// The layouts created by this type are intended to be passed to the serde-generate tool to create
/// struct bindings for Move types in source languages that use Move-based services.
/// The LayoutBuilder can operate in two modes: "deep" and "shallow".
/// In shallow mode, it will generate a single layout for the struct or type passed in by the user
/// (under the assumption that layouts for dependencies have been generated previously).
/// In deep mode, it will generate layouts for all of the (transitive) dependencies of the type passed
/// in, as well as layouts for the Move ground types like `address` and `signer`. The result is a
/// self-contained registry with no unbound typenames
pub struct SerdeLayoutBuilder<T> {
    registry: Registry,
    module_resolver: T,
    /// If true, operate in shallow mode; else, operate in deep mode
    shallow: bool,
}

impl<T: GetModule> SerdeLayoutBuilder<T> {
    /// Create a `LayoutBuilder` with an empty registry and deep layout resolution
    pub fn new(module_resolver: T) -> Self {
        Self {
            registry: Self::default_registry(),
            module_resolver,
            shallow: false,
        }
    }

    /// Create a `LayoutBuilder` with an empty registry and shallow layout resolution
    pub fn new_shallow(module_resolver: T) -> Self {
        Self {
            registry: BTreeMap::new(),
            module_resolver,
            shallow: true,
        }
    }

    /// Return a registry containing layouts for all the Move ground types (e.g., address)
    pub fn default_registry() -> Registry {
        let mut registry = BTreeMap::new();
        // add Move ground types to registry (address, signer)
        let address_layout = Box::new(Format::TupleArray {
            content: Box::new(Format::U8),
            size: AccountAddress::LENGTH,
        });
        registry.insert(
            ADDRESS.to_string(),
            ContainerFormat::NewTypeStruct(address_layout.clone()),
        );
        registry.insert(
            SIGNER.to_string(),
            ContainerFormat::NewTypeStruct(address_layout),
        );

        registry
    }

    /// Get the registry of layouts generated so far
    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Get the registry of layouts generated so far
    pub fn into_registry(self) -> Registry {
        self.registry
    }

    /// Add layouts for all types used in `t` to the registry
    pub fn build_type_layout(&mut self, t: TypeTag) -> Result<Format, T::Error> {
        self.build_normalized_type_layout(&Type::from(t), &Vec::new())
    }

    /// Add layouts for all types used in `t` to the registry
    pub fn build_struct_layout(&mut self, s: &StructTag) -> Result<Format, T::Error> {
        let serde_type_args = s
            .type_params
            .iter()
            .map(|t| self.build_type_layout(t.clone()))
            .collect::<Result<Vec<Format>, T::Error>>()?;
        let shallow = false;
        self.build_struct_layout_(&s.module_id(), &s.name, &serde_type_args, shallow)
    }

    fn build_normalized_type_layout(
        &mut self,
        t: &Type,
        input_type_args: &[Format],
    ) -> Result<Format, T::Error> {
        use Type::*;
        Ok(match t {
            Bool => Format::Bool,
            U8 => Format::U8,
            U64 => Format::U64,
            U128 => Format::U128,
            Address => Format::TypeName(ADDRESS.to_string()),
            Signer => Format::TypeName(SIGNER.to_string()),
            Struct {
                address,
                module,
                name,
                type_arguments,
            } => {
                let serde_type_args = type_arguments
                    .iter()
                    .map(|t| self.build_normalized_type_layout(t, input_type_args))
                    .collect::<Result<Vec<Format>, T::Error>>()?;
                let declaring_module = ModuleId::new(*address, module.clone());
                self.build_struct_layout_(&declaring_module, name, &serde_type_args, self.shallow)?
            }
            Vector(inner_t) => {
                if matches!(inner_t.as_ref(), U8) {
                    // specialize vector<u8> as bytes
                    Format::Bytes
                } else {
                    Format::Seq(Box::new(
                        self.build_normalized_type_layout(inner_t, input_type_args)?,
                    ))
                }
            }
            TypeParameter(i) => input_type_args[*i as usize].clone(),
            Reference(_) | MutableReference(_) => unreachable!(), // structs cannot store references
        })
    }

    fn build_struct_layout_(
        &mut self,
        module_id: &ModuleId,
        name: &Identifier,
        type_arguments: &[Format],
        shallow: bool,
    ) -> Result<Format, T::Error> {
        // build a human-readable name for the struct type. this should do the same thing as
        // StructTag::display(), but it's not easy to use that code here
        let generics: Vec<String> = type_arguments
            .iter()
            .map(|t| Self::print_format_type(t))
            .collect();
        let struct_key = format!("{}::{}<{}>", module_id, name, generics.join(","));
        if !shallow && !self.registry.contains_key(&struct_key) {
            let declaring_module = self
                .module_resolver
                .get_module_by_id(module_id)?
                .expect("Failed to resolve module");
            let def = declaring_module
                .find_struct_def_by_name(name)
                .unwrap_or_else(|| {
                    panic!(
                        "Could not find struct named {} in module {}",
                        name,
                        declaring_module.name()
                    )
                });
            let normalized_struct = Struct::new(&declaring_module, def).1;
            assert_eq!(
                normalized_struct.type_parameters.len(),
                type_arguments.len(),
                "Wrong number of type arguments for struct"
            );
            let fields = normalized_struct
                .fields
                .iter()
                .map(|f| {
                    self.build_normalized_type_layout(&f.type_, type_arguments)
                        .map(|value| Named {
                            name: f.name.to_string(),
                            value,
                        })
                })
                .collect::<Result<Vec<Named<Format>>, T::Error>>()?;

            let serde_struct = ContainerFormat::Struct(fields);
            // update cache
            self.registry.insert(struct_key.clone(), serde_struct);
        } // else, it's shallow mode or we already generated a layout for the type
        Ok(Format::TypeName(struct_key))
    }

    fn print_format_type(t: &Format) -> String {
        match t {
            Format::TypeName(s) => s.to_string(),
            Format::Bool => "bool".to_string(),
            Format::U8 => "u8".to_string(),
            Format::U64 => "u64".to_string(),
            Format::U128 => "u128".to_string(),
            Format::Bytes => "vector<u8>".to_string(),
            Format::Seq(inner) => format!("vector<{}>", Self::print_format_type(inner)),
            v => unimplemented!("Printing format value {:?}", v),
        }
    }
}

pub enum TypeLayoutBuilder {}
pub enum StructLayoutBuilder {}

#[derive(Copy, Clone, Debug)]
enum LayoutType {
    WithFields,
    Runtime,
}

impl TypeLayoutBuilder {
    /// Construct `TypeLayout` with fields from `t`.
    /// Panics if `resolver` cannot resolve a module whose types are referenced directly or
    /// transitively by `t`.
    pub fn build_with_fields(t: &TypeTag, resolver: &impl GetModule) -> Result<MoveTypeLayout> {
        Self::build(t, resolver, LayoutType::WithFields)
    }

    /// Construct a runtime `TypeLayout` from `t`.
    /// Panics if `resolver` cannot resolve a module whose types are referenced directly or
    /// transitively by `t`.
    pub fn build_runtime(t: &TypeTag, resolver: &impl GetModule) -> Result<MoveTypeLayout> {
        Self::build(t, resolver, LayoutType::Runtime)
    }

    fn build(
        t: &TypeTag,
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveTypeLayout> {
        use TypeTag::*;
        Ok(match t {
            Bool => MoveTypeLayout::Bool,
            U8 => MoveTypeLayout::U8,
            U64 => MoveTypeLayout::U64,
            U128 => MoveTypeLayout::U128,
            Address => MoveTypeLayout::Address,
            Signer => bail!("Type layouts cannot contain signer"),
            Vector(elem_t) => {
                MoveTypeLayout::Vector(Box::new(Self::build(elem_t, resolver, layout_type)?))
            }
            Struct(s) => {
                MoveTypeLayout::Struct(StructLayoutBuilder::build(s, resolver, layout_type)?)
            }
        })
    }

    fn build_from_signature_token(
        m: &CompiledModule,
        s: &SignatureToken,
        type_arguments: &[MoveTypeLayout],
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveTypeLayout> {
        use SignatureToken::*;
        Ok(match s {
            Vector(t) => MoveTypeLayout::Vector(Box::new(Self::build_from_signature_token(
                m,
                t,
                type_arguments,
                resolver,
                layout_type,
            )?)),
            Struct(shi) => MoveTypeLayout::Struct(StructLayoutBuilder::build_from_handle_idx(
                m,
                *shi,
                vec![],
                resolver,
                layout_type,
            )?),
            StructInstantiation(shi, type_actuals) => {
                let actual_layouts = type_actuals
                    .iter()
                    .map(|t| {
                        Self::build_from_signature_token(
                            m,
                            t,
                            type_arguments,
                            resolver,
                            layout_type,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?;
                MoveTypeLayout::Struct(StructLayoutBuilder::build_from_handle_idx(
                    m,
                    *shi,
                    actual_layouts,
                    resolver,
                    layout_type,
                )?)
            }
            TypeParameter(i) => type_arguments[*i as usize].clone(),
            Bool => MoveTypeLayout::Bool,
            U8 => MoveTypeLayout::U8,
            U64 => MoveTypeLayout::U64,
            U128 => MoveTypeLayout::U128,
            Address => MoveTypeLayout::Address,
            Signer => bail!("Type layouts cannot contain signer"),
            Reference(_) | MutableReference(_) => bail!("Type layouts cannot contain references"),
        })
    }
}

impl StructLayoutBuilder {
    pub fn build_runtime(s: &StructTag, resolver: &impl GetModule) -> Result<MoveStructLayout> {
        Self::build(s, resolver, LayoutType::Runtime)
    }

    pub fn build_with_fields(s: &StructTag, resolver: &impl GetModule) -> Result<MoveStructLayout> {
        Self::build(s, resolver, LayoutType::WithFields)
    }

    /// Construct an expanded `TypeLayout` from `s`.
    /// Panics if `resolver` cannot resolved a module whose types are referenced directly or
    /// transitively by `s`.
    fn build(
        s: &StructTag,
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveStructLayout> {
        let type_arguments = s
            .type_params
            .iter()
            .map(|t| TypeLayoutBuilder::build(t, resolver, layout_type))
            .collect::<Result<Vec<MoveTypeLayout>>>()?;
        Self::build_from_name(
            &s.module_id(),
            &s.name,
            type_arguments,
            resolver,
            layout_type,
        )
    }

    fn build_from_definition(
        m: &CompiledModule,
        s: &StructDefinition,
        type_arguments: Vec<MoveTypeLayout>,
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveStructLayout> {
        assert_eq!(
            m.struct_handle_at(s.struct_handle).type_parameters.len(),
            type_arguments.len(),
            "Wrong number of type arguments for struct",
        );
        match &s.field_information {
            StructFieldInformation::Native => {
                bail!("Can't extract fields for native struct")
            }
            StructFieldInformation::Declared(fields) => {
                let layouts = fields
                    .iter()
                    .map(|f| {
                        TypeLayoutBuilder::build_from_signature_token(
                            m,
                            &f.signature.0,
                            &type_arguments,
                            resolver,
                            layout_type,
                        )
                    })
                    .collect::<Result<Vec<MoveTypeLayout>>>()?;
                Ok(match layout_type {
                    LayoutType::Runtime => MoveStructLayout::Runtime(layouts),
                    LayoutType::WithFields => MoveStructLayout::WithFields(
                        fields
                            .iter()
                            .map(|f| m.identifier_at(f.name).to_owned())
                            .zip(layouts)
                            .map(|(name, layout)| MoveFieldLayout::new(name, layout))
                            .collect(),
                    ),
                })
            }
        }
    }

    fn build_from_name(
        declaring_module: &ModuleId,
        name: &IdentStr,
        type_arguments: Vec<MoveTypeLayout>,
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveStructLayout> {
        let m = resolver
            .get_module_by_id(declaring_module)
            .unwrap_or_else(|_| panic!("Error while resolving module {}", declaring_module))
            .unwrap();
        let def = m.find_struct_def_by_name(name).unwrap_or_else(|| {
            panic!(
                "Could not find struct named {} in module {}",
                name, declaring_module
            )
        });
        Self::build_from_definition(&m, def, type_arguments, resolver, layout_type)
    }

    fn build_from_handle_idx(
        m: &CompiledModule,
        s: StructHandleIndex,
        type_arguments: Vec<MoveTypeLayout>,
        resolver: &impl GetModule,
        layout_type: LayoutType,
    ) -> Result<MoveStructLayout> {
        if let Some(def) = m.find_struct_def(s) {
            // declared internally
            Self::build_from_definition(m, def, type_arguments, resolver, layout_type)
        } else {
            let handle = m.struct_handle_at(s);
            let name = m.identifier_at(handle.name);
            let declaring_module = m.module_id_for_handle(m.module_handle_at(handle.module));
            // declared externally
            Self::build_from_name(
                &declaring_module,
                name,
                type_arguments,
                resolver,
                layout_type,
            )
        }
    }
}
