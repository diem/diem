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

use crate::{cluster_swarm::ClusterSwarm, instance::Instance};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::instance::{
    ApplicationConfig::{Fullnode, Validator, Vault, LSR},
    InstanceConfig,
};
use itertools::Itertools;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;
use libra_config::config::DEFAULT_JSON_RPC_PORT;
use reqwest::Client as HttpClient;
use std::{collections::HashSet, convert::TryFrom, process::Command};

const DEFAULT_NAMESPACE: &str = "default";

const CFG_SEED: &str = "1337133713371337133713371337133713371337133713371337133713371337";
const CFG_FULLNODE_SEED: &str = "2674267426742674267426742674267426742674267426742674267426742674";

const ERROR_NOT_FOUND: u16 = 404;

#[derive(Clone)]
pub struct ClusterSwarmKube {
    client: Client,
    node_map: Arc<Mutex<HashMap<String, KubeNode>>>,
    http_client: HttpClient,
}

impl ClusterSwarmKube {
    pub async fn new() -> Result<Self> {
        let http_client = HttpClient::new();
        // This uses kubectl proxy locally to forward connections to kubernetes api server
        Command::new("/usr/local/bin/kubectl")
            .arg("proxy")
            .spawn()?;
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(2000, 60), || {
            Box::pin(async move {
                debug!("Running local kube pod healthcheck on http://127.0.0.1:8001");
                reqwest::get("http://127.0.0.1:8001").await?.text().await?;
                info!("Local kube pod healthcheck passed");
                Ok::<(), reqwest::Error>(())
            })
        })
        .await?;
        let config = Config::new(
            reqwest::Url::parse("http://127.0.0.1:8001")
                .expect("Failed to parse kubernetes endpoint url"),
        );
        let client = Client::new(config);
        let node_map = Arc::new(Mutex::new(HashMap::new()));
        Ok(Self {
            client,
            node_map,
            http_client,
        })
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

