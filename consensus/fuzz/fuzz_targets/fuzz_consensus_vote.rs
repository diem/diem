#![no_main]
#![feature(async_await)]
#[macro_use]
extern crate libfuzzer_sys;
extern crate consensus;

use consensus::chained_bft::{
    event_processor_test::NodeSetup,
    safety::vote_msg::VoteMsg,
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

fuzz_target!(|data: &[u8]| {
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
            // println!("{:?}", x);
            return;
        }
    };

    // extract vote
    let vote = match msg.has_vote() {
        true => match VoteMsg::from_proto(msg.take_vote()) {
            Ok(xx) => xx,
            Err(_) => return,
        },
        false => return,
    };

    // process vote (network.rs)
    let validator = ValidatorVerifier::new_empty();
    match vote.verify(&validator) {
        Err(_) => {
            // println!("{:?}", x);
            return;
        }
        _ => (),
    }

    block_on(async move {
        // process vote (event_process)
        node.event_processor.process_vote(vote.clone(), 0).await;
    });
});
