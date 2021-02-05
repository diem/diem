// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// Transformation which injects global invariants into the bytecode.

#[allow(unused_imports)]
use log::{debug, info, log, warn};

use crate::{
    function_data_builder::FunctionDataBuilder,
    function_target::FunctionData,
    function_target_pipeline::{FunctionTargetProcessor, FunctionTargetsHolder, FunctionVariant},
    options::{ProverOptions, PROVER_DEFAULT_OPTIONS},
    stackless_bytecode::{BorrowNode, Bytecode, Operation, PropKind},
    usage_analysis,
};

use crate::spec_translator::SpecTranslator;

use move_model::{
    ast::{ConditionKind, GlobalInvariant},
    model::{ConditionTag, FunctionEnv, QualifiedId, StructId},
};
use std::collections::BTreeSet;

const GLOBAL_INVARIANT_FAILS_MESSAGE: &str = "global memory invariant does not hold";

pub struct GlobalInvariantInstrumentationProcessor {}

impl GlobalInvariantInstrumentationProcessor {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

impl FunctionTargetProcessor for GlobalInvariantInstrumentationProcessor {
    fn process(
        &self,
        _targets: &mut FunctionTargetsHolder,
        fun_env: &FunctionEnv<'_>,
        data: FunctionData,
    ) -> FunctionData {
        if fun_env.is_native() || fun_env.is_intrinsic() {
            // Nothing to do.
            return data;
        }
        if data.variant != FunctionVariant::Verification && !fun_env.has_friend() {
            // Only need to instrument if this is a verification variant, or if the
            // function has a friend and is verified in the friends context.
            return data;
        }

        let options = fun_env
            .module_env
            .env
            .get_extension::<ProverOptions>()
            .unwrap_or_else(|| &*PROVER_DEFAULT_OPTIONS);

        Instrumenter::run(options, fun_env, data)
    }

    fn name(&self) -> String {
        "global_invariant_instrumenter".to_string()
    }
}

struct Instrumenter<'a> {
    _options: &'a ProverOptions,
    builder: FunctionDataBuilder<'a>,
}

impl<'a> Instrumenter<'a> {
    fn run(
        options: &'a ProverOptions,
        fun_env: &FunctionEnv<'a>,
        data: FunctionData,
    ) -> FunctionData {
        let builder = FunctionDataBuilder::new(fun_env, data);
        let mut instrumenter = Instrumenter {
            _options: options,
            builder,
        };
        instrumenter.instrument();
        instrumenter.builder.data
    }

    fn instrument(&mut self) {
        // Extract and clear current code
        let old_code = std::mem::take(&mut self.builder.data.code);

        // Emit entrypoint assumptions if this is a verification entry.
        if self.builder.data.variant == FunctionVariant::Verification {
            self.instrument_entrypoint();
        }

        // Generate new instrumented code.
        for bc in old_code {
            self.instrument_bytecode(bc);
        }
    }

    fn instrument_entrypoint(&mut self) {
        // Emit an assume of all invariants over memory touched by this function, and which
        // stem from modules in the dependency graph, OR which are used in invariants which
        // need to be asserted.
        //
        // In general, assumptions are only needed to assume for memory which this function
        // can know about via its dependency graph, but not for some arbitrary other memory.
        // However, there is an exception for those invariants which need to be asserted because
        // memory is updated: those can stem from modules not in the dependency graph, and they
        // need to be assumed at entry to make inductive verification possible.
        let env = self.builder.global_env();
        let mut invariants = BTreeSet::new();
        let mut invariants_for_modified_memory = BTreeSet::new();
        for mem in usage_analysis::get_used_memory(&self.builder.get_target()) {
            invariants.extend(env.get_global_invariants_for_memory(*mem));
        }
        for mem in usage_analysis::get_modified_memory(&self.builder.get_target()) {
            invariants_for_modified_memory.extend(env.get_global_invariants_for_memory(*mem));
        }

        let module_env = &self.builder.fun_env.module_env;
        let mut translated = SpecTranslator::translate_invariants(
            &mut self.builder,
            invariants.iter().filter_map(|id| {
                env.get_global_invariant(*id).filter(|inv| {
                    inv.kind == ConditionKind::Invariant
                        // Invariant needs to be either over modified memory, or declared in
                        // the transitive dependencies.
                        && (invariants_for_modified_memory.contains(id)
                            || module_env.is_transitive_dependency(inv.declaring_module))
                })
            }),
        );
        for (loc, cond) in std::mem::take(&mut translated.invariants) {
            self.builder.set_next_debug_comment(format!(
                "global invariant {}",
                loc.display(self.builder.global_env())
            ));
            self.builder
                .emit_with(|id| Bytecode::Prop(id, PropKind::Assume, cond));
        }
    }

    fn instrument_bytecode(&mut self, bc: Bytecode) {
        use BorrowNode::*;
        use Bytecode::*;
        use Operation::*;
        match &bc {
            Call(_, _, WriteBack(GlobalRoot(mem)), ..) => {
                self.emit_invariants_for_update(*mem, move |builder| {
                    builder.emit(bc);
                })
            }
            Call(_, _, MoveTo(mid, sid, _), ..) | Call(_, _, MoveFrom(mid, sid, _), ..) => self
                .emit_invariants_for_update(mid.qualified(*sid), move |builder| {
                    builder.emit(bc);
                }),
            _ => self.builder.emit(bc),
        }
    }

    fn emit_invariants_for_update<F>(&mut self, mem: QualifiedId<StructId>, emit_update: F)
    where
        F: FnOnce(&mut FunctionDataBuilder<'_>),
    {
        // Translate the invariants, computing any state to be saved as well. State saves are
        // necessary for update invariants which contain the `old(..)` expressions.
        let invariants = self.get_verified_invariants_for_mem(mem);
        let mut translated =
            SpecTranslator::translate_invariants(&mut self.builder, invariants.iter().cloned());

        // Emit all necessary state saves
        self.builder
            .set_next_debug_comment("state save for global update invariants".to_string());
        for (mem, label) in std::mem::take(&mut translated.saved_memory) {
            self.builder
                .emit_with(|id| Bytecode::SaveMem(id, label, mem));
        }
        for (var, label) in std::mem::take(&mut translated.saved_spec_vars) {
            self.builder
                .emit_with(|id| Bytecode::SaveSpecVar(id, label, var));
        }
        self.builder.clear_next_debug_comment();

        // Emit the code which performs the update on `mem`.
        emit_update(&mut self.builder);

        // Emit assertions of translated invariants.
        for (loc, cond) in std::mem::take(&mut translated.invariants) {
            self.builder.set_next_debug_comment(format!(
                "global invariant {}",
                loc.display(self.builder.global_env())
            ));
            self.builder.set_loc_and_vc_info(
                loc,
                ConditionTag::Requires,
                GLOBAL_INVARIANT_FAILS_MESSAGE,
            );
            self.builder
                .emit_with(|id| Bytecode::Prop(id, PropKind::Assert, cond));
        }
    }

    /// Returns the invariants which need to be verified if the given memory is updated.
    /// This filters out those invariants which stem from modules which are not verification
    /// target.
    fn get_verified_invariants_for_mem(
        &self,
        mem: QualifiedId<StructId>,
    ) -> Vec<&'a GlobalInvariant> {
        let env = self.builder.global_env();
        env.get_global_invariants_for_memory(mem)
            .iter()
            .filter_map(|id| {
                env.get_global_invariant(*id)
                    .filter(|inv| env.get_module(inv.declaring_module).is_target())
            })
            .collect()
    }
}
