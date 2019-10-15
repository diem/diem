use crate::consensus_provider::ConsensusProvider;
use failure::prelude::*;

pub struct PowConsensusProvider {}

impl ConsensusProvider for PowConsensusProvider {
    fn start(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn stop(&mut self) {
        unimplemented!()
    }
}