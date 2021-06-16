// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{AdminInfo, ChainInfo, Coffer, Factory, FullNode, HealthCheckError, Node, NodeId, PublicInfo, Result, Swarm, Validator, query_sequence_numbers};
use anyhow::{bail, format_err};
use debug_interface::NodeDebugClient;
use diem_client::{BlockingClient, Client as JsonRpcClient};
use diem_config::config::NodeConfig;
use diem_logger::*;
use diem_sdk::crypto::ed25519::{Ed25519PrivateKey, ED25519_PRIVATE_KEY_LENGTH, Ed25519PublicKey};
use diem_sdk::crypto::ValidCryptoMaterialStringExt;
use diem_sdk::move_types::account_address::AccountAddress;
use diem_sdk::transaction_builder::TransactionFactory;
use diem_sdk::types::{AccountKey, LocalAccount};
use diem_types::chain_id::{ChainId, NamedChain};
use diem_types::PeerId;
use k8s_openapi::api::core::v1::{Pod, Service, Node as k8sNode};
use kube::api::ListParams;
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::Client as K8sClient,
    Config,
};
use reqwest::Url;
use std::convert::TryFrom;
use std::process::Command;
use std::str::FromStr;
use std::time::Duration;
use tokio::runtime::Runtime;
use cluster_test::cluster_swarm::cluster_swarm_kube::{ClusterSwarmKube, KubeNode};
use diem_sdk::crypto::test_utils::KeyPair;

const HEALTH_CHECK_URL: &str = "http://127.0.0.1:8001";
const KUBECTL_BIN: &str = "/usr/local/bin/kubectl";
const JSON_RPC_PORT: u32 = 8080;
const VALIDATOR_LB: &str = "val";

struct K8sSwarm {
    validators: Vec<K8sNode>,
    fullnodes: Vec<K8sNode>,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    kube_client: K8sClient,
    pub chain_id: ChainId,
}

