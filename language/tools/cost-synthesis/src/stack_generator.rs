// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Random valid stack state generation.
//!
//! This module encodes the stack evolution logic that is used to evolve the execution stack from
//! one iteration to the next while synthesizing the cost of an instruction.
use crate::{
    bytecode_specifications::{frame_transition_info::frame_transitions, stack_transition_info::*},
    common::*,
};
use libra_types::{
    account_address::{AccountAddress, ADDRESS_LENGTH},
    byte_array::ByteArray,
    language_storage::ModuleId,
};
use move_core_types::identifier::Identifier;
use move_vm_runtime::{
    interpreter::InterpreterForCostSynthesis, loaded_data::loaded_module::LoadedModule, MoveVM,
};
use move_vm_state::execution_context::{ExecutionContext, SystemExecutionContext};
use move_vm_types::values::*;
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use vm::{
    access::*,
    errors::VMResult,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, FieldDefinitionIndex,
        FunctionDefinition, FunctionDefinitionIndex, FunctionHandleIndex, FunctionSignature,
        LocalIndex, MemberCount, ModuleHandle, SignatureToken, StructDefinition,
        StructDefinitionIndex, StructFieldInformation, StructHandleIndex, TableIndex,
        NO_TYPE_ACTUALS,
    },
    gas_schedule::{AbstractMemorySize, GasAlgebra, GasCarrier},
};

/// Specifies the data to be applied to the execution stack for the next valid stack state.
///
/// This contains directives and information that is used to evolve the execution stack of the VM
/// in the `stack_transition_function`.
pub struct StackState<'txn> {
    /// The loaded module that should be loaded as the "current module" in the VM and the function
    /// index for the frame that we expect to be at the top of the execution stack. Set to `None`
    /// if no particular frame is required by the instruction.
    pub module_info: (&'txn LoadedModule, Option<FunctionDefinitionIndex>),

    /// The value stack that is required for the instruction.
    pub stack: Stack,

    /// A copy of the instruction. This will later be used to call into the VM.
    pub instr: Bytecode,

    /// For certain instructions the cost is variable on the size of the data being loaded. This
    /// holds the size of data that was generated so this can be taken into account when
    /// determining the cost per byte.
    pub size: AbstractMemorySize<GasCarrier>,

    /// A sparse mapping of local index to local value for the current function frame. This will
    /// be applied to the execution stack later on in the `stack_transition_function`.
    pub local_mapping: HashMap<LocalIndex, Value>,
}

impl<'txn> StackState<'txn> {
    /// Create a new stack state with the passed-in information.
    pub fn new(
        module_info: (&'txn LoadedModule, Option<FunctionDefinitionIndex>),
        stack: Stack,
        instr: Bytecode,
        size: AbstractMemorySize<GasCarrier>,
        local_mapping: HashMap<LocalIndex, Value>,
    ) -> Self {
        Self {
            module_info,
            stack,
            instr,
            size,
            local_mapping,
        }
    }
}

/// A wrapper around the instruction being synthesized. Holds the internal state that is
/// used to generate random valid stack states.
pub struct RandomStackGenerator<'txn> {
    /// The account address that all resources will be published under
    account_address: &'txn AccountAddress,

    /// The number of iterations that this instruction will be run for. Used to implement the
    /// `Iterator` trait.
    iters: u16,

    /// The source of pseudo-randomness.
    gen: StdRng,

    /// Certain instructions require indices into the various tables within the module.
    /// We store a reference to the loaded module context that we are currently in so that we can
    /// generate valid references into these tables. When generating a module universe this is the
    /// root module that has pointers to all other modules.
    root_module: &'txn LoadedModule,

    /// The MoveVM instance, It contains all of the modules in the universe and used for
    /// runtime and loading operations.
    move_vm: &'txn MoveVM,

    /// Context for resolution
    ctx: SystemExecutionContext<'txn>,

    /// The bytecode instruction for which stack states are generated.
    op: Bytecode,

