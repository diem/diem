// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes an annotation for each function whether it is verified or inlined.

use crate::{
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::ProverOptions,
    usage_analysis,
};
use log::debug;
use move_model::model::{FunctionEnv, GlobalEnv, QualifiedInstId, StructId, VerificationScope};
use std::collections::BTreeSet;

/// The annotation for information about verification.
#[derive(Clone, Default)]
pub struct VerificationInfo {
    /// Whether the function is target of verification.
    pub verified: bool,
    /// Whether the function needs to have an inlined variant since it is called from a verified
    /// function and is not opaque.
    pub inlined: bool,
}

/// Get verification information for this function.
pub fn get_info(target: &FunctionTarget<'_>) -> VerificationInfo {
    target
        .get_annotations()
        .get::<VerificationInfo>()
        .cloned()
        .unwrap_or_else(VerificationInfo::default)
}

pub struct VerificationAnalysisProcessor();

impl VerificationAnalysisProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl FunctionTargetProcessor for VerificationAnalysisProcessor {
    fn initialize(&self, env: &GlobalEnv, _targets: &mut FunctionTargetsHolder) {
        let options = ProverOptions::get(env);

        // If we are verifying only one function or module, check that this indeed exists.
        match &options.verify_scope {
            VerificationScope::Only(name) | VerificationScope::OnlyModule(name) => {
                let for_module = matches!(&options.verify_scope, VerificationScope::OnlyModule(_));
                let mut target_exists = false;
                for module in env.get_modules() {
                    if module.is_target() {
                        if for_module {
                            target_exists = module.matches_name(name)
                        } else {
                            target_exists = module.get_functions().any(|f| f.matches_name(name));
                        }
                        if target_exists {
                            break;
                        }
                    }
                }
                if !target_exists {
                    env.error(
                        &env.unknown_loc(),
                        &format!(
                            "{} target {} does not exist in target modules",
                            if for_module { "module" } else { "function" },
                            name
                        ),
                    )
                }
            }
            _ => {}
        }
    }

    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        // When we are called, the data of this function is removed from targets so it can
        // be mutated, as per pipeline processor design. We put it back temporarily to have
        // a unique model of targets.
        let fid = fun_env.get_qualified_id();
        let variant = data.variant.clone();
        targets.insert_target_data(&fid, variant.clone(), data);

        // Check the friend relation.
        check_friend_relation(fun_env);

        let options = ProverOptions::get(fun_env.module_env.env);
        let is_verified = {
            // Whether this function is a verification target provided explicitly by the user.
            let is_explicitly_verified =
                fun_env.module_env.is_target() && fun_env.should_verify(&options.verify_scope);
            if options.verify_scope.is_exclusive() {
                // If the verification is set exclusive to function or module, don't add friends
                // for verification.
                is_explicitly_verified
            } else {
                // Get all memory mentioned in the invariants in target modules
                let target_memory = get_target_invariant_memory(fun_env.module_env.env);

                // Get all memory modified by this function.
                let fun_target = targets.get_target(fun_env, &variant);
                let modified_memory =
                    usage_analysis::get_directly_modified_memory_inst(&fun_target);

                // This function needs to be verified if it is a target or it touches target memory.
                is_explicitly_verified || !modified_memory.is_disjoint(&target_memory)
            }
        };
        if is_verified {
            debug!("marking `{}` to be verified", fun_env.get_full_name_str());
            mark_verified(fun_env, &variant, targets, &options);
        }

        targets.remove_target_data(&fid, &variant)
    }

    fn name(&self) -> String {
        "verification_analysis".to_string()
    }
}

