use crate::control_flow_graph::{BlockId, ControlFlowGraph, VMControlFlowGraph};
use std::collections::HashMap;
use vm::{
    file_format::{Bytecode, CompiledModule},
    views::FunctionDefinitionView,
};

/// Trait for finite-height abstract domains. Infinite height domains would require a more complex
/// trait.
pub trait AbstractDomain: Clone + Sized {
    fn join(&mut self, other: &Self) -> JoinResult;
}

#[derive(Debug)]
pub enum JoinResult {
    Unchanged,
    Changed,
    Error,
}

#[allow(dead_code)]
pub enum BlockPostcondition<State> {
    State(State),
    Bottom,
}

#[allow(dead_code)]
pub type InvariantMap<State> = HashMap<BlockId, BlockPostcondition<State>>;

/// Take a pre-state + instruction and mutate it to produce a post-state
/// Auxiliary data can be stored in self.
pub trait TransferFunctions {
    type State: AbstractDomain;

    fn execute(&mut self, pre: &mut Self::State, instr: &Bytecode, index: usize);
}

pub trait AbstractInterpreter: TransferFunctions {
    fn analyze_function(
        &mut self,
        initial_state: Self::State,
        function_view: &FunctionDefinitionView<CompiledModule>,
        cfg: &VMControlFlowGraph,
    ) -> InvariantMap<Self::State> {
        let mut inv_map: InvariantMap<Self::State> = InvariantMap::new();
        let entry_block_id = 0; // 0 is always the entry block
        let mut work_list = vec![entry_block_id];
        inv_map.insert(entry_block_id, BlockPostcondition::State(initial_state));

        while let Some(block_id) = work_list.pop() {
            let mut state = match inv_map.get_mut(&block_id) {
                Some(BlockPostcondition::State(state)) => state.clone(),
                Some(BlockPostcondition::Bottom) | None => panic!("Should never happen"),
            };
            self.execute_block(block_id, &mut state, &function_view, &cfg);
            let block = cfg
                .block_of_id(block_id)
                .expect("block_id is not the start offset of a block");

            for next_block_id in &block.successors {
                match inv_map.get_mut(next_block_id) {
                    Some(BlockPostcondition::State(old_state)) => {
                        let join_result = old_state.join(&state);
                        match join_result {
                            JoinResult::Unchanged => {
                                continue;
                            }
                            JoinResult::Changed => work_list.push(*next_block_id),
                            JoinResult::Error => {
                                inv_map.insert(*next_block_id, BlockPostcondition::Bottom);
                            }
                        }
                    }
                    None => {
                        inv_map.insert(*next_block_id, BlockPostcondition::State(state.clone()));
                        work_list.push(*next_block_id)
                    }
                    Some(BlockPostcondition::Bottom) => {
                        continue;
                    }
                }
            }
        }

        inv_map
    }

    fn execute_block(
        &mut self,
        block_id: BlockId,
        state: &mut Self::State,
        function_view: &FunctionDefinitionView<CompiledModule>,
        cfg: &VMControlFlowGraph,
    ) {
        let block = cfg
            .block_of_id(block_id)
            .expect("block_id is not the start offset of a block");

        for offset in block.entry..=block.exit {
            let instr = &function_view.code().code[offset as usize];
            self.execute(state, instr, offset as usize);
        }
    }
}
