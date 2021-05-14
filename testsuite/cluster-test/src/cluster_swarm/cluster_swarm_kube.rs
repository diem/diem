// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::{collections::HashMap, env, sync::Arc};

use anyhow::{bail, format_err, Result};
use async_trait::async_trait;

use diem_logger::*;
use futures::{future::try_join_all, join, lock::Mutex, Future, FutureExt, TryFuture};
use k8s_openapi::api::core::v1::{ConfigMap, Node, Pod, Service};
use kube::{
    api::{Api, DeleteParams, PostParams},
    client::Client,
    Config,
};

use crate::{cluster_swarm::ClusterSwarm, instance::Instance};
use rand::{distributions::Alphanumeric, thread_rng, Rng};

use crate::instance::{
    ApplicationConfig::{Fullnode, Validator, Vault, LSR},
    InstanceConfig,
};
use diem_config::config::DEFAULT_JSON_RPC_PORT;
use k8s_openapi::api::batch::v1::Job;
use kube::api::ListParams;
use reqwest::Client as HttpClient;
use rusoto_core::Region;
use rusoto_s3::{PutObjectRequest, S3Client, S3};
use rusoto_sts::WebIdentityProvider;
use serde::de::DeserializeOwned;
use std::{collections::HashSet, convert::TryFrom, process::Command, time::Duration};
use tokio::sync::Semaphore;

pub const CFG_SEED: &str = "1337133713371337133713371337133713371337133713371337133713371337";
const DEFAULT_NAMESPACE: &str = "default";
const ERROR_NOT_FOUND: u16 = 404;
const GENESIS_PATH: &str = "/tmp/genesis.blob";
const HEALTH_CHECK_URL: &str = "http://127.0.0.1:8001";
const KUBECTL_BIN: &str = "/usr/local/bin/kubectl";

// We use the macros below to get around the current limitations of the
// "include_str!" macro (which loads the file content at compile time, rather
// than at runtime).
// TODO(joshlind): Remove me once we support runtime file loading.

// Config file names.
macro_rules! FULLNODE_CONFIG {
    () => {
        "configs/fullnode.yaml"
    };
}
macro_rules! SAFETY_RULES_CONFIG {
    () => {
        "configs/safetyrules.yaml"
    };
}
macro_rules! VALIDATOR_CONFIG {
    () => {
        "configs/validator.yaml"
    };
}

// Fluent bit file names.
macro_rules! FLUENT_BIT_CONF {
    () => {
        "fluent-bit/fluent-bit.conf"
    };
}
macro_rules! FLUENT_BIT_PARSERS_CONF {
    () => {
        "fluent-bit/parsers.conf"
    };
}

// Template file names.
macro_rules! JOB_TEMPLATE {
    () => {
        "templates/job_template.yaml"
    };
}
macro_rules! DIEM_NODE_SERVICE_TEMPLATE {
    () => {
        "templates/diem_node_service_template.yaml"
    };
}
macro_rules! DIEM_NODE_SPEC_TEMPLATE {
    () => {
        "templates/diem_node_spec_template.yaml"
    };
}
macro_rules! LSR_SERVICE_TEMPLATE {
    () => {
        "templates/lsr_service_template.yaml"
    };
}
macro_rules! LSR_SPEC_TEMPLATE {
    () => {
        "templates/lsr_spec_template.yaml"
    };
}
macro_rules! VAULT_SERVICE_TEMPLATE {
    () => {
        "templates/vault_service_template.yaml"
    };
}
macro_rules! VAULT_SPEC_TEMPLATE {
    () => {
        "templates/vault_spec_template.yaml"
    };
}

#[derive(Clone)]
pub struct ClusterSwarmKube {
    client: Client,
    http_client: HttpClient,
    s3_client: S3Client,
    pub node_map: Arc<Mutex<HashMap<String, KubeNode>>>,
}

