// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, format_err, Result};
use async_trait::async_trait;

use futures::{future::try_join_all, lock::Mutex};
use k8s_openapi::api::core::v1::{ConfigMap, Node, Pod, Service};
use kube::{
    api::{Api, PostParams},
    client::Client,
    Config,
};
use libra_logger::*;
use util::retry;

use crate::{cluster_swarm::ClusterSwarm, instance::Instance};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::instance::{
    InstanceConfig,
    InstanceConfig::{Fullnode, Validator},
};
use itertools::Itertools;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;
use libra_config::config::DEFAULT_JSON_RPC_PORT;

const DEFAULT_NAMESPACE: &str = "default";

const CFG_SEED: &str = "1337133713371337133713371337133713371337133713371337133713371337";
const CFG_FULLNODE_SEED: &str = "2674267426742674267426742674267426742674267426742674267426742674";

const ERROR_NOT_FOUND: u16 = 404;

#[derive(Clone)]
pub struct ClusterSwarmKube {
    client: Client,
    node_map: Arc<Mutex<HashMap<InstanceConfig, Instance>>>,
}

impl ClusterSwarmKube {
    pub async fn new() -> Result<Self> {
        let result = Config::infer().await;
        let config = result.map_err(|e| format_err!("Failed to load config: {:?}", e))?;
        let client = Client::new(config);
        let node_map = Arc::new(Mutex::new(HashMap::new()));
        Ok(Self { client, node_map })
    }