    fn lsr_spec(
        &self,
        validator_index: u32,
        num_validators: u32,
        node_name: &str,
        image_tag: &str,
        lsr_backend: &str,
    ) -> Result<(Pod, Service)> {
        let pod_yaml = format!(
            include_str!("lsr_spec_template.yaml"),
            validator_index = validator_index,
            num_validators = num_validators,
            image_tag = image_tag,
            node_name = node_name,
            lsr_backend = lsr_backend,
            cfg_seed = CFG_SEED,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        let pod_spec = serde_json::value::to_value(pod_spec)?;
        let pod_spec = serde_json::from_value(pod_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        let service_yaml = format!(
            include_str!("lsr_service_template.yaml"),
            validator_index = validator_index,
        );
        let service_spec: serde_yaml::Value = serde_yaml::from_str(&service_yaml).unwrap();
        let service_spec = serde_json::value::to_value(service_spec).unwrap();
        let service_spec = serde_json::from_value(service_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        Ok((pod_spec, service_spec))
    }

    fn vault_spec(&self, validator_index: u32, node_name: &str) -> Result<(Pod, Service)> {
        let pod_yaml = format!(
            include_str!("vault_spec_template.yaml"),
            validator_index = validator_index,
            node_name = node_name,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml)?;
        let pod_spec = serde_json::value::to_value(pod_spec)?;
        let pod_spec = serde_json::from_value(pod_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        let service_yaml = format!(
            include_str!("vault_service_template.yaml"),
            validator_index = validator_index,
        );
        let service_spec: serde_yaml::Value = serde_yaml::from_str(&service_yaml).unwrap();
        let service_spec = serde_json::value::to_value(service_spec).unwrap();
        let service_spec = serde_json::from_value(service_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        Ok((pod_spec, service_spec))
    }

    fn validator_spec(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        enable_lsr: bool,
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
            enable_lsr = enable_lsr,
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
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(5000, 20), || {
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
                        if let Some(failed) = job_status.failed {
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
            .iter()
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
        if wait_jobs_results.iter().any(|r| !r) {
            bail!("one of the jobs failed")
        }
        Ok(())
    }

    async fn list_nodes(&self) -> Result<Vec<KubeNode>> {
        let node_api: Api<Node> = Api::all(self.client.clone());
        let lp = ListParams::default().labels("nodeType=validators");
        let nodes = node_api.list(&lp).await?.items;
        nodes.into_iter().map(KubeNode::try_from).collect()
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
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(5000, 30), || {
            let resource_api = resource_api.clone();
            let name = name.to_string();
            Box::pin(async move {
                match resource_api.delete(&name, &Default::default()).await {
                    Ok(_) => {}
                    Err(kube::Error::Api(ae)) => {
                        if ae.code == ERROR_NOT_FOUND {
                            debug!("{} {} deleted successfully", T::KIND, name);
                            return Ok(());
                        } else {
                            error!(
                                "delete failed for {} {} with kube::Error::Api: {}",
                                T::KIND,
                                name,
                                ae
                            )
                        }
                    }
                    Err(err) => {
                        error!("delete failed for {} {} with error: {}", T::KIND, name, err)
                    }
                }
                match resource_api.get(&name).await {
                    Ok(_) => {
                        bail!("Waiting for {} {} to be deleted..", T::KIND, name);
                    }
                    Err(kube::Error::Api(ae)) => {
                        if ae.code == ERROR_NOT_FOUND {
                            debug!("{} {} deleted successfully", T::KIND, name);
                            Ok(())
                        } else {
                            bail!("Waiting for {} {} to be deleted..", T::KIND, name)
                        }
                    }
                    Err(err) => bail!(
                        "Waiting for {} {} to be deleted... Error: {}",
                        T::KIND,
                        name,
                        err
                    ),
                }
            })
        })
        .await
        .map_err(|e| format_err!("Failed to delete {} {}: {:?}", T::KIND, name, e))
    }

    async fn remove_all_network_effects_helper(&self) -> Result<()> {
        debug!("Trying to remove_all_network_effects");
        let back_off_limit = 2;

        let jobs: Vec<Job> = self
            .list_nodes()
            .await?
            .iter()
            .map(|node| -> Result<Job, anyhow::Error> {
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
                    node_name = node.name,
                    command = "tc qdisc delete dev eth0 root || true",
                    back_off_limit = back_off_limit,
                );
                debug!("Removing network effects from node {}", node.name);
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

    pub async fn run(
        &self,
        k8s_node: &str,
        docker_image: &str,
        command: &str,
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
            node_name = k8s_node,
            command = command,
            back_off_limit = back_off_limit,
        );
        let job_spec: serde_yaml::Value = serde_yaml::from_str(&job_yaml)?;
        let job_spec = serde_json::value::to_value(job_spec)?;
        let job_spec = serde_json::from_value(job_spec)
            .map_err(|e| format_err!("serde_json::from_value failed: {}", e))?;
        debug!("Running job {} for node {}", job_name, k8s_node);
        self.run_jobs(vec![job_spec], back_off_limit).await
    }

    async fn allocate_node(&self, pod_name: &str) -> Result<KubeNode> {
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(5000, 15), || {
            Box::pin(async move { self.allocate_node_impl(pod_name).await })
        })
        .await
    }

    async fn allocate_node_impl(&self, pod_name: &str) -> Result<KubeNode> {
        let nodes = self.list_nodes().await?;
        let nodes_count = nodes.len();
        // Holding lock for read-verfy-write to avoid race conditions on this map
        let mut node_map = self.node_map.lock().await;
        if let Some(existed) = node_map.get(pod_name) {
            return Ok(existed.clone());
        }
        let used_nodes: HashSet<_> = node_map.values().map(|node| &node.name).collect();
        for node in nodes {
            if !used_nodes.contains(&node.name) {
                node_map.insert(pod_name.to_string(), node.clone());
                return Ok(node);
            }
        }
        Err(format_err!(
            "Can not find free node, got total {} nodes",
            nodes_count
        ))
    }

    pub async fn upsert_node(
        &self,
        instance_config: InstanceConfig,
        delete_data: bool,
    ) -> Result<Instance> {
        let pod_name = instance_config.pod_name();
        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            self.delete_resource::<Pod>(&pod_name).await?;
        }
        let node = self
            .allocate_node(&pod_name)
            .await
            .map_err(|e| format_err!("Failed to allocate node: {}", e))?;
        debug!("Creating pod {} on {:?}", pod_name, node);
        let (p, s): (Pod, Service) = match &instance_config.application_config {
            Validator(validator_config) => (
                self.validator_spec(
                    instance_config.validator_group,
                    validator_config.num_validators,
                    validator_config.num_fullnodes,
                    validator_config.enable_lsr,
                    &node.name,
                    &validator_config.image_tag,
                    &validator_config.config_overrides.iter().join(","),
                    delete_data,
                )?,
                self.service_spec(instance_config.pod_name()),
            ),
            Fullnode(fullnode_config) => (
                self.fullnode_spec(
                    fullnode_config.fullnode_index,
                    fullnode_config.num_fullnodes_per_validator,
                    instance_config.validator_group,
                    fullnode_config.num_validators,
                    &node.name,
                    &fullnode_config.image_tag,
                    &fullnode_config.config_overrides.iter().join(","),
                    delete_data,
                )?,
                self.service_spec(instance_config.pod_name()),
            ),
            Vault(_vault_config) => self.vault_spec(instance_config.validator_group, &node.name)?,
            LSR(lsr_config) => self.lsr_spec(
                instance_config.validator_group,
                lsr_config.num_validators,
                &node.name,
                &lsr_config.image_tag,
                &lsr_config.lsr_backend,
            )?,
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
        let ac_port = DEFAULT_JSON_RPC_PORT as u32;
        let instance = Instance::new_k8s(
            pod_name,
            node.internal_ip,
            ac_port,
            node.name.clone(),
            instance_config.clone(),
            self.http_client.clone(),
            self.clone(),
        );
        Ok(instance)
    }

    pub async fn delete_node(&self, instance_config: &InstanceConfig) -> Result<()> {
        let pod_name = instance_config.pod_name();
        let service_name = pod_name.clone();
        self.delete_resource::<Pod>(&pod_name).await?;
        self.delete_resource::<Service>(&service_name).await
    }

    async fn remove_all_network_effects(&self) -> Result<()> {
        libra_retrier::retry_async(libra_retrier::fixed_retry_strategy(5000, 3), || {
            Box::pin(async move { self.remove_all_network_effects_helper().await })
        })
        .await
    }

    pub async fn cleanup(&self) -> Result<()> {
        self.delete_all()
            .await
            .map_err(|e| format_err!("delete_all failed: {}", e))?;
        self.remove_all_network_effects()
            .await
            .map_err(|e| format_err!("remove_all_network_effects: {}", e))
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
}

#[async_trait]
impl ClusterSwarm for ClusterSwarmKube {
    async fn spawn_new_instance(
        &self,
        instance_config: InstanceConfig,
        delete_data: bool,
    ) -> Result<Instance> {
        self.upsert_node(instance_config, delete_data).await
    }

    async fn get_grafana_baseurl(&self) -> Result<String> {
        let workspace = self.get_workspace().await?;
        Ok(format!(
            "http://grafana.{}-k8s-testnet.aws.hlw3truzy4ls.com",
            workspace
        ))
    }
}

#[derive(Clone, Debug)]
pub struct KubeNode {
    pub name: String,
    pub provider_id: String,
    pub internal_ip: String,
}

impl TryFrom<Node> for KubeNode {
    type Error = anyhow::Error;

    fn try_from(node: Node) -> Result<Self, Self::Error> {
        let metadata = node
            .metadata
            .ok_or_else(|| format_err!("metadata not found for node"))?;
        let spec = node
            .spec
            .ok_or_else(|| format_err!("spec not found for node"))?;
        let provider_id = spec
            .provider_id
            .ok_or_else(|| format_err!("provider_id not found for node"))?;
        let name = metadata
            .name
            .ok_or_else(|| format_err!("node name not found"))?;
        let status = node
            .status
            .ok_or_else(|| format_err!("status not found for node"))?;
        let addresses = status
            .addresses
            .ok_or_else(|| format_err!("addresses name not found"))?;
        let internal_address = addresses
            .iter()
            .find(|a| a.type_ == "InternalIP")
            .ok_or_else(|| format_err!("internal address not found"))?;
        let internal_ip = internal_address.address.clone();
        Ok(Self {
            name,
            provider_id,
            internal_ip,
        })
    }
}
