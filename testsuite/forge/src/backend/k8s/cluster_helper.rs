// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{get_validators, k8s_retry_strategy, nodes_healthcheck, Result, Validator};
use anyhow::{bail, format_err};
use diem_logger::*;
use k8s_openapi::api::batch::v1::Job;
use kube::{api::Api, client::Client as K8sClient, Config};
use rand::Rng;
use rayon::prelude::*;
use regex::Regex;
use serde_json::Value;
use std::{
    convert::TryFrom,
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

pub(crate) fn remove_validator(validator_name: &str) -> Result<()> {
    let validator_uninstall_args = ["uninstall", "--keep-history", validator_name];
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
    Ok(())
}

fn upgrade_validator(validator_name: &str, helm_repo: &str, options: &[&str]) -> Result<()> {
    let validator_upgrade_base_args = [
        "upgrade",
        validator_name,
        &format!("{}/diem-validator", helm_repo),
    ];
    let validator_upgrade_args = [&validator_upgrade_base_args, options].concat();
    println!("{:?}", validator_upgrade_args);
    let validator_upgrade_output = Command::new(HELM_BIN)
        .stdout(Stdio::inherit())
        .args(&validator_upgrade_args)
        .output()
        .expect("failed to helm upgrade valNN");
    if !validator_upgrade_output.status.success() {
        bail!(format!(
            "Upgrade not completed: {}",
            String::from_utf8(validator_upgrade_output.stderr).unwrap()
        ));
    }

    Ok(())
}

fn upgrade_testnet(
    helm_repo: &str,
    era_num: &str,
    num_validator: usize,
    image_tag: &str,
) -> Result<()> {
    // upgrade testnet
    let testnet_upgrade_args = [
        "upgrade",
        "diem",
        &format!("{}/testnet", helm_repo),
        "--reuse-values",
        "--history-max",
        "2",
        "--set",
        &format!("genesis.era={}", era_num),
        "--set",
        &format!("genesis.numValidators={}", num_validator),
        "--set",
        &format!("imageTag={}", image_tag),
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
    println!("Testnet upgraded");

    Ok(())
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

pub fn clean_k8s_cluster(
    helm_repo: String,
    base_num_validators: usize,
    base_validator_image_tag: String,
    base_testnet_image_tag: String,
    require_validator_healthcheck: bool,
) -> Result<()> {
    assert!(base_num_validators <= MAX_NUM_VALIDATORS);

    let new_era = get_new_era().unwrap();

    // scale down. helm uninstall validators while keeping history for later
    (0..MAX_NUM_VALIDATORS).into_par_iter().for_each(|i| {
        remove_validator(&format!("val{}", i)).unwrap();
    });
    println!("All validators removed");

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

    // upgrade testnet
    upgrade_testnet(
        &helm_repo,
        &new_era,
        base_num_validators,
        &base_testnet_image_tag,
    )?;

    // wait for genesis to run again, and get the updated validators
    let rt = Runtime::new().unwrap();
    let mut validators = rt.block_on(async {
        let kube_client = k8s_client().await;
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

pub async fn k8s_client() -> K8sClient {
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
