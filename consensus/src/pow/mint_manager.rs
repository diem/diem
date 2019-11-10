use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::chain_manager::ChainManager;
use crate::pow::event_processor::EventProcessor;
use crate::pow::payload_ext::BlockPayloadExt;
use crate::state_replication::{StateComputer, TxnManager};
use atomic_refcell::AtomicRefCell;
use consensus_types::block_info::BlockInfo;
use consensus_types::{block::Block, quorum_cert::QuorumCert, vote_data::VoteData};
use cuckoo::consensus::PowService;
use libra_crypto::hash::CryptoHash;
use libra_crypto::hash::GENESIS_BLOCK_ID;
use libra_crypto::HashValue;
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::crypto_proxies::ValidatorSigner;
use libra_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use libra_types::transaction::SignedTransaction;
use network::{
    proto::{
        Block as BlockProto, ConsensusMsg,
        ConsensusMsg_oneof::{self},
    },
    validator_network::{ConsensusNetworkSender, Event},
};
use rand::Rng;
use std::collections::BTreeMap;
use std::convert::TryInto;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::TaskExecutor;

pub struct MintManager {
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
    network_sender: ConsensusNetworkSender,
    author: AccountAddress,
    self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
    block_store: Arc<ConsensusDB>,
    pow_srv: Arc<dyn PowService>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
}

impl MintManager {
    pub fn new(
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
        network_sender: ConsensusNetworkSender,
        author: AccountAddress,
        self_sender: channel::Sender<failure::Result<Event<ConsensusMsg>>>,
        block_store: Arc<ConsensusDB>,
        pow_srv: Arc<dyn PowService>,
        chain_manager: Arc<AtomicRefCell<ChainManager>>,
    ) -> Self {
        MintManager {
            txn_manager,
            state_computer,
            network_sender,
            author,
            self_sender,
            block_store,
            pow_srv,
            chain_manager,
        }
    }

    pub fn mint(&self, executor: TaskExecutor) {
        let mint_txn_manager = self.txn_manager.clone();
        let mint_state_computer = self.state_computer.clone();
        let mut mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let mut self_sender = self.self_sender.clone();
        let block_db = self.block_store.clone();
        let pow_srv = self.pow_srv.clone();
        let chain_manager = self.chain_manager.clone();

        let mint_fut = async move {
            let chain_manager_clone = chain_manager.clone();
            loop {
                match mint_txn_manager.pull_txns(1, vec![]).await {
                    Ok(txns) => {
                        let (height, parent_block) =
                            chain_manager_clone.borrow().chain_height_and_root().await;
                        if txns.len() > 0 {
                            //create block
                            let parent_block_id = parent_block.id;
                            let grandpa_block_id = parent_block.parent_block_id;
                            //QC with parent block id
                            let quorum_cert = if parent_block_id != *GENESIS_BLOCK_ID {
                                let parent_block = block_db
                                    .get_block_by_hash::<BlockPayloadExt>(&parent_block_id)
                                    .expect("block not find in database err.");
                                parent_block.quorum_cert().clone()
                            } else {
                                QuorumCert::certificate_for_genesis()
                            };

                            //                            let mut tmp = vec![];
                            //                            tmp.append(&mut genesis_txn_vec);
                            //                            let (pre_compute_parent_block_id, commit_txn_vec) =
                            //                                if parent_block_id != *GENESIS_BLOCK_ID {
                            //                                    (grandpa_block_id, txns.clone())
                            //                                } else {
                            //                                    tmp.append(&mut txns);
                            //                                    (*PRE_GENESIS_BLOCK_ID, tmp)
                            //                                };

                            //compute current block state id
                            let tmp_id = HashValue::random();
                            match mint_state_computer
                                .compute_by_hash(grandpa_block_id, parent_block_id, tmp_id, &txns)
                                .await
                            {
                                Ok(processed_vm_output) => {
                                    let executed_trees = processed_vm_output.executed_trees();
                                    let state_id = executed_trees.state_root();
                                    let txn_accumulator_hash =
                                        executed_trees.txn_accumulator().root_hash();
                                    let txn_len = executed_trees.version().expect("version err.");

                                    let parent_li = quorum_cert.ledger_info().ledger_info().clone();
                                    let parent_vd = quorum_cert.vote_data();
                                    let epoch = parent_vd.parent().epoch();
                                    let v_s = match parent_li.next_validator_set() {
                                        Some(n_v_s) => Some(n_v_s.clone()),
                                        None => None,
                                    };

                                    // vote data
                                    let parent_block_info = parent_vd.proposed().clone();
                                    let block_info = BlockInfo::new(
                                        epoch,
                                        height + 1,
                                        parent_block_id,
                                        state_id,
                                        txn_len,
                                        parent_li.timestamp_usecs(),
                                        v_s.clone(),
                                    );
                                    let vote_data = VoteData::new(block_info, parent_block_info);
                                    // ledger info
                                    let li = LedgerInfo::new(
                                        txn_len,
                                        txn_accumulator_hash,
                                        vote_data.hash(),
                                        parent_block_id,
                                        epoch,
                                        parent_li.timestamp_usecs(),
                                        v_s,
                                    );
                                    let signer = ValidatorSigner::genesis(); //TODO:change signer
                                    let signature = signer
                                        .sign_message(li.hash())
                                        .expect("Fail to sign genesis ledger info");
                                    let mut signatures = BTreeMap::new();
                                    signatures.insert(signer.author(), signature);
                                    let new_qc = QuorumCert::new(
                                        vote_data,
                                        LedgerInfoWithSignatures::new(li.clone(), signatures),
                                    );

                                    //mint
                                    let nonce = generate_nonce();
                                    let proof = pow_srv.solve(li.hash().as_ref(), nonce);
                                    let solve = match proof {
                                        Some(proof) => proof.solve,
                                        None => vec![10],
                                    };
                                    let mint_data = BlockPayloadExt { txns, nonce, solve };

                                    //block data
                                    let block = Block::<BlockPayloadExt>::new_proposal(
                                        mint_data,
                                        height + 1,
                                        parent_li.timestamp_usecs(),
                                        new_qc,
                                        &ValidatorSigner::from_int(1),
                                    );

                                    info!(
                                        "Minter : {:?} find a new block : {:?}",
                                        mint_author,
                                        block.id()
                                    );
                                    let block_pb = TryInto::<BlockProto>::try_into(block)
                                        .expect("parse block err.");

                                    // send block
                                    let msg = ConsensusMsg {
                                        message: Some(ConsensusMsg_oneof::NewBlock(block_pb)),
                                    };

                                    EventProcessor::broadcast_consensus_msg(
                                        &mut mint_network_sender,
                                        true,
                                        mint_author,
                                        &mut self_sender,
                                        msg,
                                    )
                                    .await;
                                }
                                Err(e) => {
                                    error!("{:?}", e);
                                }
                            }
                        }

//                        let mut r = rand::thread_rng();
//                        r.gen::<i32>();
//                        let sleep_time = r.gen_range(10, 20);
//                        debug!("sleep begin.");
//                        sleep(Duration::from_secs(sleep_time));
//                        debug!("sleep end.");
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