    /// The maximum size of the generated value stack.
    max_stack_size: u64,

    /// Cursor into the address pool. Used for the generation of random addresses.  We use this since we need the
    /// addresses to be unique, and we don't want a mutable reference into the underlying `root_module`.
    address_pool_index: TableIndex,

    /// A reverse lookup table to find the struct definition for a struct handle. Needed for
    /// generating an inhabitant for a struct SignatureToken. This is lazily populated.
    /// NB: The `StructDefinitionIndex`s in this table are w.r.t. the module that is given by the
    /// `ModuleId` and _not_ the `root_module`.
    struct_handle_table: HashMap<ModuleId, HashMap<Identifier, StructDefinitionIndex>>,

    /// A reverse lookup table for each code module that allows us to resolve function handles to
    /// function definitions. Also lazily populated.
    function_handle_table: HashMap<ModuleId, HashMap<Identifier, FunctionDefinitionIndex>>,

    /// The tables sizes that we have for address and string pools
    table_size: u16,
}

impl<'txn> RandomStackGenerator<'txn> {
    /// Create a new random stack state generator.
    ///
    /// It initializes each of the internal resolution tables for structs and function handles to
    /// be empty.
    pub fn new(
        account_address: &'txn AccountAddress,
        root_module: &'txn LoadedModule,
        move_vm: &'txn MoveVM,
        ctx: SystemExecutionContext<'txn>,
        op: &Bytecode,
        max_stack_size: u64,
        iters: u16,
    ) -> Self {
        let seed: [u8; 32] = [0; 32];
        Self {
            gen: StdRng::from_seed(seed),
            op: op.clone(),
            account_address,
            max_stack_size,
            root_module,
            move_vm,
            ctx,
            iters,
            table_size: iters,
            address_pool_index: 0,
            struct_handle_table: HashMap::new(),
            function_handle_table: HashMap::new(),
        }
    }

    fn to_module_id(&self, module_handle: &ModuleHandle) -> ModuleId {
        let address = *self.root_module.address_at(module_handle.address);
        let name = self.root_module.identifier_at(module_handle.name);
        ModuleId::new(address, name.into())
    }

    // Determines if the instruction gets its type/instruction info from the stack type
    // transitions, or from the type signatures available in the module(s).
    fn is_module_specific_op(&self) -> bool {
        use Bytecode::*;
        match self.op {
            MoveToSender(_, _)
            | MoveFrom(_, _)
            | ImmBorrowGlobal(_, _)
            | MutBorrowGlobal(_, _)
            | Exists(_, _)
            | Unpack(_, _)
            | Pack(_, _)
            | Call(_, _) => true,
            CopyLoc(_) | MoveLoc(_) | StLoc(_) | MutBorrowLoc(_) | ImmBorrowLoc(_)
            | ImmBorrowField(_) | MutBorrowField(_) => true,
            _ => false,
        }
    }

    // TODO: merge the following three.
    fn next_u8(&mut self, stk: &[Value]) -> u8 {
        if self.op == Bytecode::Sub && !stk.is_empty() {
            let peek: VMResult<u8> = stk
                .last()
                .expect("[Next Integer] The impossible happened: the value stack became empty while still full.")
                .copy_value()
                .value_as();
            self.gen.gen_range(
                0,
                peek.expect("[Next Integer] Unable to cast peeked stack value to a u8."),
            )
        } else {
            self.gen.gen_range(0, u8::max_value())
        }
    }

    fn next_u64(&mut self, stk: &[Value]) -> u64 {
        if self.op == Bytecode::Sub && !stk.is_empty() {
            let peek: VMResult<u64> = stk
                .last()
                .expect("[Next Integer] The impossible happened: the value stack became empty while still full.")
                .copy_value()
                .value_as();
            self.gen.gen_range(
                0,
                peek.expect("[Next Integer] Unable to cast peeked stack value to a u64."),
            )
        } else {
            u64::from(self.gen.gen_range(0, u32::max_value()))
        }
    }

