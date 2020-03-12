// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, format_err, Result};
use async_trait::async_trait;

use futures::{future::try_join_all, lock::Mutex};
use kube::{
    api::{Api, PostParams},
    client::APIClient,
    config,
};
use libra_logger::*;
use util::retry;

use crate::{cluster_swarm::ClusterSwarm, instance::Instance};
use kube::api::ListParams;
use libra_config::config::DEFAULT_JSON_RPC_PORT;

const DEFAULT_NAMESPACE: &str = "default";

const CFG_SEED: &str = "1337133713371337133713371337133713371337133713371337133713371337";
const CFG_FULLNODE_SEED: &str = "2674267426742674267426742674267426742674267426742674267426742674";

const ERROR_NOT_FOUND: u16 = 404;

pub struct ClusterSwarmKube {
    client: APIClient,
    validator_to_node: Arc<Mutex<HashMap<u32, Instance>>>,
    fullnode_to_node: Arc<Mutex<HashMap<(u32, u32), Instance>>>,
}

impl ClusterSwarmKube {
    pub async fn new() -> Result<Self> {
        let mut config = config::load_kube_config().await;
        if config.is_err() {
            config = config::incluster_config();
        }
        let config = config.map_err(|e| format_err!("Failed to load config: {:?}", e))?;
        let client = APIClient::new(config);
        let validator_to_node = Arc::new(Mutex::new(HashMap::new()));
        let fullnode_to_node = Arc::new(Mutex::new(HashMap::new()));
        Ok(Self {
            client,
            validator_to_node,
            fullnode_to_node,
        })
    }

