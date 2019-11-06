use failure::prelude::*;
use config::config::NodeConfig;
use network::{
    proto::{
        ConsensusMsg, ConsensusMsg_oneof::{self, *}, Block as BlockProto, RequestBlock, RespondBlock, BlockRetrievalStatus,
    },
    validator_network::{ConsensusNetworkEvents, ConsensusNetworkSender, Event, RpcError}};
use std::{sync::Arc, thread};
use vm_runtime::MoveVM;
use executor::Executor;
use state_synchronizer::StateSyncClient;
use crate::state_replication::{StateComputer, TxnManager};
use libra_types::transaction::{SignedTransaction, Transaction};
use crate::{
    consensus_provider::ConsensusProvider,
    state_computer::ExecutionProxy,
    txn_manager::MempoolProxy,
};
use consensus_types::{block::Block, quorum_cert::QuorumCert, vote_data::VoteData};
use std::collections::{HashSet, BTreeMap};
use crypto::HashValue;
use futures::{executor::block_on, stream::select, SinkExt, StreamExt, TryStreamExt};
use libra_types::ledger_info::{LedgerInfo, LedgerInfoWithSignatures};
use crate::chained_bft::consensusdb::ConsensusDB;
use std::time::{Duration, Instant};
use rand::{self, Rng};
use tokio::runtime::{self, TaskExecutor};
use logger::prelude::*;
use libra_mempool::proto::mempool::MempoolClient;
use executor::{StateComputeResult, ExecutedState};
use libra_types::transaction::TransactionStatus;
use libra_types::vm_error::{VMStatus, StatusCode};
use crate::counters;
use std::{convert::{TryFrom, From}, path::PathBuf};
use crypto::hash::CryptoHash;
use libra_types::crypto_proxies::ValidatorSigner;
use libra_types::account_address::AccountAddress;
use channel;
use atomic_refcell::AtomicRefCell;
use prost_ext::MessageExt;
use crypto::hash::{GENESIS_BLOCK_ID, PRE_GENESIS_BLOCK_ID};
use futures::{channel::mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use libra_types::validator_set::ValidatorSet;
use libra_types::PeerId;
use std::collections::HashMap;
use bytes::Bytes;
use futures_locks::{Mutex, RwLock};
use {
    futures::{
        compat::Future01CompatExt,
        future::{self, FutureExt, TryFutureExt},
    },
};
use std::thread::sleep;
use tokio::timer::Interval;
use std::convert::TryInto;
use cuckoo::consensus::{PowCuckoo, PowService, Proof};
use network::validator_network::PowContext;
use crate::chained_bft::consensusdb::BlockIndex;
use crate::pow::chain_manager::{ChainManager, BlockChain};
use crate::pow::sync_manager::{SyncManager, BlockRetrievalResponse};
use crate::pow::mint_manager::MintManager;
use crate::pow::event_processor::EventProcessor;

pub struct PowConsensusProvider {
    runtime: tokio::runtime::Runtime,
    event_handle: Option<EventProcessor>,
}

impl PowConsensusProvider {
    pub fn new(node_config: &mut NodeConfig,
               network_sender: ConsensusNetworkSender,
               network_events: ConsensusNetworkEvents,
               mempool_client: Arc<MempoolClient>,
               executor: Arc<Executor<MoveVM>>,
               synchronizer_client: Arc<StateSyncClient>,
               rollback_flag: bool) -> Self {
        let runtime = runtime::Builder::new()
            .name_prefix("pow-consensus-")
            .build()
            .expect("Failed to create Tokio runtime!");

        let txn_manager = Arc::new(MempoolProxy::new(mempool_client.clone()));
        let state_computer = Arc::new(ExecutionProxy::new(executor, synchronizer_client.clone()));

        let peer_id_str = node_config
            .get_validator_network_config()
            .unwrap()
            .peer_id
            .clone();
        let author =
            AccountAddress::try_from(peer_id_str.clone()).expect("Failed to parse peer id of a validator");

        let genesis_transaction = node_config
            .get_genesis_transaction()
            .expect("failed to load genesis transaction!");

        let event_handle = EventProcessor::new(network_sender, network_events, txn_manager, state_computer, author, node_config.get_storage_dir(), genesis_transaction, rollback_flag);
        Self {
            runtime,
            event_handle: Some(event_handle),
        }
    }

    pub fn event_handle(&mut self, executor: TaskExecutor) {
        match self.event_handle.take() {
            Some(mut handle) => {
                //mint
                handle.mint_manager.mint(executor.clone());

                //msg
                handle.event_process(executor.clone());

                //save
                handle.chain_manager.borrow_mut().save_block(executor.clone());

                //sync
                handle.sync_manager.borrow_mut().sync_block_msg(executor.clone());

                //TODO:orphan
            }
            _ => {}
        }
    }
}

impl ConsensusProvider for PowConsensusProvider {
    fn start(&mut self) -> Result<()> {
        let executor = self.runtime.executor();
        self.event_handle(executor);
        Ok(())
    }

    fn stop(&mut self) {
        //TODO
        // 1. stop mint
        // 2. stop process event
        unimplemented!()
    }
}