// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    backend::k8s::node::K8sNode, query_sequence_numbers, ChainInfo, FullNode, Node, Result, Swarm,
    Validator, Version,
};
use anyhow::{bail, format_err};
use diem_logger::*;
use diem_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    types::{
        chain_id::{ChainId, NamedChain},
        AccountKey, LocalAccount, PeerId,
    },
};
use k8s_openapi::api::{batch::v1::Job, core::v1::Service};
use kube::{
    api::{Api, ListParams},
    client::Client as K8sClient,
    Config,
};
use rand::Rng;
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use std::{
    collections::HashMap,
    convert::TryFrom,
    fs::File,
    io::Write,
    process::{Command, Stdio},
    str,
};
use tempfile::TempDir;
use tokio::{runtime::Runtime, time::Duration};

const HEALTH_CHECK_URL: &str = "http://127.0.0.1:8001";
const KUBECTL_BIN: &str = "kubectl";
const HELM_BIN: &str = "helm";
const JSON_RPC_PORT: u32 = 80;
const VALIDATOR_LB: &str = "validator-fullnode-lb";
const MAX_NUM_VALIDATORS: usize = 30;

pub struct K8sSwarm {
    validators: HashMap<PeerId, K8sNode>,
    fullnodes: HashMap<PeerId, K8sNode>,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    kube_client: K8sClient,
    runtime: Runtime,
    helm_repo: String,
    pub chain_id: ChainId,
}