    fn next_u128(&mut self, stk: &[Value]) -> u128 {
        if self.op == Bytecode::Sub && !stk.is_empty() {
            let peek: VMResult<u128> = stk
                .last()
                .expect("[Next Integer] The impossible happened: the value stack became empty while still full.")
                .copy_value()
                .value_as();
            self.gen.gen_range(
                0,
                peek.expect("[Next Integer] Unable to cast peeked stack value to a u128."),
            )
        } else {
            u128::from(self.gen.gen_range(0, u32::max_value()))
        }
    }

    fn next_bool(&mut self) -> bool {
        // Flip a coin
        self.gen.gen_bool(0.5)
    }

    fn next_bytearray(&mut self) -> ByteArray {
        let len: usize = self.gen.gen_range(1, BYTE_ARRAY_MAX_SIZE);
        let bytes: Vec<u8> = (0..len).map(|_| self.gen.gen::<u8>()).collect();
        ByteArray::new(bytes)
    }

    fn next_addr(&mut self, is_padding: bool) -> AccountAddress {
        if is_padding {
            AccountAddress::new(self.gen.gen())
        } else {
            let address = self
                .root_module
                .address_at(AddressPoolIndex::new(self.address_pool_index));
            self.address_pool_index = (self.address_pool_index + 1) % self.table_size;
            *address
        }
    }

    fn next_bounded_index(&mut self, bound: TableIndex) -> TableIndex {
        self.gen.gen_range(1, bound)
    }

    fn next_address_idx(&mut self) -> AddressPoolIndex {
        let len = self.root_module.address_pool().len();
        AddressPoolIndex::new(self.gen.gen_range(0, len) as TableIndex)
    }

    fn next_bytearray_idx(&mut self) -> ByteArrayPoolIndex {
        let len = self.root_module.byte_array_pool().len();
        ByteArrayPoolIndex::new(self.gen.gen_range(0, len) as TableIndex)
    }

    fn next_function_handle_idx(&mut self) -> FunctionHandleIndex {
        let table_idx =
            self.next_bounded_index(self.root_module.function_handles().len() as TableIndex);
        FunctionHandleIndex::new(table_idx)
    }

