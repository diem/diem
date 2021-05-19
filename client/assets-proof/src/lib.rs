// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! A CLI tool for collecting and aggregating a VASPs current on-chain assets into
//! an easily digestible json output.
//!
//! ### Example usage:
//!
//! ```sh
//! $ cargo run -p diem-assets-proof -- collect \
//!     --json-server 'https://testnet.diem.com/v1' \
//!     --parent-vasp 'ed34ba9396da470da8de0945555549f8' \
//!     --child-vasps 'f236d4d977cbf08fe6e8e796bb6b41bd' \
//!     --child-vasps 'ad8c1f9be3b9de371567eda247c1f290' \
//!     --child-vasps '228d25d0c232c010be8bc1613f833ea0'
//!
//! {
//!   "diem_chain_id": 2,
//!   "diem_ledger_version": 88497519,
//!   "diem_ledger_timestampusec": 1616457122815922,
//!   "accumulator_root_hash": "75e03d04f13f15450807ced2d8707fd90b3a95d71ce981396deacf6134f20b64",
//!   "all_child_vasps_valid": true,
//!   "total_unfrozen_balances": {
//!     "XUS": 100000000000
//!   },
//!   "currencies": {
//!     "XDX": {
//!       "scaling_factor": 1000000,
//!       "fractional_part": 1000,
//!       "to_xdx_exchange_rate": 1.0
//!     },
//!     "XUS": {
//!       "scaling_factor": 1000000,
//!       "fractional_part": 100,
//!       "to_xdx_exchange_rate": 1.0
//!     }
//!   },
//!   "parent_vasp": {
//!     "address": "ED34BA9396DA470DA8DE0945555549F8",
//!     "balances": {
//!       "XUS": 70000000000
//!     },
//!     "human_name": "No. 63291 VASP",
//!     "base_url": "http://localhost:54330",
//!     "num_children": 3
//!   },
//!   "child_vasps": {
//!     "228D25D0C232C010BE8BC1613F833EA0": {
//!       "result": {
//!         "balances": {
//!           "XUS": 10000000000
//!         }
//!       }
//!     },
//!     "AD8C1F9BE3B9DE371567EDA247C1F290": {
//!       "result": {
//!         "balances": {
//!           "XUS": 10000000000
//!         }
//!       }
//!     },
//!     "F236D4D977CBF08FE6E8E796BB6B41BD": {
//!       "result": {
//!         "balances": {
//!           "XUS": 10000000000
//!         }
//!       }
//!     }
//!   }
//! }
//! ```

use anyhow::{ensure, format_err, Context, Result};
use diem_client::{
    views::{AccountRoleView, AccountView, AmountView, CurrencyInfoView, MetadataView},
    Response,
};
use diem_crypto::HashValue;
use diem_types::{
    account_address::AccountAddress, account_config::constants::from_currency_code_string,
    account_state::AccountState, account_state_blob::AccountStateBlob, chain_id::ChainId,
    transaction::Version,
};
use move_core_types::identifier::Identifier;
use serde::Serialize;
use std::{collections::BTreeMap, convert::TryFrom, iter, ops::AddAssign};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "todo")]
pub struct Args {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Collect a Proof-of-Assets receipt for a Diem VASP's on-chain accounts.
    Collect(CollectOptions),
}

#[derive(Debug, StructOpt)]
pub struct CommonOptions {
    /// Chain ID of the network to connect to. If unset, it will default to whatever
    /// network the json-rpc server is currently on.
    #[structopt(short = "i", long)]
    chain_id: Option<ChainId>,

    /// The URL of the fullnode json-rpc service used to interface with Diem.
    #[structopt(short = "j", long)]
    json_server: String,

    /// Print verbose debug messages.
    #[structopt(short = "v", long)]
    verbose: bool,
}

#[derive(Debug, StructOpt)]
pub struct CollectOptions {
    #[structopt(flatten)]
    common: CommonOptions,