    fn validator_spec(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        node_name: &str,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<Vec<u8>> {
        let cfg_fullnode_seed = if num_fullnodes > 0 {
            CFG_FULLNODE_SEED
        } else {
            ""
        };
        let fluentbit_enabled = if index % 10 == 0 { "true" } else { "false" };
        let pod_yaml = format!(
            include_str!("validator_spec_template.yaml"),
            index = index,
            num_validators = num_validators,
            num_fullnodes = num_fullnodes,
            image_tag = image_tag,
            node_name = node_name,
            cfg_overrides = "",
            delete_data = delete_data,
            cfg_seed = CFG_SEED,
            cfg_fullnode_seed = cfg_fullnode_seed,
            fluentbit_enabled = fluentbit_enabled,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        serde_json::to_vec(&pod_spec).map_err(|e| format_err!("serde_json::to_vec failed: {}", e))
    }

    fn fullnode_spec(
        &self,
        fullnode_index: u32,
        num_fullnodes: u32,
        validator_index: u32,
        num_validators: u32,
        node_name: &str,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<Vec<u8>> {
        let fluentbit_enabled = if validator_index % 10 == 0 {
            "true"
        } else {
            "false"
        };
        let pod_yaml = format!(
            include_str!("fullnode_spec_template.yaml"),
            fullnode_index = fullnode_index,
            num_fullnodes = num_fullnodes,
            validator_index = validator_index,
            num_validators = num_validators,
            node_name = node_name,
            image_tag = image_tag,
            cfg_overrides = "",
            delete_data = delete_data,
            cfg_seed = CFG_SEED,
            cfg_fullnode_seed = CFG_FULLNODE_SEED,
            fluentbit_enabled = fluentbit_enabled,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        serde_json::to_vec(&pod_spec).map_err(|e| format_err!("serde_json::to_vec failed: {}", e))
    }

    async fn get_pod_node_and_ip(&self, pod_name: &str) -> Result<(String, String)> {
        retry::retry_async(retry::fixed_retry_strategy(10000, 60), || {
            let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
            let pod_name = pod_name.to_string();
            Box::pin(async move {
                match pod_api.get(&pod_name).await {
                    Ok(o) => {
                        let node_name = o.spec.node_name.ok_or_else(|| {
                            format_err!("node_name not found for pod {}", pod_name)
                        })?;
                        let pod_ip = o
                            .status
                            .ok_or_else(|| format_err!("status not found for pod {}", pod_name))?
                            .pod_ip
                            .ok_or_else(|| format_err!("pod_ip not found for pod {}", pod_name))?;
                        if node_name.is_empty() || pod_ip.is_empty() {
                            bail!(
                                "Either node_name or pod_ip was empty string for pod {}",
                                pod_name
                            )
                        } else {
                            Ok((node_name, pod_ip))
                        }
                    }
                    Err(e) => bail!("pod_api.get failed for pod {} : {:?}", pod_name, e),
                }
            })
        })
        .await
    }

    async fn delete_pod(&self, name: &str) -> Result<()> {
        debug!("Deleting Pod {}", name);
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        pod_api.delete(name, &Default::default()).await?;
        retry::retry_async(retry::fixed_retry_strategy(5000, 30), || {
            let pod_api = pod_api.clone();
            let name = name.to_string();
            Box::pin(async move {
                match pod_api.get(&name).await {
                    Ok(_) => {
                        bail!("Waiting for pod {} to be deleted..", name);
                    }
                    Err(kube::Error::Api(ae)) => {
                        if ae.code == ERROR_NOT_FOUND {
                            debug!("Pod {} deleted successfully", name);
                            Ok(())
                        } else {
                            bail!("Waiting for pod to be deleted..")
                        }
                    }
                    Err(_) => bail!("Waiting for pod {} to be deleted..", name),
                }
            })
        })
        .await
        .map_err(|e| format_err!("Failed to delete pod {}: {:?}", name, e))
    }
}

#[async_trait]
impl ClusterSwarm for ClusterSwarmKube {
    async fn validator_instances(&self) -> Vec<Instance> {
        self.validator_to_node
            .lock()
            .await
            .values()
            .cloned()
            .collect()
    }

    async fn fullnode_instances(&self) -> Vec<Instance> {
        self.fullnode_to_node
            .lock()
            .await
            .values()
            .cloned()
            .collect()
    }

    async fn upsert_validator(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()> {
        let pod_name = format!("val-{}", index);
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            self.delete_pod(&pod_name).await?;
        }
        let node_name = if let Some(instance) = self.validator_to_node.lock().await.get(&index) {
            if let Some(k8s_node) = instance.k8s_node() {
                k8s_node.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        debug!("Creating pod {} on node {:?}", pod_name, node_name);
        match pod_api
            .create(
                &PostParams::default(),
                self.validator_spec(
                    index,
                    num_validators,
                    num_fullnodes,
                    &node_name,
                    image_tag,
                    delete_data,
                )?,
            )
            .await
        {
            Ok(o) => {
                debug!("Created {}", o.metadata.name);
            }
            Err(e) => bail!("Failed to create pod {} : {}", pod_name, e),
        }
        if node_name.is_empty() {
            let (node_name, pod_ip) = self.get_pod_node_and_ip(&pod_name).await?;
            let ac_port = DEFAULT_JSON_RPC_PORT as u32;
            let instance = Instance::new_k8s(pod_name, pod_ip, ac_port, Some(node_name));
            self.validator_to_node.lock().await.insert(index, instance);
        }
        Ok(())
    }

    async fn delete_validator(&self, index: u32) -> Result<()> {
        let pod_name = format!("val-{}", index);
        self.delete_pod(&pod_name).await
    }

    async fn upsert_fullnode(
        &self,
        fullnode_index: u32,
        num_fullnodes_per_validator: u32,
        validator_index: u32,
        num_validators: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()> {
        let pod_name = format!("fn-{}-{}", validator_index, fullnode_index);
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            self.delete_pod(&pod_name).await?;
        }
        let node_name = if let Some(instance) = self
            .fullnode_to_node
            .lock()
            .await
            .get(&(validator_index, fullnode_index))
        {
            if let Some(k8s_node) = instance.k8s_node() {
                k8s_node.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        debug!("Creating pod {} on node {:?}", pod_name, node_name);
        match pod_api
            .create(
                &PostParams::default(),
                self.fullnode_spec(
                    fullnode_index,
                    num_fullnodes_per_validator,
                    validator_index,
                    num_validators,
                    &node_name,
                    image_tag,
                    delete_data,
                )?,
            )
            .await
        {
            Ok(o) => {
                debug!("Created {}", o.metadata.name);
            }
            Err(e) => bail!("Failed to create pod {} : {}", pod_name, e),
        }
        if node_name.is_empty() {
            let (node_name, pod_ip) = self.get_pod_node_and_ip(&pod_name).await?;
            let ac_port = DEFAULT_JSON_RPC_PORT as u32;
            let instance = Instance::new_k8s(pod_name, pod_ip, ac_port, Some(node_name));
            self.fullnode_to_node
                .lock()
                .await
                .insert((validator_index, fullnode_index), instance);
        }
        Ok(())
    }

    async fn delete_fullnode(&self, fullnode_index: u32, validator_index: u32) -> Result<()> {
        let pod_name = format!("fn-{}-{}", validator_index, fullnode_index);
        self.delete_pod(&pod_name).await
    }

    async fn delete_all(&self) -> Result<()> {
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        let pod_names: Vec<_> = pod_api
            .list(&ListParams {
                label_selector: Some("libra-node=true".to_string()),
                ..Default::default()
            })
            .await?
            .iter()
            .map(|pod| pod.metadata.name.clone())
            .collect();
        let delete_futures = pod_names.iter().map(|pod_name| self.delete_pod(pod_name));
        try_join_all(delete_futures).await?;
        Ok(())
    }

    async fn get_grafana_baseurl(&self) -> Result<String> {
        let data = Api::v1ConfigMap(self.client.clone())
            .within(DEFAULT_NAMESPACE)
            .get("workspace")
            .await?
            .data;
        let workspace = data
            .get("workspace")
            .ok_or_else(|| format_err!("Failed to find workspace"))?;
        Ok(format!(
            "http://grafana.{}-k8s-testnet.aws.hlw3truzy4ls.com",
            workspace
        ))
    }
}
