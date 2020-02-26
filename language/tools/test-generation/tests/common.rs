// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

extern crate test_generation;
use test_generation::{
    abstract_state::AbstractState,
    config::ALLOW_MEMORY_UNSAFE,
    summaries::{instruction_summary, Effects},
};
use vm::file_format::{Bytecode, FunctionInstantiation, StructDefInstantiation};

pub fn run_instruction(
    instruction: Bytecode,
    mut initial_state: AbstractState,
) -> (AbstractState, Bytecode) {
    let summary = instruction_summary(instruction.clone(), false);
    // The following is temporary code. If ALLOW_MEMORY_UNSAFE is false,
    // we have to ignore that precondition so that we can test the instructions.
    // When memory safety is implemented, only the false branch should be used.
    let unsatisfied_preconditions = if !ALLOW_MEMORY_UNSAFE {
        match instruction {
            Bytecode::Pop
            //| Bytecode::MoveLoc(_)
            | Bytecode::CopyLoc(_)
            | Bytecode::MutBorrowLoc(_)
            | Bytecode::ImmBorrowLoc(_)
            | Bytecode::StLoc(_)
            | Bytecode::ReadRef
            | Bytecode::WriteRef
            | Bytecode::FreezeRef
            | Bytecode::MutBorrowField(_)
            | Bytecode::MutBorrowFieldGeneric(_)
            | Bytecode::ImmBorrowFieldGeneric(_)
            | Bytecode::ImmBorrowField(_)
            | Bytecode::MutBorrowGlobal(_)
            | Bytecode::MutBorrowGlobalGeneric(_)
            | Bytecode::ImmBorrowGlobal(_)
            | Bytecode::ImmBorrowGlobalGeneric(_)
            | Bytecode::MoveToSender(_)
            | Bytecode::MoveToSenderGeneric(_) => {
                let len = summary.preconditions.len();
                summary.preconditions[..(len - 1)]
                    .iter()
                    .any(|precondition| !precondition(&initial_state))
            }
            Bytecode::Eq | Bytecode::Neq => {
                let len = summary.preconditions.len();
                summary.preconditions[..(len - 2)]
                    .iter()
                    .any(|precondition| !precondition(&initial_state))
            }
            _ => summary
                .preconditions
                .iter()
                .any(|precondition| !precondition(&initial_state)),
        }
    } else {
        summary
            .preconditions
            .iter()
            .any(|precondition| !precondition(&initial_state))
    };
    assert_eq!(
        unsatisfied_preconditions, false,
        "preconditions of instruction not satisfied"
    );
    match summary.effects {
        Effects::TyParams(instantiation, effect, instantiation_application) => {
            let (struct_idx, instantiation) = instantiation(&initial_state);
            let index = initial_state.module.add_instantiation(instantiation);
            let struct_inst = StructDefInstantiation {
                def: struct_idx,
                type_parameters: index,
            };
            let str_inst_idx = initial_state.module.add_struct_instantiation(struct_inst);
            let effects = effect(str_inst_idx);
            let instruction = instantiation_application(str_inst_idx);
            (
                effects.iter().fold(initial_state, |acc, effect| {
                    effect(&acc)
                        .unwrap_or_else(|err| panic!("Error applying instruction effect: {}", err))
                }),
                instruction,
            )
        }
        Effects::TyParamsCall(instantiation, effect, instantiation_application) => {
            let (fh_idx, instantiation) = instantiation(&initial_state);
            let index = initial_state.module.add_instantiation(instantiation);
            let func_inst = FunctionInstantiation {
                handle: fh_idx,
                type_parameters: index,
            };
            let func_inst_idx = initial_state.module.add_function_instantiation(func_inst);
            let effects = effect(func_inst_idx);
            let instruction = instantiation_application(func_inst_idx);
            (
                effects.iter().fold(initial_state, |acc, effect| {
                    effect(&acc)
                        .unwrap_or_else(|err| panic!("Error applying instruction effect: {}", err))
                }),
                instruction,
            )
        }
        Effects::NoTyParams(effects) => (
            effects.iter().fold(initial_state, |acc, effect| {
                effect(&acc)
                    .unwrap_or_else(|err| panic!("Error applying instruction effect: {}", err))
            }),
            instruction,
        ),
    }
}