    /// The account address of the parent VASP.
    #[structopt(short = "p", long)]
    parent_vasp: AccountAddress,

    /// Account addresses of the child VASPs.
    #[structopt(short = "c", long)]
    child_vasps: Vec<AccountAddress>,

    /// If set, collect the receipt at this timestamp (UTC time in microseconds
    /// since unix epoch). If this argument and `--version` are unset, collect the
    /// receipt at the latest state.
    #[structopt(short = "t", long)]
    timestamp_usecs: Option<u64>,

    /// If set, collect the receipt at this ledger version. If this argument and
    /// `--timestamp_usecs` are unset, collect the receipt at the latest state.
    #[structopt(long)]
    version: Option<Version>,
}

/// A simplified view of the parent VASP account, ignoring irrelevant info like
/// the sent/receive event keys.
#[derive(Debug, Serialize)]
pub struct ParentVASPView {
    address: AccountAddress,
    balances: BalancesView,
    human_name: String,
    base_url: String,
    /// The number of child VASP accounts under this parent VASP account.
    num_children: u64,
}

impl TryFrom<AccountView> for ParentVASPView {
    type Error = anyhow::Error;

    fn try_from(account: AccountView) -> Result<Self> {
        ensure!(
            !account.is_frozen,
            "account is currently frozen by Diem treasury compliance"
        );

        match account.role {
            AccountRoleView::ParentVASP {
                human_name,
                base_url,
                num_children,
                ..
            } => Ok(ParentVASPView {
                address: account.address,
                balances: BalancesView::new(account.balances),
                human_name,
                base_url,
                num_children,
            }),
            _ => Err(format_err!(
                "expected parent VASP account, actual account type: {:?}",
                account.role
            )),
        }
    }
}

/// A simplified view of a single child VASP account. Currently we only display
/// the balances for each account.
#[derive(Debug, Serialize)]
pub struct ChildVASPView {
    balances: BalancesView,

    #[serde(skip_serializing)]
    address: AccountAddress,

    #[serde(skip_serializing)]
    parent_vasp_address: AccountAddress,
}

impl TryFrom<AccountView> for ChildVASPView {
    type Error = anyhow::Error;

    fn try_from(account: AccountView) -> Result<Self> {
        ensure!(
            !account.is_frozen,
            "account is currently frozen by Diem treasury compliance"
        );

        match account.role {
            AccountRoleView::ChildVASP {
                parent_vasp_address,
                ..
            } => Ok(ChildVASPView {
                address: account.address,
                balances: BalancesView::new(account.balances),
                parent_vasp_address,
            }),
            _ => Err(format_err!(
                "expected child VASP account, actual account type: {:?}",
                account.role
            )),
        }
    }
}

/// A simplified view of the current on-chain currency metadata.
#[derive(Debug, Serialize)]
pub struct SimpleCurrencyView {
    /// The currency code (parsed as a valid currency Move [`Identifier`]).
    #[serde(skip_serializing)]
    currency: Identifier,

    /// Factor by which the amount is scaled before it is stored on-chain.
    scaling_factor: u64,

    /// Max fractional part of a single unit of currency allowed in a transaction.
    fractional_part: u64,

    /// Current on-chain exchange rate of the this currency to the XDX currency.
    to_xdx_exchange_rate: f32,
}

impl TryFrom<CurrencyInfoView> for SimpleCurrencyView {
    type Error = anyhow::Error;

    fn try_from(currency: CurrencyInfoView) -> Result<Self> {
        let code = from_currency_code_string(&currency.code)?;
        Ok(SimpleCurrencyView {
            currency: code,
            scaling_factor: currency.scaling_factor,
            fractional_part: currency.fractional_part,
            to_xdx_exchange_rate: currency.to_xdx_exchange_rate,
        })
    }
}

/// A set of balances by currency.
#[derive(Clone, Debug, Serialize)]
pub struct BalancesView(BTreeMap<String, u64>);

impl BalancesView {
    pub fn empty() -> Self {
        Self(BTreeMap::new())
    }

