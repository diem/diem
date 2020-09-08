// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_faucet::mint;
use libra_logger::prelude::info;
use std::fmt;
use structopt::StructOpt;
use warp::Filter;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "Libra Faucet",
    author = "The Libra Association",
    about = "Libra Testnet utitlty service for creating test account and minting test coins"
)]
struct Args {
    /// Faucet service listen address
    #[structopt(short = "a", long, default_value = "127.0.0.1")]
    pub address: String,
    /// Faucet service listen port
    #[structopt(short = "p", long, default_value = "80")]
    pub port: u16,
    /// Libra fullnode/validator server URL
    #[structopt(short = "s", long, default_value = "https://testnet.libra.org/v1")]
    pub server_url: String,
    /// Path to the private key for creating test account and minting coins.
    /// To keep Testnet simple, we used one private key for both treasury compliance account and testnet
    /// designated dealer account, hence here we only accept one private key.
    /// To manually generate a keypair, use generate-key:
    /// `cargo run -p generate-keypair -- -o <output_file_path>`
    #[structopt(short = "m", long, default_value = "/opt/libra/etc/mint.key")]
    pub mint_key_file_path: String,
    /// Chain ID of the network this client is connecting to.
    /// For mainnet: \"MAINNET\" or 1, testnet: \"TESTNET\" or 2, devnet: \"DEVNET\" or 3, \
    /// local swarm: \"TESTING\" or 4
    /// Note: Chain ID of 0 is not allowed; Use number if chain id is not predefined.
    #[structopt(short = "c", long, default_value = "2")]
    pub chain_id: libra_types::chain_id::ChainId,
}

#[tokio::main]
async fn main() {
    let args = Args::from_args();
    libra_logger::Logger::new().init();

    let address: std::net::SocketAddr = format!("{}:{}", args.address, args.port)
        .parse()
        .expect("invalid address or port number");

    info!(
        "[faucet]: chain id: {}, server url: {}",
        args.chain_id,
        args.server_url.as_str(),
    );
    let service = std::sync::Arc::new(mint::Service::new(
        args.server_url,
        args.chain_id,
        args.mint_key_file_path,
    ));

    info!("[faucet]: running on: {}", address);
    warp::serve(routes(service)).run(address).await;
}

fn routes(
    service: std::sync::Arc<mint::Service>,
) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {
    let mint = warp::any()
        .and(warp::post())
        .and(warp::any().map(move || std::sync::Arc::clone(&service)))
        .and(warp::query().map(move |params: mint::MintParams| params))
        .and_then(handle)
        .with(warp::log::custom(|info| {
            info!(
                "{} \"{} {} {:?}\" {} \"{}\" \"{}\" {:?}",
                OptFmt(info.remote_addr()),
                info.method(),
                info.path(),
                info.version(),
                info.status().as_u16(),
                OptFmt(info.referer()),
                OptFmt(info.user_agent()),
                info.elapsed(),
            )
        }));

    // POST /?amount=25&auth_key=xxx&currency_code=XXX
    let route_root = warp::path::end().and(mint.clone());
    // POST /mint?amount=25&auth_key=xxx&currency_code=XXX
    let route_mint = warp::path::path("mint").and(warp::path::end()).and(mint);

    let health = warp::path!("-" / "healthy").map(|| "libra-faucet:ok");
    health.or(route_mint.or(route_root)).boxed()
}

async fn handle(
    service: std::sync::Arc<mint::Service>,
    params: mint::MintParams,
) -> Result<Box<dyn warp::Reply>, warp::Rejection> {
    let ret = service.process(&params).await;
    match ret {
        Ok(body) => Ok(Box::new(body)),
        Err(err) => Err(warp::reject::custom(ServerInternalError(err.to_string()))),
    }
}

struct OptFmt<T>(Option<T>);

impl<T: fmt::Display> fmt::Display for OptFmt<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref t) = self.0 {
            fmt::Display::fmt(t, f)
        } else {
            f.write_str("-")
        }
    }
}

#[derive(Debug)]
struct ServerInternalError(String);
impl warp::reject::Reject for ServerInternalError {}

#[cfg(test)]
mod tests {
    use crate::routes;
    use libra_faucet::mint;
    use std::{collections::HashMap, sync::Arc};
    use warp::Filter;

    fn setup() -> (tokio::runtime::Runtime, Arc<mint::Service>) {
        setup_with_accounts(genesis_accounts())
    }

    fn setup_with_accounts(
        accounts: HashMap<String, serde_json::Value>,
    ) -> (tokio::runtime::Runtime, Arc<mint::Service>) {
        let f = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .to_path_buf();
        generate_key::generate_and_save_key(&f);

        let chain_id = libra_types::chain_id::ChainId::test();
        let rt = tokio::runtime::Runtime::new().unwrap();

        let accounts_arc = Arc::new(accounts);
        let stub = warp::any()
            .and(warp::body::json())
            .map(move |req: serde_json::Value| {
                let resp = handle_request(req, chain_id, Arc::clone(&accounts_arc));
                Ok(warp::reply::json(&resp))
            });
        let port = libra_config::utils::get_available_port();
        let server = rt.enter(move || warp::serve(stub).bind(([127, 0, 0, 1], port)));
        rt.handle().spawn(server);

        let service = mint::Service::new(
            format!("http://localhost:{}/v1", port),
            chain_id,
            f.to_str().unwrap().to_owned(),
        );
        (rt, Arc::new(service))
    }

