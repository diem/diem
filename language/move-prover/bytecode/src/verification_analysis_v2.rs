// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Analysis which computes an annotation for each function whether

use crate::{
    dataflow_domains::SetDomain,
    function_target::{FunctionData, FunctionTarget},
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::ProverOptions,
    usage_analysis,
};
use log::debug;
use move_model::{
    model::{FunId, FunctionEnv, GlobalEnv, GlobalId, ModuleEnv, QualifiedId, VerificationScope},
    pragmas::{
        CONDITION_SUSPENDABLE_PROP, DELEGATE_INVARIANTS_TO_CALLER_PRAGMA,
        DISABLE_INVARIANTS_IN_BODY_PRAGMA, VERIFY_PRAGMA,
    },
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

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
        .unwrap_or_default()
}

// Analysis info to save for global_invariant_instrumentation phase
pub struct InvariantAnalysisData {
    /// The set of all functions in target module.
    pub target_fun_ids: BTreeSet<QualifiedId<FunId>>,
    /// Functions in dependent modules that are transitively called by functions in target module.
    pub dep_fun_ids: BTreeSet<QualifiedId<FunId>>,
    /// functions where invariants are disabled by pragma disable_invariants_in_body
    pub disabled_inv_fun_set: BTreeSet<QualifiedId<FunId>>,
    /// Functions where invariants are disabled in a transitive caller, or by
    /// pragma delegate_invariant_to_caller
    pub non_inv_fun_set: BTreeSet<QualifiedId<FunId>>,
    /// global and update invariants in the target module
    pub target_invariants: BTreeSet<GlobalId>,
    /// Maps invariant ID to set of functions that modify the invariant
    /// Does not include update invariants
    pub funs_that_modify_inv: BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>>,
    /// Maps function to the set of invariants that it modifies
    /// Does not include update invariants
    pub invs_modified_by_fun: BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>>,
    /// Functions that modify some invariant in the target
    /// Does not include update invariants
    pub funs_that_modify_some_inv: BTreeSet<QualifiedId<FunId>>,
    /// functions that are in non_inv_fun_set and M[I] for some I.
    /// We have to verify the callers, which may be in friend modules.
    pub funs_that_delegate_to_caller: BTreeSet<QualifiedId<FunId>>,
    /// Functions that are not in target or deps, but that call a function
    /// in non_inv_fun_set that modifies some invariant from target module
    /// and eventually calls a function in target mod or a dependency.
    pub friend_fun_ids: BTreeSet<QualifiedId<FunId>>,
    /// For each function, give the set of invariants that are disabled in that function.
    /// This is defined as the least set satisfying set inequalities: (1) in a function where
    /// invariants are disabled, it is the set of invariants modified in the function, and
    /// (2) in a function in non_inv_fun_set, it is the least set that includes all disabled_invs
    /// for calling functions.
    pub disabled_invs_for_fun: BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>>,
}

/// Get all invariants from target modules
fn get_target_invariants(
    global_env: &GlobalEnv,
    target_modules: &[ModuleEnv],
) -> BTreeSet<GlobalId> {
    let target_mod_ids = target_modules
        .iter()
        .map(|mod_env| mod_env.get_id())
        .flat_map(|target_mod_id| global_env.get_global_invariants_by_module(target_mod_id))
        .collect();
    target_mod_ids
}

