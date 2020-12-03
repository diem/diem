// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    file_format::{
        Bytecode, CodeOffset, CodeUnit, ConstantPoolIndex, FieldHandle, FieldHandleIndex,
        FieldInstantiation, FieldInstantiationIndex, FunctionDefinition, FunctionHandle,
        FunctionHandleIndex, FunctionInstantiation, FunctionInstantiationIndex, IdentifierIndex,
        Kind, LocalIndex, ModuleHandleIndex, Signature, SignatureIndex, SignatureToken,
        StructDefInstantiation, StructDefInstantiationIndex, StructDefinition,
        StructDefinitionIndex, StructHandle, TableIndex,
    },
    proptest_types::{
        signature::{KindGen, SignatureGen, SignatureTokenGen},
        TableSize,
    },
};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
    sample::{select, Index as PropIndex},
};
use std::{
    collections::{BTreeSet, HashMap, HashSet},
    hash::Hash,
};

#[derive(Debug, Default)]
struct SignatureState {
    signatures: Vec<Signature>,
    signature_map: HashMap<Signature, SignatureIndex>,
}

impl SignatureState {
    fn new(signatures: Vec<Signature>) -> Self {
        let mut state = Self::default();
        for sig in signatures {
            state.add_signature(sig);
        }
        state
    }

    fn signatures(self) -> Vec<Signature> {
        self.signatures
    }

    fn add_signature(&mut self, sig: Signature) -> SignatureIndex {
        precondition!(self.signatures.len() < TableSize::max_value() as usize);
        if let Some(idx) = self.signature_map.get(&sig) {
            return *idx;
        }
        let idx = SignatureIndex(self.signatures.len() as u16);
        self.signatures.push(sig.clone());
        self.signature_map.insert(sig, idx);
        idx
    }
}

#[derive(Debug, Default)]
#[allow(unused)]
struct FieldHandleState {
    field_handles: Vec<FieldHandle>,
    field_map: HashMap<FieldHandle, FieldHandleIndex>,
}

impl FieldHandleState {
    #[allow(unused)]
    pub fn field_handles(self) -> Vec<FieldHandle> {
        self.field_handles
    }

    #[allow(unused)]
    fn add_field_handle(&mut self, fh: FieldHandle) -> FieldHandleIndex {
        precondition!(self.field_handles.len() < TableSize::max_value() as usize);
        if let Some(idx) = self.field_map.get(&fh) {
            return *idx;
        }
        let idx = FieldHandleIndex(self.field_handles.len() as u16);
        self.field_handles.push(fh.clone());
        self.field_map.insert(fh, idx);
        idx
    }
}

#[derive(Debug)]
#[allow(unused)]
struct InstantiationState<T>
where
    T: Eq + Clone + Hash,
{
    instantiations: Vec<T>,
    instantiation_map: HashMap<T, TableIndex>,
}

impl<T> InstantiationState<T>
where
    T: Eq + Clone + Hash,
{
    fn new() -> Self {
        InstantiationState {
            instantiations: vec![],
            instantiation_map: HashMap::new(),
        }
    }

    #[allow(unused)]
    pub fn instantiations(self) -> Vec<T> {
        self.instantiations
    }

    #[allow(unused)]
    fn add_instantiation(&mut self, inst: T) -> TableIndex {
        precondition!(self.instantiations.len() < TableSize::max_value() as usize);
        if let Some(idx) = self.instantiation_map.get(&inst) {
            return *idx;
        }
        let idx = self.instantiations.len() as TableIndex;
        self.instantiations.push(inst.clone());
        self.instantiation_map.insert(inst, idx);
        idx
    }
}

/// Represents state required to materialize final data structures for function definitions.
#[derive(Debug)]
pub struct FnHandleMaterializeState<'a> {
    module_handles_len: usize,
    identifiers_len: usize,
    struct_handles: &'a [StructHandle],
    signatures: SignatureState,
    function_handles: HashSet<(ModuleHandleIndex, IdentifierIndex)>,
}

impl<'a> FnHandleMaterializeState<'a> {
    pub fn new(
        module_handles_len: usize,
        identifiers_len: usize,
        struct_handles: &'a [StructHandle],
    ) -> Self {
        Self {
            module_handles_len,
            identifiers_len,
            struct_handles,
            signatures: SignatureState::default(),
            function_handles: HashSet::new(),
        }
    }

    pub fn signatures(self) -> Vec<Signature> {
        self.signatures.signatures()
    }

    fn add_signature(&mut self, sig: Signature) -> SignatureIndex {
        self.signatures.add_signature(sig)
    }
}