impl ClusterSwarmKube {
    pub async fn new() -> Result<Self> {
        let http_client = HttpClient::new();
        // This uses kubectl proxy locally to forward connections to kubernetes api server
        Command::new(KUBECTL_BIN).arg("proxy").spawn()?;
        diem_retrier::retry_async(k8s_retry_strategy(), || {
            Box::pin(async move {
                debug!("Running local kube pod healthcheck on {}", HEALTH_CHECK_URL);
                reqwest::get(HEALTH_CHECK_URL).await?.text().await?;
                info!("Local kube pod healthcheck passed");
                Ok::<(), reqwest::Error>(())
            })
        })
        .await?;
        let config = Config::new(
            reqwest::Url::parse(HEALTH_CHECK_URL).expect("Failed to parse kubernetes endpoint url"),
        );
        let client = Client::try_from(config)?;
        let credentials_provider = WebIdentityProvider::from_k8s_env();
        let dispatcher =
            rusoto_core::HttpClient::new().expect("failed to create request dispatcher");
        let s3_client = S3Client::new_with(dispatcher, credentials_provider, Region::UsWest2);
        let node_map = Arc::new(Mutex::new(HashMap::new()));
        Ok(Self {
            client,
            http_client,
            s3_client,
            node_map,
        })
    }

    fn service_spec(&self, peer_id: String) -> Result<Service> {
        let service_yaml = format!(
            include_str!(DIEM_NODE_SERVICE_TEMPLATE!()),
            peer_id = &peer_id
        );
        get_spec_instance_from_template(service_yaml)
    }

    fn lsr_spec(&self, pod_name: &str, node_name: &str, image_tag: &str) -> Result<(Pod, Service)> {
        let pod_yaml = format!(
            include_str!(LSR_SPEC_TEMPLATE!()),
            pod_name = pod_name,
            image_tag = image_tag,
            node_name = node_name,
        );
        let pod_spec = get_spec_instance_from_template(pod_yaml)?;

        let service_yaml = format!(include_str!(LSR_SERVICE_TEMPLATE!()), pod_name = pod_name,);
        let service_spec = get_spec_instance_from_template(service_yaml)?;

        Ok((pod_spec, service_spec))
    }

    fn vault_spec(&self, validator_index: u32, node_name: &str) -> Result<(Pod, Service)> {
        let pod_yaml = format!(
            include_str!(VAULT_SPEC_TEMPLATE!()),
            validator_index = validator_index,
            node_name = node_name,
        );
        let pod_spec = get_spec_instance_from_template(pod_yaml)?;

        let service_yaml = format!(
            include_str!(VAULT_SERVICE_TEMPLATE!()),
            validator_index = validator_index,
        );
        let service_spec = get_spec_instance_from_template(service_yaml)?;

        Ok((pod_spec, service_spec))
    }

    fn diem_node_spec(
        &self,
        pod_app: &str,
        pod_name: &str,
        node_name: &str,
        image_tag: &str,
    ) -> Result<Pod> {
        let pod_yaml = format!(
            include_str!(DIEM_NODE_SPEC_TEMPLATE!()),
            pod_app = pod_app,
            pod_name = pod_name,
            image_tag = image_tag,
            node_name = node_name,
        );
        get_spec_instance_from_template(pod_yaml)
    }

    fn job_spec(
        &self,
        k8s_node: &str,
        docker_image: &str,
        command: &str,
        job_name: &str,
        back_off_limit: u32,
    ) -> Result<(Job, String)> {
        let suffix = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>()
            .to_ascii_lowercase();
        let job_full_name = format!("{}-{}", job_name, suffix);

        let job_yaml = format!(
            include_str!(JOB_TEMPLATE!()),
            name = &job_full_name,
            label = job_name,
            image = docker_image,
            node_name = k8s_node,
            command = command,
            back_off_limit = back_off_limit,
        );
        let job_spec = get_spec_instance_from_template(job_yaml)?;
        Ok((job_spec, job_full_name))
    }