    pub fn new(balances: Vec<AmountView>) -> Self {
        Self(
            balances
                .into_iter()
                .map(|balance| (balance.currency, balance.amount))
                .collect(),
        )
    }

    pub fn merge(mut self, other: BalancesView) -> Self {
        for (currency, amount) in other.0.into_iter() {
            self.0.entry(currency).or_default().add_assign(amount);
        }
        self
    }
}

/// Truncated view of the [`MetadataView`].
///
/// Note: we try to mirror the json-rpc naming convention
/// (e.g., "diem_ledger_version" instead of "version") for consistency.
#[derive(Debug, Serialize)]
pub struct SimpleMetadataView {
    /// The ledger chain id. Used to distinguish between e.g. testnet and mainnet.
    diem_chain_id: ChainId,

    /// The ledger version that we collected this receipt at.
    diem_ledger_version: Version,

    /// The ledger timestamp (UTC time in microseconds since unix epoch) that we
    /// collected this receipt at.
    diem_ledger_timestampusec: u64,

    /// The current transaction accumulator root hash.
    accumulator_root_hash: HashValue,
}

impl From<MetadataView> for SimpleMetadataView {
    fn from(metadata: MetadataView) -> Self {
        SimpleMetadataView {
            diem_chain_id: ChainId::new(metadata.chain_id),
            diem_ledger_version: metadata.version,
            diem_ledger_timestampusec: metadata.timestamp,
            accumulator_root_hash: metadata.accumulator_root_hash,
        }
    }
}

/// A helper type for serializing `Result<T>` with serde-json.
#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultWrapper<T> {
    Result(T),
    Error(String),
}

impl<T> ResultWrapper<T> {
    pub fn new(result: Result<T>) -> ResultWrapper<T> {
        match result {
            Ok(val) => Self::Result(val),
            // Use the "alternate" display format so we get the whole error chain.
            Err(err) => Self::Error(format!("{:#}", err)),
        }
    }

    pub fn is_error(&self) -> bool {
        match self {
            Self::Result(_) => false,
            Self::Error(_) => true,
        }
    }
}

/// A receipt containing the aggregate balance of a VASP and all of its child VASP
/// accounts, in addition to each individual account's balance and on-chain metadata.
#[derive(Debug, Serialize)]
pub struct AssetsProof {
    /// The on-chain ledger metadata when this receipt was recorded.
    #[serde(flatten)]
    metadata: SimpleMetadataView,

    /// True if all child VASPs are valid, where valid means the number of expected
    /// child VASPs matches the value on-chain, and each child VASP correctly
    /// deserializes and contains the expected parent VASP address.
    all_child_vasps_valid: bool,

    /// The sum total unfrozen balance of this VASP and all of its child VASP
    /// accounts, split by currency.
    total_unfrozen_balances: BalancesView,

    /// Since the on-chain currency info is the source-of-truth, we include this
    /// metadata alongside this snapshot of the aggregate VASP balance in case then
    /// currency info changes.
    currencies: BTreeMap<Identifier, SimpleCurrencyView>,

    /// The parent VASP account and its associated balances and metadata.
    parent_vasp: ParentVASPView,

    /// The child VASP accounts and their associated balances. If we fail to
    /// retrieve/deserialize/validate a child VASP account, we don't error-out
    /// the whole process; instead, the entry for that account contains an error
    /// message.
    child_vasps: BTreeMap<AccountAddress, ResultWrapper<ChildVASPView>>,
}

impl Args {
    pub fn exec(self) -> Result<String> {
        match self.cmd {
            Command::Collect(opts) => pretty_print(opts.exec()),
        }
    }
}

impl CollectOptions {
    fn exec(&self) -> Result<AssetsProof> {
        let client = diem_client::BlockingClient::new(self.common.json_server.clone());
        self.exec_with_client(client)
    }