/// Computes and returns the set of disabled invariants for each function in disabled_inv_fun_set
/// Disabled invariants for a function are the invariants modified (directly or indirectly) by the fun
/// that are also declared to be suspendable via "invariant [suspendable] ..."
fn compute_disabled_invs_for_fun(
    global_env: &GlobalEnv,
    disabled_inv_fun_set: &BTreeSet<QualifiedId<FunId>>,
    invs_modified_by_fun: &BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>>,
) -> BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>> {
    let mut disabled_invs_for_fun: BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>> =
        BTreeMap::new();
    for module_env in global_env.get_modules() {
        for fun_env in module_env.get_functions() {
            let fun_id = fun_env.get_qualified_id();
            // If function disables invariants, get the set of invariants modified in the function
            // and keep only those that are declared to be suspendable
            if disabled_inv_fun_set.contains(&fun_id) {
                if let Some(modified_invs) = invs_modified_by_fun.get(&fun_id) {
                    let disabled_invs: BTreeSet<GlobalId> = modified_invs
                        .iter()
                        .filter(|inv_id| {
                            global_env
                                .is_property_true(
                                    &global_env
                                        .get_global_invariant(**inv_id)
                                        .unwrap()
                                        .properties,
                                    CONDITION_SUSPENDABLE_PROP,
                                )
                                .unwrap_or(false)
                        })
                        .cloned()
                        .collect();
                    debug_print_inv_set(
                        global_env,
                        &disabled_invs,
                        "$$$$$$$$$$$$$$$$\ncompute_disabled_invs_for_fun",
                    );
                    disabled_invs_for_fun.insert(fun_id, disabled_invs.clone());
                }
            }
        }
    }

    // Compute a least fixed point of disable_invs_for_fun.  Starts with disabled inv functions and
    // all invariants that they modify. Then propagate those to called functions.  They're not top-sorted
    // (which may not be good enough for recursion, in this case, I'm not sure).  So fun_ids go back
    // into the worklist until the disable_inv_set for each fun converges (worklist will be empty).
    let mut worklist: VecDeque<QualifiedId<FunId>> = disabled_inv_fun_set.iter().cloned().collect();
    while let Some(caller_fun_id) = worklist.pop_front() {
        // If None, it's ok to skip because there are no disabled_invs to propagate to called funs
        if let Some(disabled_invs_for_caller) = disabled_invs_for_fun.remove(&caller_fun_id) {
            let called_funs = global_env
                .get_function(caller_fun_id)
                .get_called_functions();
            for called_fun_id in called_funs {
                let disabled_invs_for_called = disabled_invs_for_fun
                    .entry(called_fun_id)
                    .or_insert_with(BTreeSet::new);
                // if caller has any disabled_invs that callee does not, add them to called
                // and add called to the worklist for further processing
                if !disabled_invs_for_caller.is_subset(disabled_invs_for_called) {
                    // Add missing inv_ids to called set
                    for inv_id in &disabled_invs_for_caller {
                        disabled_invs_for_called.insert(*inv_id);
                    }
                    worklist.push_back(called_fun_id);
                }
            }
            // put back disabled_invs_for_caller
            disabled_invs_for_fun.insert(caller_fun_id, disabled_invs_for_caller);
        }
    }
    disabled_invs_for_fun
}

/// Check whether function is callable from unknown sites (i.e., it is public or
/// a script fun) and modifies some invariant in the target module.
/// The second condition is an exception for functions that cannot invalidate
/// any invariants.
fn check_legal_disabled_invariants(
    _fun_env: &FunctionEnv,
    _disabled_inv_fun_set: &BTreeSet<QualifiedId<FunId>>,
    _non_inv_fun_set: &BTreeSet<QualifiedId<FunId>>,
    _funs_that_modify_some_inv: &BTreeSet<QualifiedId<FunId>>,
) {

    // TODO: this currently produces a false positive for unknown reasons. It is hard to
    //   debug because the error message does not give any info about the source of
    //   invariant disabling. To give better error messages, we need to extend the analysis
    //   to track locations.
    /*
    let global_env = fun_env.module_env.env;
    let fun_id = fun_env.get_qualified_id();
    if non_inv_fun_set.contains(&fun_id) && funs_that_modify_some_inv.contains(&fun_id) {
        // function is public or script, which means we don't know whether invariants hold
        // or not at the call site
        if fun_env.has_unknown_callers() {
            global_env.error(
                &fun_env.get_loc(),
                "Public or script functions must not be called when invariants are disabled \
                 in a transitive caller or there is a pragma delegate_invariants_to_caller",
            );
        } else if disabled_inv_fun_set.contains(&fun_id) {
            global_env.error(
                &fun_env.get_loc(),
                "Functions must not have a disable invariant pragma when invariants are disabled \
                 in a transitive caller or there is a pragma delegate_invariants_to_caller",
            );
        }
    }
     */
}

