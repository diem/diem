// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::Client,
    request::MethodRequest,
    response::{MethodResponse, Response},
    state::State,
};
use anyhow::{bail, ensure, format_err, Result};
use diem_json_rpc_types::views::AccountView;
use diem_types::{
    account_address::AccountAddress, account_config::diem_root_address,
    account_state::AccountState, account_state_blob::AccountStateWithProof,
    epoch_change::EpochChangeProof, ledger_info::LedgerInfoWithSignatures,
    on_chain_config::RegisteredCurrencies, proof::AccumulatorConsistencyProof,
    transaction::Version, trusted_state::TrustedState, waypoint::Waypoint,
};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt::Debug,
    sync::{Arc, RwLock},
};

// TODO(philiphayes): figure out retry strategy
// TODO(philiphayes): real on-disk waypoint persistence
// TODO(philiphayes): fill out rest of the methods
// TODO(philiphayes): use real error types
// TODO(philiphayes): non-verifying vs verifying client equivalence testing
// TODO(philiphayes): all clients should validate chain id (allow users to trust-on-first-use or pre-configure)
// TODO(philiphayes): we could abstract the async client so VerifyingClient takes a dyn Trait?

// TODO(philiphayes): we should really add a real StateProof type (not alias) that
// collects these types together. StateProof isn't a very descriptive name though...

type StateProof = (
    LedgerInfoWithSignatures,
    EpochChangeProof,
    AccumulatorConsistencyProof,
);

#[derive(Clone, Debug)]
pub struct VerifyingClient {
    inner: Client,
    trusted_state_store: Arc<RwLock<TrustedStateStore>>,
}

impl VerifyingClient {
    // TODO(philiphayes): construct the client ourselves? we probably want to
    // control the retries out here. For example, during sync, if we get a stale
    // state proof the retry logic should include that and not just fail immediately.
    pub fn new(inner: Client, storage: Box<dyn Storage>) -> Result<Self> {
        let trusted_state_store = TrustedStateStore::new(storage)?;
        Ok(Self {
            inner,
            trusted_state_store: Arc::new(RwLock::new(trusted_state_store)),
        })
    }

    pub fn new_with_state(
        inner: Client,
        trusted_state: TrustedState,
        storage: Box<dyn Storage>,
    ) -> Self {
        let trusted_state_store = TrustedStateStore::new_with_state(trusted_state, storage);
        Self {
            inner,
            trusted_state_store: Arc::new(RwLock::new(trusted_state_store)),
        }
    }

    /// Get a snapshot of our current trusted ledger [`Version`].
    pub fn version(&self) -> Version {
        self.trusted_state_store.read().unwrap().version()
    }

    /// Get a snapshot of our current trusted [`Waypoint`].
    pub fn waypoint(&self) -> Waypoint {
        self.trusted_state_store.read().unwrap().waypoint()
    }

    /// Get a snapshot of our current [`TrustedState`].
    pub fn trusted_state(&self) -> TrustedState {
        self.trusted_state_store
            .read()
            .unwrap()
            .trusted_state()
            .clone()
    }

    /// Issue `get_state_proof` requests until we successfully sync to the remote
    /// node's current version (unless we experience a verification error or other
    /// I/O error).
    pub async fn sync(&self) -> Result<()> {
        // TODO(philiphayes): retries
        while self.sync_one_step().await? {}
        Ok(())
    }

    /// Issue a single `get_state_proof` request and try to verify it. Returns
    /// `Ok(true)` if, after verification, we still need to sync more. Returns
    /// `Ok(false)` if we have finished syncing.
    pub async fn sync_one_step(&self) -> Result<bool> {
        let (state_proof_view, state) = self
            .inner
            .get_state_proof(self.version())
            .await?
            .into_parts();
        let state_proof = StateProof::try_from(&state_proof_view)?;

        let latest_li = state_proof.0.ledger_info();
        ensure!(latest_li.version() == state.version);
        ensure!(latest_li.timestamp_usecs() == state.timestamp_usecs);

        self.verify_and_ratchet(&state_proof)?;

        Ok(state_proof.1.more)
    }