impl K8sSwarm {
    pub async fn new() -> Result<Self> {
        println!("hhhh get into here");


        Command::new(KUBECTL_BIN).arg("proxy").spawn()?;
        diem_retrier::retry_async(k8s_retry_strategy(), || {
            Box::pin(async move {
                debug!("Running local kube pod healthcheck on {}", HEALTH_CHECK_URL);
                reqwest::get(HEALTH_CHECK_URL).await?.text().await?;
                println!("Local kube pod healthcheck passed");
                Ok::<(), reqwest::Error>(())
            })
        })
            .await?;


        let config = Config::new(
            reqwest::Url::parse(HEALTH_CHECK_URL).expect("Failed to parse kubernetes endpoint url"),
        );
        let kube_client = K8sClient::try_from(config)?;
        let mut validators = vec![];
        let mut fullnodes = vec![];
        let services = list_services(kube_client.clone()).await?;
        let mut i = 0;
        for s in &services {
            if s.name.contains(VALIDATOR_LB) {
                let node = K8sNode {
                    peer_id: PeerId::random(),
                    node_id: i,
                    ip: s.host_ip.clone(),
                    port: JSON_RPC_PORT,
                    dns: s.name.clone()
                };
                println!("hhhhhh vnode peer_id = {:?}, node_id = {:?}, ip = {:?}, dns = {:?}", node.peer_id, node.node_id, node.ip, node.dns);
                validators.push(node);
                i += 1;
            }
        }

        println!("hhhhh done here");
        let client = validators[0].json_rpc_client();
        let key = load_root_key();
        let account_key = AccountKey::from_private_key(key);
        let address = diem_sdk::types::account_config::diem_root_address();
        let sequence_number = query_sequence_numbers(&client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        let root_account = LocalAccount::new(
            address,
            account_key,
            sequence_number,
        );

        println!("hhhhh root seq = {}", root_account.sequence_number());
        let key = load_tc_key();
        let account_key = AccountKey::from_private_key(key);
        let address = diem_sdk::types::account_config::treasury_compliance_account_address();
        let sequence_number = query_sequence_numbers(&client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        let treasury_compliance_account = LocalAccount::new(
            address,
            account_key,
            sequence_number,
        );
        println!("hhhhh tc seq = {}", treasury_compliance_account.sequence_number());
        let key = load_tc_key();
        let account_key = AccountKey::from_private_key(key);
        let address = diem_sdk::types::account_config::testnet_dd_account_address();
        let sequence_number = query_sequence_numbers(&client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        let designated_dealer_account = LocalAccount::new(
            address,
            account_key,
            sequence_number,
        );
        println!("hhhhh dd seq = {}", designated_dealer_account.sequence_number());

        info!("vnode = {}, fnode = {}", validators.len(), fullnodes.len());
        Ok(Self {
            validators,
            fullnodes,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            kube_client,
            chain_id: ChainId::new(NamedChain::DEVNET.id()),
        })
    }

    fn get_url(&self) -> String {
        self.validator(NodeId::new(0))
            .json_rpc_endpoint()
            .to_string()
    }
}

impl Swarm for K8sSwarm {
    fn health_check(&mut self) -> Result<()> {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            Command::new(KUBECTL_BIN).arg("proxy").spawn()?;
            diem_retrier::retry_async(k8s_retry_strategy(), || {
                Box::pin(async move {
                    debug!("Running local kube pod healthcheck on {}", HEALTH_CHECK_URL);
                    let _res = reqwest::get(HEALTH_CHECK_URL).await.unwrap().text().await;
                    info!("Local kube pod healthcheck passed");
                    Ok(())
                })
            })
            .await
        })
    }

    fn validators<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn Validator> + 'a> {
        Box::new(self.validators.iter().map(|v| v as &'a dyn Validator))
    }

    fn validators_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a> {
        Box::new(
            self.validators
                .iter_mut()
                .map(|v| v as &'a mut dyn Validator),
        )
    }

    fn validator(&self, id: NodeId) -> &dyn Validator {
        let idx = id.as_inner();
        self.validators.iter().find(|n| n.node_id == idx).unwrap()
    }

    fn validator_mut(&mut self, id: NodeId) -> &mut dyn Validator {
        let idx = id.as_inner();
        self.validators
            .iter_mut()
            .find(|n| n.node_id == idx)
            .unwrap()
    }

    fn full_nodes<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn FullNode> + 'a> {
        Box::new(self.fullnodes.iter().map(|v| v as &'a dyn FullNode))
    }

    fn full_nodes_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn FullNode> + 'a> {
        Box::new(self.fullnodes.iter_mut().map(|v| v as &'a mut dyn FullNode))
    }

    fn full_node(&self, id: NodeId) -> &dyn FullNode {
        let idx = id.as_inner();
        self.fullnodes.iter().find(|n| n.node_id == idx).unwrap()
    }

    fn full_node_mut(&mut self, id: NodeId) -> &mut dyn FullNode {
        let idx = id.as_inner();
        self.fullnodes
            .iter_mut()
            .find(|n| n.node_id == idx)
            .unwrap()
    }

    fn add_validator(&mut self, id: NodeId) -> Result<NodeId> {
        todo!()
    }

    fn remove_validator(&mut self, id: NodeId) -> Result<()> {
        todo!()
    }

    fn add_full_node(&mut self, id: NodeId) -> Result<()> {
        todo!()
    }

    fn remove_full_node(&mut self, id: NodeId) -> Result<()> {
        todo!()
    }

    fn admin_info(&mut self) -> AdminInfo<'_> {
        let url = self.get_url();
        AdminInfo::new(
            &mut self.root_account,
            &mut self.treasury_compliance_account,
            url,
            self.chain_id,
        )
    }

    fn public_info(&mut self) -> PublicInfo<'_> {
        let url = self.get_url();
        PublicInfo::new(
            url.clone(),
            self.chain_id,
            Coffer::TreasuryCompliance {
                transaction_factory: TransactionFactory::new(self.chain_id),
                json_rpc_client: BlockingClient::new(&url),
                treasury_compliance_account: &mut self.treasury_compliance_account,
                designated_dealer_account: &mut self.designated_dealer_account,
            },
        )
    }

    fn chain_info(&mut self) -> ChainInfo<'_> {
        let url = self.get_url();
        ChainInfo::new(
            &mut self.root_account,
            &mut self.treasury_compliance_account,
            &mut self.designated_dealer_account,
            url,
            self.chain_id,
        )
    }
}

#[derive(Clone)]
struct K8sNode {
    peer_id: PeerId,
    node_id: usize,
    dns: String,
    ip: String,
    port: u32,
}

impl K8sNode {
    fn port(&self) -> u32 {
        self.port
    }

    fn dns(&self) -> String {
        self.dns.clone()
    }

    fn ip(&self) -> String {
        self.ip.clone()
    }
}

impl Node for K8sNode {
    fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(self.node_id)
    }

    fn json_rpc_endpoint(&self) -> Url {
        Url::from_str(&format!("http://{}:{}/v1", self.ip(), self.port())).expect("Invalid URL.")
    }

    fn json_rpc_client(&self) -> JsonRpcClient {
        JsonRpcClient::new(self.json_rpc_endpoint().to_string())
    }

    fn debug_client(&self) -> NodeDebugClient {
        NodeDebugClient::new(
            self.ip.clone(),
            NodeConfig::default()
                .debug_interface
                .admission_control_node_debug_port,
        )
    }

    fn get_metric(&mut self, metric_name: &str) -> Option<i64> {
        match self.debug_client().get_node_metric(metric_name) {
            Err(e) => {
                println!(
                    "error getting {} for node: {}; error: {}",
                    metric_name, self.node_id, e
                );
                None
            }
            Ok(maybeval) => {
                if maybeval.is_none() {
                    println!("Node: {} did not report {}", self.node_id, metric_name);
                }
                maybeval
            }
        }
    }

    fn config(&self) -> &NodeConfig {
        todo!()
    }

    fn start(&mut self) -> Result<()> {
        todo!()
    }

    fn stop(&mut self) -> Result<()> {
        todo!()
    }

    fn clear_storage(&mut self) -> Result<()> {
        todo!()
    }

    fn health_check(&mut self) -> Result<(), HealthCheckError> {
        let rt = Runtime::new().unwrap();
        let results = rt
            .block_on(self.json_rpc_client().batch(Vec::new()))
            .unwrap();
        if results.iter().all(Result::is_ok) {
            return Err(HealthCheckError::RpcFailure(format_err!("")));
        }

        Ok(())
    }
}

impl Validator for K8sNode {}

impl FullNode for K8sNode {}

pub struct K8sFactory {}

impl K8sFactory {
    pub fn new() -> Self {
        Self {}
    }
}

impl Factory for K8sFactory {
    fn launch_swarm(&self, _node_num: usize) -> Box<dyn Swarm> {
        let rt = Runtime::new().unwrap();
        let swarm = rt.block_on(K8sSwarm::new()).unwrap();
        Box::new(swarm)
    }
}

fn k8s_retry_strategy() -> impl Iterator<Item = Duration> {
    diem_retrier::exp_retry_strategy(1000, 5000, 30)
}

#[derive(Clone, Debug)]
pub struct KubeService {
    pub name: String,
    pub host_ip: String,
}

impl TryFrom<Service> for KubeService {
    type Error = anyhow::Error;

