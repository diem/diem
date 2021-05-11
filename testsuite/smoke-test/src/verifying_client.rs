// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    smoke_test_environment::SmokeTestEnvironment,
    test_utils::{diem_swarm_utils::get_json_rpc_url, setup_swarm_and_client_proxy},
};
use cli::client_proxy::ClientProxy;
use diem_client::{
    Client, InMemoryStorage, MethodRequest, MethodResponse, Response, Result, VerifyingClient,
};
use diem_types::{
    account_address::AccountAddress,
    account_config::constants::addresses::{
        diem_root_address, testnet_dd_account_address, treasury_compliance_account_address,
        validator_set_address,
    },
    trusted_state::TrustedState,
};
use proptest::{collection::vec, prelude::*, sample::select};
use tokio::runtime::Builder;

struct Environment {
    _env: SmokeTestEnvironment,
    pub max_batch_size: usize,
    pub client_proxy: ClientProxy,
    pub client: Client,
    pub verifying_client: VerifyingClient<InMemoryStorage>,
}

impl Environment {
    fn new() -> Self {
        let (env, client_proxy) = setup_swarm_and_client_proxy(1, 0);

        let max_batch_size = env
            .validator_swarm
            .get_node(0)
            .unwrap()
            .config()
            .json_rpc
            .batch_size_limit as usize;

        let url = get_json_rpc_url(&env.validator_swarm, 0);
        let client = Client::new(url);

        let genesis_waypoint = env.validator_swarm.config.waypoint;
        let trusted_state = TrustedState::from(genesis_waypoint);
        let storage = InMemoryStorage::new();
        let verifying_client =
            VerifyingClient::new_with_state(client.clone(), trusted_state, storage);

        Self {
            _env: env,
            max_batch_size,
            client_proxy,
            client,
            verifying_client,
        }
    }

    fn fund_new_account(&mut self) -> AccountAddress {
        let account = self.client_proxy.create_next_account(false).unwrap();
        let idx = format!("{}", account.index);
        self.client_proxy
            .mint_coins(&["mintb", &idx, "1000", "XUS"], true)
            .unwrap();
        account.address
    }

    /// Send a request using both verifying and non-verifying clients.
    async fn request(
        &self,
        request: MethodRequest,
    ) -> (
        Result<Response<MethodResponse>>,
        Result<Response<MethodResponse>>,
    ) {
        let send_nv = self.client.request(request.clone());
        let send_v = self.verifying_client.request(request);

        tokio::join!(send_nv, send_v)
    }

    /// Send a request batch using both verifying and non-verifying clients.
    async fn batch(
        &self,
        batch: Vec<MethodRequest>,
    ) -> (
        Result<Vec<Result<Response<MethodResponse>>>>,
        Result<Vec<Result<Response<MethodResponse>>>>,
    ) {
        let send_nv = self.client.batch(batch.clone());
        let send_v = self.verifying_client.batch(batch);

        tokio::join!(send_nv, send_v)
    }
}

/// Running a request against a `VerifyingClient` and non-verifying `Client`
/// at the same state version should always return the same response.
fn assert_responses_equal(
    recv_nv: Result<Response<MethodResponse>>,
    recv_v: Result<Response<MethodResponse>>,
) {
    match (recv_nv, recv_v) {
        (Ok(resp_nv), Ok(resp_v)) => {
            let (inner_nv, state_nv) = resp_nv.into_parts();
            let (inner_v, state_v) = resp_v.into_parts();

            assert_eq!(state_nv.chain_id, state_v.chain_id);
            assert_eq!(inner_nv, inner_v);
        }
        (Err(_), Err(_)) => (),
        (recv_nv, recv_v) => {
            panic!(
                "client handling mismatch:\n\
                 1. non-verifying client response: {:?}\n\
                 2.     verifying client response: {:?}",
                recv_nv, recv_v,
            );
        }
    }
}

