// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes an annotation for each function whether

use crate::{
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::ProverOptions,
    usage_analysis,
};
use move_model::{
    model::{FunId, FunctionEnv, GlobalEnv, GlobalId, QualifiedId},
    pragmas::{DELEGATE_INVARIANTS_TO_CALLER_PRAGMA, DISABLE_INVARIANTS_IN_BODY_PRAGMA},
};
use std::collections::{BTreeMap, BTreeSet};

/// The annotation for information about verification.
#[derive(Clone, Default)]
pub struct VerificationInfoV2 {
    /// Whether the function is target of verification.
    pub verified: bool,
    /// Whether the function needs to have an inlined variant since it is called from a verified
    /// function and is not opaque.
    pub inlined: bool,
}

/// Get verification information for this function.
pub fn get_info(target: &FunctionTarget<'_>) -> VerificationInfoV2 {
    target
        .get_annotations()
        .get::<VerificationInfoV2>()
        .cloned()
        .unwrap_or_else(VerificationInfoV2::default)
}

// Analysis info to save for global_invariant_instrumentation phase
pub struct InvariantAnalysisData {
    /// global invariants in the target module
    pub target_invariants: BTreeSet<GlobalId>,
    /// Invariants that appear in target module (?) or dependencies
    /// Functions where invariants are disabled in body
    pub disabled_inv_fun_set: BTreeSet<QualifiedId<FunId>>,
    /// Functions where invariants are disabled in a transitive caller (or by pragma)
    pub non_inv_fun_set: BTreeSet<QualifiedId<FunId>>,
    /// The set of all functions in target module.
    pub target_fun_ids: BTreeSet<QualifiedId<FunId>>,
    /// Functions in dependent modules that are transitively called by functions in target module.
    pub dep_fun_ids: BTreeSet<QualifiedId<FunId>>,
    /// Maps invariant ID to set of functions that modify the invariant
    pub funs_that_modify_inv: BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>>,
    /// Functions that modify some invariant in the target
    pub funs_that_modify_some_inv: BTreeSet<QualifiedId<FunId>>,
    /// functions that are in non_inv_fun_set and M[I] for some I.
    /// We have to verify the callers, which may be in friend modules.
    pub funs_that_delegate_to_caller: BTreeSet<QualifiedId<FunId>>,
    /// Functions that are not in target or deps, but that call a function
    /// in non_inv_fun_set that modifies some invariant from target module
    /// and eventually calls a function in target mod or a dependency.
    pub friend_fun_ids: BTreeSet<QualifiedId<FunId>>,
}

/// Get all invariants from target module
fn get_target_invariants(global_env: &GlobalEnv) -> BTreeSet<GlobalId> {
    let module_id = global_env.get_target_module().get_id();
    global_env.get_global_invariants_by_module(module_id)
}

fn compute_disabled_inv_fun_set(global_env: &GlobalEnv) -> BTreeSet<QualifiedId<FunId>> {
    let mut disabled_inv_fun_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    for module_env in global_env.get_modules() {
        for fun_env in module_env.get_functions() {
            let fun_id = fun_env.get_qualified_id();
            if fun_env.is_pragma_true(DISABLE_INVARIANTS_IN_BODY_PRAGMA, || false) {
                disabled_inv_fun_set.insert(fun_id);
            }
        }
    }
    disabled_inv_fun_set
}