    fn try_from(service: Service) -> Result<Self, Self::Error> {
        let metadata = service.metadata;
        let name = metadata
            .name
            .ok_or_else(|| format_err!("node name not found"))?;
        let spec = service
            .spec
            .ok_or_else(|| format_err!("spec not found for node"))?;
        let host_ip = spec.cluster_ip.unwrap_or_default();
        Ok(Self {
            name,
            host_ip,
        })
    }
}

pub async fn list_services(client: K8sClient) -> Result<Vec<KubeService>> {
    println!("hhhh get in list pods");
    let node_api: Api<Service> = Api::all(client);
    let lp = ListParams::default();
    let services = node_api.list(&lp).await?.items;
    services.into_iter().map(KubeService::try_from).collect()
}

async fn list_nodes(client: K8sClient) -> Result<Vec<KubeNode>> {
    let node_api: Api<k8sNode> = Api::all(client);
    let lp = ListParams::default().labels("nodeType=validators");
    let nodes = node_api.list(&lp).await?.items;
    nodes.into_iter().map(KubeNode::try_from).collect()
}

fn parse_node_id(s: &str) -> Result<(usize)> {
    let v = s.split('-').collect::<Vec<&str>>();
    if v.len() < 5 {
        return Err(format_err!("Failed to parse {:?} node id format", s));
    }
    let idx: usize = v[0][3..].parse().unwrap();
    Ok((idx))
}

// TODO remove hard code key
fn load_root_key() -> Ed25519PrivateKey {
    get_mint_key_pair_from_file("/tmp/mint.key").private_key
}

// TODO remove hard code key
fn load_tc_key() -> Ed25519PrivateKey {
    get_mint_key_pair_from_file("/tmp/mint.key").private_key
}

fn get_mint_key_pair_from_file(
    mint_file: &str,
) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let mint_key: Ed25519PrivateKey = generate_key::load_key(mint_file);
    KeyPair::from(mint_key)
}
