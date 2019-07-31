extern crate test_generation;
use test_generation::{abstract_state::AbstractState, summaries::instruction_summary};
use vm::file_format::Bytecode;

pub fn run_instruction(instruction: Bytecode, initial_state: AbstractState) -> AbstractState {
    let instruction_summary = instruction_summary(instruction);
    let mut preconditions_satisfied = true;
    for precondition in instruction_summary.preconditions.into_iter() {
        if !precondition(&initial_state) {
            preconditions_satisfied = false;
        }
    }
    assert_eq!(
        preconditions_satisfied, true,
        "preconditions of instruction not satisfied"
    );
    let mut state2 = initial_state.clone();
    for effect in instruction_summary.effects.into_iter() {
        state2 = effect(&state2);
    }
    state2
}