    fn exec_with_client(&self, client: impl Client) -> Result<AssetsProof> {
        // TODO(philiphayes): do we need json-rpc request batching? Currently we
        // make a separate request per account, which is a bit inefficient but not
        // particularly problematic as long as there are not too many child accounts.

        // Get the current on-chain metadata (version, timestamp, chain id).
        let metadata = self
            .resolve_metadata(&client)
            .context("Failed to resolve current chain metadata")?;
        let target_version = metadata.diem_ledger_version;

        if let Some(expected_chain_id) = self.common.chain_id {
            ensure!(
                expected_chain_id == metadata.diem_chain_id,
                "Remote service's chain doesn't match our expected chain id: actual: {}, expected: {}",
                metadata.diem_chain_id,
                expected_chain_id,
            );
        }

        // Get the current on-chain currency metadata.
        let currencies = client
            .get_currencies()?
            .into_inner()
            .into_iter()
            .map(SimpleCurrencyView::try_from)
            .collect::<Result<Vec<_>>>()
            .context("Invalid currency metadata")?;

        // Get the parent VASP account.
        let parent_vasp = client
            .get_account_by_version(self.parent_vasp, target_version)
            .context("Failed to retrieve parent VASP account")?
            .into_inner()
            .ok_or_else(|| {
                format_err!(
                    "No parent VASP account at the address: '{}'",
                    self.parent_vasp
                )
            })
            .and_then(ParentVASPView::try_from)
            .context("Invalid parent VASP account")?;

        let mut all_child_vasps_valid = true;

        // Just _warn_ here rather than return an error. For VASPs with a large
        // number of child VASP accounts, there's a tricky TOCTOU issue with
        // requesting the list of child accounts and then checking the total
        // assets, where the set of child accounts might have changed in between.
        //
        // At the worst, we underestimate the total assets, which is less problematic
        // for e.g. auditing solvency, though more problematic for e.g. tax
        // reporting...
        if parent_vasp.num_children != self.child_vasps.len() as u64 {
            all_child_vasps_valid = false;
            eprintln!(
                "WARNING: The actual number of child VASP accounts on-chain \
                 doesn't match the expected number: actual: {}, expected: {}",
                parent_vasp.num_children,
                self.child_vasps.len(),
            );
        }

        // Get all the child VASPs and validate they are in fact this parent's child
        // and not frozen by treasury compliance.
        let child_vasps = self
            .child_vasps
            .iter()
            .map(|child_vasp_address| -> (AccountAddress, ResultWrapper<ChildVASPView>) {
                let maybe_account_view = client
                    .get_account_by_version(*child_vasp_address, target_version)
                    .context("Failed to retrieve child VASP account")
                    .map(Response::into_inner)
                    .and_then(|opt_account_view| opt_account_view.ok_or_else(|| format_err!("no child VASP account at the address")));

                let maybe_child_vasp = maybe_account_view
                    .and_then(|account_view| ChildVASPView::try_from(account_view).context("invalid child VASP account"))
                    .and_then(|child_vasp| {
                        ensure!(
                            child_vasp.parent_vasp_address == parent_vasp.address,
                            "Child VASP's parent VASP account doesn't match: child_vasp.parent_vasp_address: '{}'",
                            child_vasp.parent_vasp_address,
                        );
                        Ok(child_vasp)
                    });

                if maybe_child_vasp.is_err() {
                    all_child_vasps_valid = false;
                }

                (*child_vasp_address, ResultWrapper::new(maybe_child_vasp))
            })
            .collect::<BTreeMap<_, _>>();

        // Aggregate all of the parent and child VASP balances.
        let child_vasp_balances = child_vasps.values().filter_map(|maybe_child_vasp| {
            let child_vasp = match maybe_child_vasp {
                ResultWrapper::Result(child_vasp) => child_vasp,
                ResultWrapper::Error(_) => return None,
            };
            Some(child_vasp.balances.clone())
        });
        let vasp_balances = iter::once(parent_vasp.balances.clone()).chain(child_vasp_balances);
        let total_unfrozen_balances =
            vasp_balances.fold(BalancesView::empty(), BalancesView::merge);

        let currencies = currencies
            .into_iter()
            .map(|currency_info| (currency_info.currency.clone(), currency_info))
            .collect::<BTreeMap<_, _>>();

        Ok(AssetsProof {
            metadata,
            all_child_vasps_valid,
            total_unfrozen_balances,
            currencies,
            parent_vasp,
            child_vasps,
        })
    }

