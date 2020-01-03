use crate::chained_bft::consensusdb::ConsensusDB;
use crate::pow::chain_manager::ChainManager;
use crate::pow::event_processor::EventProcessor;
use crate::pow::mine_state::{BlockIndex, MineStateManager};
use crate::state_replication::{StateComputer, TxnManager};
use anyhow::Result;
use async_std::sync::Sender;
use atomic_refcell::AtomicRefCell;
use consensus_types::{
    block::Block,
    block_data::BlockData,
    payload_ext::{genesis_id, BlockPayloadExt},
    quorum_cert::QuorumCert,
    vote_data::VoteData,
};
use futures::{channel::mpsc, SinkExt, StreamExt};
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
use miner::types::{MineState, Proof};
use network::{
    proto::{
        Block as BlockProto, ConsensusMsg,
        ConsensusMsg_oneof::{self},
    },
    validator_network::{ConsensusNetworkSender, Event},
};
use rand::{rngs::StdRng, SeedableRng};
use std::collections::{BTreeMap, HashMap};
use std::convert::TryInto;
use std::sync::Arc;
use tokio::runtime::Handle;

pub struct MintManager {
    txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
    state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
    network_sender: ConsensusNetworkSender,
    author: AccountAddress,
    self_sender: channel::Sender<Result<Event<ConsensusMsg>>>,
    block_store: Arc<ConsensusDB>,
    chain_manager: Arc<AtomicRefCell<ChainManager>>,
    mine_state: MineStateManager<BlockIndex>,
}

impl MintManager {
    pub fn new(
        txn_manager: Arc<dyn TxnManager<Payload = Vec<SignedTransaction>>>,
        state_computer: Arc<dyn StateComputer<Payload = Vec<SignedTransaction>>>,
        network_sender: ConsensusNetworkSender,
        author: AccountAddress,
        self_sender: channel::Sender<Result<Event<ConsensusMsg>>>,
        block_store: Arc<ConsensusDB>,
        chain_manager: Arc<AtomicRefCell<ChainManager>>,
        mine_state: MineStateManager<BlockIndex>,
    ) -> Self {
        MintManager {
            txn_manager,
            state_computer,
            network_sender,
            author,
            self_sender,
            block_store,
            chain_manager,
            mine_state,
        }
    }

    pub fn mint(
        &self,
        executor: Handle,
        self_pri_key: Ed25519PrivateKey,
        mut new_block_receiver: mpsc::Receiver<u64>,
    ) {
        let mint_txn_manager = self.txn_manager.clone();
        let mint_state_computer = self.state_computer.clone();
        let mint_network_sender = self.network_sender.clone();
        let mint_author = self.author;
        let self_sender = self.self_sender.clone();
        let block_db = self.block_store.clone();
        let chain_manager = self.chain_manager.clone();
        let mut mine_state = self.mine_state.clone();
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
        let signer_account_address = signer.author();
        let wait_executor = executor.clone();
        let mint_fut = async move {
            let mut proof_sender_map: HashMap<u64, Sender<Option<Proof>>> = HashMap::new();
            let (block_data_sender, mut block_data_receiver) = mpsc::channel(1024);
            loop {
                ::futures::select! {
                    block_data = block_data_receiver.select_next_some() => {
                        //block data
                        let block = Block::<BlockPayloadExt>::new_proposal_from_block_data(
                            block_data,
                            &signer,
                        );

                        info!(
                            "Peer : {:?}, Minter : {:?} find a new block : {:?}",
                            mint_author,
                            self_signer_address,
                            block.id()
                        );
                        let block_pb = TryInto::<BlockProto>::try_into(block)
                            .expect("parse block err.");

                        // send block
                        let msg = ConsensusMsg {
                            message: Some(ConsensusMsg_oneof::NewBlock(block_pb)),
                        };

                        EventProcessor::broadcast_consensus_msg(
                            &mut mint_network_sender.clone(),
                            true,
                            mint_author,
                            &mut self_sender.clone(),
                            msg,
                        )
                            .await;
                    }
                    latest_height = new_block_receiver.select_next_some() => {
                        {
                            for key in proof_sender_map.keys() {
                                if let Some(tmp_tx) = proof_sender_map.get(key) {
                                    tmp_tx.send(None).await;
                                }
                            }
                        }

                        match mint_txn_manager.pull_txns(100, vec![]).await {
                            Ok(txns) => {
                                if let Some((height, parent_block)) =
                                    chain_manager.borrow().chain_height_and_root().await
                                {
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
                                            let vote_data = VoteData::new(
                                                current_block_info.clone(),
                                                parent_block_info,
                                            );
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
                                            mine_state.set_latest_block(parent_block_id);
                                            let (rx, tx) = mine_state.mine_block(li.hash().to_vec());
                                            {
                                                proof_sender_map.clear();
                                                proof_sender_map.insert(latest_height, tx);
                                            }
                                            let wait_mint_network_sender = mint_network_sender.clone();
                                            let wait_self_sender = self_sender.clone();
                                            let mut wait_block_data_sender = block_data_sender.clone();
                                            let wait_fut = async move {
                                                if let Some(proof) = rx.recv().await.unwrap() {
                                                    let mint_data = BlockPayloadExt {
                                                        txns,
                                                        nonce: proof.nonce,
                                                        solve: proof.solution,
                                                        target: proof.target.to_vec(),
                                                        algo: proof.algo.into(),
                                                    };

                                                    let block_data = BlockData::<BlockPayloadExt>::new_proposal(
                                                        mint_data,
                                                        signer_account_address,
                                                        height + 1,
                                                        timestamp_usecs,
                                                        new_qc,
                                                    );

                                                    wait_block_data_sender.send(block_data).await.unwrap();
                                                }
                                            };

                                            wait_executor.clone().spawn(wait_fut);
                                        }
                                        Err(e) => {
                                            error!("{:?}", e);
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    complete => {
                       break;
                   }
                }
            }
        };
        executor.spawn(mint_fut);
    }
}

fn network_keypair() -> (X25519StaticPrivateKey, X25519StaticPublicKey) {
    let seed = [0u8; 32];
    let mut fast_rng = StdRng::from_seed(seed);
    compat::generate_keypair(&mut fast_rng)
}
