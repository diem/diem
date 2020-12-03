// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    function_target::{FunctionTarget, FunctionTargetData},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder},
    stackless_bytecode::{Bytecode, Operation},
};
use diem_types::account_config;
use move_core_types::language_storage::{StructTag, TypeTag};
use spec_lang::{
    env::{FunctionEnv, GlobalEnv, QualifiedId, StructId},
    ty::Type,
};
use std::collections::BTreeSet;

pub fn get_used_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageAnnotation>()
        .expect("Invariant violation: target not analyzed")
        .used_memory
}

pub fn get_modified_memory<'env>(
    target: &'env FunctionTarget,
) -> &'env BTreeSet<QualifiedId<StructId>> {
    &target
        .get_annotations()
        .get::<UsageAnnotation>()
        .expect("Invariant violation: target not analyzed")
        .modified_memory
}

/// Get all closed types that may be packed by (1) genesis and (2) all transaction scripts.
/// This makes some simplifying assumptions that are not correct in general, but hold for the
/// current Diem Framework:
/// - Transaction scripts have at most 1 type argument
/// - The only values that can be bound to a transaction script type argument are XUS and
///   XDX. Passing any other values will lead to an aborted transaction.
/// The first assumption is checked and will trigger an assert failure if violated. The second
/// is unchecked, but would be a nice property for the prover.
pub fn get_packed_types(env: &GlobalEnv, targets: &FunctionTargetsHolder) -> BTreeSet<StructTag> {
    let mut packed_types = BTreeSet::new();
    for module_env in env.get_modules() {
        let module_name = module_env.get_identifier().to_string();
        let is_script = module_env.is_script_module();
        if is_script || module_name == "Genesis" {
            for func_env in module_env.get_functions() {
                let fun_target = targets.get_target(&func_env);
                let annotation = fun_target
                    .get_annotations()
                    .get::<UsageAnnotation>()
                    .expect(
                        "Invariant violation: usage analysis should be run before calling this",
                    );
                packed_types.extend(annotation.closed_types.clone());
                // instantiate the tx script open types with XUS, XDX
                if is_script {
                    let num_type_parameters = func_env.get_type_parameters().len();
                    assert!(num_type_parameters <= 1, "Assuming that transaction scripts have <= 1 type parameters for simplicity. If there can be >1 type parameter, the code here must account for all permutations of type params");

                    if num_type_parameters == 1 {
                        let coin_types: Vec<Type> =
                            vec![account_config::xus_tag(), account_config::xdx_type_tag()]
                                .into_iter()
                                .map(|t| Type::from_type_tag(t, env))
                                .collect();
                        for open_ty in &annotation.open_types {
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

/// The annotation for usage of functions. This is computed by the function target processor.
#[derive(Default)]
struct UsageAnnotation {
    // The memory which is directly and transitively accessed by this function.
    used_memory: BTreeSet<QualifiedId<StructId>>,
    // The memory which is directly and transitiviely modfied by this function.
    modified_memory: BTreeSet<QualifiedId<StructId>>,
    // Closed types (i.e., with no free type variables) that may be directly or transitively packed by this function.
    closed_types: BTreeSet<StructTag>,
    // Open types (i.e., with free type variables) that may be directly or transitively packed by this function.
    open_types: BTreeSet<Type>,
}

pub struct UsageProcessor();

impl UsageProcessor {
    pub fn new() -> Box<Self> {
        Box::new(UsageProcessor())
    }
}

impl FunctionTargetProcessor for UsageProcessor {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        func_env: &FunctionEnv<'_>,
        mut data: FunctionTargetData,
    ) -> FunctionTargetData {
        let mut annotation = UsageAnnotation::default();

        let func_target = FunctionTarget::new(func_env, &data);
        func_target.get_modify_targets().keys().for_each(|target| {
            annotation.modified_memory.insert(*target);
        });
        if !func_env.is_native() {
            self.analyze(&mut annotation, func_target, targets);
        }
        data.annotations.set(annotation);
        data
    }

    fn name(&self) -> String {
        "usage_analysis".to_string()
    }
}

impl UsageProcessor {
    fn analyze(
        &self,
        annotation: &mut UsageAnnotation,
        func_target: FunctionTarget<'_>,
        targets: &FunctionTargetsHolder,
    ) {
        use Bytecode::*;
        use Operation::*;

        for code in func_target.get_bytecode() {
            if let Call(_, _, oper, _) = code {
                match oper {
                    Function(mid, fid, types) => {
                        let func_env = func_target.global_env().get_function(mid.qualified(*fid));
                        if !func_env.is_native() {
                            let func_target = targets.get_target(&func_env);
                            let summary = func_target
                                .get_annotations()
                                .get::<UsageAnnotation>()
                                .unwrap_or_else(|| {
                                    panic!(
                                        "Failed to look up summary for {:?} in module {:?}",
                                        func_target.func_env.get_identifier(),
                                        func_target.func_env.module_env.get_identifier()
                                    )
                                });

                            annotation.modified_memory.extend(&summary.modified_memory);
                            annotation.used_memory.extend(&summary.used_memory);
                            // add closed types
                            for ty in &summary.closed_types {
                                annotation.closed_types.insert(ty.clone());
                            }
                            // instantiate open types with the type parameters at this call site
                            for open_ty in &summary.open_types {
                                let specialized_ty = open_ty.instantiate(types);
                                if specialized_ty.is_open() {
                                    annotation.open_types.insert(specialized_ty);
                                } else if let Some(TypeTag::Struct(s)) =
                                    specialized_ty.into_type_tag(func_target.global_env())
                                {
                                    annotation.closed_types.insert(s);
                                } else {
                                    panic!("Invariant violation: struct type {:?} became non-struct type after substitution", open_ty)
                                }
                            }
                        }
                    }
                    MoveTo(mid, sid, _) | MoveFrom(mid, sid, _) | BorrowGlobal(mid, sid, _) => {
                        annotation.modified_memory.insert(mid.qualified(*sid));
                        annotation.used_memory.insert(mid.qualified(*sid));
                    }
                    Exists(mid, sid, _) | GetField(mid, sid, ..) | GetGlobal(mid, sid, _) => {
                        annotation.used_memory.insert(mid.qualified(*sid));
                    }
                    Pack(mid, sid, types) => {
                        let env = func_target.global_env();
                        match env.get_struct_tag(*mid, *sid, types) {
                            Some(tag) => {
                                // type is closed. add to closed types summary
                                annotation.closed_types.insert(tag);
                            }
                            None => {
                                // type is open. add to open types summary
                                annotation.open_types.insert(Type::Struct(
                                    *mid,
                                    *sid,
                                    types.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