#[derive(Clone, Debug)]
pub struct FunctionHandleGen {
    module: PropIndex,
    name: PropIndex,
    parameters: SignatureGen,
    return_: SignatureGen,
    type_parameters: Vec<KindGen>,
}

impl FunctionHandleGen {
    pub fn strategy(
        param_count: impl Into<SizeRange>,
        return_count: impl Into<SizeRange>,
        kind_count: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        let return_count = return_count.into();
        let param_count = param_count.into();
        (
            any::<PropIndex>(),
            any::<PropIndex>(),
            SignatureGen::strategy(param_count),
            SignatureGen::strategy(return_count),
            vec(KindGen::strategy(), kind_count),
        )
            .prop_map(
                |(module, name, parameters, return_, type_parameters)| Self {
                    module,
                    name,
                    parameters,
                    return_,
                    type_parameters,
                },
            )
    }

    pub fn materialize(self, state: &mut FnHandleMaterializeState) -> Option<FunctionHandle> {
        let mod_ = ModuleHandleIndex(self.module.index(state.module_handles_len) as TableIndex);
        let mod_idx = if mod_.0 == 0 {
            ModuleHandleIndex(1)
        } else {
            mod_
        };
        let iden_idx = IdentifierIndex(self.name.index(state.identifiers_len) as TableIndex);
        if state.function_handles.contains(&(mod_idx, iden_idx)) {
            return None;
        }
        state.function_handles.insert((mod_idx, iden_idx));
        let parameters = self.parameters.materialize(state.struct_handles);
        let params_idx = state.add_signature(parameters);
        let return_ = self.return_.materialize(state.struct_handles);
        let return_idx = state.add_signature(return_);
        let type_parameters = self
            .type_parameters
            .into_iter()
            .map(|kind| kind.materialize())
            .collect();
        Some(FunctionHandle {
            module: mod_idx,
            name: iden_idx,
            parameters: params_idx,
            return_: return_idx,
            type_parameters,
        })
    }
}

/// Represents state required to materialize final data structures for function definitions.
#[derive(Debug)]
pub struct FnDefnMaterializeState<'a> {
    identifiers_len: usize,
    constant_pool_len: usize,
    struct_handles: &'a [StructHandle],
    struct_defs: &'a [StructDefinition],
    signatures: SignatureState,
    function_handles: Vec<FunctionHandle>,
    struct_def_to_field_count: HashMap<usize, usize>,
    def_function_handles: HashSet<(ModuleHandleIndex, IdentifierIndex)>,
    field_handles: FieldHandleState,
    type_instantiations: InstantiationState<StructDefInstantiation>,
    function_instantiations: InstantiationState<FunctionInstantiation>,
    field_instantiations: InstantiationState<FieldInstantiation>,
}

impl<'a> FnDefnMaterializeState<'a> {
    pub fn new(
        identifiers_len: usize,
        constant_pool_len: usize,
        struct_handles: &'a [StructHandle],
        struct_defs: &'a [StructDefinition],
        signatures: Vec<Signature>,
        function_handles: Vec<FunctionHandle>,
        struct_def_to_field_count: HashMap<usize, usize>,
    ) -> Self {
        Self {
            identifiers_len,
            constant_pool_len,
            struct_handles,
            struct_defs,
            signatures: SignatureState::new(signatures),
            function_handles,
            struct_def_to_field_count,
            def_function_handles: HashSet::new(),
            field_handles: FieldHandleState::default(),
            type_instantiations: InstantiationState::new(),
            function_instantiations: InstantiationState::new(),
            field_instantiations: InstantiationState::new(),
        }
    }

    pub fn return_tables(
        self,
    ) -> (
        Vec<Signature>,
        Vec<FunctionHandle>,
        Vec<FieldHandle>,
        Vec<StructDefInstantiation>,
        Vec<FunctionInstantiation>,
        Vec<FieldInstantiation>,
    ) {
        (
            self.signatures.signatures(),
            self.function_handles,
            self.field_handles.field_handles(),
            self.type_instantiations.instantiations(),
            self.function_instantiations.instantiations(),
            self.field_instantiations.instantiations(),
        )
    }

    fn add_signature(&mut self, sig: Signature) -> SignatureIndex {
        self.signatures.add_signature(sig)
    }

    fn add_function_handle(&mut self, handle: FunctionHandle) -> FunctionHandleIndex {
        precondition!(self.function_handles.len() < TableSize::max_value() as usize);
        self.function_handles.push(handle);
        FunctionHandleIndex((self.function_handles.len() - 1) as TableIndex)
    }

