use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::chain_manager::ChainManager;
use crate::pow::event_processor::EventProcessor;
use crate::pow::payload_ext::{genesis_id, BlockPayloadExt};
use crate::state_replication::{StateComputer, TxnManager};
use atomic_refcell::AtomicRefCell;
use consensus_types::{block::Block, quorum_cert::QuorumCert, vote_data::VoteData};
use cuckoo::consensus::PowService;
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_crypto::x25519::{X25519StaticPrivateKey, X25519StaticPublicKey};
use libra_crypto::HashValue;
use libra_crypto::{hash::CryptoHash, x25519::compat, PrivateKey};
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use libra_types::block_info::BlockInfo;
use libra_types::block_metadata::BlockMetadata;
use libra_types::crypto_proxies::ValidatorSigner;
use libra_types::transaction::SignedTransaction;
use libra_types::{
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    validator_public_keys::ValidatorPublicKeys,
    validator_set::ValidatorSet,
};
use miner::types::{MineCtx, MineState, MineStateManager};
use network::{
    proto::{
        Block as BlockProto, ConsensusMsg,
        ConsensusMsg_oneof::{self},
    },
    validator_network::{ConsensusNetworkSender, Event},
};
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng};
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
    _pow_srv: Arc<dyn PowService>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
    mine_state: MineStateManager,
    self_key: Option<Ed25519PrivateKey>,
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
        mine_state: MineStateManager,
        pri_key: Ed25519PrivateKey,
    ) -> Self {
        MintManager {
            txn_manager,
            state_computer,
            network_sender,
            author,
            self_sender,
            block_store,
            _pow_srv: pow_srv,
            chain_manager,
            mine_state,
            self_key: Some(pri_key),
        }
    }

    pub fn mint(&mut self, executor: TaskExecutor) {
        let mint_txn_manager = self.txn_manager.clone();
        let mint_state_computer = self.state_computer.clone();
        let mut mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let mut self_sender = self.self_sender.clone();
        let block_db = self.block_store.clone();
        let chain_manager = self.chain_manager.clone();
        let mut mine_state = self.mine_state.clone();
        let self_pri_key = self.self_key.take().expect("self_key is none.");
        let self_pub_key = self_pri_key.public_key();
        let self_signer_address = AccountAddress::from_public_key(&self_pub_key);
        let (_tmp_pri_key, tmp_pub_key) = network_keypair();
        let keys = vec![ValidatorPublicKeys::new(
            self_signer_address,
            self_pub_key.clone(),
            100,
            self_pub_key,
            tmp_pub_key,
        )];
        let signer = ValidatorSigner::new(self_signer_address, self_pri_key);
        let mint_fut = async move {
            loop {
                match mint_txn_manager.pull_txns(100, vec![]).await {
                    Ok(txns) => {
                        let (height, parent_block) =
                            chain_manager.borrow().chain_height_and_root().await;
                        //create block
                        let parent_block_id = parent_block.id();
                        let grandpa_block_id = parent_block.parent_id();
                        //QC with parent block id
                        let quorum_cert = if parent_block_id != genesis_id() {
                            let parent_block = block_db
                                .get_block_by_hash::<BlockPayloadExt>(&parent_block_id)
                                .expect("block not find in database err.");
                            parent_block.quorum_cert().clone()
                        } else {
                            QuorumCert::certificate_for_genesis_from_ledger_info(
                                &LedgerInfo::genesis(),
                                genesis_id(),
                            )
                        };

                        //compute current block state id
                        let timestamp_usecs =
                            quorum_cert.ledger_info().ledger_info().timestamp_usecs() + 10;
                        let tmp_id = HashValue::random();
                        let block_meta_data = BlockMetadata::new(
                            parent_block_id.clone(),
                            timestamp_usecs,
                            BTreeMap::new(),
                            self_signer_address,
                        );
                        match mint_state_computer
                            .compute_by_hash(
                                &grandpa_block_id,
                                &parent_block_id,
                                &tmp_id,
                                vec![(block_meta_data.clone(), txns.clone())],
                            )
                            .await
                        {
                            Ok(processed_vm_output) => {
                                let executed_trees = processed_vm_output.executed_trees();
                                let state_id = executed_trees.state_root();
                                let txn_accumulator_hash =
                                    executed_trees.txn_accumulator().root_hash();
                                let txn_len = executed_trees.version().expect("version err.");

                                let parent_vd = quorum_cert.vote_data();
                                let epoch = parent_vd.parent().epoch();

                                // vote data
                                let parent_block_info = parent_vd.proposed().clone();
                                let current_block_info = BlockInfo::new(
                                    epoch,
                                    height + 1,
                                    parent_block_id.clone(),
                                    txn_accumulator_hash,
                                    txn_len,
                                    timestamp_usecs,
                                    Some(ValidatorSet::new(keys.clone())),
                                );
                                let vote_data =
                                    VoteData::new(current_block_info.clone(), parent_block_info);
                                let li = LedgerInfo::new(current_block_info, state_id);

                                let signature = signer
                                    .sign_message(li.hash())
                                    .expect("Fail to sign genesis ledger info");
                                let mut signatures = BTreeMap::new();
                                signatures.insert(self_signer_address, signature);
                                let new_qc = QuorumCert::new(
                                    vote_data,
                                    LedgerInfoWithSignatures::new(li.clone(), signatures),
                                );

                                //mint
                                let nonce = generate_nonce();

                                let (rx, _tx) = mine_state.mine_block(MineCtx {
                                    header: li.hash().to_vec(),
                                    nonce,
                                });

                                let solve = rx.recv().await.unwrap();
                                let mint_data = BlockPayloadExt { txns, nonce, solve };

                                //block data
                                let block = Block::<BlockPayloadExt>::new_proposal(
                                    mint_data,
                                    height + 1,
                                    timestamp_usecs,
                                    new_qc,
                                    &signer,
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

                        let mut r = rand::thread_rng();
                        r.gen::<i32>();
                        let sleep_time = r.gen_range(1, 5);
                        debug!("sleep begin.");
                        sleep(Duration::from_secs(sleep_time));
                        debug!("sleep end.");
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

fn network_keypair() -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
    let seed = [0u8; 32];
    let mut fast_rng = StdRng::from_seed(seed);
    compat::generate_keypair(&mut fast_rng)
}
