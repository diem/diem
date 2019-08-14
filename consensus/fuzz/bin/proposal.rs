#![feature(async_await)]

extern crate consensus;

use consensus::chained_bft::{
    consensus_types::proposal_msg::ProposalMsg,
    event_processor::ProcessProposalResult,
    event_processor_test::NodeSetup,
    test_utils::{consensus_runtime, MockStorage, TestPayload},
};
use futures::executor::block_on;
use lazy_static::lazy_static;
use network::proto::ConsensusMsg;
use nextgen_crypto::ed25519::*;
use proto_conv::FromProto;
use protobuf::Message as proto;
use tokio::runtime;
use types::{validator_signer::ValidatorSigner, validator_verifier::ValidatorVerifier};

lazy_static! {
    static ref STATIC_RUNTIME: runtime::Runtime = consensus_runtime();
    static ref FUZZING_SIGNER: ValidatorSigner<Ed25519PrivateKey> = ValidatorSigner::random(None);
}

fn main() {
    let argument = std::env::args().last().unwrap();
    println!("reading proposal from file: {}", argument);
    let proposal = std::fs::read(argument).unwrap();
    fuzz(&proposal);
}

fn fuzz(data: &[u8]) {
    // create node
    let signer = FUZZING_SIGNER.clone();
    let mut peers = vec![];
    peers.push(signer.author());
    let proposer_author = peers[0];
    let peers = std::sync::Arc::new(peers);
    let (storage, initial_data) = MockStorage::<TestPayload>::start_for_fuzzing();
    let mut node = NodeSetup::new(
        None,
        STATIC_RUNTIME.executor(),
        signer,
        proposer_author,
        peers,
        storage,
        initial_data,
        false,
    );

    // proto parse
    let mut msg: ConsensusMsg = match protobuf::parse_from_bytes(data) {
        Ok(xx) => xx,
        Err(_) => {
            return;
        }
    };

    // extract proposal
    let proposal = match msg.has_proposal() {
        true => match ProposalMsg::<TestPayload>::from_proto(msg.take_proposal()) {
            Ok(xx) => xx,
            Err(_) => return,
        },
        false => return,
    };

    // process proposal (network.rs)
    let validator = ValidatorVerifier::new_empty();
    match proposal.verify(&validator) {
        Err(_) => {
            return;
        }
        _ => (),
    }

    block_on(async move {
        // process proposal (event_process)
        match node
            .event_processor
            .process_proposal(proposal.clone())
            .await
        {
            ProcessProposalResult::Done(Some(_)) => (),
            _ => return,
        };

        // process winning proposal (event_process)
        node.event_processor
            .process_winning_proposal(proposal)
            .await;
    });
}