    async fn wait_job_completion(
        &self,
        job_name: &str,
        back_off_limit: u32,
        killed: bool,
    ) -> Result<bool> {
        diem_retrier::retry_async(k8s_retry_strategy(), || {
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
                    Err(e) => {
                        if killed {
                            info!("Job {} has been killed already", job_name);
                            return Ok(true);
                        }
                        bail!("job_api.get failed for job {} : {:?}", job_name, e)
                    }
                }
            })
        })
        .await
    }

    pub async fn kill_job(&self, job_full_name: &str) -> Result<()> {
        let dp = DeleteParams::default();
        let job_api: Api<Job> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        job_api
            .delete(job_full_name, &dp)
            .await?
            .map_left(|o| debug!("Deleting Job: {:?}", o.status))
            .map_right(|s| debug!("Deleted Job: {:?}", s));
        let back_off_limit = 0;
        // the job night have been deleted already, so we do not handle error
        match self
            .wait_job_completion(job_full_name, back_off_limit, true)
            .await
        {
            Ok(_) => debug!("Killing job {} returned job_status.success", job_full_name),
            Err(error) => info!(
                "Killing job {} returned job_status.failed: {}",
                job_full_name, error
            ),
        }

        Ok(())
    }

    // just ensures jobs are started, but does not wait on completion
    async fn start_jobs(&self, jobs: Vec<Job>) -> Result<Vec<String>> {
        let pp = PostParams::default();
        let job_api: Api<Job> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let create_jobs_futures = jobs.iter().map(|job| job_api.create(&pp, job));
        let job_names: Vec<String> = try_join_all_limit(create_jobs_futures.collect())
            .await?
            .iter()
            .map(|job| -> Result<String, anyhow::Error> {
                Ok(job
                    .metadata
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found for job {:?}", &job))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        Ok(job_names)
    }

    async fn run_jobs(&self, jobs: Vec<Job>, back_off_limit: u32) -> Result<()> {
        let job_names: Vec<String> = self.start_jobs(jobs).await?;
        let wait_jobs_futures = job_names
            .iter()
            .map(|job_name| self.wait_job_completion(job_name, back_off_limit, false));
        let wait_jobs_results = try_join_all_limit(wait_jobs_futures.collect()).await?;
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
        diem_retrier::retry_async(k8s_retry_strategy(), || {
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
                    .map(char::from)
                    .collect::<String>()
                    .to_ascii_lowercase();
                let job_name = format!("remove-network-effects-{}", suffix);
                let job_yaml = format!(
                    include_str!(JOB_TEMPLATE!()),
                    name = &job_name,
                    label = "remove-network-effects",
                    image = "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
                    node_name = node.name,
                    command = "tc qdisc delete dev eth0 root || true",
                    back_off_limit = back_off_limit,
                );
                debug!("Removing network effects from node {}", node.name);
                get_spec_instance_from_template(job_yaml)
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

    pub async fn spawn_job(
        &self,
        k8s_node: &str,
        docker_image: &str,
        command: &str,
        job_name: &str,
    ) -> Result<String> {
        let back_off_limit = 0;
        let (job_spec, job_full_name) =
            self.job_spec(k8s_node, docker_image, command, job_name, back_off_limit)?;
        debug!("Starting job {} for node {}", job_name, k8s_node);
        self.start_jobs(vec![job_spec]).await?;
        Ok(job_full_name)
    }

    pub async fn run(
        &self,
        k8s_node: &str,
        docker_image: &str,
        command: &str,
        job_name: &str,
    ) -> Result<()> {
        let back_off_limit = 0;
        let (job_spec, _) =
            self.job_spec(k8s_node, docker_image, command, job_name, back_off_limit)?;
        debug!("Running job {} for node {}", job_name, k8s_node);
        self.run_jobs(vec![job_spec], back_off_limit).await
    }

    pub async fn allocate_node(&self, pod_name: &str) -> Result<KubeNode> {
        diem_retrier::retry_async(k8s_retry_strategy(), || {
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

    pub async fn upsert_node(&self, instance_config: InstanceConfig) -> Result<Instance> {
        let pod_name = instance_config.pod_name();
        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            self.delete_resource::<Pod>(&pod_name).await?;
        }
        let node = self
            .allocate_node(&pod_name)
            .await
            .map_err(|e| format_err!("Failed to allocate node: {}", e))?;
        debug!(
            "Configuring fluent-bit, genesis and config for pod {} on {}",
            pod_name, node.name
        );
        if instance_config.application_config.needs_fluentbit() {
            self.config_fluentbit("events", &pod_name, &node.name)
                .await?;
        }
        if instance_config.application_config.needs_genesis() {
            self.put_genesis_file(&pod_name, &node.name).await?;
        }
        if instance_config.application_config.needs_config() {
            self.generate_config(&instance_config, &pod_name, &node.name)
                .await?;
        }
        debug!("Creating pod {} on {:?}", pod_name, node);
        let (p, s): (Pod, Service) = match &instance_config.application_config {
            Validator(validator_config) => (
                self.diem_node_spec(
                    "diem-validator",
                    pod_name.as_str(),
                    &node.name,
                    &validator_config.image_tag,
                )?,
                self.service_spec(pod_name.clone())?,
            ),
            Fullnode(fullnode_config) => (
                self.diem_node_spec(
                    "diem-fullnode",
                    pod_name.as_str(),
                    &node.name,
                    &fullnode_config.image_tag,
                )?,
                self.service_spec(pod_name.clone())?,
            ),
            Vault(_vault_config) => {
                self.vault_spec(instance_config.validator_group.index_only(), &node.name)?
            }
            LSR(lsr_config) => {
                self.lsr_spec(pod_name.as_str(), &node.name, &lsr_config.image_tag)?
            }
        };
        match pod_api.create(&PostParams::default(), &p).await {
            Ok(o) => {
                debug!(
                    "Created pod {}",
                    o.metadata
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
            pod_name.clone(),
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
        diem_retrier::retry_async(k8s_retry_strategy(), || {
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

    pub async fn delete<T>(&self) -> Result<()>
    where
        T: k8s_openapi::Resource
            + Clone
            + serde::de::DeserializeOwned
            + kube::api::Meta
            + Send
            + Sync,
    {
        let api: Api<T> = Api::namespaced(self.client.clone(), DEFAULT_NAMESPACE);
        let resource_names: Vec<String> = api
            .list(&ListParams {
                label_selector: Some("diem-node=true".to_string()),
                ..Default::default()
            })
            .await?
            .iter()
            .map(|res| -> Result<String, anyhow::Error> {
                Ok(res
                    .meta()
                    .name
                    .as_ref()
                    .ok_or_else(|| format_err!("name not found"))?
                    .clone())
            })
            .collect::<Result<_, _>>()?;
        let delete_futures = resource_names
            .iter()
            .map(|resource_names| self.delete_resource::<T>(resource_names));
        try_join_all_limit(delete_futures.collect()).await?;
        Ok(())
    }

    pub async fn delete_all(&self) -> Result<()> {
        let del_pod = self.delete::<Pod>().boxed();
        let del_service = self.delete::<Service>().boxed();
        let del_job = self.delete::<Job>().boxed();
        let _ = join!(del_pod, del_service, del_job);
        Ok(())
    }

    /// Runs command on the provided host in separate utility container based on cluster-test-util image
    pub async fn util_cmd<S: AsRef<str>>(
        &self,
        command: S,
        k8s_node: &str,
        job_name: &str,
    ) -> Result<()> {
        self.run(
            k8s_node,
            "853397791086.dkr.ecr.us-west-2.amazonaws.com/cluster-test-util:latest",
            command.as_ref(),
            job_name,
        )
        .await
    }

    async fn config_fluentbit(&self, input_tag: &str, pod_name: &str, node: &str) -> Result<()> {
        let parsers_config = include_str!(FLUENT_BIT_PARSERS_CONF!()).to_string();
        let fluentbit_config = format!(
            include_str!(FLUENT_BIT_CONF!()),
            input_tag = input_tag,
            pod_name = pod_name
        );
        let dir = "/opt/diem/data/fluent-bit/";
        self.put_file(
            node,
            pod_name,
            format!("{}parser.conf", dir).as_str(),
            parsers_config.as_bytes(),
        )
        .await?;
        self.put_file(
            node,
            pod_name,
            format!("{}fluent-bit.conf", dir).as_str(),
            fluentbit_config.as_bytes(),
        )
        .await?;
        Ok(())
    }

    async fn put_genesis_file(&self, pod_name: &str, node: &str) -> Result<()> {
        let genesis = std::fs::read(GENESIS_PATH)
            .map_err(|e| format_err!("Failed to read {} : {}", GENESIS_PATH, e))?;
        self.put_file(
            node,
            pod_name,
            "/opt/diem/etc/genesis.blob",
            genesis.as_slice(),
        )
        .await?;
        Ok(())
    }

    async fn generate_config(
        &self,
        instance_config: &InstanceConfig,
        pod_name: &str,
        node: &str,
    ) -> Result<()> {
        let node_config = match &instance_config.application_config {
            Validator(validator_config) => Some(format!(
                include_str!(VALIDATOR_CONFIG!()),
                vault_addr = validator_config
                    .vault_addr
                    .as_ref()
                    .unwrap_or(&"".to_string()),
                vault_ns = validator_config
                    .vault_namespace
                    .as_ref()
                    .unwrap_or(&"".to_string()),
                safety_rules_addr = validator_config
                    .safety_rules_addr
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )),
            Fullnode(fullnode_config) => Some(format!(
                include_str!(FULLNODE_CONFIG!()),
                vault_addr = fullnode_config
                    .vault_addr
                    .as_ref()
                    .unwrap_or(&"".to_string()),
                vault_ns = fullnode_config
                    .vault_namespace
                    .as_ref()
                    .unwrap_or(&"".to_string()),
                seed_peer_ip = fullnode_config.seed_peer_ip,
            )),
            LSR(lsr_config) => Some(format!(
                include_str!(SAFETY_RULES_CONFIG!()),
                vault_addr = lsr_config.vault_addr.as_ref().unwrap_or(&"".to_string()),
                vault_ns = lsr_config
                    .vault_namespace
                    .as_ref()
                    .unwrap_or(&"".to_string()),
            )),
            _ => None,
        };

        if let Some(node_config) = node_config {
            self.put_file(
                node,
                pod_name,
                "/opt/diem/etc/node.yaml",
                node_config.as_bytes(),
            )
            .await?;
        }

        Ok(())
    }
}

/// Retrieves a spec instance of type T from a T template file.
fn get_spec_instance_from_template<T: DeserializeOwned>(template_yaml: String) -> Result<T> {
    let spec: serde_yaml::Value = serde_yaml::from_str(&template_yaml)?;
    let spec = serde_json::value::to_value(spec)?;
    serde_json::from_value(spec).map_err(|e| format_err!("serde_json::from_value failed: {}", e))
}

#[async_trait]
impl ClusterSwarm for ClusterSwarmKube {
    async fn spawn_new_instance(&self, instance_config: InstanceConfig) -> Result<Instance> {
        self.upsert_node(instance_config).await
    }

    async fn clean_data(&self, node: &str) -> Result<()> {
        self.util_cmd("rm -rf /opt/diem/data/*", node, "clean-data")
            .await
    }

    async fn get_node_name(&self, pod_name: &str) -> Result<String> {
        let node = self.allocate_node(pod_name).await?;
        Ok(node.name)
    }

    async fn get_grafana_baseurl(&self) -> Result<String> {
        let workspace = self.get_workspace().await?;
        Ok(format!(
            "http://grafana.{}-k8s-testnet.aws.hlw3truzy4ls.com",
            workspace
        ))
    }

    async fn put_file(&self, node: &str, pod_name: &str, path: &str, content: &[u8]) -> Result<()> {
        let bucket = "toro-cluster-test-flamegraphs";
        let run_id = env::var("RUN_ID").expect("RUN_ID is not set.");
        diem_retrier::retry_async(k8s_retry_strategy(), || {
            let run_id = &run_id;
            let content = content.to_vec();
            Box::pin(async move {
                self.s3_client
                    .put_object(PutObjectRequest {
                        bucket: bucket.to_string(),
                        key: format!("data/{}/{}/{}", run_id, pod_name, path),
                        body: Some(content.into()),
                        ..Default::default()
                    })
                    .await
                    .map_err(|e| format_err!("put_object failed : {}", e))
            })
        })
        .await?;
        self.util_cmd(
            format!(
                "aws s3 cp s3://{}/data/{}/{}/{path} {path}",
                bucket,
                run_id,
                pod_name,
                path = path
            ),
            node,
            "put-file",
        )
        .await
        .map_err(|e| format_err!("aws s3 cp failed : {}", e))?;
        Ok(())
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
        let metadata = node.metadata;
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

async fn try_join_all_limit<O, E, F: Future<Output = std::result::Result<O, E>>>(
    futures: Vec<F>,
) -> std::result::Result<Vec<O>, E> {
    let semaphore = Semaphore::new(32);
    let futures = futures
        .into_iter()
        .map(|f| acquire_and_execute(&semaphore, f));
    try_join_all(futures).await
}

async fn acquire_and_execute<F: TryFuture>(semaphore: &Semaphore, f: F) -> F::Output {
    let _permit = semaphore.acquire().await;
    f.await
}

fn k8s_retry_strategy() -> impl Iterator<Item = Duration> {
    diem_retrier::exp_retry_strategy(1000, 5000, 30)
}