    fn get_signature_from_type_params(&mut self, kinds: &[Kind]) -> Signature {
        let mut type_params = vec![];
        for kind in kinds {
            match kind {
                Kind::Copyable | Kind::All => type_params.push(SignatureToken::U64),
                Kind::Resource => type_params.push(SignatureToken::Signer),
            }
        }
        Signature(type_params)
    }

    fn add_signature_from_type_params(&mut self, kinds: &[Kind]) -> SignatureIndex {
        let sig = self.get_signature_from_type_params(kinds);
        self.signatures.add_signature(sig)
    }

    fn get_function_instantiation(&mut self, fh_idx: usize) -> FunctionInstantiationIndex {
        let kinds = self.function_handles[fh_idx].type_parameters.clone();
        let sig_idx = self.add_signature_from_type_params(&kinds);
        let fi = FunctionInstantiation {
            handle: FunctionHandleIndex(fh_idx as TableIndex),
            type_parameters: sig_idx,
        };
        FunctionInstantiationIndex(self.function_instantiations.add_instantiation(fi))
    }

    fn get_type_instantiation(&mut self, sd_idx: usize) -> StructDefInstantiationIndex {
        let sd = &self.struct_defs[sd_idx];
        let kinds = self.struct_handles[sd.struct_handle.0 as usize]
            .type_parameters
            .clone();
        let sig_idx = self.add_signature_from_type_params(&kinds);
        let si = StructDefInstantiation {
            def: StructDefinitionIndex(sd_idx as TableIndex),
            type_parameters: sig_idx,
        };
        StructDefInstantiationIndex(self.type_instantiations.add_instantiation(si))
    }
}

#[derive(Clone, Debug)]
pub struct FunctionDefinitionGen {
    name: PropIndex,
    parameters: SignatureGen,
    return_: SignatureGen,
    is_public: bool,
    acquires: Vec<PropIndex>,
    code: CodeUnitGen,
}

impl FunctionDefinitionGen {
    pub fn strategy(
        return_count: impl Into<SizeRange>,
        arg_count: impl Into<SizeRange>,
        _kind_count: impl Into<SizeRange>,
        acquires_count: impl Into<SizeRange>,
        code_len: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        let return_count = return_count.into();
        let arg_count = arg_count.into();
        (
            any::<PropIndex>(),
            SignatureGen::strategy(arg_count.clone()),
            SignatureGen::strategy(return_count),
            any::<bool>(),
            vec(any::<PropIndex>(), acquires_count.into()),
            CodeUnitGen::strategy(arg_count, code_len),
        )
            .prop_map(
                |(name, parameters, return_, is_public, acquires, code)| Self {
                    name,
                    parameters,
                    return_,
                    is_public,
                    acquires,
                    code,
                },
            )
    }

    pub fn materialize(self, state: &mut FnDefnMaterializeState) -> Option<FunctionDefinition> {
        // This precondition should never fail because the table size cannot be greater
        // than TableSize::max_value()
        let iden_idx = IdentifierIndex(self.name.index(state.identifiers_len) as TableIndex);
        if state
            .def_function_handles
            .contains(&(ModuleHandleIndex(0), iden_idx))
        {
            return None;
        }
        state
            .def_function_handles
            .insert((ModuleHandleIndex(0), iden_idx));

        let parameters = self.parameters.materialize(state.struct_handles);
        let params_idx = state.add_signature(parameters);
        let return_ = self.return_.materialize(state.struct_handles);
        let return_idx = state.add_signature(return_);
        let handle = FunctionHandle {
            module: ModuleHandleIndex(0),
            name: iden_idx,
            parameters: params_idx,
            return_: return_idx,
            type_parameters: vec![],
        };
        let function_handle = state.add_function_handle(handle);
        let mut acquires_set = BTreeSet::new();
        for acquire in self.acquires {
            acquires_set.insert(StructDefinitionIndex(
                acquire.index(state.struct_defs.len()) as TableIndex,
            ));
        }
        let acquires_global_resources = acquires_set.into_iter().collect();
        // TODO: consider generating native functions?
        Some(FunctionDefinition {
            function: function_handle,
            is_public: self.is_public,
            acquires_global_resources,
            code: Some(self.code.materialize(state)),
        })
    }
}

#[derive(Clone, Debug)]
struct CodeUnitGen {
    locals_signature: Vec<SignatureTokenGen>,
    code: Vec<BytecodeGen>,
}

impl CodeUnitGen {
    fn strategy(
        arg_count: impl Into<SizeRange>,
        code_len: impl Into<SizeRange>,
    ) -> impl Strategy<Value = Self> {
        (
            vec(SignatureTokenGen::strategy(), arg_count),
            vec(BytecodeGen::garbage_strategy(), code_len),
        )
            .prop_map(|(locals_signature, code)| Self {
                locals_signature,
                code,
            })
    }