    /// Verify and ratchet forward our trusted state using a state proof.
    pub fn verify_and_ratchet(&self, state_proof: &StateProof) -> Result<()> {
        let (latest_li, epoch_change_proof, _) = state_proof;

        let trusted_state = self.trusted_state();
        let change = trusted_state.verify_and_ratchet(latest_li, epoch_change_proof)?;

        if let Some(new_state) = change.new_state() {
            self.trusted_state_store
                .write()
                .unwrap()
                .ratchet(new_state)?;
        }

        // TODO(philiphayes): verify the accumulator consistency proof; no-one
        // seems to actually verify this anywhere?

        Ok(())
    }

    pub async fn get_account(
        &self,
        address: AccountAddress,
    ) -> Result<Response<Option<AccountView>>> {
        Ok(self
            .send_via_batch(MethodRequest::GetAccount((address,)))
            .await?
            .and_then(MethodResponse::try_into_get_account)?)
    }

    /// Send a single request via `VerifyingClient::batch`.
    async fn send_via_batch(&self, request: MethodRequest) -> Result<Response<MethodResponse>> {
        let mut responses = self.batch(vec![request]).await?.into_iter();
        let response = match responses.next() {
            Some(response) => response,
            None => bail!("response batch is empty"),
        };
        ensure!(responses.next().is_none(), "too many responses");
        response
    }

    pub async fn batch(
        &self,
        requests: Vec<MethodRequest>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        // transform each request into verifying sub-request batches
        let batch = VerifyingBatch::from_batch(requests);

        // flatten and collect sub-request batches into flat list of requests
        let mut requests = batch.collect_requests();
        // append get_state_proof request
        let start_version = self.version();
        requests.push(MethodRequest::get_state_proof(start_version));

        // actually send the batch
        let responses = self.inner.batch(requests).await?;

        // TODO(philiphayes): REMOVE. for now, just convert the errors until we
        // use the real error type.
        let mut responses = responses
            .into_iter()
            .map(|result| result.map_err(anyhow::Error::new))
            .collect::<Vec<_>>();

        // extract and verify the state proof
        let (state_proof_response, state) = responses.pop().unwrap()?.into_parts();
        let state_proof_view = state_proof_response.try_into_get_state_proof()?;
        let state_proof = StateProof::try_from(&state_proof_view)?;

        // check the response metadata matches the state proof
        let latest_li = state_proof.0.ledger_info();
        ensure!(state.version == latest_li.version());
        ensure!(state.timestamp_usecs == latest_li.timestamp_usecs());

        // try to ratchet our trusted state using the state proof
        self.verify_and_ratchet(&state_proof)?;

        // remote says we're too far behind and need to sync. we have to throw
        // out the batch since we can't verify any proofs
        if state_proof.1.more {
            bail!("needs sync");
        }

        // unflatten subresponses, verify, and collect into one response per request
        batch.validate_responses(start_version, &state, &state_proof, responses)
    }
}

struct VerifyingBatch {
    requests: Vec<VerifyingRequest>,
}

impl VerifyingBatch {
    fn from_batch(requests: Vec<MethodRequest>) -> Self {
        Self {
            requests: requests.into_iter().map(VerifyingRequest::from).collect(),
        }
    }

    fn num_subrequests(&self) -> usize {
        self.requests
            .iter()
            .map(|request| request.subrequests.len())
            .sum()
    }

    fn collect_requests(&self) -> Vec<MethodRequest> {
        self.requests
            .iter()
            .flat_map(|request| request.subrequests.iter().cloned())
            .collect()
    }

