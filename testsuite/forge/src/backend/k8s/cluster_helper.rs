// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{get_validators, k8s_retry_strategy, nodes_healthcheck, Result, Validator};
use anyhow::{bail, format_err};
use diem_logger::*;
use hyper::{Client, Uri};
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;
use k8s_openapi::api::batch::v1::Job;
use kube::{api::Api, client::Client as K8sClient, Config};
use rand::Rng;
use rayon::prelude::*;
use regex::Regex;
use rusoto_core::Region;
use rusoto_credential::EnvironmentProvider;
use rusoto_eks::{
    DescribeUpdateRequest, Eks, EksClient, NodegroupScalingConfig, UpdateNodegroupConfigRequest,
};
use rusoto_sts::WebIdentityProvider;
use serde_json::Value;
use std::{
    cmp,
    convert::TryFrom,
    env,
    fs::File,
    io::Write,
    process::{Command, Stdio},
    str,
};
use tempfile::TempDir;
use tokio::runtime::Runtime;

const HELM_BIN: &str = "helm";
const KUBECTL_BIN: &str = "kubectl";
const MAX_NUM_VALIDATORS: usize = 30;
const HEALTH_CHECK_URL: &str = "http://127.0.0.1:8001";

async fn wait_genesis_job(kube_client: &K8sClient, era: &str) -> Result<()> {
    diem_retrier::retry_async(k8s_retry_strategy(), || {
        let jobs: Api<Job> = Api::namespaced(kube_client.clone(), "default");
        Box::pin(async move {
            let job_name = format!("diem-testnet-genesis-e{}", era);
            debug!("Running get job: {}", &job_name);
            let genesis_job = jobs.get_status(&job_name).await.unwrap();
            debug!("Status: {:?}", genesis_job.status);
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

pub fn set_validator_image_tag(
    validator_name: &str,
    image_tag: &str,
    helm_repo: &str,
) -> Result<()> {
    let validator_upgrade_options = [
        "--reuse-values",
        "--history-max",
        "2",
        "--set",
        &format!("imageTag={}", image_tag),
    ];
    upgrade_validator(validator_name, helm_repo, &validator_upgrade_options)
}

pub(crate) fn remove_helm_release(release_name: &str) -> Result<()> {
    let release_uninstall_args = ["uninstall", "--keep-history", release_name];
    println!("{:?}", release_uninstall_args);
    let release_uninstall_output = Command::new(HELM_BIN)
        .stdout(Stdio::inherit())
        .args(&release_uninstall_args)
        .output()
        .expect("failed to helm uninstall valNN");

    let uninstalled_re = Regex::new(r"already deleted").unwrap();
    let uninstall_stderr = String::from_utf8(release_uninstall_output.stderr).unwrap();
    let already_uninstalled = uninstalled_re.is_match(&uninstall_stderr);
    assert!(
        release_uninstall_output.status.success() || already_uninstalled,
        "{}",
        uninstall_stderr
    );
    Ok(())
}

fn helm_release_patch(release_name: &str, version: usize) -> Result<()> {
    // trick helm into letting us upgrade later
    // https://phoenixnap.com/kb/helm-has-no-deployed-releases#ftoc-heading-5
    let helm_patch_args = [
        "patch",
        "secret",
        &format!("sh.helm.release.v1.{}.v{}", release_name, version),
        "--type=merge",
        "-p",
        "{\"metadata\":{\"labels\":{\"status\":\"deployed\"}}}",
    ];
    println!("{:?}", helm_patch_args);
    let helm_patch_output = Command::new(KUBECTL_BIN)
        .stdout(Stdio::inherit())
        .args(&helm_patch_args)
        .output()
        .expect("failed to kubectl patch secret valNN");
    assert!(
        helm_patch_output.status.success(),
        "{}",
        String::from_utf8(helm_patch_output.stderr).unwrap()
    );

    Ok(())
}

fn upgrade_helm_release(release_name: &str, helm_chart: &str, options: &[&str]) -> Result<()> {
    let upgrade_base_args = ["upgrade", release_name, helm_chart];
    let upgrade_args = [&upgrade_base_args, options].concat();
    println!("{:?}", upgrade_args);
    let upgrade_output = Command::new(HELM_BIN)
        .stdout(Stdio::inherit())
        .args(&upgrade_args)
        .output()
        .unwrap_or_else(|_| {
            panic!(
                "failed to helm upgrade release {} with chart {}",
                release_name, helm_chart
            )
        });
    if !upgrade_output.status.success() {
        bail!(format!(
            "Upgrade not completed: {}",
            String::from_utf8(upgrade_output.stderr).unwrap()
        ));
    }

    Ok(())
}

fn upgrade_validator(validator_name: &str, helm_repo: &str, options: &[&str]) -> Result<()> {
    upgrade_helm_release(
        validator_name,
        &format!("{}/diem-validator", helm_repo),
        options,
    )
}

fn upgrade_testnet(helm_repo: &str, options: &[&str]) -> Result<()> {
    upgrade_helm_release("diem", &format!("{}/testnet", helm_repo), options)
}

fn get_helm_status(helm_release_name: &str) -> Result<Value> {
    let status_args = ["status", helm_release_name, "-o", "json"];
    println!("{:?}", status_args);
    let raw_helm_values = Command::new(HELM_BIN)
        .args(&status_args)
        .output()
        .unwrap_or_else(|_| panic!("failed to helm status {}", helm_release_name));

    let helm_values = String::from_utf8(raw_helm_values.stdout).unwrap();
    serde_json::from_str(&helm_values)
        .map_err(|e| format_err!("failed to deserialize helm values: {}", e))
}

fn get_helm_values(helm_release_name: &str) -> Result<Value> {
    let mut v: Value = get_helm_status(helm_release_name)
        .map_err(|e| format_err!("failed to helm get values diem: {}", e))?;
    Ok(v["config"].take())
}

pub fn uninstall_from_k8s_cluster() -> Result<()> {
    // helm uninstall validators while keeping history for later
    (0..MAX_NUM_VALIDATORS).into_par_iter().for_each(|i| {
        remove_helm_release(&format!("val{}", i)).unwrap();
    });
    println!("All validators removed");

    // NOTE: for now, do not remove testnet helm chart since it is more expensive
    // remove_helm_release("diem").unwrap();
    // println!("Testnet release removed");
    Ok(())
}

pub fn clean_k8s_cluster(
    helm_repo: String,
    base_num_validators: usize,
    base_validator_image_tag: String,
    base_testnet_image_tag: String,
    require_validator_healthcheck: bool,
) -> Result<()> {
    assert!(base_num_validators <= MAX_NUM_VALIDATORS);

    let new_era = get_new_era().unwrap();

    let tmp_dir = TempDir::new().expect("Could not create temp dir");

    // prepare for scale up. get the helm values to upgrade later
    (0..base_num_validators).into_par_iter().for_each(|i| {
        let v: Value = get_helm_status(&format!("val{}", i)).unwrap();
        let version = v["version"].as_i64().expect("not a i64") as usize;
        let config = &v["config"];

        let era: &str = &era_to_string(&v["config"]["chain"]["era"]).unwrap();
        assert!(
            !&new_era.eq(era),
            "New era {} is the same as past release era {}",
            new_era,
            era
        );

        // store the helm values for later use
        let file_path = tmp_dir.path().join(format!("val{}_status.json", i));
        println!("Wrote helm values to: {:?}", &file_path);
        let mut file = File::create(file_path).expect("Could not create file in temp dir");
        file.write_all(&config.to_string().into_bytes())
            .expect("Could not write to file");

        helm_release_patch(&format!("val{}", i), version).unwrap();
    });
    println!("All validators prepare for upgrade");

    // upgrade validators in parallel
    (0..base_num_validators).into_par_iter().for_each(|i| {
        let file_path = tmp_dir
            .path()
            .join(format!("val{}_status.json", i))
            .display()
            .to_string();
        let validator_upgrade_options = [
            "-f",
            &file_path,
            "--install",
            "--history-max",
            "2",
            "--set",
            &format!("chain.era={}", &new_era),
            "--set",
            &format!("imageTag={}", &base_validator_image_tag),
        ];
        upgrade_validator(&format!("val{}", i), &helm_repo, &validator_upgrade_options).unwrap();
    });
    println!("All validators upgraded");

    // get testnet values
    let v: Value = get_helm_status("diem").unwrap();
    let version = v["version"].as_i64().expect("not a i64") as usize;
    let config = &v["config"];

    // prep testnet chart for release
    helm_release_patch("diem", version).unwrap();

    // store the helm values for later use
    let file_path = tmp_dir.path().join("diem_status.json");
    println!("Wrote helm values to: {:?}", &file_path);
    let mut file = File::create(file_path).expect("Could not create file in temp dir");
    file.write_all(&config.to_string().into_bytes())
        .expect("Could not write to file");
    let file_path_str = tmp_dir
        .path()
        .join("diem_status.json")
        .display()
        .to_string();
    let testnet_upgrade_options = [
        "-f",
        &file_path_str,
        "--install",
        "--history-max",
        "2",
        "--set",
        &format!("genesis.era={}", &new_era),
        "--set",
        &format!("genesis.numValidators={}", base_num_validators),
        "--set",
        &format!("imageTag={}", &base_testnet_image_tag),
        "--set",
        "monitoring.prometheus.useHttps=false",
    ];

    // upgrade testnet
    upgrade_testnet(&helm_repo, &testnet_upgrade_options)?;

    // wait for genesis to run again, and get the updated validators
    let rt = Runtime::new().unwrap();
    let mut validators = rt.block_on(async {
        let kube_client = create_k8s_client().await;
        wait_genesis_job(&kube_client, &new_era).await.unwrap();
        let vals = get_validators(kube_client.clone(), &base_validator_image_tag)
            .await
            .unwrap();
        vals
    });
    let all_nodes = Box::new(validators.values_mut().map(|v| v as &mut dyn Validator));

    // healthcheck on each of the validators wait until they all healthy
    if require_validator_healthcheck {
        return nodes_healthcheck(all_nodes);
    }
    Ok(())
}

fn get_new_era() -> Result<String> {
    let v: Value = get_helm_values("diem")?;
    println!("{}", v["genesis"]["era"]);
    let chain_era: &str = &era_to_string(&v["genesis"]["era"]).unwrap();

    // get the new era
    let mut rng = rand::thread_rng();
    let new_era: &str = &format!("fg{}", rng.gen::<u32>());
    println!("genesis.era: {} --> {}", chain_era, new_era);
    Ok(new_era.to_string())
}

// sometimes helm will try to interpret era as a number in scientific notation
fn era_to_string(era_value: &Value) -> Result<String> {
    match era_value {
        Value::Number(num) => Ok(format!("{}", num)),
        Value::String(s) => Ok(s.to_string()),
        _ => bail!("Era is not a number {}", era_value),
    }
}

pub async fn create_k8s_client() -> K8sClient {
    let _ = Command::new(KUBECTL_BIN).arg("proxy").spawn();
    let _ = diem_retrier::retry_async(k8s_retry_strategy(), || {
        Box::pin(async move {
            debug!("Running local kube pod healthcheck on {}", HEALTH_CHECK_URL);
            reqwest::get(HEALTH_CHECK_URL).await?.text().await?;
            println!("Local kube pod healthcheck passed");
            Ok::<(), reqwest::Error>(())
        })
    })
    .await;
    let config = Config::new(
        reqwest::Url::parse(HEALTH_CHECK_URL).expect("Failed to parse kubernetes endpoint url"),
    );
    K8sClient::try_from(config).unwrap()
}

fn create_eks_client(auth_with_k8s_env: bool) -> Result<EksClient> {
    let connector = HttpsConnector::new();
    let http_connector: hyper_proxy::ProxyConnector<
        hyper_tls::HttpsConnector<hyper::client::HttpConnector>,
    >;
    if let Ok(proxy_url) = std::env::var("HTTP_PROXY") {
        let proxy = Proxy::new(Intercept::All, proxy_url.parse::<Uri>()?);
        http_connector = ProxyConnector::from_proxy(connector, proxy)?;
    } else {
        http_connector = ProxyConnector::new(connector)?;
    }
    let mut hyper_builder = Client::builder();
    // disabling due to connection closed issue
    hyper_builder.pool_max_idle_per_host(0);
    let dispatcher = rusoto_core::HttpClient::from_builder(hyper_builder, http_connector);
    let eks_client = if auth_with_k8s_env {
        EksClient::new_with(
            dispatcher,
            WebIdentityProvider::from_k8s_env(),
            Region::UsWest2,
        )
    } else {
        EksClient::new_with(dispatcher, EnvironmentProvider::default(), Region::UsWest2)
    };
    Ok(eks_client)
}

fn create_update_nodegroup_config_request(
    cluster_name: &str,
    nodegroup_name: &str,
    nodegroup_scaling_config: NodegroupScalingConfig,
) -> UpdateNodegroupConfigRequest {
    UpdateNodegroupConfigRequest {
        client_request_token: None,
        cluster_name: cluster_name.to_string(),
        labels: None,
        nodegroup_name: nodegroup_name.to_string(),
        scaling_config: Some(nodegroup_scaling_config),
    }
}

async fn submit_update_nodegroup_config_request(
    eks_client: &EksClient,
    cluster_name: &str,
    nodegroup_name: &str,
    nodegroup_scaling_config: NodegroupScalingConfig,
) -> Result<String> {
    let update_nodegroup_request = create_update_nodegroup_config_request(
        cluster_name,
        nodegroup_name,
        nodegroup_scaling_config,
    );
    let update_response = eks_client
        .update_nodegroup_config(update_nodegroup_request)
        .await;
    let update_id = update_response.unwrap().update.unwrap().id.unwrap();
    println!(
        "Created {} nodegroup update request with ID: {}",
        nodegroup_name, update_id
    );
    Ok(update_id)
}

pub fn set_eks_nodegroup_size(
    cluster_name: String,
    num_validators: usize,
    auth_with_k8s_env: bool,
) -> Result<()> {
    // https://github.com/lucdew/rusoto-example/blob/master/src/client.rs
    // Create rusoto client through an http proxy
    let eks_client = create_eks_client(auth_with_k8s_env)?;
    println!("Created rusoto http client");

    // nodegroup scaling factors
    let max_surge = 2; // multiplier for max size
    let num_validators: i64 = num_validators as i64;
    let idle_utilities_size = 5; // keep extra utilities nodes around for forge pods and monitoring
    let validator_scaling = NodegroupScalingConfig {
        desired_size: Some(cmp::max(num_validators * 3, 1)),
        max_size: Some(cmp::max((num_validators * 3 + 1) * max_surge, 1)),
        min_size: Some(cmp::max(num_validators * 3, 1)),
    };
    let utilities_scaling = NodegroupScalingConfig {
        desired_size: Some(cmp::max(num_validators * 3, idle_utilities_size)),
        max_size: Some(cmp::max(
            num_validators * 3 * max_surge,
            idle_utilities_size,
        )),
        min_size: Some(cmp::max(num_validators * 3, idle_utilities_size)),
    };
    let trusted_scaling = NodegroupScalingConfig {
        desired_size: Some(num_validators),
        max_size: Some(cmp::max(num_validators * max_surge, 1)),
        min_size: Some(num_validators),
    };

    // submit the scaling requests
    let rt = Runtime::new()?;
    let validators_update_id = rt.block_on(submit_update_nodegroup_config_request(
        &eks_client,
        &cluster_name,
        "validators",
        validator_scaling,
    ))?;
    let utilities_update_id = rt.block_on(submit_update_nodegroup_config_request(
        &eks_client,
        &cluster_name,
        "utilities",
        utilities_scaling,
    ))?;
    let trusted_update_id = rt.block_on(submit_update_nodegroup_config_request(
        &eks_client,
        &cluster_name,
        "trusted",
        trusted_scaling,
    ))?;

    // wait for nodegroup updates
    let updates: Vec<(&str, &str)> = vec![
        ("validators", &validators_update_id),
        ("utilities", &utilities_update_id),
        ("trusted", &trusted_update_id),
    ];
    updates
        .into_par_iter()
        .for_each(|(nodegroup_name, update_id)| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                diem_retrier::retry_async(k8s_retry_strategy(), || {
                    let client = eks_client.clone();
                    let describe_update_request = DescribeUpdateRequest {
                        addon_name: None,
                        name: cluster_name.clone(),
                        nodegroup_name: Some(nodegroup_name.to_string()),
                        update_id: update_id.to_string(),
                    };
                    Box::pin(async move {
                        let describe_update = client
                            .describe_update(describe_update_request)
                            .await
                            .unwrap()
                            .update
                            .unwrap();
                        if let Some(s) = describe_update.status {
                            match s.as_str() {
                                "Failed" => bail!("Nodegroup update failed"),
                                "Successful" => {
                                    println!(
                                        "{} nodegroup update {} successful!!!",
                                        &nodegroup_name, update_id
                                    );
                                    Ok(())
                                }
                                &_ => {
                                    println!(
                                        "Waiting for {} update {}: {} ...",
                                        &nodegroup_name, update_id, s
                                    );
                                    bail!("Waiting for valid update status")
                                }
                            }
                        } else {
                            bail!("Failed to describe nodegroup update")
                        }
                    })
                })
                .await
            })
            .unwrap();
        });

    Ok(())
}
