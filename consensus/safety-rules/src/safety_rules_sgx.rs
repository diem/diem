/* lwg: safetyrule that leverages sgx */

/* from t_safety_rules */
use crate::{ConsensusState, Error, safety_rules_sgx_runner, t_safety_rules::TSafetyRules};
use consensus_types::{
        block::Block, block_data::BlockData, timeout::Timeout, vote::Vote,
            vote_proposal::MaybeSignedVoteProposal,
};
use libra_crypto::ed25519::Ed25519Signature;
use libra_types::epoch_change::EpochChangeProof;
use std::io::{Write};
use std::net::{TcpStream, Shutdown};

pub struct SafetyRulesSGX {
    stream: TcpStream,
}

impl SafetyRulesSGX {

    fn send(&self, buf: &[u8]) {
        let mut stream = TcpStream::connect(safety_rules_sgx_runner::LSR_SGX_ADDRESS).unwrap();
        stream.write(buf).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
    }

    pub fn new() -> Self {
        safety_rules_sgx_runner::start_lsr_enclave();
        let mut stream = TcpStream::connect(safety_rules_sgx_runner::LSR_SGX_ADDRESS).unwrap();
        stream.write("hello...".as_bytes()).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        Self { stream }
    }
}

impl TSafetyRules for SafetyRulesSGX {

    fn initialize(&mut self, proof: &EpochChangeProof) -> Result<(), Error> {
        let mut msg: Vec<u8> = "req:init\n".as_bytes().iter().cloned().collect();
        msg.extend(lcs::to_bytes(proof).unwrap());
        self.send(msg.as_ref());
        Ok(())
    }

    fn consensus_state(&mut self) -> Result<ConsensusState, Error> {
        let msg: Vec<u8> = "req:consensus_state\n".as_bytes().iter().cloned().collect();
        self.send(msg.as_ref());
        Err(Error::NotInitialized("Unimplemented".into()))
    }

    fn construct_and_sign_vote(&mut self, maybe_signed_vote_proposal: &MaybeSignedVoteProposal) -> Result<Vote, Error> {
        let mut msg: Vec<u8> = "req:construct_and_sign_vote\n".as_bytes().iter().cloned().collect();
        msg.extend(lcs::to_bytes(maybe_signed_vote_proposal).unwrap());
        self.send(msg.as_ref());
        Err(Error::NotInitialized("Unimplemented".into()))
    }

    fn sign_proposal(&mut self, block_data: BlockData) -> Result<Block, Error> {
        let mut msg: Vec<u8> = "req:sign_proposal\n".as_bytes().iter().cloned().collect();
        msg.extend(lcs::to_bytes(&block_data).unwrap());
        self.send(msg.as_ref());
        Err(Error::NotInitialized("Unimplemented".into()))
    }

    fn sign_timeout(&mut self, timeout: &Timeout) -> Result<Ed25519Signature, Error> {
        let mut msg: Vec<u8> = "req:sign_timeout\n".as_bytes().iter().cloned().collect();
        msg.extend(lcs::to_bytes(timeout).unwrap());
        self.send(msg.as_ref());
        Err(Error::NotInitialized("Unimplemented".into()))
    }
}


