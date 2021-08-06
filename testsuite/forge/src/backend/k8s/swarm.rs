// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backend::k8s::node::K8sNode, k8s_client, query_sequence_numbers, remove_validator,
    set_validator_image_tag, ChainInfo, FullNode, Node, Result, Swarm, Validator, Version,
};
use anyhow::{anyhow, bail, format_err};
use diem_logger::*;
use diem_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    types::{
        chain_id::{ChainId, NamedChain},
        AccountKey, LocalAccount, PeerId,
    },
};
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::{Api, ListParams},
    client::Client as K8sClient,
};
use std::{collections::HashMap, convert::TryFrom, str, sync::Arc};
use tokio::{runtime::Runtime, time::Duration};

const JSON_RPC_PORT: u32 = 80;
const VALIDATOR_LB: &str = "validator-fullnode-lb";

pub struct K8sSwarm {
    validators: HashMap<PeerId, K8sNode>,
    fullnodes: HashMap<PeerId, K8sNode>,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    kube_client: K8sClient,
    helm_repo: String,
    versions: Arc<HashMap<Version, String>>,
    pub chain_id: ChainId,
}

impl K8sSwarm {
    pub async fn new(
        root_key: &[u8],
        treasury_compliance_key: &[u8],
        helm_repo: &str,
        image_tag: &str,
        base_image_tag: &str,
    ) -> Result<Self> {
        let kube_client = k8s_client().await;
        let fullnodes = HashMap::new();
        let validators = get_validators(kube_client.clone(), image_tag).await?;

        let client = validators.values().next().unwrap().json_rpc_client();
        let key = load_root_key(root_key);
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
        let root_account = LocalAccount::new(address, account_key, sequence_number);

        let key = load_tc_key(treasury_compliance_key);
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
        let treasury_compliance_account = LocalAccount::new(address, account_key, sequence_number);

        let key = load_tc_key(treasury_compliance_key);
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
        let designated_dealer_account = LocalAccount::new(address, account_key, sequence_number);

        let mut versions = HashMap::new();
        let base_version = Version::new(0, base_image_tag.to_string());
        let cur_version = Version::new(1, image_tag.to_string());
        versions.insert(cur_version, image_tag.to_string());
        versions.insert(base_version, base_image_tag.to_string());

        Ok(Self {
            validators,
            fullnodes,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            kube_client,
            chain_id: ChainId::new(NamedChain::DEVNET.id()),
            helm_repo: helm_repo.to_string(),
            versions: Arc::new(versions),
        })
    }

    fn get_url(&self) -> String {
        self.validators
            .values()
            .next()
            .unwrap()
            .json_rpc_endpoint()
            .to_string()
    }

    #[allow(dead_code)]
    fn get_kube_client(&self) -> K8sClient {
        self.kube_client.clone()
    }
}

impl Swarm for K8sSwarm {
    fn health_check(&mut self) -> Result<()> {
        nodes_healthcheck(Box::new(
            self.validators
                .values_mut()
                .map(|v| v as &mut dyn Validator),
        ))
    }

