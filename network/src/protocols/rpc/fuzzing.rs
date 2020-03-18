// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    common::NegotiatedSubstream,
    peer::PeerNotification,
    protocols::rpc::{self, RpcNotification},
    ProtocolId,
};
use futures::{
    future::{self, FutureExt},
    io::{AsyncReadExt, AsyncWriteExt},
    stream::StreamExt,
};
use libra_proptest_helpers::ValueGenerator;
use libra_types::PeerId;
use memsocket::MemorySocket;
use proptest::{arbitrary::any, collection::vec, prop_oneof, strategy::Strategy};
use std::{io, time::Duration};
use tokio::runtime;
use tokio_util::codec::{Encoder, LengthDelimitedCodec};

// Length of unsigned varint prefix in bytes for a u128-sized length
const MAX_UVI_PREFIX_BYTES: usize = 19;

// Corpus size classes
const MAX_SMALL_MSG_BYTES: usize = 32;
const MAX_MEDIUM_MSG_BYTES: usize = 280;

const MOCK_PEER_ID: PeerId = PeerId::DEFAULT;
const INBOUND_RPC_TIMEOUT: Duration = Duration::from_secs(30);
const TEST_PROTOCOL: ProtocolId = ProtocolId::ConsensusRpc;

#[test]
fn test_fuzzer() {
    let mut gen = ValueGenerator::new();
    let data = generate_corpus(&mut gen);
    fuzzer(&data);
}

// Generate some random, well-formed, unsigned-varint length-prefixed byte arrays
// for our fuzzer corpus to act as serialized inbound rpc calls.
pub fn generate_corpus(gen: &mut ValueGenerator) -> Vec<u8> {
    let small_data_strat = vec(any::<u8>(), 0..MAX_SMALL_MSG_BYTES);
    let medium_data_strat = vec(any::<u8>(), 0..MAX_MEDIUM_MSG_BYTES);

    // bias corpus generation to prefer small message sizes
    let data_strat = prop_oneof![small_data_strat, medium_data_strat];

    let length_prefixed_data_strat = data_strat.prop_map(|data| {
        let max_len = data.len() + MAX_UVI_PREFIX_BYTES;
        let mut buf = bytes::BytesMut::with_capacity(max_len);
        let mut codec = LengthDelimitedCodec::new();
        codec
            .encode(bytes::Bytes::from(data), &mut buf)
            .expect("Failed to create uvi-prefixed data for corpus");
        buf.freeze().to_vec()
    });

    gen.generate(length_prefixed_data_strat)
}

// Fuzz the inbound rpc protocol.
pub fn fuzzer(data: &[u8]) {
    let (notification_tx, mut notification_rx) = channel::new_test(8);
    let (mut dialer_substream, listener_substream) = MemorySocket::new_pair();
    let listener_substream = NegotiatedSubstream {
        protocol: TEST_PROTOCOL,
        substream: listener_substream,
    };
    let peer_notif = PeerNotification::NewSubstream(MOCK_PEER_ID, listener_substream);

    // run the rpc inbound protocol using the in-memory substream
    let f_handle_inbound =
        rpc::handle_inbound_substream(notification_tx, peer_notif, INBOUND_RPC_TIMEOUT)
            .map(|_| io::Result::Ok(()));

    // mock the notification channel to echo the fuzzer data back to the dialer
    // as an rpc response
    let f_respond_inbound = async move {
        if let Some(notif) = notification_rx.next().await {
            match notif {
                RpcNotification::RecvRpc(req) => {
                    let protocol = req.protocol;
                    let data = req.data;
                    let res_tx = req.res_tx;
                    assert_eq!(protocol, TEST_PROTOCOL);
                    let _ = res_tx.send(Ok(data));
                }
            }
        }

        io::Result::Ok(())
    };

    // send the fuzzer data over the in-memory substream to the listener
    let f_outbound = async move {
        // write the fuzzer data into the in-memory substream
        dialer_substream.write_all(data).await?;
        dialer_substream.flush().await?;
        // half-close the write-side
        dialer_substream.close().await?;

        // drain whatever bytes the listener sends us
        let mut buf = Vec::new();
        let _ = dialer_substream.read_to_end(&mut buf).await?;

        // when testing, we only run the fuzzer with well-formed inputs, so we
        // should successfully reach this point and read the same data back
        if cfg!(test) {
            assert_eq!(data, &*buf);
        }

        io::Result::Ok(())
    };

    let f = future::try_join3(f_handle_inbound, f_respond_inbound, f_outbound);
    // we need to use tokio runtime since Rpc uses tokio timers
    let res = runtime::Runtime::new().unwrap().block_on(f);

    // there should be no errors when testing with well-formed inputs
    if cfg!(test) {
        res.expect("Fuzzing should succeed when run on the corpus");
    }
}
