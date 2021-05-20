// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::Client,
    error::{Error, Result},
    request::MethodRequest,
    response::{MethodResponse, Response},
    state::State,
};
use diem_json_rpc_types::views::{AccountView, CurrencyInfoView, EventView};
use diem_types::{
    account_address::AccountAddress,
    account_config::diem_root_address,
    account_state::AccountState,
    account_state_blob::AccountStateWithProof,
    contract_event::EventWithProof,
    epoch_change::EpochChangeProof,
    event::EventKey,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::RegisteredCurrencies,
    proof::AccumulatorConsistencyProof,
    transaction::Version,
    trusted_state::TrustedState,
    waypoint::Waypoint,
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
pub struct VerifyingClient<S> {
    inner: Client,
    trusted_state_store: Arc<RwLock<TrustedStateStore<S>>>,
}

impl<S: Storage> VerifyingClient<S> {
    // TODO(philiphayes): construct the client ourselves? we probably want to
    // control the retries out here. For example, during sync, if we get a stale
    // state proof the retry logic should include that and not just fail immediately.
    pub fn new(inner: Client, storage: S) -> Result<Self> {
        let trusted_state_store = TrustedStateStore::new(storage)?;
        Ok(Self {
            inner,
            trusted_state_store: Arc::new(RwLock::new(trusted_state_store)),
        })
    }

    pub fn new_with_state(inner: Client, trusted_state: TrustedState, storage: S) -> Self {
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
        let state_proof = StateProof::try_from(&state_proof_view).map_err(Error::decode)?;

        // check the response metadata matches the state proof
        verify_latest_li_matches_state(state_proof.0.ledger_info(), &state)?;

        // try to ratchet our trusted state using the state proof
        self.verify_and_ratchet(&state_proof)?;

        Ok(state_proof.1.more)
    }

    /// Verify and ratchet forward our trusted state using a state proof.
    pub fn verify_and_ratchet(&self, state_proof: &StateProof) -> Result<()> {
        let (latest_li, epoch_change_proof, _) = state_proof;

        // TODO(philiphayes): stale error case
        let trusted_state = self.trusted_state();
        let change = trusted_state
            .verify_and_ratchet(latest_li, epoch_change_proof)
            .map_err(Error::invalid_proof)?;

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
        self.request(MethodRequest::get_account(address))
            .await?
            .and_then(MethodResponse::try_into_get_account)
    }

    pub async fn get_account_by_version(
        &self,
        address: AccountAddress,
        version: Version,
    ) -> Result<Response<Option<AccountView>>> {
        self.request(MethodRequest::get_account_by_version(address, version))
            .await?
            .and_then(MethodResponse::try_into_get_account)
    }

    pub async fn get_events(
        &self,
        key: EventKey,
        start_seq: u64,
        limit: u64,
    ) -> Result<Response<Vec<EventView>>> {
        self.request(MethodRequest::get_events(key, start_seq, limit))
            .await?
            .and_then(MethodResponse::try_into_get_events)
    }

    pub async fn get_currencies(&self) -> Result<Response<Vec<CurrencyInfoView>>> {
        self.request(MethodRequest::get_currencies())
            .await?
            .and_then(MethodResponse::try_into_get_currencies)
    }

    pub async fn get_network_status(&self) -> Result<Response<u64>> {
        self.request(MethodRequest::get_network_status())
            .await?
            .and_then(MethodResponse::try_into_get_network_status)
    }

    /// Send a single request via `VerifyingClient::batch`.
    pub async fn request(&self, request: MethodRequest) -> Result<Response<MethodResponse>> {
        let mut responses = self.batch(vec![request]).await?.into_iter();
        let response = match responses.next() {
            Some(response) => response,
            None => {
                return Err(Error::rpc_response(
                    "expected one response, received empty response batch",
                ))
            }
        };
        let rest = responses.as_slice();
        if !rest.is_empty() {
            return Err(Error::rpc_response(format!(
                "expected one response, received unexpected responses: {:?}",
                rest,
            )));
        }
        response
    }

    pub fn actual_batch_size(requests: &[MethodRequest]) -> usize {
        let actual_requests = VerifyingBatch::from_batch(requests.to_vec()).collect_requests();
        actual_requests.len() + 1 /* get_state_proof */
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
        let mut responses = self.inner.batch(requests).await?;

        // extract and verify the state proof
        let (state_proof_response, state) = responses.pop().unwrap()?.into_parts();
        let state_proof_view = state_proof_response.try_into_get_state_proof()?;
        let state_proof = StateProof::try_from(&state_proof_view).map_err(Error::decode)?;

        // check the response metadata matches the state proof
        verify_latest_li_matches_state(state_proof.0.ledger_info(), &state)?;

        // try to ratchet our trusted state using the state proof
        self.verify_and_ratchet(&state_proof)?;

        // remote says we're too far behind and need to sync. we have to throw
        // out the batch since we can't verify any proofs
        if state_proof.1.more {
            // TODO(philiphayes): what is the right behaviour here? it would obv.
            // be more convenient to just call `self.sync` here and then retry,
            // but maybe a client would like to control the syncs itself?
            return Err(Error::unknown(
                "our client is too far behind, we need to sync",
            ));
        }

        // unflatten subresponses, verify, and collect into one response per request
        batch.validate_responses(start_version, &state, &state_proof, responses)
    }
}

/// Check that certain metadata (version and timestamp) in a `LedgerInfo` matches
/// the response `State`.
fn verify_latest_li_matches_state(latest_li: &LedgerInfo, state: &State) -> Result<()> {
    if latest_li.version() != state.version {
        return Err(Error::invalid_proof(format!(
            "latest LedgerInfo version ({}) doesn't match response version ({})",
            latest_li.version(),
            state.version,
        )));
    }
    if latest_li.timestamp_usecs() != state.timestamp_usecs {
        return Err(Error::invalid_proof(format!(
            "latest LedgerInfo timestamp ({}) doesn't match response timestamp ({})",
            latest_li.timestamp_usecs(),
            state.timestamp_usecs,
        )));
    }
    Ok(())
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
        let num_subrequests = self.num_subrequests();
        if num_subrequests != responses.len() {
            return Err(Error::rpc_response(format!(
                "expected {} responses, received {} responses in batch",
                num_subrequests,
                responses.len()
            )));
        }

        for response in responses.iter().flatten() {
            if response.state() != state {
                return Err(Error::rpc_response(format!(
                    "expected all responses in batch to have the same metadata: {:?}, \
                         received unexpected response metadata: {:?}",
                    state,
                    response.state(),
                )));
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
        if subresponses.len() != self.subrequests.len() {
            return Err(Error::rpc_response(format!(
                "expected {} subresponses for our request {:?}, received {} subresponses in batch",
                self.subrequests.len(),
                self.request.method(),
                subresponses.len(),
            )));
        }

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
            MethodRequest::GetAccount(address, version) => verifying_get_account(address, version),
            MethodRequest::GetEvents(key, start_seq, limit) => {
                verifying_get_events(key, start_seq, limit)
            }
            MethodRequest::GetCurrencies([]) => verifying_get_currencies(),
            MethodRequest::GetNetworkStatus([]) => verifying_get_network_status(),
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

fn verifying_get_account(address: AccountAddress, version: Option<Version>) -> VerifyingRequest {
    let request = MethodRequest::GetAccount(address, version);
    let subrequests = vec![
        MethodRequest::GetAccountStateWithProof(diem_root_address(), version, None),
        MethodRequest::GetAccountStateWithProof(address, version, None),
    ];
    let callback: RequestCallback = |ctxt, subresponses| {
        match subresponses {
        [MethodResponse::GetAccountStateWithProof(ref diem_root), MethodResponse::GetAccountStateWithProof(ref account)] =>
        {
            let diem_root_with_proof =
                AccountStateWithProof::try_from(diem_root).map_err(Error::decode)?;
            let account_state_with_proof =
                AccountStateWithProof::try_from(account).map_err(Error::decode)?;

            let (address, version) = match ctxt.request {
                MethodRequest::GetAccount(address, version) => (*address, *version),
                request => panic!("programmer error: unexpected request: {:?}", request),
            };
            let latest_li = ctxt.state_proof.0.ledger_info();
            let ledger_version = latest_li.version();
            let version = version.unwrap_or(ledger_version);

            diem_root_with_proof
                .verify(latest_li, version, diem_root_address())
                .map_err(Error::invalid_proof)?;
            account_state_with_proof
                .verify(latest_li, version, address)
                .map_err(Error::invalid_proof)?;

            // TODO(philiphayes): it seems wasteful to lookup the whole diem_root
            // account, verify the proofs, etc... just to get a list of supported
            // currency codes. Would it make sense for the AccountView to just
            // list _all_ BalanceResource's under balance? Then we could avoid
            // this extra lookup. It's even more problematic when a batch
            // includes many GetAccount requests, as this batch handling logic
            // isn't (currently) smart enough to dedup the diem_root lookups.
            let diem_root_blob = diem_root_with_proof
                .blob
                .ok_or_else(|| Error::unknown("missing diem_root account"))?;
            let diem_root = AccountState::try_from(&diem_root_blob)
                .map_err(Error::decode)?;
            let currencies: Option<RegisteredCurrencies> = diem_root.get_config()
                .map_err(Error::decode)?;
            let currency_codes = currencies
                .as_ref()
                .map(RegisteredCurrencies::currency_codes)
                .ok_or_else(|| Error::unknown("diem_root has no registered currencies"))?;

            let maybe_account_view = account_state_with_proof
                .blob
                .map(|blob| {
                    let account_state = AccountState::try_from(&blob)
                        .map_err(Error::decode)?;
                    AccountView::try_from_account_state(
                        address,
                        account_state,
                        &currency_codes,
                        version,
                    ).map_err(Error::decode)
                })
                .transpose()?;

            Ok(MethodResponse::GetAccount(maybe_account_view))
        }
        subresponses => return Err(Error::rpc_response(format!(
                    "expected [GetAccountStateWithProof, GetAccountStateWithProof] subresponses, received: {:?}",
                    subresponses,
                    ))),
    }
    };
    VerifyingRequest::new(request, subrequests, callback)
}

fn verifying_get_events(key: EventKey, start_seq: u64, limit: u64) -> VerifyingRequest {
    let request = MethodRequest::GetEvents(key, start_seq, limit);
    let subrequests = vec![MethodRequest::GetEventsWithProofs(key, start_seq, limit)];

    let callback: RequestCallback = |ctxt, subresponses| {
        let event_with_proof_views = match subresponses {
            [MethodResponse::GetEventsWithProofs(ref inner)] => inner,
            subresponses => {
                return Err(Error::rpc_response(format!(
                    "expected [GetEventsWithProofs] subresponses, received: {:?}",
                    subresponses,
                )))
            }
        };

        let (key, start_seq, limit) = match ctxt.request {
            MethodRequest::GetEvents(key, start_seq, limit) => (key, *start_seq, *limit),
            request => panic!("programmer error: unexpected request: {:?}", request),
        };

        // Make sure we didn't get more than we requested. Note that remote can
        // always return a shorter prefix than is on-chain and we don't consider
        // that an invalid response.
        let num_received = event_with_proof_views.len() as u64;
        if num_received > limit {
            return Err(Error::rpc_response(format!(
                "more events than limit: limit {} events, received {} events",
                limit, num_received,
            )));
        }

        let latest_li = ctxt.state_proof.0.ledger_info();
        let event_views = event_with_proof_views
            .iter()
            .enumerate()
            .map(|(offset, event_with_proof_view)| {
                // Deserialize the diem-core type from the json-rpc view type.
                let event_with_proof =
                    EventWithProof::try_from(event_with_proof_view).map_err(Error::decode)?;

                // Actually verify the proof. Once verified, we should be guaranteed
                // that this event exists on-chain in the `key` event stream with
                // the given sequence number and transaction version.
                let txn_version = event_with_proof.transaction_version;
                event_with_proof
                    .verify(
                        latest_li,
                        key,
                        start_seq + offset as u64,
                        txn_version,
                        event_with_proof.event_index,
                    )
                    .map_err(Error::invalid_proof)?;

                // Project into the json-rpc type
                let event_view = EventView::try_from((txn_version, event_with_proof.event))
                    .map_err(Error::decode)?;

                Ok(event_view)
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(MethodResponse::GetEvents(event_views))
    };
    VerifyingRequest::new(request, subrequests, callback)
}

fn verifying_get_currencies() -> VerifyingRequest {
    let request = MethodRequest::GetCurrencies([]);
    let subrequests = vec![MethodRequest::GetAccountStateWithProof(
        diem_root_address(),
        None,
        None,
    )];
    let callback: RequestCallback = |ctxt, subresponses| {
        let diem_root = match subresponses {
            [MethodResponse::GetAccountStateWithProof(ref diem_root)] => diem_root,
            subresponses => {
                return Err(Error::rpc_response(format!(
                    "expected [GetAccountStateWithProof] subresponses, received: {:?}",
                    subresponses,
                )))
            }
        };

        let diem_root_with_proof =
            AccountStateWithProof::try_from(diem_root).map_err(Error::decode)?;

        let latest_li = ctxt.state_proof.0.ledger_info();
        let version = latest_li.version();

        diem_root_with_proof
            .verify(latest_li, version, diem_root_address())
            .map_err(Error::invalid_proof)?;

        // Deserialize the DiemRoot account state, pull out its currency infos,
        // and project them into json-rpc currency views.
        let diem_root_blob = diem_root_with_proof
            .blob
            .ok_or_else(|| Error::unknown("missing diem_root account"))?;
        let diem_root = AccountState::try_from(&diem_root_blob).map_err(Error::decode)?;
        let currency_infos = diem_root
            .get_registered_currency_info_resources()
            .map_err(Error::decode)?;
        let currency_views = currency_infos.iter().map(CurrencyInfoView::from).collect();

        Ok(MethodResponse::GetCurrencies(currency_views))
    };
    VerifyingRequest::new(request, subrequests, callback)
}

fn verifying_get_network_status() -> VerifyingRequest {
    let request = MethodRequest::get_network_status();
    let subrequests = vec![MethodRequest::get_network_status()];
    let callback: RequestCallback = |_ctxt, subresponses| {
        let status = match subresponses {
            [MethodResponse::GetNetworkStatus(ref status)] => *status,
            subresponses => {
                return Err(Error::rpc_response(format!(
                    "expected [GetNetworkStatus] subresponses, received: {:?}",
                    subresponses,
                )))
            }
        };
        Ok(MethodResponse::GetNetworkStatus(status))
    };
    VerifyingRequest::new(request, subrequests, callback)
}

mod private {
    pub trait Sealed {}

    impl Sealed for super::InMemoryStorage {}
}

// TODO(philiphayes): unseal `Storage` trait once verifying client stabilizes.
pub trait Storage: private::Sealed + Debug {
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
            .ok_or_else(|| Error::unknown("key not set"))
    }

    fn set(&mut self, key: &str, value: Vec<u8>) -> Result<()> {
        self.data.insert(key.to_owned(), value);
        Ok(())
    }
}

pub const TRUSTED_STATE_KEY: &str = "trusted_state";

#[derive(Debug)]
struct TrustedStateStore<S> {
    trusted_state: TrustedState,
    storage: S,
}

impl<S: Storage> TrustedStateStore<S> {
    fn new(storage: S) -> Result<Self> {
        let trusted_state = storage
            .get(TRUSTED_STATE_KEY)
            .and_then(|bytes| bcs::from_bytes(&bytes).map_err(Error::decode))?;

        Ok(Self {
            trusted_state,
            storage,
        })
    }

    fn new_with_state(trusted_state: TrustedState, storage: S) -> Self {
        let maybe_stored_state: Result<TrustedState> = storage
            .get(TRUSTED_STATE_KEY)
            .and_then(|bytes| bcs::from_bytes(&bytes).map_err(Error::decode));

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
            let trusted_state_bytes = bcs::to_bytes(&self.trusted_state).map_err(Error::decode)?;
            self.storage.set(TRUSTED_STATE_KEY, trusted_state_bytes)?;
        }

        Ok(())
    }
}
