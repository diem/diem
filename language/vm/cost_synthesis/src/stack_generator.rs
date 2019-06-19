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
use rand::{rngs::StdRng, Rng, SeedableRng};
use std::collections::HashMap;
use types::{account_address::AccountAddress, byte_array::ByteArray, language_storage::CodeKey};
use vm::{
    access::*,
    assert_ok,
    file_format::{
        AddressPoolIndex, ByteArrayPoolIndex, Bytecode, CodeOffset, FieldDefinitionIndex,
        FunctionDefinition, FunctionDefinitionIndex, FunctionHandleIndex, LocalIndex, MemberCount,
        ModuleHandle, SignatureToken, StringPoolIndex, StructDefinition, StructDefinitionIndex,
        StructHandleIndex, TableIndex,
    },
};
use vm_runtime::{
    code_cache::module_cache::ModuleCache, execution_stack::ExecutionStack,
    loaded_data::loaded_module::LoadedModule, value::*,
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
    pub size: u64,

    /// A sparse mapping of local index to local value for the current function frame. This will
    /// be applied to the execution stack later on in the `stack_transition_function`.
    pub local_mapping: HashMap<LocalIndex, Local>,
}

impl<'txn> StackState<'txn> {
    /// Create a new stack state with the passed-in information.
    pub fn new(
        module_info: (&'txn LoadedModule, Option<FunctionDefinitionIndex>),
        stack: Stack,
        instr: Bytecode,
        size: u64,
        local_mapping: HashMap<LocalIndex, Local>,
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
pub struct RandomStackGenerator<'alloc, 'txn>
where
    'alloc: 'txn,
{
    /// The number of iterations that this instruction will be run for. Used to implement the
    /// `Iterator` trait.
    iters: u64,

    /// The source of pseudo-randomness.
    gen: StdRng,

    /// Certain instructions require indices into the various tables within the module.
    /// We store a reference to the loaded module context that we are currently in so that we can
    /// generate valid references into these tables. When generating a module universe this is the
    /// root module that has pointers to all other modules.
    root_module: &'txn LoadedModule,

    /// The module cache for all of the other modules in the universe. We need this in order to
    /// resolve struct and function handles to other modules other then the root module.
    module_cache: &'txn ModuleCache<'alloc>,

    /// The bytecode instruction for which stack states are generated.
    op: Bytecode,

    /// The maximum size of the generated value stack.
    max_stack_size: u64,

    /// Cursor into the string pool. Used for the generation of random strings.
    string_pool_index: u64,

    /// Cursor into the address pool. Used for the generation of random addresses.  We use this
    /// since we need the addresses to be unique for e.g. CreateAccount, and we don't want a
    /// mutable reference into the underlying `root_module`.
    address_pool_index: u64,

    /// A reverse lookup table to find the struct definition for a struct handle. Needed for
    /// generating an inhabitant for a struct SignatureToken. This is lazily populated.
    struct_handle_table: HashMap<CodeKey, HashMap<String, StructDefinitionIndex>>,

    /// A reverse lookup table for each code module that allows us to resolve function handles to
    /// function definitions. Also lazily populated.
    function_handle_table: HashMap<CodeKey, HashMap<String, FunctionDefinitionIndex>>,
}

impl<'alloc, 'txn> RandomStackGenerator<'alloc, 'txn>
where
    'alloc: 'txn,
{
    /// Create a new random stack state generator.
    ///
    /// It initializes each of the internal resolution tables for structs and function handles to
    /// be empty.
    pub fn new(
        root_module: &'txn LoadedModule,
        module_cache: &'txn ModuleCache<'alloc>,
        op: &Bytecode,
        max_stack_size: u64,
        iters: u64,
    ) -> Self {
        let seed: [u8; 32] = [0; 32];
        Self {
            gen: StdRng::from_seed(seed),
            op: op.clone(),
            max_stack_size,
            root_module,
            module_cache,
            iters,
            string_pool_index: iters,
            address_pool_index: iters,
            struct_handle_table: HashMap::new(),
            function_handle_table: HashMap::new(),
        }
    }

    fn to_code_key(&self, module_handle: &ModuleHandle) -> CodeKey {
        let address = *self.root_module.module.address_at(module_handle.address);
        let name = self.root_module.module.string_at(module_handle.name);
        CodeKey::new(address, name.to_string())
    }

    // Determines if the instruction gets its type/instruction info from the stack type
    // transitions, or from the type signatures available in the module(s).
    fn is_module_specific_op(&self) -> bool {
        use Bytecode::*;
        match self.op {
            Unpack(_) | Pack(_) | Call(_) => true,
            CopyLoc(_) | MoveLoc(_) | StLoc(_) | BorrowLoc(_) | BorrowField(_) => true,
            _ => false,
        }
    }

    // Certain operations are only valid if their values come from module-specific data. In
    // particular, CreateLibraAccount. But, they may eventually be more of these as well.
    fn points_to_module_data(&self) -> bool {
        use Bytecode::*;
        match self.op {
            CreateAccount => true,
            _ => false,
        }
    }

    fn next_int(&mut self, stk: &[Local]) -> u64 {
        if self.op == Bytecode::Sub && !stk.is_empty() {
            let peek: Option<u64> = stk.last().expect("[Next Integer] The impossible happened: the value stack became empty while still full.").clone().value().expect("[Next Integer] Invalid integer stack value encountered when peeking at the generated stack.").into();
            self.gen.gen_range(
                0,
                peek.expect("[Next Integer] Unable to cast peeked stack value to an integer."),
            )
        } else {
            u64::from(self.gen.gen_range(0, u32::max_value()))
        }
    }

    fn next_bool(&mut self) -> bool {
        // Flip a coin
        self.gen.gen_bool(1.0 / 2.0)
    }

    fn next_bytearray(&mut self) -> ByteArray {
        let len: usize = self.gen.gen_range(1, BYTE_ARRAY_MAX_SIZE);
        let bytes: Vec<u8> = (0..len).map(|_| self.gen.gen::<u8>()).collect();
        ByteArray::new(bytes)
    }

    // Strings and addresses are already randomly generated in the module that we create these
    // pools from so we simply pop off from them. This assumes that the module was generated with
    // at least `self.iters` number of strings and addresses. In the case where we are just padding
    // the stack, or where the instructions semantics don't require having an address in the
    // address pool, we don't waste our pools and generate a random value.
    fn next_str(&mut self, is_padding: bool) -> String {
        if !self.points_to_module_data() || is_padding {
            let len: usize = self.gen.gen_range(1, MAX_STRING_SIZE);
            (0..len).map(|_| self.gen.gen::<char>()).collect::<String>()
        } else {
            let string = self.root_module.module.as_inner().string_pool
                [self.string_pool_index as usize]
                .clone();
            self.string_pool_index = self
                .string_pool_index
                .checked_sub(1)
                .expect("Exhausted strings in string pool");
            string
        }
    }

    fn next_addr(&mut self, is_padding: bool) -> AccountAddress {
        if !self.points_to_module_data() || is_padding {
            AccountAddress::random()
        } else {
            let address =
                self.root_module.module.as_inner().address_pool[self.address_pool_index as usize];
            self.address_pool_index = self
                .address_pool_index
                .checked_sub(1)
                .expect("Exhausted account addresses in address pool");
            address
        }
    }

    fn next_bounded_index(&mut self, bound: TableIndex) -> TableIndex {
        self.gen.gen_range(1, bound)
    }

    fn next_string_idx(&mut self) -> StringPoolIndex {
        let len = self.root_module.module.string_pool().len();
        StringPoolIndex::new(self.gen.gen_range(0, len) as TableIndex)
    }

    fn next_address_idx(&mut self) -> AddressPoolIndex {
        let len = self.root_module.module.address_pool().len();
        AddressPoolIndex::new(self.gen.gen_range(0, len) as TableIndex)
    }

    fn next_bytearray_idx(&mut self) -> ByteArrayPoolIndex {
        let len = self.root_module.module.byte_array_pool().len();
        ByteArrayPoolIndex::new(self.gen.gen_range(0, len) as TableIndex)
    }

    fn next_function_handle_idx(&mut self) -> FunctionHandleIndex {
        let table_idx =
            self.next_bounded_index(self.root_module.module.function_handles().len() as TableIndex);
        FunctionHandleIndex::new(table_idx)
    }

    fn next_stack_value(&mut self, stk: &[Local], is_padding: bool) -> Local {
        match self.gen.gen_range(0, 5) {
            0 => Local::u64(self.next_int(stk)),
            1 => Local::bool(self.next_bool()),
            2 => Local::string(self.next_str(is_padding)),
            3 => Local::bytearray(self.next_bytearray()),
            _ => Local::address(self.next_addr(is_padding)),
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
        Local,
    ) {
        // We pick a random function from the module in which to store the local
        let function_handle_idx = self.next_function_handle_idx();
        let (module, function_definition, function_def_idx) =
            self.resolve_function_handle(function_handle_idx);
        let type_sig = &module
            .module
            .locals_signature_at(function_definition.code.locals)
            .0;
        // Pick a random local within that function in which we'll store the local
        let local_index = self.gen.gen_range(0, type_sig.len());
        let type_tok = type_sig[local_index].clone();
        let stack_local = self.resolve_to_value(type_tok, &[]);
        (
            module,
            local_index as LocalIndex,
            function_def_idx,
            stack_local,
        )
    }

    fn fill_instruction_arg(&mut self) -> Bytecode {
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
            .module
            .function_def_at(function_idx)
            .code
            .code
            .len();
        match self.op {
            BrTrue(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                BrTrue(index as CodeOffset)
            }
            BrFalse(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                BrFalse(index as CodeOffset)
            }
            Branch(_) => {
                let index = self.next_bounded_index(frame_len as TableIndex);
                Branch(index as CodeOffset)
            }
            LdConst(_) => LdConst(self.next_int(&[])),
            LdStr(_) => LdStr(self.next_string_idx()),
            LdByteArray(_) => LdByteArray(self.next_bytearray_idx()),
            LdAddr(_) => LdAddr(self.next_address_idx()),
            _ => self.op.clone(),
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
        let struct_handle = self
            .root_module
            .module
            .struct_handle_at(struct_handle_index);
        let struct_name = self.root_module.string_at(struct_handle.name);
        let module_handle = self
            .root_module
            .module
            .module_handle_at(struct_handle.module);
        let code_key = self.to_code_key(module_handle);
        let module = self
            .module_cache
            .get_loaded_module(&code_key)
            .expect("[Module Lookup] Error while looking up module")
            .expect("[Module Lookup] Unable to find module");
        let struct_def_idx = if self.struct_handle_table.contains_key(&code_key) {
            self.struct_handle_table
                .get(&code_key)
                .expect("[Struct Definition Lookup] Unable to get struct handles for module")
                .get(struct_name)
        } else {
            let entry = self.struct_handle_table.entry(code_key).or_insert_with(|| {
                module
                    .module
                    .struct_defs()
                    .enumerate()
                    .map(|(struct_def_index, struct_def)| {
                        let handle = module.module.struct_handle_at(struct_def.struct_handle);
                        let name = module.module.string_at(handle.name).to_string();
                        (
                            name,
                            StructDefinitionIndex::new(struct_def_index as TableIndex),
                        )
                    })
                    .collect()
            });
            entry.get(struct_name)
        }
        .expect("[Struct Definition Lookup] Unable to get struct definition for struct handle");

        let struct_def = module.module.struct_def_at(*struct_def_idx);
        (module, struct_def, *struct_def_idx)
    }

    fn resolve_function_handle(
        &mut self,
        function_handle_index: FunctionHandleIndex,
    ) -> (
        &'txn LoadedModule,
        &'txn FunctionDefinition,
        FunctionDefinitionIndex,
    ) {
        let function_handle = self
            .root_module
            .module
            .function_handle_at(function_handle_index);
        let function_name = self.root_module.string_at(function_handle.name);
        let module_handle = self
            .root_module
            .module
            .module_handle_at(function_handle.module);
        let code_key = self.to_code_key(module_handle);
        let module = self
            .module_cache
            .get_loaded_module(&code_key)
            .expect("[Module Lookup] Error while looking up module")
            .expect("[Module Lookup] Unable to find module");
        let function_def_idx = if self.function_handle_table.contains_key(&code_key) {
            *self
                .function_handle_table
                .get(&code_key)
                .expect("[Function Definition Lookup] Unable to get function handles for module")
                .get(function_name)
                .expect(
                    "[Function Definition Lookup] Unable to get function definition for struct handle",
                )
        } else {
            let entry = self
                .function_handle_table
                .entry(code_key)
                .or_insert_with(|| {
                    module
                        .module
                        .function_defs()
                        .enumerate()
                        .map(|(function_def_index, function_def)| {
                            let handle = module.module.function_handle_at(function_def.function);
                            let name = module.module.string_at(handle.name).to_string();
                            (
                                name,
                                FunctionDefinitionIndex::new(function_def_index as TableIndex),
                            )
                        })
                        .collect()
                });
            *entry.get(function_name).expect("FOO")
        };

        let function_def = module.module.function_def_at(function_def_idx);
        (module, function_def, function_def_idx)
    }

    // Build an inhabitant of the type given by `sig_token`. We pass the current stack state in
    // since for certain instructions (...Sub) we need to generate number pairs that when
    // subtracted from each other do not cause overflow.
    fn resolve_to_value(&mut self, sig_token: SignatureToken, stk: &[Local]) -> Local {
        match sig_token {
            SignatureToken::Bool => Local::bool(self.next_bool()),
            SignatureToken::U64 => Local::u64(self.next_int(stk)),
            SignatureToken::String => Local::string(self.next_str(false)),
            SignatureToken::Address => Local::address(self.next_addr(false)),
            SignatureToken::Reference(box sig) | SignatureToken::MutableReference(box sig) => {
                let underlying_value = self.resolve_to_value(sig, stk);
                underlying_value
                    .borrow_local()
                    .expect("Unable to generate valid reference value")
            }
            SignatureToken::ByteArray => Local::bytearray(self.next_bytearray()),
            SignatureToken::Struct(struct_handle_idx) => {
                assert!(self.root_module.module.struct_defs().len() > 1);
                let struct_definition = self
                    .root_module
                    .module
                    .struct_def_at(self.resolve_struct_handle(struct_handle_idx).2);
                let num_fields = struct_definition.field_count as usize;
                let index = struct_definition.fields;
                let fields = self
                    .root_module
                    .module
                    .field_def_range(num_fields as MemberCount, index);
                let mutvals = fields
                    .map(|field| {
                        self.resolve_to_value(
                            self.root_module
                                .module
                                .type_signature_at(field.signature)
                                .0
                                .clone(),
                            stk,
                        )
                        .value()
                        .expect("[Struct Generation] Unable to get underlying value for generated struct field.")
                    })
                    .collect();
                Local::struct_(mutvals)
            }
        }
    }

    // Generate starting state of the stack based upon the type transition in the call info table.
    fn generate_from_type(&mut self, typ: SignatureTy, stk: &[Local]) -> Local {
        // If the underlying type is a variable type, then we can choose any type that we want.
        let typ = if typ.is_variable() {
            let underlying_type = typ.underlying();
            let index = self.gen.gen_range(0, underlying_type.len());
            underlying_type[index].clone()
        } else {
            typ.underlying()
                .first()
                .expect("Unable to get underlying type for sigty in generate_from_type")
                .clone()
        };
        self.resolve_to_value(typ.0, stk)
    }

    // Certain instructions require specific stack states; e.g. Pack() requires the correct number
    // and type of locals to already exist at the top of the value stack when the instruction is
    // encountered. We therefore need to generate the stack state for certain operations _not_ on
    // their call info, but on the possible calls that we could have in the module/other modules
    // that we are aware of.
    fn generate_from_module_info(&mut self) -> StackState<'txn> {
        use Bytecode::*;
        match self.op {
            Call(_) => {
                let function_handle_idx = self.next_function_handle_idx();
                let function_idx = self.resolve_function_handle(function_handle_idx).2;
                let function_handle = self
                    .root_module
                    .module
                    .function_handle_at(function_handle_idx);
                let function_sig = self
                    .root_module
                    .module
                    .function_signature_at(function_handle.signature);
                let stack = function_sig.arg_types.clone().into_iter().fold(
                    Vec::new(),
                    |mut acc, sig_tok| {
                        acc.push(self.resolve_to_value(sig_tok, &acc));
                        acc
                    },
                );
                let size = stack.iter().fold(0, |acc, local| local.size() + acc);
                StackState::new(
                    (self.root_module, Some(function_idx)),
                    self.random_pad(stack),
                    Call(function_handle_idx),
                    size,
                    HashMap::new(),
                )
            }
            Pack(_struct_def_idx) => {
                let struct_def_bound = self.root_module.module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_definition = self.root_module.module.struct_def_at(random_struct_idx);
                let num_fields = struct_definition.field_count as usize;
                let index = struct_definition.fields;
                let fields = self
                    .root_module
                    .module
                    .field_def_range(num_fields as MemberCount, index);
                let stack: Stack = fields
                    .map(|field| {
                        let ty = self
                            .root_module
                            .module
                            .type_signature_at(field.signature)
                            .0
                            .clone();
                        self.resolve_to_value(ty, &[])
                    })
                    .collect();
                let size = stack.iter().fold(0, |acc, local| local.size() + acc);
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(stack),
                    Pack(random_struct_idx),
                    size,
                    HashMap::new(),
                )
            }
            Unpack(_struct_def_idx) => {
                let struct_def_bound = self.root_module.module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_handle_idx = self
                    .root_module
                    .module
                    .struct_def_at(random_struct_idx)
                    .struct_handle;
                let struct_stack =
                    self.resolve_to_value(SignatureToken::Struct(struct_handle_idx), &[]);
                let size = struct_stack.size() as u64;
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(vec![struct_stack]),
                    Unpack(random_struct_idx),
                    size,
                    HashMap::new(),
                )
            }
            BorrowField(_) => {
                // First grab a random struct
                let struct_def_bound = self.root_module.module.struct_defs().len() as TableIndex;
                let random_struct_idx =
                    StructDefinitionIndex::new(self.next_bounded_index(struct_def_bound));
                let struct_definition = self.root_module.module.struct_def_at(random_struct_idx);
                let num_fields = struct_definition.field_count;
                // Grab a random field within that struct to borrow
                let field_index = self.gen.gen_range(0, num_fields);
                let struct_stack = self.resolve_to_value(
                    SignatureToken::Reference(Box::new(SignatureToken::Struct(
                        struct_definition.struct_handle,
                    ))),
                    &[],
                );
                let field_size = struct_stack
                    .borrow_field(u32::from(field_index))
                    .expect("[BorrowField] Unable to borrow field of generated struct to get field size.")
                    .size();
                StackState::new(
                    (self.root_module, None),
                    self.random_pad(vec![struct_stack]),
                    BorrowField(FieldDefinitionIndex::new(field_index)),
                    field_size,
                    HashMap::new(),
                )
            }
            StLoc(_) => {
                let (module, local_idx, function_idx, stack_local) = self.next_local_state();
                let size = stack_local.size();
                StackState::new(
                    (module, Some(function_idx)),
                    self.random_pad(vec![stack_local]),
                    StLoc(local_idx as LocalIndex),
                    size,
                    HashMap::new(),
                )
            }
            CopyLoc(_) | MoveLoc(_) | BorrowLoc(_) => {
                let (module, local_idx, function_idx, frame_local) = self.next_local_state();
                let size = frame_local.size();
                let mut locals_mapping = HashMap::new();
                locals_mapping.insert(local_idx as LocalIndex, frame_local);
                StackState::new(
                    (module, Some(function_idx)),
                    self.random_pad(Vec::new()),
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
            let size = starting_stack.iter().fold(0, |acc, x| acc + x.size());
            StackState::new(
                (self.root_module, None),
                self.random_pad(starting_stack),
                self.fill_instruction_arg(),
                size,
                HashMap::new(),
            )
        })
    }

    /// Applies the `stack_state` to the VM's execution stack.
    ///
    /// We don't use the `instr` in the stack state within this function. We therefore pull it out
    /// since we are grabbing ownership of the other fields of the struct and return it to be
    /// used elsewhere.
    pub fn stack_transition<P>(
        stk: &mut ExecutionStack<'alloc, 'txn, P>,
        stack_state: StackState<'alloc>,
    ) -> Bytecode
    where
        P: ModuleCache<'alloc>,
    {
        // Set the value stack
        stk.set_stack(stack_state.stack);

        // Perform the frame transition (if there is any needed)
        frame_transitions(stk, &stack_state.instr, stack_state.module_info);

        // Populate the locals of the frame
        for (local_index, local) in stack_state.local_mapping.into_iter() {
            assert_ok!(stk
                .top_frame_mut()
                .expect("[Stack Transition] Unable to get top frame on execution stack.")
                .store_local(local_index, local));
        }
        stack_state.instr
    }
}

impl<'alloc, 'txn> Iterator for RandomStackGenerator<'alloc, 'txn>
where
    'alloc: 'txn,
{
    type Item = StackState<'txn>;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_stack()
    }
}