    fn validate_responses(
        &self,
        start_version: Version,
        state: &State,
        state_proof: &StateProof,
        responses: Vec<Result<Response<MethodResponse>>>,
    ) -> Result<Vec<Result<Response<MethodResponse>>>> {
        ensure!(
            self.num_subrequests() == responses.len(),
            "unexpected number of responses"
        );

        for response in &responses {
            if let Ok(response) = response {
                ensure!(response.state() == state);
            }
        }

        let mut responses_iter = responses.into_iter();

        Ok(self
            .requests
            .iter()
            .map(|request| {
                let n = request.subrequests.len();
                let subresponses = responses_iter.by_ref().take(n).collect();
                request.validate_subresponses(start_version, state, state_proof, subresponses)
            })
            .collect())
    }
}

struct RequestContext<'a> {
    #[allow(dead_code)]
    start_version: Version,

    #[allow(dead_code)]
    state: &'a State,

    state_proof: &'a StateProof,

    request: &'a MethodRequest,

    #[allow(dead_code)]
    subrequests: &'a [MethodRequest],
}

type RequestCallback = fn(RequestContext<'_>, &[MethodResponse]) -> Result<MethodResponse>;

struct VerifyingRequest {
    request: MethodRequest,
    subrequests: Vec<MethodRequest>,
    callback: RequestCallback,
}

impl VerifyingRequest {
    fn new(
        request: MethodRequest,
        subrequests: Vec<MethodRequest>,
        callback: RequestCallback,
    ) -> Self {
        Self {
            request,
            subrequests,
            callback,
        }
    }

    // TODO(philiphayes): this would be easier if the Error's were cloneable...

    fn validate_subresponses(
        &self,
        start_version: Version,
        state: &State,
        state_proof: &StateProof,
        subresponses: Vec<Result<Response<MethodResponse>>>,
    ) -> Result<Response<MethodResponse>> {
        ensure!(!subresponses.is_empty(), "no responses");
        ensure!(
            subresponses.len() == self.subrequests.len(),
            "unexpected number of responses"
        );

        let ctxt = RequestContext {
            start_version,
            state,
            state_proof,
            request: &self.request,
            subrequests: &self.subrequests.as_slice(),
        };

        // TODO(philiphayes): coalesce the Result's somehow so we don't lose error info.

        let subresponses_only = subresponses
            .into_iter()
            .map(|result| result.map(Response::into_inner))
            .collect::<Result<Vec<_>>>()?;

        let response = (self.callback)(ctxt, subresponses_only.as_slice())?;
        Ok(Response::new(response, state.clone()))
    }
}

impl From<MethodRequest> for VerifyingRequest {
    fn from(request: MethodRequest) -> Self {
        match request {
            MethodRequest::GetAccount((address,)) => verifying_get_account(address),
            _ => todo!(),
        }
    }
}

// TODO(philiphayes): add separate type for each MethodRequest? like:
//
// ```
// struct GetAccount((address,));
//
// enum MethodRequest {
//     GetAccount(GetAccount),
//     // ..
// }
// ```
//
// would allow the from(MethodRequest) above to call a method on the enum inner
// instead of these ad-hoc methods i think

fn verifying_get_account(address: AccountAddress) -> VerifyingRequest {
    let request = MethodRequest::GetAccount((address,));
    let subrequests = vec![
        MethodRequest::GetAccountStateWithProof(diem_root_address(), None, None),
        MethodRequest::GetAccountStateWithProof(address, None, None),
    ];
    let callback: RequestCallback = |ctxt, subresponses| match subresponses {
        [MethodResponse::GetAccountStateWithProof(ref diem_root), MethodResponse::GetAccountStateWithProof(ref account)] =>
        {
            let diem_root_with_proof = AccountStateWithProof::try_from(diem_root)?;
            let account_state_with_proof = AccountStateWithProof::try_from(account)?;

            let latest_li = ctxt.state_proof.0.ledger_info();
            let state_version = latest_li.version();

            let address = match ctxt.request {
                MethodRequest::GetAccount((address,)) => *address,
                _ => panic!("should not happen"),
            };

            diem_root_with_proof.verify(latest_li, state_version, diem_root_address())?;
            account_state_with_proof.verify(latest_li, state_version, address)?;

            // TODO(philiphayes): it seems wasteful to lookup the whole diem_root
            // account, verify the proofs, etc... just to get a list of supported
            // currency codes. Would it make sense for the AccountView to just
            // list _all_ BalanceResource's under balance? Then we could avoid
            // this extra lookup. It's even more problematic when a batch
            // includes many GetAccount requests, as this batch handling logic
            // isn't (currently) smart enough to dedup the diem_root lookups.
            let diem_root_blob = diem_root_with_proof
                .blob
                .ok_or_else(|| format_err!("missing diem_root accound somehow"))?;
            let diem_root = AccountState::try_from(&diem_root_blob)?;
            let currencies: Option<RegisteredCurrencies> = diem_root.get_config()?;
            let currency_codes = currencies
                .as_ref()
                .map(RegisteredCurrencies::currency_codes)
                .ok_or_else(|| format_err!("diem_root missing registered currencies"))?;

            let maybe_account_view = account_state_with_proof
                .blob
                .map(|blob| {
                    let account_state = AccountState::try_from(&blob)?;
                    AccountView::try_from_account_state(
                        address,
                        account_state,
                        &currency_codes,
                        state_version,
                    )
                })
                .transpose()?;

            Ok(MethodResponse::GetAccount(maybe_account_view))
        }
        _ => bail!("invalid responses"),
    };
    VerifyingRequest::new(request, subrequests, callback)
}

// TODO(philiphayes): reuse secure/storage?

pub trait Storage: Debug {
    fn get(&self, key: &str) -> Result<Vec<u8>>;
    fn set(&mut self, key: &str, value: Vec<u8>) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct InMemoryStorage {
    data: HashMap<String, Vec<u8>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for InMemoryStorage {
    fn get(&self, key: &str) -> Result<Vec<u8>> {
        self.data
            .get(key)
            .map(Clone::clone)
            .ok_or_else(|| format_err!("key not set"))
    }

    fn set(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.data.insert(key.to_owned(), value);
        Ok(())
    }
}

pub const TRUSTED_STATE_KEY: &str = "trusted_state";

#[derive(Debug)]
struct TrustedStateStore {
    trusted_state: TrustedState,
    storage: Box<dyn Storage>,
}

impl TrustedStateStore {
    fn new(storage: Box<dyn Storage>) -> Result<Self> {
        let trusted_state = storage
            .get(TRUSTED_STATE_KEY)
            .and_then(|bytes| bcs::from_bytes(&bytes).map_err(Into::into))?;

        Ok(Self {
            trusted_state,
            storage,
        })
    }

    fn new_with_state(trusted_state: TrustedState, storage: Box<dyn Storage>) -> Self {
        let maybe_stored_state: Result<TrustedState> = storage
            .get(TRUSTED_STATE_KEY)
            .and_then(|bytes| bcs::from_bytes(&bytes).map_err(Into::into));

        let trusted_state = if let Ok(stored_state) = maybe_stored_state {
            if trusted_state.version() > stored_state.version() {
                trusted_state
            } else {
                stored_state
            }
        } else {
            trusted_state
        };

        Self {
            trusted_state,
            storage,
        }
    }

    fn version(&self) -> Version {
        self.trusted_state.version()
    }

    fn waypoint(&self) -> Waypoint {
        self.trusted_state.waypoint()
    }

    fn trusted_state(&self) -> &TrustedState {
        &self.trusted_state
    }

    fn ratchet(&mut self, new_state: TrustedState) -> Result<()> {
        if new_state.version() > self.trusted_state.version() {
            self.trusted_state = new_state;
            let trusted_state_bytes = bcs::to_bytes(&self.trusted_state)?;
            self.storage.set(TRUSTED_STATE_KEY, trusted_state_bytes)?;
        }

        Ok(())
    }
}
