// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::{anyhow, bail, format_err, Result};
use diem_client::{
    views::AccountStateWithProofView, AccountAddress, BlockingClient, MethodRequest, MethodResponse,
};
use diem_read_write_set::ReadWriteSetAnalysis;
use diem_state_view::StateView;
use diem_transaction_replay::DiemDebugger;
use diem_types::access_path::AccessPath;
use diem_types::transaction::TransactionOutput;
use diem_types::{
    account_state::AccountState, account_state_blob::AccountStateBlob, transaction::Transaction,
};
use diem_vm::{DiemVM, VMExecutor};
use move_binary_format::CompiledModule;
use move_cli::sandbox::utils::{
    mode::{Mode, ModeType},
    on_disk_state_view::OnDiskStateView,
};
use move_core_types::language_storage::ResourceKey;

pub struct OnDiskStateViewReader {
    view: OnDiskStateView,
}

struct AccountAddressWithState {
    address: AccountAddress,
    state: AccountState,
}

pub struct PrefetchTransactionReplayerConfig {
    build_dir: PathBuf,
    storage_dir: PathBuf,
    rpc_url: String,
    module_mode_type: ModeType,
}

pub struct PrefetchTransactionReplayer {
    state: OnDiskStateViewReader,
    debugger: DiemDebugger,
    rpc_client: BlockingClient,
}

impl StateView for OnDiskStateViewReader {
    fn get(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>> {
        access_path.get_struct_tag().map_or(Ok(None), |tag| {
            self.view.get_resource_bytes(access_path.address, tag)
        })
    }

    fn is_genesis(&self) -> bool {
        false
    }
}

impl PrefetchTransactionReplayer {
    pub fn from_config(config: PrefetchTransactionReplayerConfig) -> Result<Self> {
        let mode = Mode::new(config.module_mode_type);
        let state = OnDiskStateViewReader {
            view: mode.prepare_state(&config.build_dir, &config.storage_dir)?,
        };
        let debugger = DiemDebugger::json_rpc_with_config(
            &config.rpc_url,
            config.build_dir,
            config.storage_dir,
        )?;

        let rpc_client = BlockingClient::new(&config.rpc_url);

        Ok(PrefetchTransactionReplayer {
            state,
            debugger,
            rpc_client,
        })
    }

    fn get_keys_read_for_tx(
        &self,
        tx: &Transaction,
        modules: &[CompiledModule],
    ) -> Result<Vec<ResourceKey>> {
        let signed_tx = match tx {
            Transaction::UserTransaction(u) => u,
            _ => bail!("can only process user transactions"),
        };

        let rw_analysis = ReadWriteSetAnalysis::new(read_write_set::analyze(modules)?);
        rw_analysis.get_keys_read(signed_tx, &self.state.view)
    }

    fn get_unique_addresses_from_keys(&self, key_reads: &[ResourceKey]) -> Vec<AccountAddress> {
        key_reads
            .iter()
            .map(|k| k.address)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect()
    }

    fn extract_account_state(
        &self,
        state_with_proof: &AccountStateWithProofView,
    ) -> Option<AccountState> {
        // NOTE: if this actually has a chance to fail, it might be better to change
        // the function to return Result<Option<AccountState>, _>
        state_with_proof.blob.as_ref().and_then(|bytes| {
            bcs::from_bytes::<AccountStateBlob>(bytes)
                .map_err(|e| anyhow!(e.to_string()))
                .and_then(|account_state_blob| AccountState::try_from(&account_state_blob))
                .ok()
        })
    }

    fn fetch_accounts_storage(
        &self,
        addresses: &[AccountAddress],
    ) -> Result<Vec<AccountAddressWithState>> {
        let requests = addresses
            .iter()
            .map(|a| MethodRequest::get_account_state_with_proof(*a, None, None))
            .collect();

        let results = self
            .rpc_client
            .batch(requests)?
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?;

        let account_states: Vec<AccountAddressWithState> = results
            .into_iter()
            .zip(addresses.iter())
            .filter_map(|(r, a)| match r.into_inner() {
                MethodResponse::GetAccountStateWithProof(v) => self
                    .extract_account_state(&v)
                    .map(|state| AccountAddressWithState { address: *a, state }),
                _ => None,
            })
            .collect();

        Ok(account_states)
    }

    fn populate_storage_for_transactions(
        &self,
        txs: &[Transaction],
        version: Option<u64>,
    ) -> Result<()> {
        let replay_version = match version {
            Some(v) => v,
            None => self.debugger.get_latest_version()?,
        };

        let modules = self
            .debugger
            .get_diem_framework_modules_at_version(replay_version, true)?;

        let mut keys = vec![];
        for tx in txs.iter() {
            let new_keys = self.get_keys_read_for_tx(tx, &modules)?;
            keys.extend(new_keys);
        }
        let unique_addresses = self.get_unique_addresses_from_keys(&keys);

        let accounts_states = self.fetch_accounts_storage(&unique_addresses)?;
        for account in accounts_states {
            self.debugger
                .save_account_state(account.address, &account.state)?;
        }
        Ok(())
    }

    pub fn execute_transactions_at(
        &self,
        txs: Vec<Transaction>,
        version: Option<u64>,
    ) -> Result<Vec<TransactionOutput>> {
        self.populate_storage_for_transactions(&txs, version)?;
        DiemVM::execute_block(txs, &self.state)
            .map_err(|err| format_err!("Unexpected VM Error: {:?}", err))
    }
}