// Find functions called from a context where invariant checking is disabled.
// Whenever a function has this property, so do its callees.
// TODO: Error checking
fn compute_non_inv_fun_set(global_env: &GlobalEnv) -> BTreeSet<QualifiedId<FunId>> {
    let mut fun_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    // invariant: If a function is in fun_set and not in worklist,
    // then all the functions it calls are also in fun_set
    // or in worklist.  When worklist is empty, all callees of a function
    // in fun_set will also be in fun_set.
    // Each function is added at most once to the worklist.
    let mut worklist = vec![];
    for module_env in global_env.get_modules() {
        for fun_env in module_env.get_functions() {
            if fun_env.is_pragma_true(DISABLE_INVARIANTS_IN_BODY_PRAGMA, || false) {
                let fun_id = fun_env.get_qualified_id();
                worklist.push(fun_id);
            }
            if fun_env.is_pragma_true(DELEGATE_INVARIANTS_TO_CALLER_PRAGMA, || false) {
                let fun_id = fun_env.get_qualified_id();
                if fun_set.insert(fun_id) {
                    // Add to work_list only if fun_id is not in fun_set (may have inferred
                    // this from a caller already).
                    worklist.push(fun_id);
                }
            }
            // This is a little faster than getting the transitive callees of each function
            while let Some(called_fun_id) = worklist.pop() {
                let called_funs = global_env
                    .get_function(called_fun_id)
                    .get_called_functions();
                for called_fun_id in called_funs {
                    if fun_set.insert(called_fun_id) {
                        // Add to work_list only if fun_id is not in fun_set
                        worklist.push(called_fun_id);
                    }
                }
            }
        }
    }
    fun_set
}

/// Collect all functions that are defined in the target module, or called transitively
/// from those functions.
fn compute_dep_fun_ids(
    global_env: &GlobalEnv,
    target_fun_ids: &BTreeSet<QualifiedId<FunId>>,
) -> BTreeSet<QualifiedId<FunId>> {
    let mut dep_fun_ids = BTreeSet::new();
    let mut worklist = vec![];
    worklist.extend(target_fun_ids);
    while let Some(fun_id) = worklist.pop() {
        // Find the called functions that are not already in called_from_target
        let called = global_env.get_function(fun_id).get_called_functions();
        for called_fun_id in called {
            if !dep_fun_ids.insert(called_fun_id) {
                worklist.push(called_fun_id);
            }
        }
    }
    dep_fun_ids
}

/// Compute a map from each invariant to the set of functions that modify state
/// appearing in the invariant. Return that, and a second value that is the union
/// of functions over all invariants in the first map.
fn compute_funs_that_modify_inv(
    global_env: &GlobalEnv,
    target_invariants: &BTreeSet<GlobalId>,
    targets: &mut FunctionTargetsHolder,
    variant: FunctionVariant,
) -> (
    BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>>,
    BTreeSet<QualifiedId<FunId>>,
) {
    let mut funs_that_modify_inv: BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>> =
        BTreeMap::new();
    let mut funs_that_modify_some_inv: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    for inv_id in target_invariants {
        // Collect the global state used by inv_id (this is computed in usage_analysis.rs)
        let inv_mem_use = global_env
            .get_global_invariant(*inv_id)
            .unwrap()
            .mem_usage
            .clone();
        // set of functions that modify state in inv_id that we are building
        let mut fun_id_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
        // Iterate over all functions in the module cluster
        for module_env in global_env.get_modules() {
            for fun_env in module_env.get_functions() {
                // Get all memory modified by this function.
                let fun_target = targets.get_target(&fun_env, &variant);
                let modified_memory =
                    usage_analysis::get_directly_modified_memory_inst(&fun_target);
                // Add functions to set if it modifies mem used in invariant
                if !modified_memory.is_disjoint(&inv_mem_use) {
                    let fun_id = fun_env.get_qualified_id();
                    fun_id_set.insert(fun_id);
                    funs_that_modify_some_inv.insert(fun_id);
                }
            }
        }
        if !fun_id_set.is_empty() {
            funs_that_modify_inv.insert(*inv_id, fun_id_set);
        }
    }
    (funs_that_modify_inv, funs_that_modify_some_inv)
}

