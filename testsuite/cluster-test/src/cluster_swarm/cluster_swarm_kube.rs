// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{bail, format_err, Result};
use async_trait::async_trait;

use futures::lock::Mutex;
use kube::{
    api::{Api, PostParams},
    client::APIClient,
    config,
};
use slog_scope::*;

use util::retry;

use crate::cluster_swarm::ClusterSwarm;
use crate::instance::Instance;
use libra_config::config::AdmissionControlConfig;

const DEFAULT_NAMESPACE: &str = "default";

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
    ) -> Vec<u8> {
        let pod_yaml = format!(
            r#"
apiVersion: v1
kind: Pod
metadata:
  name: validator-{index}
  labels:
    app: libra-validator
    libra-node: "true"
  annotations:
    prometheus.io/should_be_scraped: "true"
spec:
  hostNetwork: true
  serviceAccountName: validator
  nodeSelector:
    nodeType: validators
  nodeName: "{node_name}"
  initContainers:
  - name: init
    image: bitnami/kubectl:1.17
    volumeMounts:
    - mountPath: /opt/libra/data
      name: data
    securityContext:
      runAsUser: 0 # To get permissions to write to /opt/libra/data
    command:
    - "bash"
    - "-c"
    - |
      set -x;
      if [[ {delete_data} = true ]]; then
        rm -rf /opt/libra/data/*
      fi
      CFG_SEED_PEER_IP=$(kubectl get pod/validator-0 -o=jsonpath='{{.status.podIP}}');
      while [ -z "${{CFG_SEED_PEER_IP}}" ]; do
        sleep 5;
        CFG_SEED_PEER_IP=$(kubectl get pod/validator-0 -o=jsonpath='{{.status.podIP}}');
        echo "Waiting for pod/validator-0 IP Address";
      done;
      echo -n "${{CFG_SEED_PEER_IP}}" > /opt/libra/data/seed_peer_ip
      until [ $(kubectl get pods -l app=libra-validator | grep ^validator | grep -e Init -e Running | wc -l) = "{num_validators}" ]; do
        sleep 3;
        echo "Waiting for all validator pods to be scheduled";
      done
  containers:
  - name: main
    image: 853397791086.dkr.ecr.us-west-2.amazonaws.com/libra_validator:{image_tag}
    imagePullPolicy: Always
    resources:
      requests:
        cpu: 7800m
    ports:
    - containerPort: 6180
    - containerPort: 6181
    - containerPort: 8000
    - containerPort: 9101
    - containerPort: 6191
    volumeMounts:
    - mountPath: /opt/libra/data
      name: data
    env:
    - name: CFG_NODE_INDEX
      value: "{index}"
    - name: CFG_NUM_VALIDATORS
      value: "{num_validators}"
    - name: CFG_NUM_FULLNODES
      value: "{num_fullnodes}"
    - name: CFG_SEED
      value: "1337133713371337133713371337133713371337133713371337133713371337"
    - name: CFG_FULLNODE_SEED
      value: "2674267426742674267426742674267426742674267426742674267426742674"
    - name: RUST_LOG
      value: "debug"
    - name: RUST_BACKTRACE
      value: "1"
    - name: CFG_OVERRIDES
      value: "{cfg_overrides}"
    - name: MY_POD_IP
      valueFrom:
        fieldRef:
          fieldPath: status.podIP
    command:
      - "bash"
      - "-c"
      - |
        set -x;
        export CFG_LISTEN_ADDR=$MY_POD_IP;
        export CFG_SEED_PEER_IP=$(cat /opt/libra/data/seed_peer_ip);
        exec bash /docker-run-dynamic.sh
  volumes:
  - name: data
    hostPath:
      path: /data
      type: Directory
  affinity:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
          - key: libra-node
            operator: Exists
        topologyKey: "kubernetes.io/hostname"
  terminationGracePeriodSeconds: 5
  tolerations:
  - key: "validators"
    operator: "Exists"
    effect: "NoSchedule"
"#,
            index = index,
            num_validators = num_validators,
            num_fullnodes = num_fullnodes,
            image_tag = image_tag,
            node_name = node_name,
            cfg_overrides = "",
            delete_data = delete_data,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml).unwrap();
        serde_json::to_vec(&pod_spec).unwrap()
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
    ) -> Vec<u8> {
        let pod_yaml = format!(
            r#"
apiVersion: v1
kind: Pod
metadata:
  name: fullnode-{validator_index}-{fullnode_index}
  labels:
    app: libra-fullnode
    libra-node: "true"
  annotations:
    prometheus.io/should_be_scraped: "true"
spec:
  hostNetwork: true
  serviceAccountName: fullnode
  nodeSelector:
    nodeType: validators
  nodeName: "{node_name}"
  initContainers:
  - name: init
    image: bitnami/kubectl:1.17
    volumeMounts:
    - mountPath: /opt/libra/data
      name: data
    securityContext:
      runAsUser: 0 # To get permissions to write to /opt/libra/data
    command:
    - "bash"
    - "-c"
    - |
      set -x;
      if [[ {delete_data} = true ]]; then
        rm -rf /opt/libra/data/*
      fi
      CFG_SEED_PEER_IP=$(kubectl get pod/validator-{validator_index} -o=jsonpath='{{.status.podIP}}');
      while [ -z "${{CFG_SEED_PEER_IP}}" ]; do
        sleep 5;
        CFG_SEED_PEER_IP=$(kubectl get pod/validator-{validator_index} -o=jsonpath='{{.status.podIP}}');
        echo "Waiting for pod/validator-{validator_index} IP Address";
      done;
      echo -n "${{CFG_SEED_PEER_IP}}" > /opt/libra/data/seed_peer_ip
  containers:
  - name: main
    image: 853397791086.dkr.ecr.us-west-2.amazonaws.com/libra_validator:{image_tag}
    imagePullPolicy: Always
    resources:
      requests:
        cpu: 7800m
    ports:
    - containerPort: 6180
    - containerPort: 6181
    - containerPort: 8000
    - containerPort: 9101
    - containerPort: 6191
    volumeMounts:
    - mountPath: /opt/libra/data
      name: data
    env:
    - name: CFG_NUM_VALIDATORS
      value: "{num_validators}"
    - name: CFG_NUM_FULLNODES
      value: "{num_fullnodes}"
    - name: CFG_FULLNODE_INDEX
      value: "{fullnode_index}"
    - name: CFG_SEED
      value: "1337133713371337133713371337133713371337133713371337133713371337"
    - name: CFG_FULLNODE_SEED
      value: "2674267426742674267426742674267426742674267426742674267426742674"
    - name: RUST_LOG
      value: "debug"
    - name: RUST_BACKTRACE
      value: "1"
    - name: CFG_OVERRIDES
      value: "{cfg_overrides}"
    - name: MY_POD_IP
      valueFrom:
        fieldRef:
          fieldPath: status.podIP
    command:
      - "bash"
      - "-c"
      - |
        set -x;
        export CFG_LISTEN_ADDR=$MY_POD_IP;
        export CFG_SEED_PEER_IP=$(cat /opt/libra/data/seed_peer_ip);
        exec bash /docker-run-dynamic-fullnode.sh
  volumes:
  - name: data
    hostPath:
      path: /data
      type: Directory
  affinity:
    podAntiAffinity:
      requiredDuringSchedulingIgnoredDuringExecution:
      - labelSelector:
          matchExpressions:
          - key: libra-node
            operator: Exists
        topologyKey: "kubernetes.io/hostname"
  terminationGracePeriodSeconds: 5
  tolerations:
  - key: "validators"
    operator: "Exists"
    effect: "NoSchedule"
"#,
            fullnode_index = fullnode_index,
            num_fullnodes = num_fullnodes,
            validator_index = validator_index,
            num_validators = num_validators,
            node_name = node_name,
            image_tag = image_tag,
            cfg_overrides = "",
            delete_data = delete_data,
        );
        let pod_spec: serde_yaml::Value = serde_yaml::from_str(&pod_yaml).unwrap();
        serde_json::to_vec(&pod_spec).unwrap()
    }

    async fn get_pod_node_and_ip(&self, pod_name: &str) -> Result<(String, String)> {
        retry::retry_async(retry::fixed_retry_strategy(5000, 60), || {
            let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
            let pod_name = pod_name.to_string();
            Box::pin(async move {
                match pod_api.get(&pod_name).await {
                    Ok(o) => {
                        let node_name = o.spec.node_name.unwrap_or_default();
                        let pod_ip = o.status.unwrap_or_default().pod_ip.unwrap_or_default();
                        if node_name.is_empty() || pod_ip.is_empty() {
                            bail!("node_name not found for pod {}", pod_name)
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
        retry::retry_async(retry::fixed_retry_strategy(2000, 30), || {
            let pod_api = pod_api.clone();
            let name: String = name.to_string();
            Box::pin(async move {
                match pod_api.get(&name).await {
                    Ok(_) => {
                        bail!("Waiting for pod {} to be deleted..", name);
                    }
                    Err(kube::Error::Api(ae)) => {
                        if ae.code == ERROR_NOT_FOUND {
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
    async fn upsert_validator(
        &self,
        index: u32,
        num_validators: u32,
        num_fullnodes: u32,
        image_tag: &str,
        delete_data: bool,
    ) -> Result<()> {
        debug!("upsert_validator");
        let pod_name = format!("validator-{}", index);
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            debug!("Found pod, deleting it");
            self.delete_pod(&pod_name).await?;
        }
        let node_name = if let Some(instance) = self.validator_to_node.lock().await.get(&index) {
            instance.k8s_node().to_string()
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
                ),
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
            let ac_port = AdmissionControlConfig::default().address.port() as u32;
            let instance = Instance::new_k8s(pod_name, pod_ip, ac_port, node_name);
            self.validator_to_node.lock().await.insert(index, instance);
        }
        Ok(())
    }

    async fn delete_validator(&self, index: u32) -> Result<()> {
        debug!("delete_validator");
        let pod_name = format!("validator-{}", index);
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
        debug!("upsert_fullnode");
        let pod_name = format!("fullnode-{}-{}", validator_index, fullnode_index);
        let pod_api = Api::v1Pod(self.client.clone()).within(DEFAULT_NAMESPACE);
        if pod_api.get(&pod_name).await.is_ok() {
            debug!("Found pod, deleting it");
            self.delete_pod(&pod_name).await?;
        }
        let node_name = if let Some(instance) = self
            .fullnode_to_node
            .lock()
            .await
            .get(&(validator_index, fullnode_index))
        {
            instance.k8s_node().to_string()
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
                ),
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
            let ac_port = AdmissionControlConfig::default().address.port() as u32;
            let instance = Instance::new_k8s(pod_name, pod_ip, ac_port, node_name);
            self.fullnode_to_node
                .lock()
                .await
                .insert((validator_index, fullnode_index), instance);
        }
        Ok(())
    }

    async fn delete_fullnode(&self, fullnode_index: u32, validator_index: u32) -> Result<()> {
        debug!("delete_fullnode");
        let pod_name = format!("fullnode-{}-{}", validator_index, fullnode_index);
        self.delete_pod(&pod_name).await
    }
}
