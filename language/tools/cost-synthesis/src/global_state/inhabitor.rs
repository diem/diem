// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Random valid type inhabitant generation.
use libra_types::{account_address::AccountAddress, language_storage::ModuleId};
use move_core_types::identifier::Identifier;
use move_vm_runtime::{loaded_data::loaded_module::LoadedModule, MoveVM};
use move_vm_state::execution_context::SystemExecutionContext;
use move_vm_types::{
    loaded_data::{struct_def::StructDef, types::Type},
    values::*,
};
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use vm::{
    access::*,
    file_format::{
        MemberCount, ModuleHandle, SignatureToken, StructDefinition, StructDefinitionIndex,
        StructFieldInformation, StructHandleIndex, TableIndex,
    },
};

/// A wrapper around state that is used to generate random valid inhabitants for types.
pub struct RandomInhabitor<'txn> {
    /// The source of pseudo-randomness.
    gen: StdRng,

    /// Certain instructions require indices into the various tables within the module.
    /// We store a reference to the loaded module context that we are currently in so that we can
    /// generate valid references into these tables. When generating a module universe this is the
    /// root module that has pointers to all other modules.
    root_module: &'txn LoadedModule,

    /// The Move VM for all the modules in the universe. We need this in order to
    /// resolve struct and function handles to other modules other then the root module.
    move_vm: &'txn MoveVM,

    /// Context for resolution
    ctx: &'txn SystemExecutionContext<'txn>,

    /// A reverse lookup table to find the struct definition for a struct handle. Needed for
    /// generating an inhabitant for a struct SignatureToken. This is lazily populated.
    struct_handle_table: HashMap<ModuleId, HashMap<Identifier, StructDefinitionIndex>>,
}

impl<'txn> RandomInhabitor<'txn> {
    /// Create a new random type inhabitor.
    ///
    /// It initializes each of the internal resolution tables for structs and function handles to
    /// be empty.
    pub fn new(
        root_module: &'txn LoadedModule,
        move_vm: &'txn MoveVM,
        ctx: &'txn SystemExecutionContext<'txn>,
    ) -> Self {
        let seed: [u8; 32] = [0; 32];
        Self {
            gen: StdRng::from_seed(seed),
            root_module,
            move_vm,
            ctx,
            struct_handle_table: HashMap::new(),
        }
    }

    fn to_module_id(&self, module_handle: &ModuleHandle) -> ModuleId {
        let address = *self.root_module.address_at(module_handle.address);
        let name = self.root_module.identifier_at(module_handle.name);
        ModuleId::new(address, name.into())
    }

    fn next_u8(&mut self) -> u8 {
        self.gen.gen_range(0, u8::max_value())
    }

    fn next_u64(&mut self) -> u64 {
        u64::from(self.gen.gen_range(0, u32::max_value()))
    }

    fn next_u128(&mut self) -> u128 {
        u128::from(self.gen.gen_range(0, u32::max_value()))
    }

    fn next_bool(&mut self) -> bool {
        // Flip a coin
        self.gen.gen_bool(0.5)
    }

    fn next_addr(&mut self) -> AccountAddress {
        AccountAddress::new(self.gen.gen())
    }

    fn resolve_struct_handle(
        &mut self,
        struct_handle_index: StructHandleIndex,
    ) -> (
        &'txn LoadedModule,
        &'txn StructDefinition,
        StructDefinitionIndex,
    ) {
        let struct_handle = self.root_module.struct_handle_at(struct_handle_index);
        let struct_name = self.root_module.identifier_at(struct_handle.name);
        let module_handle = self.root_module.module_handle_at(struct_handle.module);
        let module_id = self.to_module_id(module_handle);
        let module = self
            .move_vm
            .get_loaded_module(&module_id, self.ctx)
            .expect("[Module Lookup] Runtime error while looking up module");
        let struct_def_idx = self
            .struct_handle_table
            .entry(module_id)
            .or_insert_with(|| {
                module
                    .struct_defs()
                    .iter()
                    .enumerate()
                    .map(|(struct_def_index, struct_def)| {
                        let handle = module.struct_handle_at(struct_def.struct_handle);
                        let name = module.identifier_at(handle.name).to_owned();
                        (
                            name,
                            StructDefinitionIndex::new(struct_def_index as TableIndex),
                        )
                    })
                    .collect()
            })
            .get(struct_name)
            .expect("[Struct Definition Lookup] Unable to get struct definition for struct handle");
        let struct_def = module.struct_def_at(*struct_def_idx);
        (module, struct_def, *struct_def_idx)
    }

    /// Build an inhabitant of the type given by `sig_token`. Note that as opposed to the
    /// inhabitant generation that is performed in the `StackGenerator` this does _not_ take the
    /// instruction and generates inhabitants in a semantically agnostic way.
    pub fn inhabit(&mut self, sig_token: &SignatureToken) -> (Type, Value) {
        match sig_token {
            SignatureToken::Bool => (Type::Bool, Value::bool(self.next_bool())),
            SignatureToken::U8 => (Type::U8, Value::u8(self.next_u8())),
            SignatureToken::U64 => (Type::U64, Value::u64(self.next_u64())),
            SignatureToken::U128 => (Type::U128, Value::u128(self.next_u128())),
            SignatureToken::Address => (Type::Address, Value::address(self.next_addr())),
            SignatureToken::Reference(_sig) | SignatureToken::MutableReference(_sig) => {
                panic!("cannot inhabit references");
            }
            SignatureToken::Struct(struct_handle_idx, _) => {
                assert!(self.root_module.struct_defs().len() > 1);
                let struct_definition = self
                    .root_module
                    .struct_def_at(self.resolve_struct_handle(*struct_handle_idx).2);
                let (num_fields, index) = match struct_definition.field_information {
                    StructFieldInformation::Native => {
                        panic!("[Struct Generation] Unexpected native struct")
                    }
                    StructFieldInformation::Declared {
                        field_count,
                        fields,
                    } => (field_count as usize, fields),
                };
                let fields = self
                    .root_module
                    .field_def_range(num_fields as MemberCount, index);
                let (layouts, values): (Vec<_>, Vec<_>) = fields
                    .iter()
                    .map(|field| {
                        self.inhabit(&self.root_module.type_signature_at(field.signature).0)
                    })
                    .unzip();
                (
                    Type::Struct(StructDef::new(layouts)),
                    Value::struct_(Struct::pack(values)),
                )
            }
            SignatureToken::Vector(_) => unimplemented!(),
            SignatureToken::TypeParameter(_) => unimplemented!(),
        }
    }
}