/// Compute the set of functions that are friend modules of target or deps, but not in
/// target or deps, and that call a function in non_inv_fun_set that modifies some target
/// invariant.  The Prover needs to verify that these functions preserve the target invariants.
fn compute_friend_fun_ids(
    global_env: &GlobalEnv,
    target_fun_ids: &BTreeSet<QualifiedId<FunId>>,
    dep_fun_ids: &BTreeSet<QualifiedId<FunId>>,
    funs_that_delegate_to_caller: &BTreeSet<QualifiedId<FunId>>,
) -> BTreeSet<QualifiedId<FunId>> {
    let mut friend_fun_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    let mut worklist: Vec<QualifiedId<FunId>> = target_fun_ids.iter().cloned().collect();
    worklist.extend(dep_fun_ids.iter().cloned());
    while let Some(fun_id) = worklist.pop() {
        if funs_that_delegate_to_caller.contains(&fun_id) {
            let fun_env = global_env.get_function(fun_id);
            let callers = fun_env.get_calling_functions();
            for caller_fun_id in callers {
                // Exclude callers that are in target or dep modules, because we will verify them, anyway.
                // We also don't need to put them in the worklist, because they were in there initially.
                // Also, don't need to process if it's already in friend_fun_set
                if !target_fun_ids.contains(&caller_fun_id)
                    && !dep_fun_ids.contains(&caller_fun_id)
                    && friend_fun_set.insert(caller_fun_id)
                {
                    worklist.push(caller_fun_id);
                }
            }
        }
    }
    friend_fun_set
}

/// Debugging function to print a set of function id's using their
/// symbolic function names.
#[allow(dead_code)]
fn print_fun_id_set(
    global_env: &GlobalEnv,
    fun_ids: &BTreeSet<QualifiedId<FunId>>,
    set_name: &str,
) {
    println!(
        "{}: {:?}",
        set_name,
        fun_ids
            .iter()
            .map(|fun_id| global_env.get_function(*fun_id).get_name_string())
            .collect::<Vec<_>>()
    );
}

/// Print sets and maps computed during verification analysis
/// TODO: Complete this and write it properly as Display
#[allow(dead_code)]
fn print_invariant_analysis_data(global_env: &GlobalEnv, inv_ana_data: &InvariantAnalysisData) {
    println!("target_invariants <can't print yet>");
    print_fun_id_set(
        global_env,
        &inv_ana_data.disabled_inv_fun_set,
        "disabled_inv_fun_set",
    );
    print_fun_id_set(global_env, &inv_ana_data.non_inv_fun_set, "non_inv_fun_set");
    print_fun_id_set(global_env, &inv_ana_data.target_fun_ids, "target_fun_ids");
    //    print_fun_id_set(global_env, inv_ana_data.funs_that_modify_inv, "funs_that_modify_inv");
    println!("funs_that_modify_inv <can't print yet>");
    print_fun_id_set(
        global_env,
        &inv_ana_data.funs_that_modify_some_inv,
        "funs_that_modify_some_inv",
    );
    print_fun_id_set(
        global_env,
        &inv_ana_data.funs_that_delegate_to_caller,
        "funs_that_delegate_to_caller",
    );
    print_fun_id_set(global_env, &inv_ana_data.friend_fun_ids, "friend_fun_ids");
}

pub struct VerificationAnalysisProcessorV2();

impl VerificationAnalysisProcessorV2 {
    pub fn new() -> Box<Self> {
        Box::new(Self())
    }
}

impl FunctionTargetProcessor for VerificationAnalysisProcessorV2 {
    fn process(
        &self,
        targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        // When we are called, the data of this function is removed from targets so it can
        // be mutated, as per pipeline processor design. We put it back temporarily to have
        // a unique model of targets.
        let global_env = fun_env.module_env.env;
        let fun_id = fun_env.get_qualified_id();
        let variant = data.variant.clone();
        targets.insert_target_data(&fun_id, variant.clone(), data);
        let inv_ana_data = global_env.get_extension::<InvariantAnalysisData>().unwrap();
        let target_fun_ids = &inv_ana_data.target_fun_ids;
        let dep_fun_ids = &inv_ana_data.dep_fun_ids;
        let friend_fun_ids = &inv_ana_data.friend_fun_ids;
        let funs_that_modify_some_inv = &inv_ana_data.funs_that_modify_some_inv;
        // verify the function if
        // * user did not have a pragma verify=false on it, and
        //    - it is in target module, or
        //    - it is in dep and can modify an invariant, or
        //    - it is a friend caller [ modify invariant is factored in. Seems inconsistent? ]
        let options = ProverOptions::get(global_env);
        if !fun_env.is_explicitly_not_verified(&options.verify_scope)
            && (target_fun_ids.contains(&fun_id)
                || (dep_fun_ids.contains(&fun_id) && funs_that_modify_some_inv.contains(&fun_id))
                || friend_fun_ids.contains(&fun_id))
        {
            mark_verified(fun_env, variant.clone(), targets);
        }

        targets.remove_target_data(&fun_id, &variant)
    }