    /// Resolve the current chain metadata according the the config.
    ///
    /// 1. If the version is pre-specified, then we get the metadata for that
    ///    specific version.
    /// 2. If a timestamp is pre-specified, then get the metadata for the last
    ///    state at or before that timestamp.
    /// 3. Otherwise, we just get the latest metadata.
    fn resolve_metadata(&self, client: &impl Client) -> Result<SimpleMetadataView> {
        let maybe_response = if let Some(version) = self.version {
            client.get_metadata_by_version(version)
        } else if let Some(timestamp_usecs) = self.timestamp_usecs {
            let current_metadata = client.get_metadata()?;
            let latest_version = current_metadata.state().version;
            let past_version =
                client.get_last_version_before_timestamp(timestamp_usecs, latest_version)?;
            client.get_metadata_by_version(past_version)
        } else {
            client.get_metadata()
        };

        let metadata = maybe_response?.into_inner();
        Ok(SimpleMetadataView::from(metadata))
    }
}

/// A small trait abstracting over the Diem json-rpc client so we can mock during
/// testing.
pub trait Client {
    fn get_last_version_before_timestamp(
        &self,
        timestamp_usecs: u64,
        version: Version,
    ) -> Result<Version>;

    fn get_metadata(&self) -> Result<Response<MetadataView>>;

    fn get_metadata_by_version(&self, version: Version) -> Result<Response<MetadataView>>;

    fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>>;

    fn get_account_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<Response<Option<AccountView>>>;
}

impl Client for diem_client::BlockingClient {
    fn get_last_version_before_timestamp(
        &self,
        _timestamp_usecs: u64,
        _version: Version,
    ) -> Result<Version> {
        todo!()
    }

    fn get_metadata(&self) -> Result<Response<MetadataView>> {
        self.get_metadata().context("Failed to get ledger metadata")
    }

    fn get_metadata_by_version(&self, version: Version) -> Result<Response<MetadataView>> {
        self.get_metadata_by_version(version)
            .with_context(|| format!("Failed to get ledger metadata: version={}", version))
    }

    fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>> {
        self.get_currencies()
            .context("Failed to get current supported ledger currencies")
    }

    fn get_account_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<Response<Option<AccountView>>> {
        // HACK: until the follwing PR lands and hits release (https://github.com/diem/diem/pull/7983),
        // there is no `get_account_by_version` API available. However, we can
        // get around this by calling `get_account_state_with_proof`, which lets
        // us specify a version. We then just discard the state proof, deserialize
        // the `AccountState` and reconstitute the `AccountView` from that.

        let response =
            self.get_account_state_with_proof(address, Some(version), None /* to_version */)?;

        let (account_state_with_proof, response_state) = response.into_parts();
        let account_blob_view = match account_state_with_proof.blob {
            Some(account_blob) => account_blob,
            None => return Ok(Response::new(None, response_state)),
        };
        let account_blob: AccountStateBlob = bcs::from_bytes(&account_blob_view.as_ref())
            .context("Failed to deserialize AccountStateBlob")?;

        let account_state =
            AccountState::try_from(&account_blob).context("Failed to deserialize account state")?;
        let account_view = AccountView::try_from_account_state(address, account_state, version)
            .context("Failed to project account state into account view")?;
        Ok(Response::new(Some(account_view), response_state))
    }
}

/// For pretty printing outputs in JSON
pub fn pretty_print<T: Serialize>(result: Result<T>) -> Result<String> {
    result.map(|val| serde_json::to_string_pretty(&val).unwrap())
}