    fn validators<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn Validator> + 'a> {
        Box::new(self.validators.values().map(|v| v as &'a dyn Validator))
    }

    fn validators_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a> {
        Box::new(
            self.validators
                .values_mut()
                .map(|v| v as &'a mut dyn Validator),
        )
    }

    fn validator(&self, id: PeerId) -> Option<&dyn Validator> {
        self.validators.get(&id).map(|v| v as &dyn Validator)
    }

    fn validator_mut(&mut self, id: PeerId) -> Option<&mut dyn Validator> {
        self.validators
            .get_mut(&id)
            .map(|v| v as &mut dyn Validator)
    }

    fn upgrade_validator(&mut self, id: PeerId, version: &Version) -> Result<()> {
        let validator = self
            .validators
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Invalid id: {}", id))?;
        let version = self
            .versions
            .get(version)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid version: {:?}", version))?;
        set_validator_image_tag(validator.name(), &version, &self.helm_repo)
    }

    fn full_nodes<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn FullNode> + 'a> {
        Box::new(self.fullnodes.values().map(|v| v as &'a dyn FullNode))
    }

    fn full_nodes_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn FullNode> + 'a> {
        Box::new(
            self.fullnodes
                .values_mut()
                .map(|v| v as &'a mut dyn FullNode),
        )
    }

    fn full_node(&self, id: PeerId) -> Option<&dyn FullNode> {
        self.fullnodes.get(&id).map(|v| v as &dyn FullNode)
    }

    fn full_node_mut(&mut self, id: PeerId) -> Option<&mut dyn FullNode> {
        self.fullnodes.get_mut(&id).map(|v| v as &mut dyn FullNode)
    }

    fn add_validator(&mut self, _id: PeerId) -> Result<PeerId> {
        todo!()
    }

    fn remove_validator(&mut self, id: PeerId) -> Result<()> {
        remove_validator(self.validator(id).unwrap().name())
    }

    fn add_full_node(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn remove_full_node(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a> {
        Box::new(self.versions.keys().cloned())
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

    fn logs_location(&mut self) -> String {
        todo!()
    }
}

pub(crate) fn k8s_retry_strategy() -> impl Iterator<Item = Duration> {
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
        Ok(Self { name, host_ip })
    }
}

async fn list_services(client: K8sClient) -> Result<Vec<KubeService>> {
    let node_api: Api<Service> = Api::all(client);
    let lp = ListParams::default();
    let services = node_api.list(&lp).await?.items;
    services.into_iter().map(KubeService::try_from).collect()
}

pub(crate) async fn get_validators(
    client: K8sClient,
    image_tag: &str,
) -> Result<HashMap<PeerId, K8sNode>> {
    let services = list_services(client).await?;
    services
        .into_iter()
        .filter(|s| s.name.contains(VALIDATOR_LB))
        .map(|s| {
            let node_id = parse_node_id(&s.name).expect("error to parse node id");
            let node = K8sNode {
                name: format!("val{}", node_id),
                // TODO: fetch this from running node
                peer_id: PeerId::random(),
                node_id,
                ip: s.host_ip.clone(),
                port: JSON_RPC_PORT,
                dns: s.name,
                version: Version::new(0, image_tag.to_string()),
                runtime: Runtime::new().unwrap(),
            };
            Ok((node.peer_id(), node))
        })
        .collect::<Result<HashMap<_, _>>>()
}

fn parse_node_id(s: &str) -> Result<usize> {
    let v = s.split('-').collect::<Vec<&str>>();
    if v.len() < 5 {
        return Err(format_err!("Failed to parse {:?} node id format", s));
    }
    let idx: usize = v[0][3..].parse().unwrap();
    Ok(idx)
}

fn load_root_key(root_key_bytes: &[u8]) -> Ed25519PrivateKey {
    Ed25519PrivateKey::try_from(root_key_bytes).unwrap()
}

fn load_tc_key(tc_key_bytes: &[u8]) -> Ed25519PrivateKey {
    Ed25519PrivateKey::try_from(tc_key_bytes).unwrap()
}

pub fn nodes_healthcheck<'a>(
    nodes: Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a>,
) -> Result<()> {
    let unhealthy_nodes = nodes
        .filter_map(|node| {
            let node_name = node.name().to_string();
            println!("Attempting health check: {}", node_name);
            // perform healthcheck with retry, returning unhealthy
            let check = diem_retrier::retry(k8s_retry_strategy(), || match node.health_check() {
                Ok(_) => {
                    println!("Node {} healthy", node_name);
                    Ok(())
                }
                Err(ref x) => {
                    debug!("Node {} unhealthy: {}", node_name, x);
                    Err(())
                }
            });
            if check.is_err() {
                return Some(node_name);
            }
            None
        })
        .collect::<Vec<_>>();
    if !unhealthy_nodes.is_empty() {
        bail!("Unhealthy validators after cleanup: {:?}", unhealthy_nodes);
    }
    println!("All validators healthy after cleanup!");
    Ok(())
}
