// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    compositional_analysis::{CompositionalAnalysis, SummaryCache},
    dataflow_analysis::{DataflowAnalysis, TransferFunctions},
    dataflow_domains::{AbstractDomain, JoinResult, SetDomain},
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    stackless_bytecode::{Bytecode, Operation},
};
use move_binary_format::file_format::CodeOffset;
use move_core_types::language_storage::{StructTag, TypeTag};
use move_model::{
    model::{FunctionEnv, GlobalEnv},
    ty::Type,
};
use std::collections::BTreeSet;

/// Get all closed types that may be packed by (1) genesis and (2) all transaction scripts.
/// This makes some simplifying assumptions that are not correct in general, but hold for the
/// current Diem Framework:
/// - Transaction scripts have at most 1 type argument
/// - The only values that can be bound to a transaction script type argument are XUS and
///   XDX. Passing any other values will lead to an aborted transaction.
/// The first assumption is checked and will trigger an assert failure if violated. The second
/// is unchecked, but would be a nice property for the prover.
pub fn get_packed_types(
    env: &GlobalEnv,
    targets: &FunctionTargetsHolder,
    coin_types: Vec<Type>,
) -> BTreeSet<StructTag> {
    let mut packed_types = BTreeSet::new();
    for module_env in env.get_modules() {
        let module_name = module_env.get_identifier().to_string();
        let is_script = module_env.is_script_module();
        if is_script || module_name == "Genesis" {
            for func_env in module_env.get_functions() {
                let fun_target = targets.get_target(&func_env, &FunctionVariant::Baseline);
                let annotation = fun_target
                    .get_annotations()
                    .get::<PackedTypesState>()
                    .expect(
                        "Invariant violation: usage analysis should be run before calling this",
                    );

                packed_types.extend(annotation.closed_types.clone());
                // instantiate the tx script open types with XUS, XDX
                if is_script {
                    let num_type_parameters = func_env.get_type_parameters().len();
                    assert!(num_type_parameters <= 1, "Assuming that transaction scripts have <= 1 type parameters for simplicity. If there can be >1 type parameter, the code here must account for all permutations of type params");

                    if num_type_parameters == 1 {
                        for open_ty in annotation.open_types.iter() {
                            for coin_ty in &coin_types {
                                match open_ty.instantiate(vec![coin_ty.clone()].as_slice()).into_type_tag(env) {
                                    Some(TypeTag::Struct(s)) =>     {
                                        packed_types.insert(s);
                                    }
                                    _ => panic!("Invariant violation: failed to specialize tx script open type {:?} into struct", open_ty),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    packed_types
}

#[derive(Debug, Clone, Default, Eq, PartialOrd, PartialEq)]
struct PackedTypesState {
    // Closed types (i.e., with no free type variables) that may be directly or transitively packed by this function.
    closed_types: SetDomain<StructTag>,
    // Open types (i.e., with free type variables) that may be directly or transitively packed by this function.
    open_types: SetDomain<Type>,
}

impl AbstractDomain for PackedTypesState {
    // TODO: would be cool to add a derive(Join) macro for this
    fn join(&mut self, other: &Self) -> JoinResult {
        match (
            self.closed_types.join(&other.closed_types),
            self.open_types.join(&other.open_types),
        ) {
            (JoinResult::Unchanged, JoinResult::Unchanged) => JoinResult::Unchanged,
            _ => JoinResult::Changed,
        }
    }
}

struct PackedTypesAnalysis<'a> {
    cache: SummaryCache<'a>,
}

impl<'a> TransferFunctions for PackedTypesAnalysis<'a> {
    type State = PackedTypesState;
    const BACKWARD: bool = false;

    fn execute(&self, state: &mut Self::State, instr: &Bytecode, _offset: CodeOffset) {
        use Bytecode::*;
        use Operation::*;

        if let Call(_, _, oper, ..) = instr {
            match oper {
                Pack(mid, sid, types) => {
                    let env = self.cache.global_env();
                    match env.get_struct_tag(*mid, *sid, types) {
                        Some(tag) => {
                            // type is closed
                            state.closed_types.insert(tag);
                        }
                        None => {
                            // type is open
                            state
                                .open_types
                                .insert(Type::Struct(*mid, *sid, types.clone()));
                        }
                    }
                }
                Function(mid, fid, types) => {
                    if let Some(summary) = self
                        .cache
                        .get::<PackedTypesState>(mid.qualified(*fid), &FunctionVariant::Baseline)
                    {
                        // add closed types
                        for ty in summary.closed_types.iter() {
                            state.closed_types.insert(ty.clone());
                        }
                        // instantiate open types with the type parameters at this call site
                        for open_ty in summary.open_types.iter() {
                            let specialized_ty = open_ty.instantiate(types);
                            if specialized_ty.is_open() {
                                state.open_types.insert(specialized_ty);
                            } else if let Some(TypeTag::Struct(s)) =
                                specialized_ty.into_type_tag(self.cache.global_env())
                            {
                                state.closed_types.insert(s);
                            } else {
                                panic!("Invariant violation: struct type {:?} became non-struct type after substitution", open_ty)
                            }
                        }
                    }
                }
                OpaqueCallBegin(_, _, _) | OpaqueCallEnd(_, _, _) => {
                    // skip
                }
                _ => (),
            }
        }
    }
}

impl<'a> DataflowAnalysis for PackedTypesAnalysis<'a> {}
impl<'a> CompositionalAnalysis<PackedTypesState> for PackedTypesAnalysis<'a> {
    fn to_summary(
        &self,
        state: PackedTypesState,
        _fun_target: &FunctionTarget,
    ) -> PackedTypesState {
        state
    }
}

pub struct PackedTypesProcessor();
impl PackedTypesProcessor {
    pub fn new() -> Box<Self> {
        Box::new(PackedTypesProcessor())
    }
}

impl FunctionTargetProcessor for PackedTypesProcessor {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionData,
    ) -> FunctionData {
        let initial_state = PackedTypesState::default();
        let fun_target = FunctionTarget::new(func_env, &data);
        let cache = SummaryCache::new(targets, func_env.module_env.env);
        let analysis = PackedTypesAnalysis { cache };
        let summary = analysis.summarize(&fun_target, initial_state);
        data.annotations.set(summary);
        data
    }

    fn name(&self) -> String {
        "packed_types_analysis".to_string()
    }
}