    fn next_resource(&mut self) -> StructDefinitionIndex {
        let resources: Vec<_> = self
            .root_module
            .struct_defs()
            .iter()
            .enumerate()
            .filter_map(|(idx, struct_def)| {
                let is_nominal_resource = self
                    .root_module
                    .struct_handle_at(struct_def.struct_handle)
                    .is_nominal_resource;
                if is_nominal_resource {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect();
        if resources.is_empty() {
            panic!("Every module must have at least one resource, but the root module doesn't have a resource type defined.");
        }
        let rand_resource_idx = self.gen.gen_range(0, resources.len());
        let struct_def_idx = resources[rand_resource_idx];
        StructDefinitionIndex::new(struct_def_idx as TableIndex)
    }

    fn next_stack_value(&mut self, stk: &[Value], is_padding: bool) -> Value {
        match self.gen.gen_range(0, 6) {
            0 => Value::u8(self.next_u8(stk)),
            1 => Value::u64(self.next_u64(stk)),
            2 => Value::u128(self.next_u128(stk)),
            3 => Value::bool(self.next_bool()),
            4 => Value::byte_array(self.next_bytearray()),
            _ => Value::address(self.next_addr(is_padding)),
        }
    }

    // Pick a random function, and random local within that function. Then generate an inhabitant
    // for that local's type.
    fn next_local_state(
        &mut self,
    ) -> (
        &'txn LoadedModule,
        LocalIndex,
        FunctionDefinitionIndex,
        Stack,
    ) {
        // We pick a random function from the module in which to store the local
        let function_handle_idx = self.next_function_handle_idx();
        let (module, function_definition, function_def_idx, function_sig) =
            self.resolve_function_handle(function_handle_idx);
        let type_sig = &module
            .locals_signature_at(function_definition.code.locals)
            .0;
        // Pick a random local within that function in which we'll store the local
        let local_index = self.gen.gen_range(0, type_sig.len());
        let type_tok = &type_sig[local_index];
        let stack_local = self.resolve_to_value(type_tok, &[]);
        let mut stack = vec![stack_local];
        for type_tok in function_sig.arg_types.iter() {
            stack.push(self.resolve_to_value(type_tok, &[]))
        }
        (module, local_index as LocalIndex, function_def_idx, stack)
    }

    fn fill_instruction_arg(&mut self) -> (Bytecode, usize) {
        use Bytecode::*;
        // For branching we need to know the size of the code within the top frame on the execution
        // stack (the frame that the instruction will be executing in) so that we don't jump off
        // the end of the function. Because of this, we need to get the frame that we're in first.
        // Since we only generate one instruction at a time, for branching instructions we know
        // that we won't be pushing any (non-default) frames on to the execution stack -- and
        // therefore that the function at `DEFAULT_FUNCTION_IDX` will be the top frame on the
        // execution stack. Because of this, we can safely pick the default function as our frame
        // here.
        let function_idx = FunctionDefinitionIndex::new(DEFAULT_FUNCTION_IDX);
        let frame_len = self
            .root_module
            .function_def_at(function_idx)
            .code
            .code
            .len();
        match self.op {
            BrTrue(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                (BrTrue(index as CodeOffset), 1)
            }
            BrFalse(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                (BrFalse(index as CodeOffset), 1)
            }
            Branch(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                (Branch(index as CodeOffset), 1)
            }
            LdU8(_) => {
                let i = self.next_u8(&[]);
                (LdU8(i), 1)
            }
            LdU64(_) => {
                let i = self.next_u64(&[]);
                (LdU64(i), 1)
            }
            LdU128(_) => {
                let i = self.next_u128(&[]);
                (LdU128(i), 1)
            }
            LdByteArray(_) => {
                let bytearray_idx = self.next_bytearray_idx();
                let bytearray_size = self.root_module.byte_array_at(bytearray_idx).len();
                (LdByteArray(bytearray_idx), bytearray_size)
            }
            LdAddr(_) => (LdAddr(self.next_address_idx()), ADDRESS_LENGTH),
            _ => (self.op.clone(), 0),
        }
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
            .get_loaded_module(&module_id, &self.ctx)
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

    fn resolve_function_handle(
        &mut self,
        function_handle_index: FunctionHandleIndex,
    ) -> (
        &'txn LoadedModule,
        &'txn FunctionDefinition,
        FunctionDefinitionIndex,
        &'txn FunctionSignature,
    ) {
        let function_handle = self.root_module.function_handle_at(function_handle_index);
        let function_signature = self
            .root_module
            .function_signature_at(function_handle.signature);
        let function_name = self.root_module.identifier_at(function_handle.name);
        let module_handle = self.root_module.module_handle_at(function_handle.module);
        let module_id = self.to_module_id(module_handle);
        let module = self
            .move_vm
            .get_loaded_module(&module_id, &self.ctx)
            .expect("[Module Lookup] Runtime error while looking up module");
        let function_def_idx = *self
            .function_handle_table
            .entry(module_id)
            .or_insert_with(|| {
                module
                    .function_defs()
                    .iter()
                    .enumerate()
                    .map(|(function_def_index, function_def)| {
                        let handle = module.function_handle_at(function_def.function);
                        let name = module.identifier_at(handle.name).to_owned();
                        (
                            name,
                            FunctionDefinitionIndex::new(function_def_index as TableIndex),
                        )
                    })
                    .collect()
            })
            .get(function_name)
            .unwrap();

        let function_def = module.function_def_at(function_def_idx);
        (module, function_def, function_def_idx, function_signature)
    }

    // Build an inhabitant of the type given by `sig_token`. We pass the current stack state in
    // since for certain instructions (...Sub) we need to generate number pairs that when
    // subtracted from each other do not cause overflow.
    fn resolve_to_value(&mut self, sig_token: &SignatureToken, stk: &[Value]) -> Value {
        match sig_token {
            SignatureToken::Bool => Value::bool(self.next_bool()),
            SignatureToken::U8 => Value::u8(self.next_u8(stk)),
            SignatureToken::U64 => Value::u64(self.next_u64(stk)),
            SignatureToken::U128 => Value::u128(self.next_u128(stk)),
            SignatureToken::Address => Value::address(self.next_addr(false)),
            SignatureToken::Reference(sig) | SignatureToken::MutableReference(sig) => {
                // TODO: The following code creates a dangling reference.
                // Find a better way to handle this.
                let mut locals = Locals::new(1);
                locals
                    .store_loc(0, self.resolve_to_value(sig, stk))
                    .unwrap();
                locals.borrow_loc(0).unwrap()
            }
            SignatureToken::ByteArray => Value::byte_array(self.next_bytearray()),
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
                    } => (field_count, fields),
                };
                let fields = self
                    .root_module
                    .field_def_range(num_fields as MemberCount, index);
                let values = fields.iter().map(|field| {
                    self.resolve_to_value(
                        &self.root_module.type_signature_at(field.signature).0,
                        stk,
                    )
                });
                Value::struct_(Struct::pack(values))
            }
            SignatureToken::Vector(_) => unimplemented!(),
            SignatureToken::TypeParameter(_) => unimplemented!(),
        }
    }

    // Generate starting state of the stack based upon the type transition in the call info table.
    fn generate_from_type(&mut self, typ: SignatureTy, stk: &[Value]) -> Value {
        let is_variable = typ.is_variable();
        let underlying = typ.underlying();
        // If the underlying type is a variable type, then we can choose any type that we want.
        let typ = if is_variable {
            let index = self.gen.gen_range(0, underlying.len());
            &underlying[index]
        } else {
            underlying
                .first()
                .expect("Unable to get underlying type for sigty in generate_from_type")
        };
        self.resolve_to_value(&typ.0, stk)
    }

    // Certain instructions require specific stack states; e.g. Pack() requires the correct number
    // and type of locals to already exist at the top of the value stack when the instruction is
    // encountered. We therefore need to generate the stack state for certain operations _not_ on
    // their call info, but on the possible calls that we could have in the module/other modules
    // that we are aware of.
    fn generate_from_module_info(&mut self) -> StackState<'txn> {
        use Bytecode::*;
        match &self.op {
            MoveToSender(_, _) => {
                let struct_handle_idx = self.next_resource();
                // We can just pick a random address -- this is incorrect by the bytecode semantics
                // (since we're moving to an account that doesn't exist), but since we don't need
                // correctness beyond this instruction it's OK.
                let struct_definition = self.root_module.struct_def_at(struct_handle_idx);
                let struct_stack = self.resolve_to_value(
                    &SignatureToken::Struct(struct_definition.struct_handle, vec![]),
                    &[],
                );
                let size = struct_stack.size();
                let stack = vec![struct_stack];
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    MoveToSender(struct_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            MoveFrom(_, _) => {
                let struct_handle_idx = self.next_resource();
                let addr = Value::address(*self.account_address);
                let size = addr.size();
                let stack = vec![addr];
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    MoveFrom(struct_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            MutBorrowGlobal(_, _) => {
                let struct_handle_idx = self.next_resource();
                let addr = Value::address(*self.account_address);
                let size = addr.size();
                let stack = vec![addr];
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    MutBorrowGlobal(struct_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            ImmBorrowGlobal(_, _) => {
                let struct_handle_idx = self.next_resource();
                let addr = Value::address(*self.account_address);
                let size = addr.size();
                let stack = vec![addr];
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    ImmBorrowGlobal(struct_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            Exists(_, _) => {
                let next_struct_handle_idx = self.next_resource();
                // Flip a coin to determine if the resource should exist or not.
                let addr = if self.next_bool() {
                    Value::address(*self.account_address)
                } else {
                    Value::address(self.next_addr(true))
                };
                let size = addr.size();
                let stack = vec![addr];
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    Exists(next_struct_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            Call(_, _) => {
                let function_handle_idx = self.next_function_handle_idx();
                let function_handle = self.root_module.function_handle_at(function_handle_idx);
                let function_sig = self
                    .root_module
                    .function_signature_at(function_handle.signature);
                let stack = function_sig
                    .arg_types
                    .iter()
                    .fold(Vec::new(), |mut acc, sig_tok| {
                        acc.push(self.resolve_to_value(sig_tok, &acc));
                        acc
                    });
                let size = stack.iter().fold(AbstractMemorySize::new(0), |acc, local| {
                    acc.add(local.size())
                });
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    Call(function_handle_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            Pack(_struct_def_idx, _) => {
                let struct_def_bound = self.root_module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_definition = self.root_module.struct_def_at(random_struct_idx);
                let (num_fields, index) = match struct_definition.field_information {
                    StructFieldInformation::Native => {
                        panic!("[Struct Pack] Unexpected native struct")
                    }
                    StructFieldInformation::Declared {
                        field_count,
                        fields,
                    } => (field_count as usize, fields),
                };
                let fields = self
                    .root_module
                    .field_def_range(num_fields as MemberCount, index);
                let stack: Stack = fields
                    .iter()
                    .map(|field| {
                        let ty = self
                            .root_module
                            .type_signature_at(field.signature)
                            .0
                            .clone();
                        self.resolve_to_value(&ty, &[])
                    })
                    .collect();
                let size = stack.iter().fold(AbstractMemorySize::new(0), |acc, local| {
                    acc.add(local.size())
                });
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    Pack(random_struct_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            Unpack(_struct_def_idx, _) => {
                let struct_def_bound = self.root_module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_handle_idx = self
                    .root_module
                    .struct_def_at(random_struct_idx)
                    .struct_handle;
                let struct_stack =
                    self.resolve_to_value(&SignatureToken::Struct(struct_handle_idx, vec![]), &[]);
                let size = struct_stack.size();
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(vec![struct_stack]),
                    Unpack(random_struct_idx, NO_TYPE_ACTUALS),
                    size,
                    HashMap::new(),
                )
            }
            ImmBorrowField(_) | MutBorrowField(_) => {
                // First grab a random struct
                let struct_def_bound = self.root_module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_definition = self.root_module.struct_def_at(random_struct_idx);
                let num_fields = struct_definition.declared_field_count().unwrap();
                // Grab a random field within that struct to borrow
                let field_index = self.gen.gen_range(0, num_fields);
                let struct_stack = self.resolve_to_value(
                    &SignatureToken::Reference(Box::new(SignatureToken::Struct(
                        struct_definition.struct_handle,
                        vec![],
                    ))),
                    &[],
                );
                let field_size = struct_stack
                    .copy_value()
                    .value_as::<StructRef>()
                    .expect("[BorrowField] Struct should be a reference.")
                    .borrow_field(usize::from(field_index))
                    .expect("[BorrowField] Unable to borrow field of generated struct to get field size.")
                    .size();
                let fdi = FieldDefinitionIndex::new(field_index);
                let op = match self.op {
                    ImmBorrowField(_) => ImmBorrowField(fdi),
                    MutBorrowField(_) => MutBorrowField(fdi),
                    _ => panic!("[BorrowField] Impossible case for op"),
                };
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(vec![struct_stack]),
                    op,
                    field_size,
                    HashMap::new(),
                )
            }
            StLoc(_) => {
                let (module, local_idx, function_idx, stack_local) = self.next_local_state();
                let size = stack_local[0].size();
                StackState::new(
                    (module, Some(function_idx)),
                    self.random_pad(stack_local),
                    StLoc(local_idx as LocalIndex),
                    size,
                    HashMap::new(),
                )
            }
            CopyLoc(_) | MoveLoc(_) | MutBorrowLoc(_) | ImmBorrowLoc(_) => {
                let (module, local_idx, function_idx, mut frame_local) = self.next_local_state();
                let size = frame_local[0].size();
                let mut locals_mapping = HashMap::new();
                locals_mapping.insert(local_idx as LocalIndex, frame_local.remove(0));
                StackState::new(
                    (module, Some(function_idx)),
                    self.random_pad(frame_local),
                    CopyLoc(local_idx as LocalIndex),
                    size,
                    locals_mapping,
                )
            }
            _ => unimplemented!(),
        }
    }

    // Take the stack, and then randomly pad it up to the given stack limit
    fn random_pad(&mut self, mut stk: Stack) -> Stack {
        // max amount we can pad while being legal
        let len = stk.len() as u64;
        let max_pad_amt = if len > self.max_stack_size {
            1
        } else {
            self.max_stack_size - (stk.len() as u64)
        };
        let rand_len = self.gen.gen_range(1, max_pad_amt);
        // Generate the random stack prefix
        let mut stk_prefix: Vec<_> = (0..rand_len)
            .map(|_| self.next_stack_value(&stk, true))
            .collect();
        stk_prefix.append(&mut stk);
        stk_prefix
    }

    /// Generate a new valid random stack state. Return `None` if `iters` many stacks have been
    /// produced with this instance.
    pub fn next_stack(&mut self) -> Option<StackState<'txn>> {
        if self.iters == 0 {
            return None;
        }
        // Clear the VM's data cache -- otherwise we'll windup grabbing the data from
        // the cache on subsequent iterations and across future instructions that
        // effect global memory.
        self.ctx.clear();
        self.iters -= 1;
        Some(if self.is_module_specific_op() {
            self.generate_from_module_info()
        } else {
            let info = call_details(&self.op);
            // Pick a random input/output argument configuration for the opcode
            let index = self.gen.gen_range(0, info.len());
            let args = info[index].in_args.clone();
            let starting_stack: Stack = args.into_iter().fold(Vec::new(), |mut acc, x| {
                // We pass in a context since we need to enforce certain relationships between
                // stack values for valid execution.
                acc.push(self.generate_from_type(x, &acc));
                acc
            });
            let (instr_arg, arg_size) = self.fill_instruction_arg();
            let size = starting_stack
                .iter()
                .fold(AbstractMemorySize::new(arg_size as GasCarrier), |acc, x| {
                    acc.add(x.size())
                });
            StackState::new(
                (self.root_module, None),
                self.random_pad(starting_stack),
                instr_arg,
                size,
                HashMap::new(),
            )
        })
    }

    pub fn execute_code_snippet(
        &mut self,
        interp: &mut InterpreterForCostSynthesis<'txn>,
        code: &[Bytecode],
    ) -> VMResult<()> {
        interp.execute_code_snippet(self.move_vm, &mut self.ctx, code)
    }

    /// Applies the `stack_state` to the VM's execution stack.
    ///
    /// We don't use the `instr` in the stack state within this function. We therefore pull it out
    /// since we are grabbing ownership of the other fields of the struct and return it to be
    /// used elsewhere.
    pub fn stack_transition(
        interpreter: &mut InterpreterForCostSynthesis<'txn>,
        stack_state: StackState<'txn>,
    ) -> (Bytecode, AbstractMemorySize<GasCarrier>) {
        // Set the value stack
        // This needs to happen before the frame transition.
        interpreter.set_stack(stack_state.stack);

        // Perform the frame transition (if there is any needed)
        frame_transitions(interpreter, &stack_state.instr, stack_state.module_info);

        // Populate the locals of the frame
        interpreter.load_call(stack_state.local_mapping);
        (stack_state.instr, stack_state.size)
    }
}
