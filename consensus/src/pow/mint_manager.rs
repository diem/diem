use libra_types::transaction::SignedTransaction;
use libra_types::crypto_proxies::ValidatorSigner;
use std::collections::BTreeMap;
use libra_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use consensus_types::{block::Block, quorum_cert::QuorumCert, vote_data::VoteData};
use cuckoo::consensus::{PowCuckoo, PowService, Proof};
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, RequestBlock, RespondBlock, BlockRetrievalStatus,
    },
    validator_network::{PowContext, ConsensusNetworkSender, Event}
};
use logger::prelude::*;
use std::thread::sleep;
use failure::prelude::*;
use {
    futures::{
        compat::Future01CompatExt,
        future::{self, FutureExt, TryFutureExt},
    },
};
use crate::state_replication::{StateComputer, TxnManager};
use std::sync::Arc;
use std::time::Duration;
use futures::channel::mpsc;
use libra_types::account_address::AccountAddress;
use crate::chained_bft::consensusdb::ConsensusDB;
use atomic_refcell::AtomicRefCell;
use crate::pow::chain_manager::ChainManager;
use crypto::hash::{GENESIS_BLOCK_ID, PRE_GENESIS_BLOCK_ID};
use tokio::runtime::{self, TaskExecutor};
use crate::pow::pow_consensus_provider::EventHandle;
use rand::Rng;
use crypto::hash::CryptoHash;

pub struct MintManager {
    txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
    block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
    network_sender: ConsensusNetworkSender,
    author: AccountAddress,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    block_store: Arc<ConsensusDB>,
    pow_srv: Arc<PowService>,
    genesis_txn: Vec<SignedTransaction>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
}

impl MintManager {

    pub fn new(txn_manager: Arc<dyn TxnManager<Payload=Vec<SignedTransaction>>>,
               state_computer: Arc<dyn StateComputer<Payload=Vec<SignedTransaction>>>,
               block_cache_sender: mpsc::Sender<Block<Vec<SignedTransaction>>>,
               network_sender: ConsensusNetworkSender,
               author: AccountAddress,
               self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
               block_store: Arc<ConsensusDB>,
               pow_srv: Arc<PowService>,
               genesis_txn: Vec<SignedTransaction>,
               chain_manager: Arc<AtomicRefCell<ChainManager>>) -> Self {
        MintManager {txn_manager, state_computer, block_cache_sender, network_sender, author,
            self_sender, block_store, pow_srv, genesis_txn, chain_manager}
    }

    pub fn mint(&self, executor: TaskExecutor) {
        let mint_txn_manager = self.txn_manager.clone();
        let mint_state_computer = self.state_computer.clone();
        let mut block_cache_sender = self.block_cache_sender.clone();
        let mut mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let mut self_sender = self.self_sender.clone();
        let block_db = self.block_store.clone();
        let pow_srv = self.pow_srv.clone();
        let genesis_txn_vec = self.genesis_txn.clone();
        let chain_manager = self.chain_manager.clone();

        let mint_fut = async move {
            let chain_manager_clone = chain_manager.clone();
            loop {
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
                        let (height, parent_block) = chain_manager_clone.borrow().chain_height_and_root().await;
                        chain_manager_clone.borrow().print_block_chain_root(mint_author.clone()).await;
                        if txns.len() > 0 {
                            //create block
                            let parent_block_id = parent_block.id;
                            let grandpa_block_id = parent_block.parent_block_id;
                            //QC with parent block id
                            let quorum_cert = if parent_block_id != *GENESIS_BLOCK_ID {
                                let parent_block = block_db.get_block_by_hash::<Vec<SignedTransaction>>(&parent_block_id).expect("block not find in database err.");
                                parent_block.quorum_cert().clone()
                            } else {
                                QuorumCert::certificate_for_genesis()
                            };

                            let mut tmp = vec![];
                            tmp.push(genesis_txn_vec.clone());
                            let (pre_compute_parent_block_id, commit_txn_vec) = if parent_block_id != *GENESIS_BLOCK_ID {
                                (grandpa_block_id, vec![txns.clone()].to_vec())
                            } else {
                                tmp.push(txns.clone());
                                (*PRE_GENESIS_BLOCK_ID, tmp)
                            };

                            //compute current block state id
                            match mint_state_computer.pre_compute(pre_compute_parent_block_id, commit_txn_vec).await {
                                Ok((compute_state, state_id)) => {
                                    let txn_len = compute_state.version();
                                    let vote_data = VoteData::new(parent_block_id,state_id, quorum_cert.certified_block_round(), parent_block_id, quorum_cert.parent_block_round());
                                    let parent_li = quorum_cert.ledger_info().ledger_info().clone();
                                    let v_s = match parent_li.next_validator_set() {
                                        Some(n_v_s) => {
                                            Some(n_v_s.clone())
                                        }
                                        None => {
                                            None
                                        }
                                    };
                                    let li = LedgerInfo::new(txn_len, compute_state.root_hash(), vote_data.hash(), parent_block_id, parent_li.epoch_num(), parent_li.timestamp_usecs(), v_s);
                                    let signer = ValidatorSigner::genesis();//TODO:change signer
                                    let signature = signer.sign_message(li.hash()).expect("Fail to sign genesis ledger info");
                                    let mut signatures = BTreeMap::new();
                                    signatures.insert(signer.author(), signature);
                                    let new_qc = QuorumCert::new(vote_data, LedgerInfoWithSignatures::new(li.clone(), signatures));

                                    let block = Block::<Vec<SignedTransaction>>::new_internal(
                                        txns,
                                        0,
                                        height + 1,
                                        0,
                                        new_qc,
                                        &ValidatorSigner::from_int(1),
                                    );

                                    let block_pb = Into::<BlockProto>::into(block);

                                    // send block
                                    let msg = ConsensusMsg {
                                        message: Some(ConsensusMsg_oneof::NewBlock(block_pb)),
                                    };
                                    let nonce = generate_nonce();
                                    let proof = pow_srv.solve(li.hash().as_ref(), nonce);
                                    let solve = match proof {
                                        Some(proof) => proof.solve,
                                        None => vec![]
                                    };
                                    let pow_ctx = PowContext {
                                        header_hash: li.hash().to_vec(),
                                        nonce,
                                        solve,
                                    };
                                    EventHandle::broadcast_consensus_msg(&mut mint_network_sender, true, mint_author, &mut self_sender, msg, Some(pow_ctx)).await;
                                }
                                Err(e) => {
                                    println!("{:?}", e);
                                },
                            }
                        }

                        let mut r = rand::thread_rng();
                        r.gen::<i32>();
                        let sleep_time = r.gen_range(10, 20);
                        println!("sleep begin.");
                        sleep(Duration::from_secs(sleep_time));
                        println!("sleep end.");
                    }
                    _ => {}
                }


            }
        };
        executor.spawn(mint_fut);
    }
}

fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen::<u64>();
    rng.gen_range(0, u64::max_value())
}