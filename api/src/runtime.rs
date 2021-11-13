// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{context::Context, index};

use diem_config::config::{ApiConfig, JsonRpcConfig, NodeConfig};
use diem_mempool::MempoolClientSender;
use diem_types::{chain_id::ChainId, protocol_spec::DpnProto};
use futures::join;
use storage_interface::MoveDbReader;
use warp::{Filter, Reply};

use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::runtime::{Builder, Runtime};

/// Creates HTTP server (warp-based) serves for both REST and JSON-RPC API.
/// When api and json-rpc are configured with same port, both API will be served for the port.
/// When api and json-rpc are configured with different port, both API will be served for
/// both ports.
/// Returns corresponding Tokio runtime
pub fn bootstrap(
    config: &NodeConfig,
    chain_id: ChainId,
    db: Arc<dyn MoveDbReader<DpnProto>>,
    mp_sender: MempoolClientSender,
) -> anyhow::Result<Runtime> {
    let runtime = Builder::new_multi_thread()
        .thread_name("api")
        .enable_all()
        .build()
        .expect("[api] failed to create runtime");

    let role = config.base.role;
    let json_rpc_config = config.json_rpc.clone();
    let api_config = config.api.clone();
    let api = WebServer::from(api_config.clone());
    let jsonrpc = WebServer::from(json_rpc_config.clone());

    runtime.spawn(async move {
        let context = Context::new(chain_id, db, mp_sender, role, json_rpc_config, api_config);
        let routes = index::routes(context);
        if api.address.port() == jsonrpc.address.port() {
            // when we rollout api, it's likely there is no api configuration for diem
            // node, we will load default api configurations (127.0.0.1:8080).
            // if ip is unspecified(0.0.0.0), we assume it is properly configured and
            // prefer the api web server configuration over jsonrpc configuration.
            if api.address.ip() == jsonrpc.address.ip() || api.address.ip().is_unspecified() {
                api.serve(routes).await;
            } else {
                jsonrpc.serve(routes).await;
            }
        } else {
            join!(api.serve(routes.clone()), jsonrpc.serve(routes));
        }
    });
    Ok(runtime)
}

#[derive(Clone, Debug, PartialEq)]
struct WebServer {
    pub address: SocketAddr,
    pub tls_cert_path: Option<String>,
    pub tls_key_path: Option<String>,
}

impl From<ApiConfig> for WebServer {
    fn from(cfg: ApiConfig) -> Self {
        Self::new(cfg.address, cfg.tls_cert_path, cfg.tls_key_path)
    }
}

impl From<JsonRpcConfig> for WebServer {
    fn from(cfg: JsonRpcConfig) -> Self {
        Self::new(cfg.address, cfg.tls_cert_path, cfg.tls_key_path)
    }
}

impl WebServer {
    pub fn new(
        address: SocketAddr,
        tls_cert_path: Option<String>,
        tls_key_path: Option<String>,
    ) -> Self {
        Self {
            address,
            tls_cert_path,
            tls_key_path,
        }
    }

    pub async fn serve<F>(&self, routes: F)
    where
        F: Filter<Error = Infallible> + Clone + Sync + Send + 'static,
        F::Extract: Reply,
    {
        match &self.tls_cert_path {
            None => warp::serve(routes).bind(self.address).await,
            Some(cert_path) => {
                warp::serve(routes)
                    .tls()
                    .cert_path(cert_path)
                    .key_path(self.tls_key_path.as_ref().unwrap())
                    .bind(self.address)
                    .await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use diem_config::config::NodeConfig;
    use diem_types::chain_id::ChainId;
    use serde_json::json;

    use crate::{
        runtime::bootstrap,
        tests::{new_test_context, TestContext},
    };

    #[test]
    fn test_bootstrap_jsonprc_and_api_configured_same_address() {
        let mut cfg = NodeConfig::default();
        cfg.randomize_ports();
        // same port and ip
        cfg.api.address = cfg.json_rpc.address;
        bootstrap_with_config(cfg);
    }

    #[test]
    fn test_bootstrap_jsonprc_and_api_configured_at_different_port() {
        let mut cfg = NodeConfig::default();
        cfg.randomize_ports();
        bootstrap_with_config(cfg);
    }

    #[test]
    fn test_bootstrap_same_port_and_jsonprc_with_unspecified_ip() {
        let mut cfg = NodeConfig::default();
        cfg.randomize_ports();
        cfg.json_rpc.address = format!("0.0.0.0:{}", cfg.api.address.port())
            .parse()
            .unwrap();
        bootstrap_with_config(cfg);
    }

    #[test]
    fn test_bootstrap_same_port_and_api_with_unspecified_ip() {
        let mut cfg = NodeConfig::default();
        cfg.randomize_ports();
        cfg.api.address = format!("0.0.0.0:{}", cfg.json_rpc.address.port())
            .parse()
            .unwrap();

        bootstrap_with_config(cfg);
    }

    pub fn bootstrap_with_config(cfg: NodeConfig) {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let context = runtime.block_on(new_test_context_async());
        let ret = bootstrap(
            &cfg,
            ChainId::test(),
            context.db.clone(),
            context.mempool.ac_client.clone(),
        );
        assert!(ret.is_ok());

        assert_web_server(cfg.api.address.port());
        assert_web_server(cfg.json_rpc.address.port());
    }

    pub fn assert_web_server(port: u16) {
        let base_url = format!("http://localhost:{}", port);
        let client = reqwest::blocking::Client::new();
        // first call have retry to ensure the server is ready to serve
        let api_resp = with_retry(|| Ok(client.get(&base_url).send()?)).unwrap();
        assert_eq!(api_resp.status(), 200);
        let healthy_check_resp = client
            .get(format!("{}/-/healthy", base_url))
            .send()
            .unwrap();
        assert_eq!(healthy_check_resp.status(), 200);
        let jsonrpc_resp = client
            .post(&base_url)
            .json(&json!({"jsonrpc": "2.0", "method": "get_metadata", "id": 1}))
            .send()
            .unwrap();
        assert_eq!(jsonrpc_resp.status(), 200);
    }

    fn with_retry<F>(f: F) -> anyhow::Result<reqwest::blocking::Response>
    where
        F: Fn() -> anyhow::Result<reqwest::blocking::Response>,
    {
        let mut remaining_attempts = 60;
        loop {
            match f() {
                Ok(r) => return Ok(r),
                Err(_) if remaining_attempts > 0 => {
                    remaining_attempts -= 1;
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => return Err(error),
            }
        }
    }

    pub async fn new_test_context_async() -> TestContext {
        new_test_context()
    }
}