    fn service_spec(&self, peer_id: String) -> Service {
        let service_yaml = format!(
            include_str!("libra_node_service_template.yaml"),
            peer_id = &peer_id
        );
        let service_spec: serde_yaml::Value = serde_yaml::from_str(&service_yaml).unwrap();
        let service_spec = serde_json::value::to_value(service_spec).unwrap();
        serde_json::from_value(service_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))
            .unwrap()
    }

    fn validator_spec(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        node_name: &str,
        image_tag: &str,
        cfg_overrides: &str,
        delete_data: bool,
    ) -> Result<Pod> {
        let cfg_fullnode_seed = if num_fullnodes > 0 {
            CFG_FULLNODE_SEED
        } else {
            ""
        };
        let fluentbit_enabled = "true";
        let pod_yaml = format!(
            include_str!("validator_spec_template.yaml"),
            index = index,
            num_validators = num_validators,
            num_fullnodes = num_fullnodes,
            image_tag = image_tag,
            node_name = node_name,
            cfg_overrides = cfg_overrides,
            delete_data = delete_data,
            cfg_seed = CFG_SEED,
            cfg_fullnode_seed = cfg_fullnode_seed,
            fluentbit_enabled = fluentbit_enabled,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        let pod_spec = serde_json::value::to_value(pod_spec)?;
        serde_json::from_value(pod_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))
    }

    fn fullnode_spec(
        &self,
        fullnode_index: u32,
        num_fullnodes: u32,
        validator_index: u32,
        num_validators: u32,
        node_name: &str,
        image_tag: &str,
        cfg_overrides: &str,
        delete_data: bool,
    ) -> Result<Pod> {
        let fluentbit_enabled = "true";
        let pod_yaml = format!(
            include_str!("fullnode_spec_template.yaml"),
            fullnode_index = fullnode_index,
            num_fullnodes = num_fullnodes,
            validator_index = validator_index,
            num_validators = num_validators,
            node_name = node_name,
            image_tag = image_tag,
            cfg_overrides = cfg_overrides,
            delete_data = delete_data,
            cfg_seed = CFG_SEED,
            cfg_fullnode_seed = CFG_FULLNODE_SEED,
            fluentbit_enabled = fluentbit_enabled,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        let pod_spec = serde_json::value::to_value(pod_spec)?;
        serde_json::from_value(pod_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))
    }

    async fn wait_job_completion(&self, job_name: &str, back_off_limit: u32) -> Result<bool> {
        retry::retry_async(retry::fixed_retry_strategy(5000, 20), || {
            let job_api: Api<Job> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
            let job_name = job_name.to_string();
            Box::pin(async move {
                match job_api.get(&job_name).await {
                    Ok(job) => {
                        let job_status = job.status.as_ref().ok_or_else(|| {
                            format_err!("status not found for job {}: {:?}", job_name, &job)
                        })?;
                        if let Some(succeeded) = job_status.succeeded {
                            if succeeded == 1 {
                                return Ok(true);
                            }
                        }
                        if let Some(failed) = job_status.succeeded {
                            if failed as u32 == back_off_limit + 1 {
                                error!("job {} failed to complete", job_name);
                                return Ok(false);
                            }
                        }
                        bail!("job in still in progress {}", job_name)
                    }
                    Err(e) => bail!("job_api.get failed for job {} : {:?}", job_name, e),
                }
            })
        })
        .await
    }

    async fn run_jobs(&self, jobs: Vec<Job>, back_off_limit: u32) -> Result<()> {
        let pp = PostParams::default();
        let job_api: Api<Job> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let create_jobs_futures = jobs.iter().map(|job| job_api.create(&pp, job));
        let job_names: Vec<String> = try_join_all(create_jobs_futures)
            .await?
            .into_iter()
            .map(|job| -> Result<String, anyhow::Error> {
                Ok(job
                    .metadata
                    .as_ref()
                    .ok_or_else(|| format_err!("metadata not found for job {:?}", &job))?
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for job {:?}", &job))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        let wait_jobs_futures = job_names
            .iter()
            .map(|job_name| self.wait_job_completion(job_name, back_off_limit));
        let wait_jobs_results = try_join_all(wait_jobs_futures).await?;
        if wait_jobs_results.into_iter().any(|r| !r) {
            bail!("one of the jobs failed")
        }
        Ok(())
    }

    async fn get_pod_node_and_ip(&self, pod_name: &str) -> Result<(String, String)> {
        retry::retry_async(retry::fixed_retry_strategy(10000, 60), || {
            let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
            let pod_name = pod_name.to_string();
            Box::pin(async move {
                match pod_api.get(&pod_name).await {
                    Ok(o) => {
                        let node_name = o
                            .spec
                            .as_ref()
                            .ok_or_else(|| format_err!("spec not found for pod {}, spec: {:?}", pod_name, o))?
                            .node_name
                            .as_ref()
                            .ok_or_else(|| {
                                format_err!("node_name not found for pod {}, spec: {:?}", pod_name, o)
                            })?;
                        let pod_ip = o
                            .status
                            .as_ref()
                            .ok_or_else(|| format_err!("status not found for pod {}, spec: {:?}", pod_name, o))?
                            .pod_ip
                            .as_ref()
                            .ok_or_else(|| format_err!("pod_ip not found for pod {}, spec: {:?}", pod_name, o))?;
                        if node_name.is_empty() || pod_ip.is_empty() {
                            bail!(
                                "Either node_name or pod_ip was empty string for pod {}, spec: {:?}",
                                pod_name, o
                            )
                        } else {
                            Ok((node_name.clone(), pod_ip.clone()))
                        }
                    }
                    Err(e) => bail!("pod_api.get failed for pod {} : {:?}", pod_name, e),
                }
            })
        })
        .await
    }

    async fn delete_resource<T>(&self, name: &str) -> Result<()>
    where
        T: k8s_openapi::Resource
            + Clone
            + serde::de::DeserializeOwned
            + kube::api::Meta
            + Send
            + Sync,
    {
        debug!("Deleting {} {}", T::KIND, name);
        let resource_api: Api<T> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        resource_api.delete(name, &Default::default()).await?;
        retry::retry_async(retry::fixed_retry_strategy(5000, 30), || {
            let pod_api = resource_api.clone();
            let name = name.to_string();
            Box::pin(async move {
                match pod_api.get(&name).await {
                    Ok(_) => {
                        bail!("Waiting for {} {} to be deleted..", T::KIND, name);
                    }
                    Err(kube::Error::Api(ae)) => {
                        if ae.code == ERROR_NOT_FOUND {
                            debug!("{} {} deleted successfully", T::KIND, name);
                            Ok(())
                        } else {
                            bail!("Waiting for {} to be deleted..", T::KIND)
                        }
                    }
                    Err(_) => bail!("Waiting for {} {} to be deleted..", T::KIND, name),
                }
            })
        })
        .await
        .map_err(|e| format_err!("Failed to delete {} {}: {:?}", T::KIND, name, e))
    }

    async fn remove_all_network_effects_helper(&self) -> Result<()> {
        debug!("Trying to remove_all_network_effects");
        let back_off_limit = 2;

        let node_api: Api<Node> = Api::all(self.client.clone());
        let lp = ListParams::default().labels("nodeType=validators");
        let jobs: Vec<Job> = node_api
            .list(&lp)
            .await?
            .iter()
            .map(|node| -> Result<Job, anyhow::Error> {
                let node_name = node
                    .metadata
                    .as_ref()
                    .ok_or_else(|| format_err!("metadata not found for node"))?
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for node"))?
                    .clone();
                let suffix = thread_rng()
                    .sample_iter(&Alphanumeric)
                    .take(10)
                    .collect::<String>()
                    .to_ascii_lowercase();
                let job_name = format!("remove-network-effects-{}", suffix);
                let job_yaml = format!(
                    include_str!("job_template.yaml"),
                    name = &job_name,
                    label = "remove-network-effects",
                    image = "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
                    node_name = node_name,
                    command = "tc qdisc delete dev eth0 root || true",
                    back_off_limit = back_off_limit,
                );
                debug!("Removing network effects from node {}", node_name);
                let job_spec: serde_yaml::Value = serde_yaml::from_str(&job_yaml)?;
                let job_spec = serde_json::value::to_value(job_spec)?;
                serde_json::from_value(job_spec)
                    .map_err(|e| format_err!("serde_json::from_value failed: {}", e))
            })
            .collect::<Result<_, _>>()?;
        self.run_jobs(jobs, back_off_limit).await
    }

    pub async fn get_workspace(&self) -> Result<String> {
        let cm_api: Api<ConfigMap> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let data = cm_api
            .get("workspace")
            .await?
            .data
            .ok_or_else(|| format_err!("data not found for ConfigMap"))?;
        let workspace = data
            .get("workspace")
            .ok_or_else(|| format_err!("Failed to find workspace"))?;
        Ok(workspace.clone())
    }
}

#[async_trait]
impl ClusterSwarm for ClusterSwarmKube {
    async fn remove_all_network_effects(&self) -> Result<()> {
        retry::retry_async(retry::fixed_retry_strategy(5000, 3), || {
            Box::pin(async move { self.remove_all_network_effects_helper().await })
        })
        .await
    }

    async fn run(
        &self,
        instance: &Instance,
        docker_image: &str,
        command: String,
        job_name: &str,
    ) -> Result<()> {
        let back_off_limit = 0;
        let suffix = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect::<String>()
            .to_ascii_lowercase();
        let job_full_name = format!("{}-{}", job_name, suffix);
        let job_yaml = format!(
            include_str!("job_template.yaml"),
            name = &job_full_name,
            label = job_name,
            image = docker_image,
            node_name = instance
                .k8s_node()
                .ok_or_else(|| { format_err!("k8s_node not found for Instance") })?,
            command = &command,
            back_off_limit = back_off_limit,
        );
        let job_spec: serde_yaml::Value = serde_yaml::from_str(&job_yaml)?;
        let job_spec = serde_json::value::to_value(job_spec)?;
        let job_spec = serde_json::from_value(job_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        debug!("Running job {} for instance {}", job_name, instance);
        self.run_jobs(vec![job_spec], back_off_limit).await
    }

    async fn validator_instances(&self) -> Vec<Instance> {
        self.node_map
            .lock()
            .await
            .values()
            .filter(|instance| {
                if let Some(Validator(_)) = instance.instance_config() {
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    async fn fullnode_instances(&self) -> Vec<Instance> {
        self.node_map
            .lock()
            .await
            .values()
            .filter(|instance| {
                if let Some(Fullnode(_)) = instance.instance_config() {
                    true
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    async fn upsert_node(&self, instance_config: InstanceConfig, delete_data: bool) -> Result<()> {
        let pod_name = match &instance_config {
            Validator(validator_config) => format!("val-{}", validator_config.index),
            Fullnode(fullnode_config) => format!(
                "fn-{}-{}",
                fullnode_config.validator_index, fullnode_config.fullnode_index
            ),
        };
        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            self.delete_resource::<Pod>(&pod_name).await?;
        }
        let node_name = if let Some(instance) = self.node_map.lock().await.get(&instance_config) {
            if let Some(k8s_node) = instance.k8s_node() {
                k8s_node.to_string()
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        debug!("Creating pod {} on node {:?}", pod_name, node_name);
        let (p, s): (Pod, Service) = match &instance_config {
            Validator(validator_config) => (
                self.validator_spec(
                    validator_config.index,
                    validator_config.num_validators,
                    validator_config.num_fullnodes,
                    &node_name,
                    &validator_config.image_tag,
                    &validator_config.config_overrides.iter().join(","),
                    delete_data,
                )?,
                self.service_spec(format!("val-{}", validator_config.index)),
            ),
            Fullnode(fullnode_config) => (
                self.fullnode_spec(
                    fullnode_config.fullnode_index,
                    fullnode_config.num_fullnodes_per_validator,
                    fullnode_config.validator_index,
                    fullnode_config.num_validators,
                    &node_name,
                    &fullnode_config.image_tag,
                    &fullnode_config.config_overrides.iter().join(","),
                    delete_data,
                )?,
                self.service_spec(format!(
                    "fn-{}-{}",
                    fullnode_config.validator_index, fullnode_config.fullnode_index
                )),
            ),
        };
        match pod_api.create(&PostParams::default(), &p).await {
            Ok(o) => {
                debug!(
                    "Created pod {}",
                    o.metadata
                        .as_ref()
                        .ok_or_else(|| { format_err!("metadata not found for pod {}", pod_name) })?
                        .name
                        .as_ref()
                        .ok_or_else(|| { format_err!("name not found for pod {}", pod_name) })?
                );
            }
            Err(e) => bail!("Failed to create pod {} : {}", pod_name, e),
        }
        let service_api: Api<Service> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        match service_api.create(&PostParams::default(), &s).await {
            Ok(o) => {
                debug!(
                    "Created service {}",
                    o.metadata
                        .as_ref()
                        .ok_or_else(|| { format_err!("metadata not found for service") })?
                        .name
                        .as_ref()
                        .ok_or_else(|| { format_err!("name not found for service") })?
                );
            }
            Err(kube::Error::Api(ae)) => {
                if ae.code == 409 {
                    // 409 == service already exists
                    debug!("Service already exists. Skipping")
                } else {
                    bail!("Failed to create service : {}", ae)
                }
            }
            Err(e) => bail!("Failed to create service : {}", e),
        }
        if node_name.is_empty() {
            let (node_name, pod_ip) = self.get_pod_node_and_ip(&pod_name).await?;
            let ac_port = DEFAULT_JSON_RPC_PORT as u32;
            let instance = Instance::new_k8s(
                pod_name,
                pod_ip,
                ac_port,
                Some(node_name),
                instance_config.clone(),
            );
            self.node_map.lock().await.insert(instance_config, instance);
        }
        Ok(())
    }

    async fn delete_node(&self, instance_config: InstanceConfig) -> Result<()> {
        let pod_name = match instance_config {
            Validator(validator_config) => format!("val-{}", validator_config.index),
            Fullnode(fullnode_config) => format!(
                "fn-{}-{}",
                fullnode_config.validator_index, fullnode_config.fullnode_index
            ),
        };
        let service_name = pod_name.clone();
        self.delete_resource::<Pod>(&pod_name).await?;
        self.delete_resource::<Service>(&service_name).await
    }

    async fn delete_all(&self) -> Result<()> {
        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let pod_names: Vec<String> = pod_api
            .list(&ListParams {
                label_selector: Some("libra-node=true".to_string()),
                ..Default::default()
            })
            .await?
            .iter()
            .map(|pod| -> Result<String, anyhow::Error> {
                Ok(pod
                    .metadata
                    .as_ref()
                    .ok_or_else(|| format_err!("metadata not found for pod"))?
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for pod"))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        let delete_futures = pod_names
            .iter()
            .map(|pod_name| self.delete_resource::<Pod>(pod_name));
        try_join_all(delete_futures).await?;
        let service_api: Api<Service> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let service_names: Vec<String> = service_api
            .list(&ListParams {
                label_selector: Some("libra-node=true".to_string()),
                ..Default::default()
            })
            .await?
            .iter()
            .map(|service| -> Result<String, anyhow::Error> {
                Ok(service
                    .metadata
                    .as_ref()
                    .ok_or_else(|| format_err!("metadata not found for service"))?
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for service"))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        let delete_futures = service_names
            .iter()
            .map(|service_name| self.delete_resource::<Service>(service_name));
        try_join_all(delete_futures).await?;
        let job_api: Api<Job> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let job_names: Vec<String> = job_api
            .list(&ListParams {
                label_selector: Some("libra-node=true".to_string()),
                ..Default::default()
            })
            .await?
            .iter()
            .map(|job| -> Result<String, anyhow::Error> {
                Ok(job
                    .metadata
                    .as_ref()
                    .ok_or_else(|| format_err!("metadata not found for job"))?
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for job"))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        let delete_futures = job_names
            .iter()
            .map(|job_name| self.delete_resource::<Job>(job_name));
        try_join_all(delete_futures).await?;
        Ok(())
    }

    async fn get_grafana_baseurl(&self) -> Result<String> {
        let workspace = self.get_workspace().await?;
        Ok(format!(
            "http://grafana.{}-k8s-testnet.aws.hlw3truzy4ls.com",
            workspace
        ))
    }
}