    fn name(&self) -> String {
        "verification_analysis_v2".to_string()
    }

    fn initialize(&self, global_env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        let target_mod_env = global_env.get_target_module();
        let target_invariants = get_target_invariants(global_env);
        let disabled_inv_fun_set = compute_disabled_inv_fun_set(global_env);
        let non_inv_fun_set = compute_non_inv_fun_set(global_env);
        let (funs_that_modify_inv, funs_that_modify_some_inv) = compute_funs_that_modify_inv(
            global_env,
            &target_invariants,
            targets,
            FunctionVariant::Baseline,
        );
        let target_fun_ids: BTreeSet<QualifiedId<FunId>> = target_mod_env
            .get_functions()
            .map(|fun| fun.get_qualified_id())
            .collect();
        let dep_fun_ids = compute_dep_fun_ids(global_env, &target_fun_ids);
        let funs_that_delegate_to_caller = non_inv_fun_set
            .intersection(&funs_that_modify_some_inv)
            .cloned()
            .collect();
        let friend_fun_ids = compute_friend_fun_ids(
            global_env,
            &target_fun_ids,
            &dep_fun_ids,
            &funs_that_delegate_to_caller,
        );
        let inv_ana_data = InvariantAnalysisData {
            target_invariants,
            disabled_inv_fun_set,
            non_inv_fun_set,
            target_fun_ids,
            dep_fun_ids,
            funs_that_modify_inv,
            funs_that_modify_some_inv,
            funs_that_delegate_to_caller,
            friend_fun_ids,
        };

        // TODO: Print when debugging.
        //        print_invariant_analysis_data(global_env, &inv_ana_data);

        global_env.set_extension(inv_ana_data);
    }
}

/// Mark this function as being verified. If it has a friend and is verified only in the
/// friends context, mark the friend instead. This also marks all functions directly or
/// indirectly called by this function as inlined if they are not opaque.
fn mark_verified(
    fun_env: &FunctionEnv<'_>,
    variant: FunctionVariant,
    targets: &mut FunctionTargetsHolder,
) {
    let actual_env = fun_env.get_transitive_friend();
    if actual_env.get_qualified_id() != fun_env.get_qualified_id() {
        // Instead of verifying this function directly, we mark the friend as being verified,
        // and this function as inlined.
        mark_inlined(fun_env, variant.clone(), targets);
    }
    // The user can override with `pragma verify = false`, so respect this.
    let options = ProverOptions::get(fun_env.module_env.env);
    if !actual_env.is_explicitly_not_verified(&options.verify_scope) {
        let mut info = targets
            .get_data_mut(&actual_env.get_qualified_id(), &variant)
            .expect("function data available")
            .annotations
            .get_or_default_mut::<VerificationInfoV2>();
        if !info.verified {
            info.verified = true;
            mark_callees_inlined(&actual_env, variant, targets);
        }
    }
}

/// Mark this function as inlined if it is not opaque, and if it is
/// are called from a verified function via a chain of zero-or-more
/// inline functions.  If it is not called from a verified function,
/// it does not need to be inlined.
fn mark_inlined(
    fun_env: &FunctionEnv<'_>,
    variant: FunctionVariant,
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
        .get_data_mut(&fun_env.get_qualified_id(), &variant)
        .expect("function data defined");
    let info = data.annotations.get_or_default_mut::<VerificationInfoV2>();
    if !info.inlined {
        info.inlined = true;
        mark_callees_inlined(fun_env, variant, targets);
    }
}

/// Continue transitively marking callees as inlined.
fn mark_callees_inlined(
    fun_env: &FunctionEnv<'_>,
    variant: FunctionVariant,
    targets: &mut FunctionTargetsHolder,
) {
    for callee in fun_env.get_called_functions() {
        let callee_env = fun_env.module_env.env.get_function(callee);
        if !callee_env.is_opaque() {
            mark_inlined(&callee_env, variant.clone(), targets);
        }
    }
}
