#![recursion_limit="256"]
mod chain_state;
pub use chain_state::{ChainStateMsg, ChainStateRequest, ChainStateResponse};

use tokio::runtime::Handle;
use futures::{future, StreamExt};
use tokio::runtime::{Builder, Runtime};
use tokio::time::{interval};
use network::{
    proto::{
        ChainStateMsg_oneof,
    },
    validator_network::{ChainStateNetworkSender, ChainStateNetworkEvents, Event},
};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use libra_prost_ext::MessageExt;
use std::convert::TryInto;
use libra_logger::prelude::*;
use consensus_types::payload_ext::BlockPayloadExt;
use libra_logger::prelude::*;
use std::convert::TryFrom;

pub struct ChainStateRuntime {
    _rt: Runtime,
}

impl ChainStateRuntime {
    pub fn bootstrap(
        chain_state_network_sender: ChainStateNetworkSender,
        chain_state_network_events: ChainStateNetworkEvents,
    ) -> Self {
        let runtime = Builder::new()
            .thread_name("chain-state-")
            .threaded_scheduler()
            .enable_all()
            .build()
            .expect("[state synchronizer] failed to create runtime");
        let executor = runtime.handle();

        let cs_handle = ChainStateResponseHandle::new(chain_state_network_events);
        executor.clone().spawn(cs_handle.start());

        let task_executor = executor.clone();
        let f = async move {
            interval(Duration::from_secs(30))
                .for_each(move |_| {
                    Self::chain_state_network_task(task_executor.clone(), chain_state_network_sender.clone());
                    future::ready(())
                })
                .await;
        };
        executor.spawn(f);

        Self {
            _rt:runtime,
        }
    }

    fn chain_state_network_task(handle:Handle, chain_state_network_sender: ChainStateNetworkSender) {
        info!("chain_state begin.");
        let f = async move {
            let current = SystemTime::now();
            let nonce = current.duration_since(UNIX_EPOCH)
                .expect("Time went backwards").as_micros() as u64;
            let req = ChainStateRequest::new(nonce);
            let msg: ChainStateMsg<BlockPayloadExt> = ChainStateMsg::CsReq(req);
            let proto:network::proto::ChainStateMsg = msg.clone().try_into().expect("into err.");
            let msg_raw = proto.to_bytes().unwrap();
            if let Err(err) = chain_state_network_sender
                .clone().broadcast_bytes(msg_raw, vec![])
                .await
            {
                error!(
                    "Error broadcasting chain_state  error: {:?}, msg: {:?}",
                    err, msg
                );
            }
        };
        handle.spawn(f);
    }
}

pub struct ChainStateResponseHandle {
    chain_state_network_events: Option<ChainStateNetworkEvents>,
}

impl ChainStateResponseHandle {
    pub fn new(chain_state_network_events: ChainStateNetworkEvents) -> Self {
        Self {
            chain_state_network_events: Some(chain_state_network_events),
        }
    }

    pub async fn start(mut self) {
        let mut chain_state_network_events = self.chain_state_network_events
            .take().expect("chain_state_network_events is none.");
        loop {
            ::futures::select! {
                network_event = chain_state_network_events.select_next_some() => {
                    match network_event {
                        Ok(msg) => {
                            match msg {
                                Event::Message((peer_id, msg)) => {
                                    let msg = match msg.message {
                                        Some(msg) => msg,
                                        None => {
                                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                            continue;
                                        }
                                    };

                                    match msg.clone() {
                                        ChainStateMsg_oneof::CsResp(resp) => {
                                            let cs_resp: ChainStateResponse<BlockPayloadExt>  = ChainStateResponse::try_from(resp).expect("parse err.");
                                            debug!("{:?}", cs_resp);
                                        }
                                        _ => {
                                            warn!("Unexpected msg from {}: {:?}", peer_id, msg);
                                            continue;
                                        }
                                    }
                                }
                                Event::RpcRequest((peer_id, _msg, _callback)) => {
                                    info!("RpcRequest from {:?} ", peer_id);
                                }
                                Event::NewPeer(peer_id) => {
                                    info!("Peer {:?} connected", peer_id);
                                }
                                Event::LostPeer(peer_id) => {
                                    info!("Peer {:?} disconnected", peer_id);
                                }
                            };
                        },
                        Err(e) => {
                            warn!("{:?}", e);
                        }
                    }
                },
            }
        }
    }
}