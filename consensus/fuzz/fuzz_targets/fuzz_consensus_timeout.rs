#![no_main]
#![feature(async_await)]
#[macro_use]
extern crate libfuzzer_sys;
extern crate consensus;

use consensus::chained_bft::{
    consensus_types::timeout_msg::TimeoutMsg,
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

    // extract timeout
    let timeout_msg = match msg.has_timeout_msg() {
        true => match TimeoutMsg::from_proto(msg.take_timeout_msg()) {
            Ok(xx) => xx,
            Err(_) => return,
        },
        false => return,
    };

    // process timeout (network.rs)
    let validator = ValidatorVerifier::new_empty();
    match timeout_msg.verify(&validator) {
        Err(_) => {
            // println!("{:?}", x);
            return;
        }
        _ => (),
    }

    block_on(async move {
        // process timeout message (event_process)
        node.event_processor
            .process_timeout_msg(timeout_msg.clone(), 0)
            .await;
    });
});
