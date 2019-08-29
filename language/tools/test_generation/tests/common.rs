extern crate test_generation;
use test_generation::{abstract_state::AbstractState, summaries::instruction_summary};
use vm::file_format::Bytecode;

pub fn run_instruction(instruction: Bytecode, initial_state: AbstractState) -> AbstractState {
    let summary = instruction_summary(instruction);
    let unsatisfied_preconditions = summary
        .preconditions
        .iter()
        .any(|precondition| !precondition(&initial_state));
    assert_eq!(
        unsatisfied_preconditions, false,
        "preconditions of instruction not satisfied"
    );
    summary.effects.iter().fold(initial_state, |acc, effect| {
        effect(&acc).unwrap_or_else(|err| panic!("Error applying instruction effect: {}", err))
    })
}