    #[test]
    fn test_healthy() {
        let (mut rt, service) = setup();
        let filter = routes(service);
        let resp = rt.block_on(async {
            warp::test::request()
                .method("GET")
                .path("/-/healthy")
                .reply(&filter)
                .await
        });
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.body(), "libra-faucet:ok");
    }

    #[test]
    fn test_mint() {
        let (mut rt, service) = setup();
        let filter = routes(service);

        let auth_key = "459c77a38803bd53f3adee52703810e3a74fd7c46952c497e75afb0a7932586d";
        for path in vec!["/", "/mint"] {
            let resp = rt.block_on(async {
                warp::test::request()
                    .method("POST")
                    .path(
                        format!(
                            "{}?auth_key={}&amount=1000000&currency_code=LBR",
                            path, auth_key
                        )
                        .as_str(),
                    )
                    .reply(&filter)
                    .await
            });
            assert_eq!(resp.body(), "2389"); // 2388+1
        }
    }

    #[test]
    fn test_mint_invalid_auth_key() {
        let (mut rt, service) = setup();
        let filter = routes(service);

        let auth_key = "invalid-auth-key";
        let resp = rt.block_on(async {
            warp::test::request()
                .method("POST")
                .path(
                    format!(
                        "/mint?auth_key={}&amount=1000000&currency_code=LBR",
                        auth_key
                    )
                    .as_str(),
                )
                .reply(&filter)
                .await
        });
        assert_eq!(resp.body(), "Invalid query string");
    }

    #[test]
    fn test_mint_fullnode_error() {
        let (mut rt, service) = setup_with_accounts(HashMap::new());
        let filter = routes(service);

        let auth_key = "459c77a38803bd53f3adee52703810e3a74fd7c46952c497e75afb0a7932586d";
        let resp = rt.block_on(async {
            warp::test::request()
                .method("POST")
                .path(
                    format!(
                        "/mint?auth_key={}&amount=1000000&currency_code=LBR",
                        auth_key
                    )
                    .as_str(),
                )
                .reply(&filter)
                .await
        });
        assert_eq!(
            resp.body(),
            "Unhandled rejection: ServerInternalError(\"treasury compliance account not found\")"
        );
    }

    fn handle_request(
        req: serde_json::Value,
        chain_id: libra_types::chain_id::ChainId,
        accounts: Arc<HashMap<String, serde_json::Value>>,
    ) -> serde_json::Value {
        if let serde_json::Value::Array(reqs) = req {
            reqs.iter()
                .map(move |req| handle_request(req.clone(), chain_id, Arc::clone(&accounts)))
                .collect()
        } else {
            serde_json::json!({
                "id": req["id"],
                "jsonrpc": "2.0",
                "libra_chain_id": chain_id.id(),
                "libra_ledger_timestampusec": 1599587372,
                "libra_ledger_version": 2052770,
                "result":match req["method"].as_str() {
                    Some("submit") => {
                        let raw: &str = req["params"][0].as_str().unwrap();
                        let txn: libra_types::transaction::SignedTransaction =
                            lcs::from_bytes(&hex::decode(raw).unwrap()).unwrap();
                        assert_eq!(txn.chain_id(), chain_id);
                        None
                    }
                    Some("get_account") => {
                        let address: &str = req["params"][0].as_str().unwrap();
                        accounts.get(address)
                    }
                    _ => panic!("unexpected method"),
                }
            })
        }
    }

    fn genesis_accounts() -> HashMap<String, serde_json::Value> {
        let mut accounts: HashMap<String, serde_json::Value> = HashMap::new();
        accounts.insert("0000000000000000000000000b1e55ed".to_owned(), serde_json::json!({
            "authentication_key": "47aee3951cfcffe581112185e1699ea0e6f1565495c581dfac665239a414eafc",
            "balances": [],
            "delegated_key_rotation_capability": false,
            "delegated_withdrawal_capability": false,
            "is_frozen": false,
            "received_events_key": "00000000000000000000000000000000000000000b1e55ed",
            "role": {
                "type": "unknown"
            },
            "sent_events_key": "01000000000000000000000000000000000000000b1e55ed",
            "sequence_number": 0
        }));
        accounts.insert("000000000000000000000000000000dd".to_owned(), serde_json::json!({
            "authentication_key": "47aee3951cfcffe581112185e1699ea0e6f1565495c581dfac665239a414eafc",
            "balances": [
                {
                    "amount": 461168581,
                    "currency": "Coin1"
                },
                {
                    "amount": 461168581,
                    "currency": "Coin2"
                },
                {
                    "amount": 922135149,
                    "currency": "LBR"
                }
            ],
            "delegated_key_rotation_capability": false,
            "delegated_withdrawal_capability": false,
            "is_frozen": false,
            "received_events_key": "0100000000000000000000000000000000000000000000dd",
            "role": {
                "base_url": "",
                "compliance_key": "",
                "expiration_time": 1999587372,
                "human_name": "moneybags",
                "preburn_balances": [
                    {
                        "amount": 0,
                        "currency": "Coin1"
                    },
                    {
                        "amount": 0,
                        "currency": "Coin2"
                    }
                ],
                "compliance_key_rotation_events_key": "0200000000000000000000000000000000000000000000dd",
                "base_url_rotation_events_key": "0300000000000000000000000000000000000000000000dd",
                "received_mint_events_key": "0000000000000000000000000000000000000000000000dd",
                "type": "designated_dealer"
            },
            "sent_events_key": "0200000000000000000000000000000000000000000000dd",
            "sequence_number": 2388
        }));
        accounts
    }
}