/// Running a batch of requests against a `VerifyingClient` and non-verifying
/// `Client` at the same state version should always return the same responses.
fn assert_batches_equal(
    recv_nv: Result<Vec<Result<Response<MethodResponse>>>>,
    recv_v: Result<Vec<Result<Response<MethodResponse>>>>,
) {
    let (batch_nv, batch_v) = match (recv_nv, recv_v) {
        (Ok(batch_nv), Ok(batch_v)) => (batch_nv, batch_v),
        (Err(_), Err(_)) => return,
        (recv_nv, recv_v) => {
            panic!(
                "client batch handling mismatch:\n\
                 1. non-verifying client batch response: {:?}\n\
                 2.     verifying client batch response: {:?}",
                recv_nv, recv_v,
            );
        }
    };

    if batch_nv.len() != batch_v.len() {
        panic!(
            "clients returned different batch sizes: {} vs {}\n\
             1. non-verifying client batch: {:?}\n\
             2.     verifying client batch: {:?}",
            batch_nv.len(),
            batch_v.len(),
            batch_nv,
            batch_v,
        );
    }

    for (resp_nv, resp_v) in batch_nv.into_iter().zip(batch_v.into_iter()) {
        assert_responses_equal(resp_nv, resp_v);
    }
}

/// A `Strategy` for generating a `MethodRequest` at or before the given version
/// and querying from the given set of accounts.
fn arb_request(
    accounts: &[AccountAddress],
    current_state_version: u64,
) -> impl Strategy<Value = MethodRequest> {
    let arb_account = select(accounts.to_owned());
    let arb_version = prop_oneof! [
        5 => Just(current_state_version),
        // query a historical version
        20 => 0u64..current_state_version,
        // occasionally choose an invalid version
        1 => Just(u64::MAX / 2),
    ];
    (arb_account, arb_version)
        .prop_map(|(address, version)| MethodRequest::get_account_by_version(address, version))
}

/// A `Strategy` for generating a batch of `MethodRequest`s with the added restriction
/// that it will still be accepted by the JSON-RPC server after going through the
/// VerifyingClient request transformation.
fn arb_batch(
    max_batch_size: usize,
    accounts: &[AccountAddress],
    current_state_version: u64,
) -> impl Strategy<Value = Vec<MethodRequest>> {
    vec(
        arb_request(accounts, current_state_version),
        1..max_batch_size,
    )
    .prop_filter(
        "batch rejected: actual size too large; won't be accepted by the JSON-RPC server",
        move |batch| {
            let actual_batch_size =
                VerifyingClient::<InMemoryStorage>::actual_batch_size(batch.as_slice());
            actual_batch_size <= max_batch_size
        },
    )
}

#[test]
fn test_client_equivalence() {
    let rt = Builder::new_current_thread().enable_all().build().unwrap();
    let mut env = Environment::new();

    // Accounts we can query from
    let accounts = vec![
        // Some standard diem accounts
        diem_root_address(),
        validator_set_address(),
        treasury_compliance_account_address(),
        testnet_dd_account_address(),
        // Fund some new accounts
        env.fund_new_account(),
        env.fund_new_account(),
        env.fund_new_account(),
        // Some random, likely non-existent accounts
        AccountAddress::ZERO,
        AccountAddress::random(),
        AccountAddress::random(),
    ];

    // Sync the verifying client
    rt.block_on(env.verifying_client.sync()).unwrap();
    let current_version = env.verifying_client.version();

    // Generate random requests and ensure that both verifying and non-verifying
    // clients handle each request identically.
    proptest!(|(request in arb_request(&accounts, current_version))| {
        let (recv_nv, recv_v) = rt.block_on(env.request(request));
        assert_responses_equal(recv_nv, recv_v);
    });

    // Do the same but with random request batches instead of single requests.
    proptest!(|(batch in arb_batch(env.max_batch_size, &accounts, current_version))| {
        let (recv_nv, recv_v) = rt.block_on(env.batch(batch));
        assert_batches_equal(recv_nv, recv_v);
    });
}