// Using pragmas, find functions called from a context where invariant
// checking is disabled.
// disabled_inv_fun set are disabled by pragma
// non_inv_fun_set are disabled by pragma or because they're called from
// another function in disabled_inv_fun_set or non_inv_fun_set.
fn compute_disabled_and_non_inv_fun_sets(
    global_env: &GlobalEnv,
) -> (BTreeSet<QualifiedId<FunId>>, BTreeSet<QualifiedId<FunId>>) {
    let mut non_inv_fun_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    let mut disabled_inv_fun_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    // invariant: If a function is in non_inv_fun_set and not in worklist,
    // then all the functions it calls are also in fun_set
    // or in worklist.  When worklist is empty, all callees of a function
    // in non_inv_fun_set will also be in non_inv_fun_set.
    // Each function is added at most once to the worklist.
    let mut worklist = vec![];
    for module_env in global_env.get_modules() {
        for fun_env in module_env.get_functions() {
            if fun_env.is_pragma_true(DISABLE_INVARIANTS_IN_BODY_PRAGMA, || false) {
                let fun_id = fun_env.get_qualified_id();
                disabled_inv_fun_set.insert(fun_id);
                worklist.push(fun_id);
            }
            if fun_env.is_pragma_true(DELEGATE_INVARIANTS_TO_CALLER_PRAGMA, || false) {
                let fun_id = fun_env.get_qualified_id();
                if non_inv_fun_set.insert(fun_id) {
                    // Add to work_list only if fun_id is not in non_inv_fun_set (may have inferred
                    // this from a caller already).
                    worklist.push(fun_id);
                }
            }
            // Downward closure of non_inv_fun_set
            while let Some(called_fun_id) = worklist.pop() {
                let called_funs = global_env
                    .get_function(called_fun_id)
                    .get_called_functions();
                for called_fun_id in called_funs {
                    if non_inv_fun_set.insert(called_fun_id) {
                        // Add to work_list only if fun_id is not in fun_set
                        worklist.push(called_fun_id);
                    }
                }
            }
        }
    }
    (disabled_inv_fun_set, non_inv_fun_set)
}

/// Collect all functions that are defined in the target module, or called transitively
/// from those functions.
/// TODO: This is not very efficient.  It would be better to compute the transitive closure.
fn compute_dep_fun_ids(
    global_env: &GlobalEnv,
    target_modules: &[ModuleEnv],
) -> BTreeSet<QualifiedId<FunId>> {
    let mut dep_fun_ids = BTreeSet::new();
    for module_env in global_env.get_modules() {
        for target_env in target_modules {
            if target_env.is_transitive_dependency(module_env.get_id()) {
                for fun_env in module_env.get_functions() {
                    dep_fun_ids.insert(fun_env.get_qualified_id());
                }
            }
        }
    }
    dep_fun_ids
}

