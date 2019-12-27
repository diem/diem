use crate::chained_bft::consensusdb::ConsensusDB;
use chain_state::{ChainStateMsg, ChainStateRequest, ChainStateResponse};
use consensus_types::block::Block;
use consensus_types::payload_ext::BlockPayloadExt;
use futures::StreamExt;
use libra_logger::prelude::*;
use network::proto::{ChainStateMsg as ChainStateMsgProto, ChainStateMsg_oneof};
use network::validator_network::{ChainStateNetworkEvents, ChainStateNetworkSender, Event};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::sync::Arc;

pub struct ChainStateRequestHandle {
    chain_state_network_sender: ChainStateNetworkSender,
    chain_state_network_events: Option<ChainStateNetworkEvents>,
    block_store: Arc<ConsensusDB>,
}

impl ChainStateRequestHandle {
    pub fn new(
        chain_state_network_sender: ChainStateNetworkSender,
        chain_state_network_events: ChainStateNetworkEvents,
        block_store: Arc<ConsensusDB>,
    ) -> Self {
        Self {
            chain_state_network_sender,
            chain_state_network_events: Some(chain_state_network_events),
            block_store,
        }
    }

    pub async fn start(mut self) {
        let mut chain_state_network_events = self
            .chain_state_network_events
            .take()
            .expect("chain_state_network_events is none.");
        let chain_state_network_sender = self.chain_state_network_sender.clone();
        let block_store = self.block_store.clone();
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
                                        ChainStateMsg_oneof::CsReq(req) => {
                                            let cs_resp = ChainStateRequest::try_from(req).expect("parse err.");
                                            debug!("{:?}", cs_resp);
                                            let block: Option<Block<BlockPayloadExt>> = block_store.latest_block();
                                            let pb = match block {
                                                Some(b) => {
                                                    let cs_resp = ChainStateResponse::new(cs_resp.nonce(),b);
                                                    let msg: ChainStateMsg<BlockPayloadExt> = ChainStateMsg::CsResp(cs_resp);
                                                    msg.try_into().expect("into err.")
                                                },
                                                None => {
                                                    ChainStateMsgProto::default()
                                                }
                                            };
                                            if let Err(err) = chain_state_network_sender
                                                .clone().send_to(peer_id, pb)
                                                .await
                                            {
                                                error!(
                                                    "Error send chain_state resp error: {:?}, msg: {:?}",
                                                    err, msg
                                                );
                                            }
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
                complete => {
                   break;
                }
            }
        }
    }
}