impl K8sSwarm {
    pub async fn new(
        root_key: &[u8],
        treasury_compliance_key: &[u8],
        helm_repo: &str,
    ) -> Result<Self> {
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
        let fullnodes = HashMap::new();
        let validators = get_validators(kube_client.clone()).await?;

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

        Ok(Self {
            validators,
            fullnodes,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            kube_client,
            runtime: Runtime::new().unwrap(),
            chain_id: ChainId::new(NamedChain::DEVNET.id()),
            helm_repo: helm_repo.to_string(),
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

impl Drop for K8sSwarm {
    // When the K8sSwarm struct goes out of scope we need to wipe the chain state
    fn drop(&mut self) {
        clean_k8s_cluster(self.helm_repo.clone(), self.validators.len())
            .map_err(|err| format_err!("Failed to clean k8s cluster with new genesis: {}", err))
            .unwrap();
    }
}

impl Swarm for K8sSwarm {
    fn health_check(&mut self) -> Result<()> {
        self.runtime.block_on(async {
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

    fn upgrade_validator(&mut self, _id: PeerId, _version: &Version) -> Result<()> {
        todo!()
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

    fn remove_validator(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn add_full_node(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn remove_full_node(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a> {
        todo!()
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
        Ok(Self { name, host_ip })
    }
}

async fn list_services(client: K8sClient) -> Result<Vec<KubeService>> {
    let node_api: Api<Service> = Api::all(client);
    let lp = ListParams::default();
    let services = node_api.list(&lp).await?.items;
    services.into_iter().map(KubeService::try_from).collect()
}

async fn get_validators(client: K8sClient) -> Result<HashMap<PeerId, K8sNode>> {
    let services = list_services(client).await?;
    services
        .into_iter()
        .filter(|s| s.name.contains(VALIDATOR_LB))
        .map(|s| {
            let node_id = parse_node_id(&s.name).expect("error to parse node id");
            let node = K8sNode {
                name: format!("val-{}", node_id),
                // TODO: fetch this from running node
                peer_id: PeerId::random(),
                node_id,
                ip: s.host_ip.clone(),
                port: JSON_RPC_PORT,
                dns: s.name,
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

async fn wait_genesis_job(kube_client: &K8sClient, era: usize) -> Result<(), anyhow::Error> {
    diem_retrier::retry_async(k8s_retry_strategy(), || {
        let jobs: Api<Job> = Api::namespaced(kube_client.clone(), "default");
        Box::pin(async move {
            let job_name = format!("diem-testnet-genesis-e{}", era);
            println!("Running get job: {}", &job_name);
            let genesis_job = jobs.get_status(&job_name).await.unwrap();
            println!("Status: {:?}", genesis_job.status);
            let status = genesis_job.status.unwrap();
            match status.succeeded {
                Some(1) => {
                    println!("Genesis job completed");
                    Ok(())
                }
                _ => bail!("Genesis job not completed"),
            }
        })
    })
    .await
}

pub fn clean_k8s_cluster(
    helm_repo: String,
    base_num_validators: usize,
) -> Result<(), anyhow::Error> {
    assert!(base_num_validators <= MAX_NUM_VALIDATORS);

    // get the previous chain era
    let raw_helm_values = Command::new(HELM_BIN)
        .arg("get")
        .arg("values")
        .arg("diem")
        .arg("--output")
        .arg("json")
        .output()
        .unwrap();

    // parse genesis
    let helm_values = String::from_utf8(raw_helm_values.stdout).unwrap();
    let v: Value = serde_json::from_str(&helm_values).unwrap();
    let chain_era = v["genesis"]["era"].as_i64().expect("not a i64") as usize;
    let curr_num_validators = v["genesis"]["numValidators"].as_i64().expect("not a i64") as usize;

    // get the new era
    let mut rng = rand::thread_rng();
    let new_era = rng.gen::<u32>() as usize;
    println!("genesis.era: {} --> {}", chain_era, new_era);
    println!(
        "genesis.numValidators: {} --> {}",
        curr_num_validators, base_num_validators
    );

    // scale down. helm uninstall validators while keeping history for later
    (0..MAX_NUM_VALIDATORS).into_par_iter().for_each(|i| {
        let validator_uninstall_args = ["uninstall", "--keep-history", &format!("val{}", i)];
        println!("{:?}", validator_uninstall_args);
        let validator_uninstall_output = Command::new(HELM_BIN)
            .stdout(Stdio::inherit())
            .args(&validator_uninstall_args)
            .output()
            .expect("failed to helm uninstall valNN");

        let uninstalled_re = Regex::new(r"already deleted").unwrap();
        let uninstall_stderr = String::from_utf8(validator_uninstall_output.stderr).unwrap();
        let already_uninstalled = uninstalled_re.is_match(&uninstall_stderr);
        assert!(
            validator_uninstall_output.status.success() || already_uninstalled,
            "{}",
            uninstall_stderr
        );
    });

    let tmp_dir = TempDir::new().expect("Could not create temp dir");

    // prepare for scale up. get the helm values to upgrade later
    (0..base_num_validators).into_par_iter().for_each(|i| {
        let validator_status_args = ["status", &format!("val{}", i), "-o", "json"];
        println!("{:?}", validator_status_args);
        let validator_status_output = Command::new(HELM_BIN)
            .args(&validator_status_args)
            .output()
            .expect("failed to helm status valNN");
        assert!(
            validator_status_output.status.success(),
            "{}",
            String::from_utf8(validator_status_output.stderr).unwrap()
        );
        let helm_status = String::from_utf8(validator_status_output.stdout).unwrap();
        let v: Value = serde_json::from_str(&helm_status).unwrap();
        let version = v["version"].as_i64().expect("not a i64") as usize;
        let config = &v["config"];

        let era = v["config"]["chain"]["era"].as_i64().expect("not a i64") as usize;
        assert!(new_era != era, "Era is the same as past release");

        // store the helm values for later use
        let file_path = tmp_dir.path().join(format!("val{}_status.json", i));
        println!("Wrote helm values to: {:?}", &file_path);
        let mut file = File::create(file_path).expect("Could not create file in temp dir");
        file.write_all(&config.to_string().into_bytes())
            .expect("Could not write to file");

        // trick helm into letting us upgrade later
        // https://phoenixnap.com/kb/helm-has-no-deployed-releases#ftoc-heading-5
        let validator_helm_patch_args = [
            "patch",
            "secret",
            &format!("sh.helm.release.v1.val{}.v{}", i, version),
            "--type=merge",
            "-p",
            "{\"metadata\":{\"labels\":{\"status\":\"deployed\"}}}",
        ];
        println!("{:?}", validator_helm_patch_args);
        let validator_helm_patch_output = Command::new(KUBECTL_BIN)
            .stdout(Stdio::inherit())
            .args(&validator_helm_patch_args)
            .output()
            .expect("failed to kubectl patch secret valNN");
        assert!(
            validator_helm_patch_output.status.success(),
            "{}",
            String::from_utf8(validator_helm_patch_output.stderr).unwrap()
        );
    });

    // upgrade validators in parallel
    (0..base_num_validators).into_par_iter().for_each(|i| {
        let rt_thread = Runtime::new().unwrap();
        let file_path = tmp_dir
            .path()
            .join(format!("val{}_status.json", i))
            .display()
            .to_string();
        let validator_upgrade_args = [
            "upgrade",
            &format!("val{}", i),
            &format!("{}/diem-validator", helm_repo),
            "-f",
            &file_path,
            "--install",
            "--history-max",
            "2",
            "--set",
            &format!("chain.era={}", new_era),
        ];
        rt_thread
            .block_on(async {
                diem_retrier::retry_async(k8s_retry_strategy(), || {
                    Box::pin(async move {
                        println!("{:?}", validator_upgrade_args);
                        let validator_upgrade_output = Command::new(HELM_BIN)
                            .stdout(Stdio::inherit())
                            .args(&validator_upgrade_args)
                            .output()
                            .expect("failed to helm upgrade valNN");
                        if validator_upgrade_output.status.success() {
                            return Ok(());
                        }
                        bail!(format!(
                            "Upgrade not completed: {}",
                            String::from_utf8(validator_upgrade_output.stderr).unwrap()
                        ));
                    })
                })
                .await
            })
            .expect("Block on helm upgrade failed");
    });

    // upgrade testnet
    let testnet_upgrade_args = [
        "upgrade",
        "diem",
        &format!("{}/testnet", helm_repo),
        "--reuse-values",
        "--history-max",
        "2",
        "--set",
        &format!("genesis.era={}", new_era),
        "--set",
        &format!("genesis.numValidators={}", base_num_validators),
    ];
    println!("{:?}", testnet_upgrade_args);
    let testnet_upgrade_output = Command::new(HELM_BIN)
        .stdout(Stdio::inherit())
        .args(&testnet_upgrade_args)
        .output()
        .expect("failed to helm upgrade diem");
    assert!(
        testnet_upgrade_output.status.success(),
        "{}",
        String::from_utf8(testnet_upgrade_output.stderr).unwrap()
    );

    let rt = Runtime::new().unwrap();

    // wait for genesis to run again, and get the updated validators
    let mut validators = rt.block_on(async {
        let mut kube_proxy = Command::new(KUBECTL_BIN).arg("proxy").spawn().unwrap();
        diem_retrier::retry_async(k8s_retry_strategy(), || {
            Box::pin(async move {
                debug!("Running local kube pod healthcheck on {}", HEALTH_CHECK_URL);
                reqwest::get(HEALTH_CHECK_URL).await?.text().await?;
                println!("Local kube pod healthcheck passed");
                Ok::<(), reqwest::Error>(())
            })
        })
        .await
        .expect("Failed kube proxy healthcheck");
        let config = Config::new(
            reqwest::Url::parse(HEALTH_CHECK_URL).expect("Failed to parse kubernetes endpoint url"),
        );
        let kube_client = K8sClient::try_from(config).unwrap();
        wait_genesis_job(&kube_client, new_era).await.unwrap();
        let vals = get_validators(kube_client.clone()).await.unwrap();
        kube_proxy.kill().unwrap();
        vals
    });

    // healthcheck on each of the validators
    let unhealthy_validators = validators
        .iter_mut()
        .filter_map(|(_, val)| {
            let val_name = val.name.clone();
            println!("Attempting health check: {}", val_name);
            // perform healthcheck with retry, returning unhealthy
            let check = diem_retrier::retry(k8s_retry_strategy(), || match val.health_check() {
                Ok(_) => {
                    println!("Validator {} healthy", val_name);
                    Ok(())
                }
                Err(ref x) => {
                    println!("Validator {} unhealthy: {}", val_name, x);
                    Err(())
                }
            });
            if check.is_err() {
                return Some(val);
            }
            None
        })
        .collect::<Vec<_>>();
    if !unhealthy_validators.is_empty() {
        bail!(
            "Unhealthy validators after cleanup: {:?}",
            unhealthy_validators
        );
    } else {
        println!("All validators healthy after cleanup!");
    }
    Ok(())
}