/// Compute a map from each invariant to the set of functions that modify state
/// appearing in the invariant. Return that, and a second value that is the union
/// of functions over all invariants in the first map.
/// This is not applied to update invariants?
fn compute_funs_that_modify_inv(
    global_env: &GlobalEnv,
    target_invariants: &BTreeSet<GlobalId>,
    targets: &mut FunctionTargetsHolder,
    variant: FunctionVariant,
) -> (
    BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>>,
    BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>>,
    BTreeSet<QualifiedId<FunId>>,
) {
    let mut funs_that_modify_inv: BTreeMap<GlobalId, BTreeSet<QualifiedId<FunId>>> =
        BTreeMap::new();
    let mut funs_that_modify_some_inv: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
    let mut invs_modified_by_fun: BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>> =
        BTreeMap::new();
    for inv_id in target_invariants {
        // Collect the global state used by inv_id (this is computed in usage_analysis.rs)
        let inv_mem_use: SetDomain<_> = global_env
            .get_global_invariant(*inv_id)
            .unwrap()
            .mem_usage
            .iter()
            .cloned()
            .collect();
        // set of functions that modify state in inv_id that we are building
        let mut fun_id_set: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
        // Iterate over all functions in the module cluster
        for module_env in global_env.get_modules() {
            for fun_env in module_env.get_functions() {
                // Get all memory modified by this function.
                let fun_target = targets.get_target(&fun_env, &variant);
                let modified_memory = usage_analysis::get_modified_memory_inst(&fun_target);
                // Add functions to set if it modifies mem used in invariant
                // TODO: This should be using unification.
                if !modified_memory.is_disjoint(&inv_mem_use) {
                    let fun_id = fun_env.get_qualified_id();
                    fun_id_set.insert(fun_id);
                    funs_that_modify_some_inv.insert(fun_id);
                    let inv_set = invs_modified_by_fun
                        .entry(fun_id)
                        .or_insert_with(BTreeSet::new);
                    inv_set.insert(*inv_id);
                }
            }
        }
        if !fun_id_set.is_empty() {
            funs_that_modify_inv.insert(*inv_id, fun_id_set);
        }
    }
    (
        funs_that_modify_inv,
        invs_modified_by_fun,
        funs_that_modify_some_inv,
    )
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
        // Check for legacy friend pragma
        // TODO: Delete when we stop using pragma friend in DiemFramework
        let fun_env = global_env.get_function(fun_id);
        let friend_env = fun_env.get_transitive_friend();
        let friend_id = friend_env.get_qualified_id();
        // if no transitive friend, it just returns the original fun_env
        if friend_id != fun_env.get_qualified_id() && friend_fun_set.insert(friend_id) {
            worklist.push(friend_id);
        }
        if funs_that_delegate_to_caller.contains(&fun_id) {
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

#[allow(dead_code)]
/// Debug print: Print global id and body of each invariant, so we can just print the global
/// id's in sets for compactness
fn debug_print_global_ids(global_env: &GlobalEnv, global_ids: &BTreeSet<GlobalId>) {
    for inv_id in global_ids {
        debug_print_inv_full(global_env, inv_id);
    }
}

/// Debugging function to print a set of function id's using their
/// symbolic function names.
#[allow(dead_code)]
fn debug_print_fun_id_set(
    global_env: &GlobalEnv,
    fun_ids: &BTreeSet<QualifiedId<FunId>>,
    set_name: &str,
) {
    debug!(
        "****************\n{}: {:?}",
        set_name,
        fun_ids
            .iter()
            .map(|fun_id| global_env.get_function(*fun_id).get_name_string())
            .collect::<Vec<_>>()
    );
}

/// Debugging code to print sets of invariants
#[allow(dead_code)]
pub fn debug_print_inv_set(
    global_env: &GlobalEnv,
    global_ids: &BTreeSet<GlobalId>,
    set_name: &str,
) {
    if global_ids.is_empty() {
        return;
    }
    debug!("{}:", set_name);
    // for global_id in global_ids {
    //     debug!("global_id: {:?}", *global_id);
    // }
    debug!("++++++++++++++++\n{}:", set_name);
    for inv_id in global_ids {
        debug_print_inv_full(global_env, inv_id);
    }
}

/// Given global id of invariant, prints the global ID and the source code
/// of the invariant
#[allow(dead_code)]
fn debug_print_inv_full(global_env: &GlobalEnv, inv_id: &GlobalId) {
    let inv = global_env.get_global_invariant(*inv_id);
    let loc = &inv.unwrap().loc;
    debug!(
        "{:?} {:?}: {}",
        *inv_id,
        inv.unwrap().kind,
        global_env.get_source(loc).unwrap(),
    );
}

#[allow(dead_code)]
fn debug_print_fun_inv_map(
    global_env: &GlobalEnv,
    fun_inv_map: &BTreeMap<QualifiedId<FunId>, BTreeSet<GlobalId>>,
    map_name: &str,
) {
    debug!("****************\nMAP NAME {}:", map_name);
    for (fun_id, inv_id_set) in fun_inv_map.iter() {
        let fname = global_env.get_function(*fun_id).get_name_string();
        debug!("FUNCTION {}:", fname);
        for inv_id in inv_id_set {
            debug_print_inv_full(global_env, inv_id);
        }
        //        debug_print_inv_set(global_env, inv_id_set, &fname);
    }
}

// global_env.get_function(*fun_id).get_name_string();

/// Print sets and maps computed during verification analysis
#[allow(dead_code)]
fn debug_print_invariant_analysis_data(
    global_env: &GlobalEnv,
    inv_ana_data: &InvariantAnalysisData,
) {
    debug_print_fun_id_set(global_env, &inv_ana_data.target_fun_ids, "target_fun_ids");
    debug_print_fun_id_set(global_env, &inv_ana_data.dep_fun_ids, "dep_fun_ids");
    debug_print_fun_id_set(
        global_env,
        &inv_ana_data.disabled_inv_fun_set,
        "disabled_inv_fun_set",
    );
    debug_print_fun_id_set(global_env, &inv_ana_data.non_inv_fun_set, "non_inv_fun_set");
    debug_print_inv_set(
        global_env,
        &inv_ana_data.target_invariants,
        "target_invariants",
    );

    // "funs_modified_by_inv" map

    debug_print_fun_inv_map(
        global_env,
        &inv_ana_data.invs_modified_by_fun,
        "invs_modified_by_fun",
    );

    debug_print_fun_id_set(
        global_env,
        &inv_ana_data.funs_that_modify_some_inv,
        "funs_that_modify_some_inv",
    );
    debug_print_fun_id_set(
        global_env,
        &inv_ana_data.funs_that_delegate_to_caller,
        "funs_that_delegate_to_caller",
    );
    debug_print_fun_id_set(global_env, &inv_ana_data.friend_fun_ids, "friend_fun_ids");

    debug_print_fun_inv_map(
        global_env,
        &inv_ana_data.disabled_invs_for_fun,
        "disabled_invs_for_fun",
    );
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
        let global_env = fun_env.module_env.env;
        let fun_id = fun_env.get_qualified_id();
        let variant = data.variant.clone();
        // When this is called, the data of this function is removed from targets so it can
        // be mutated, as per pipeline processor design. We put it back temporarily to have
        // a unique model of targets.
        targets.insert_target_data(&fun_id, variant.clone(), data);
        let inv_ana_data = global_env.get_extension::<InvariantAnalysisData>().unwrap();
        let target_fun_ids = &inv_ana_data.target_fun_ids;
        let dep_fun_ids = &inv_ana_data.dep_fun_ids;
        let friend_fun_ids = &inv_ana_data.friend_fun_ids;
        let funs_that_modify_some_inv = &inv_ana_data.funs_that_modify_some_inv;
        // Logic to decide whether to verify this function
        // Never verify if "pragma verify = false;"
        if fun_env.is_pragma_true(VERIFY_PRAGMA, || true) {
            let is_in_target_mod = target_fun_ids.contains(&fun_id);
            let is_in_deps_and_modifies_inv =
                dep_fun_ids.contains(&fun_id) && funs_that_modify_some_inv.contains(&fun_id);
            let is_in_friends = friend_fun_ids.contains(&fun_id);
            let is_normally_verified =
                is_in_target_mod || is_in_deps_and_modifies_inv || is_in_friends;
            let options = ProverOptions::get(global_env);
            let is_verified = match &options.verify_scope {
                VerificationScope::Public => {
                    (is_in_target_mod && fun_env.is_exposed())
                        || is_in_deps_and_modifies_inv
                        || is_in_friends
                }
                VerificationScope::All => is_normally_verified,
                VerificationScope::Only(function_name) => {
                    fun_env.matches_name(function_name) && is_in_target_mod
                }
                VerificationScope::OnlyModule(module_name) => {
                    is_in_target_mod && fun_env.module_env.matches_name(module_name)
                }
                VerificationScope::None => false,
            };
            if is_verified {
                debug!("marking `{}` to be verified", fun_env.get_full_name_str());
                mark_verified(fun_env, variant.clone(), targets);
            }
        }

        targets.remove_target_data(&fun_id, &variant)
    }

    fn name(&self) -> String {
        "verification_analysis_v2".to_string()
    }

    fn initialize(&self, global_env: &GlobalEnv, targets: &mut FunctionTargetsHolder) {
        let options = ProverOptions::get(global_env);

        // If we are verifying only one function or module, check that this indeed exists.
        match &options.verify_scope {
            VerificationScope::Only(name) | VerificationScope::OnlyModule(name) => {
                let for_module = matches!(&options.verify_scope, VerificationScope::OnlyModule(_));
                let mut target_exists = false;
                for module in global_env.get_modules() {
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
                    global_env.error(
                        &global_env.unknown_loc(),
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

        let target_modules = global_env.get_target_modules();
        let target_fun_ids: BTreeSet<QualifiedId<FunId>> = target_modules
            .iter()
            .flat_map(|mod_env| mod_env.get_functions())
            .map(|fun| fun.get_qualified_id())
            .collect();
        let dep_fun_ids = compute_dep_fun_ids(global_env, &target_modules);
        let (disabled_inv_fun_set, non_inv_fun_set) =
            compute_disabled_and_non_inv_fun_sets(global_env);
        let target_invariants = get_target_invariants(global_env, &target_modules);
        let (funs_that_modify_inv, invs_modified_by_fun, funs_that_modify_some_inv) =
            compute_funs_that_modify_inv(
                global_env,
                &target_invariants,
                targets,
                FunctionVariant::Baseline,
            );
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
        let disabled_invs_for_fun =
            compute_disabled_invs_for_fun(global_env, &disabled_inv_fun_set, &invs_modified_by_fun);

        // Check for public or script functions that are in non_inv_fun_set
        for module_env in global_env.get_modules() {
            for fun_env in module_env.get_functions() {
                check_legal_disabled_invariants(
                    &fun_env,
                    &disabled_inv_fun_set,
                    &non_inv_fun_set,
                    &funs_that_modify_some_inv,
                );
            }
        }
        let inv_ana_data = InvariantAnalysisData {
            target_fun_ids,
            dep_fun_ids,
            disabled_inv_fun_set,
            non_inv_fun_set,
            target_invariants,
            funs_that_modify_inv,
            invs_modified_by_fun,
            funs_that_modify_some_inv,
            funs_that_delegate_to_caller,
            friend_fun_ids,
            disabled_invs_for_fun,
        };

        // Note: To print verbose debugging info, use
        debug_print_invariant_analysis_data(global_env, &inv_ana_data);

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