    fn materialize(self, state: &mut FnDefnMaterializeState) -> CodeUnit {
        let locals_signature = Signature(
            self.locals_signature
                .into_iter()
                .map(|sig| sig.materialize(state.struct_handles))
                .collect(),
        );

        let mut code = vec![];
        for bytecode_gen in self.code {
            if let Some(bytecode) = bytecode_gen.materialize(state, code.len(), &locals_signature) {
                code.push(bytecode)
            }
        }

        CodeUnit {
            locals: state.add_signature(locals_signature),
            code,
        }
    }
}

#[derive(Clone, Debug)]
enum BytecodeGen {
    // "Simple" means this doesn't refer to any other indexes.
    Simple(Bytecode),
    // All of these refer to other indexes.
    LdConst(PropIndex),

    MutBorrowField((PropIndex, PropIndex)),
    ImmBorrowField((PropIndex, PropIndex)),

    Call(PropIndex),

    Pack(PropIndex),
    Unpack(PropIndex),
    Exists(PropIndex),
    MutBorrowGlobal(PropIndex),
    ImmBorrowGlobal(PropIndex),
    MoveFrom(PropIndex),
    MoveTo(PropIndex),
    BrTrue(PropIndex),
    BrFalse(PropIndex),
    Branch(PropIndex),
    CopyLoc(PropIndex),
    MoveLoc(PropIndex),
    StLoc(PropIndex),
    MutBorrowLoc(PropIndex),
    ImmBorrowLoc(PropIndex),
}

impl BytecodeGen {
    // This just generates nonsensical bytecodes. This will be cleaned up later as the generation
    // model is refined.
    fn garbage_strategy() -> impl Strategy<Value = Self> {
        use BytecodeGen::*;

        prop_oneof![
            Self::simple_bytecode_strategy().prop_map(Simple),
            any::<PropIndex>().prop_map(LdConst),
            (any::<PropIndex>(), any::<PropIndex>()).prop_map(ImmBorrowField),
            (any::<PropIndex>(), any::<PropIndex>()).prop_map(MutBorrowField),
            any::<PropIndex>().prop_map(Call),
            any::<PropIndex>().prop_map(Pack),
            any::<PropIndex>().prop_map(Unpack),
            any::<PropIndex>().prop_map(Exists),
            any::<PropIndex>().prop_map(ImmBorrowGlobal),
            any::<PropIndex>().prop_map(MutBorrowGlobal),
            any::<PropIndex>().prop_map(MoveFrom),
            any::<PropIndex>().prop_map(MoveTo),
            any::<PropIndex>().prop_map(BrTrue),
            any::<PropIndex>().prop_map(BrFalse),
            any::<PropIndex>().prop_map(Branch),
            any::<PropIndex>().prop_map(CopyLoc),
            any::<PropIndex>().prop_map(MoveLoc),
            any::<PropIndex>().prop_map(StLoc),
            any::<PropIndex>().prop_map(MutBorrowLoc),
            any::<PropIndex>().prop_map(ImmBorrowLoc),
        ]
    }