/// Checks whether the friend relation is correctly used.
fn check_friend_relation(fun_env: &FunctionEnv<'_>) {
    if fun_env.has_friend() {
        let env = fun_env.module_env.env;
        let loc = fun_env.get_loc();
        if fun_env.is_opaque() {
            env.error(&loc, "function with a friend cannot be declared as opaque");
        }
        let calling_functions = fun_env.get_calling_functions();

        // Construct a set containing all friends of the function in the system.
        // Right now there can be at most one friend.
        let mut friends = BTreeSet::new();
        if let Some(friend_env) = fun_env.get_friend_env() {
            friends.insert(friend_env.get_qualified_id());
        }

        // If the set of callers is not a subset of the friends set,
        // then some non-friend calls the function so generate an error message.
        if !calling_functions.is_subset(&friends) {
            env.error(&loc, &format!("function `{}` is called by other functions while it can only be called by its friend {}",
                                         fun_env.get_name_string(),
                                         fun_env.get_friend_name().unwrap())
                );
        }
    }
}

/// Compute the set of resources which are used in invariants which are target of
/// verification.
fn get_target_invariant_memory(env: &GlobalEnv) -> BTreeSet<QualifiedInstId<StructId>> {
    let mut target_resources = BTreeSet::new();
    for module_env in env.get_modules() {
        if module_env.is_target() {
            let module_id = module_env.get_id();
            let mentioned_resources: BTreeSet<QualifiedInstId<StructId>> = env
                .get_global_invariants_by_module(module_id)
                .iter()
                .flat_map(|id| env.get_global_invariant(*id).unwrap().mem_usage.clone())
                .collect();
            target_resources.extend(mentioned_resources);
        }
    }
    target_resources
}

/// Mark this function as being verified. If it has a friend and is verified only in the
/// friends context, mark the friend instead. This also marks all functions directly or
/// indirectly called by this function as inlined if they are not opaque.
fn mark_verified(
    fun_env: &FunctionEnv<'_>,
    variant: &FunctionVariant,
    targets: &mut FunctionTargetsHolder,
    options: &ProverOptions,
) {
    let actual_env = fun_env.get_transitive_friend();
    if actual_env.get_qualified_id() != fun_env.get_qualified_id() {
        // Instead of verifying this function directly, we mark the friend as being verified,
        // and this function as inlined.
        mark_inlined(fun_env, variant, targets);
    }
    // The user can override with `pragma verify = false` to verify the friend, so respect this.
    if !actual_env.is_explicitly_not_verified(&options.verify_scope) {
        let mut info = targets
            .get_data_mut(&actual_env.get_qualified_id(), variant)
            .expect("function data available")
            .annotations
            .get_or_default_mut::<VerificationInfo>();
        if !info.verified {
            info.verified = true;
            mark_callees_inlined(&actual_env, variant, targets);
        }
    }
}

/// Mark this function as inlined if it is not opaque, as it is being called
/// directly or indirectly from a verified function.
fn mark_inlined(
    fun_env: &FunctionEnv<'_>,
    variant: &FunctionVariant,
    targets: &mut FunctionTargetsHolder,
) {
    if fun_env.is_native() || fun_env.is_intrinsic() {
        return;
    }
    debug_assert!(
        targets.get_target_variants(fun_env).contains(&variant),
        "`{}` has variant `{:?}`",
        fun_env.get_name().display(fun_env.symbol_pool()),
        variant
    );
    let data = targets
        .get_data_mut(&fun_env.get_qualified_id(), variant)
        .expect("function data defined");
    let info = data.annotations.get_or_default_mut::<VerificationInfo>();
    if !info.inlined {
        info.inlined = true;
        mark_callees_inlined(fun_env, variant, targets);
    }
}

/// Continue transitively marking callees as inlined.
fn mark_callees_inlined(
    fun_env: &FunctionEnv<'_>,
    variant: &FunctionVariant,
    targets: &mut FunctionTargetsHolder,
) {
    for callee in fun_env.get_called_functions() {
        let callee_env = fun_env.module_env.env.get_function(callee);
        if !callee_env.is_opaque() {
            mark_inlined(&callee_env, variant, targets);
        }
    }
}
