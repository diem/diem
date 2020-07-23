// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{fuzz_data_to_value, FuzzTargetImpl};
use consensus::round_manager_fuzzing::fuzz_proposal;
use consensus_types::proposal_msg::ProposalMsg;

#[derive(Clone, Debug, Default)]
pub struct ConsensusProposal;

impl FuzzTargetImpl for ConsensusProposal {
    fn name(&self) -> &'static str {
        module_name!()
    }

    fn description(&self) -> &'static str {
        "Consensus proposal messages"
    }

    fn fuzz(&self, data: &[u8]) {
        let proposal: ProposalMsg = fuzz_data_to_value(data);
        fuzz_proposal(proposal);
    }
}