    fn materialize(
        self,
        state: &mut FnDefnMaterializeState,
        code_len: usize,
        locals_signature: &Signature,
    ) -> Option<Bytecode> {
        let bytecode = match self {
            BytecodeGen::Simple(bytecode) => bytecode,
            BytecodeGen::LdConst(idx) => {
                if state.constant_pool_len == 0 {
                    return None;
                }
                Bytecode::LdConst(ConstantPoolIndex(
                    idx.index(state.constant_pool_len) as TableIndex
                ))
            }
            BytecodeGen::MutBorrowField((def, field)) => {
                let sd_idx = def.index(state.struct_defs.len());
                let field_count = state.struct_def_to_field_count.get(&sd_idx)?;
                if *field_count == 0 {
                    return None;
                }
                let fh_idx = state.field_handles.add_field_handle(FieldHandle {
                    owner: StructDefinitionIndex(sd_idx as TableIndex),
                    field: field.index(*field_count) as TableIndex,
                });

                let kinds = &state.struct_handles
                    [state.struct_defs[sd_idx].struct_handle.0 as usize]
                    .type_parameters;
                if kinds.is_empty() {
                    Bytecode::MutBorrowField(fh_idx)
                } else {
                    let sig_idx = state.add_signature_from_type_params(&kinds.clone());
                    let fi_idx = state
                        .field_instantiations
                        .add_instantiation(FieldInstantiation {
                            handle: fh_idx,
                            type_parameters: sig_idx,
                        });
                    Bytecode::MutBorrowFieldGeneric(FieldInstantiationIndex(fi_idx))
                }
            }
            BytecodeGen::ImmBorrowField((def, field)) => {
                let sd_idx = def.index(state.struct_defs.len());
                let field_count = state.struct_def_to_field_count.get(&sd_idx)?;
                if *field_count == 0 {
                    return None;
                }
                let fh_idx = state.field_handles.add_field_handle(FieldHandle {
                    owner: StructDefinitionIndex(sd_idx as TableIndex),
                    field: field.index(*field_count) as TableIndex,
                });

                let kinds = &state.struct_handles
                    [state.struct_defs[sd_idx].struct_handle.0 as usize]
                    .type_parameters;
                if kinds.is_empty() {
                    Bytecode::ImmBorrowField(fh_idx)
                } else {
                    let sig_idx = state.add_signature_from_type_params(&kinds.clone());
                    let fi_idx = state
                        .field_instantiations
                        .add_instantiation(FieldInstantiation {
                            handle: fh_idx,
                            type_parameters: sig_idx,
                        });
                    Bytecode::ImmBorrowFieldGeneric(FieldInstantiationIndex(fi_idx))
                }
            }
            BytecodeGen::Call(idx) => {
                let func_handles_len = state.function_handles.len();
                let fh_idx = idx.index(func_handles_len);

                if state.function_handles[fh_idx].type_parameters.is_empty() {
                    Bytecode::Call(FunctionHandleIndex(fh_idx as TableIndex))
                } else {
                    Bytecode::CallGeneric(state.get_function_instantiation(fh_idx))
                }
            }
            BytecodeGen::Pack(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::Pack(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::PackGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::Unpack(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::Unpack(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::UnpackGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::Exists(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::Exists(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::ExistsGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::ImmBorrowGlobal(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::ImmBorrowGlobal(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::ImmBorrowGlobalGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::MutBorrowGlobal(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::MutBorrowGlobal(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::MutBorrowGlobalGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::MoveFrom(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::MoveFrom(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::MoveFromGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::MoveTo(idx) => {
                let struct_defs_len = state.struct_defs.len();
                let sd_idx = idx.index(struct_defs_len);

                let sd = &state.struct_defs[sd_idx];
                if state.struct_handles[sd.struct_handle.0 as usize]
                    .type_parameters
                    .is_empty()
                {
                    Bytecode::MoveTo(StructDefinitionIndex(sd_idx as TableIndex))
                } else {
                    Bytecode::MoveToGeneric(state.get_type_instantiation(sd_idx))
                }
            }
            BytecodeGen::BrTrue(idx) => {
                if code_len == 0 {
                    return None;
                }
                Bytecode::BrTrue(idx.index(code_len) as CodeOffset)
            }
            BytecodeGen::BrFalse(idx) => {
                if code_len == 0 {
                    return None;
                }
                Bytecode::BrFalse(idx.index(code_len) as CodeOffset)
            }
            BytecodeGen::Branch(idx) => {
                if code_len == 0 {
                    return None;
                }
                Bytecode::Branch(idx.index(code_len) as CodeOffset)
            }
            BytecodeGen::CopyLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::CopyLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::MoveLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::MoveLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::StLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::StLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::MutBorrowLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::MutBorrowLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
            BytecodeGen::ImmBorrowLoc(idx) => {
                if locals_signature.is_empty() {
                    return None;
                }
                Bytecode::ImmBorrowLoc(idx.index(locals_signature.len()) as LocalIndex)
            }
        };

        Some(bytecode)
    }

    fn simple_bytecode_strategy() -> impl Strategy<Value = Bytecode> {
        prop_oneof![
            // The numbers are relative weights, somewhat arbitrarily picked.
            9 => Self::just_bytecode_strategy(),
            1 => any::<u64>().prop_map(Bytecode::LdU64),
            1 => any::<u8>().prop_map(Bytecode::LdU8),
            1 => any::<u128>().prop_map(Bytecode::LdU128),
        ]
    }

    fn just_bytecode_strategy() -> impl Strategy<Value = Bytecode> {
        use Bytecode::*;

        static JUST_BYTECODES: &[Bytecode] = &[
            FreezeRef, Pop, Ret, LdTrue, LdFalse, ReadRef, WriteRef, Add, Sub, Mul, Mod, Div,
            BitOr, BitAnd, Xor, Or, And, Eq, Neq, Lt, Gt, Le, Ge, Abort, CastU8, CastU64, CastU128,
            Not, Nop, Shl, Shr,
        ];
        select(JUST_BYTECODES)
    }
}
