use crate::{
    aws,
    cluster::Cluster,
    cluster_swarm::{cluster_swarm_kube::ClusterSwarmKube, ClusterSwarm},
};
use anyhow::{format_err, Result};
use libra_logger::info;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct ClusterBuilderParams {
    #[structopt(long, default_value = "1")]
    pub fullnodes_per_validator: u32,
    #[structopt(long, use_delimiter = true, default_value = "")]
    pub cfg: Vec<String>,
    #[structopt(long, parse(try_from_str), default_value = "30")]
    pub num_validators: u32,
    #[structopt(long)]
    pub enable_lsr: bool,
    #[structopt(
        long,
        help = "Backend used by lsr. Possible Values are in-memory, on-disk, vault",
        default_value = "vault"
    )]
    pub lsr_backend: String,
}

pub struct ClusterBuilder {
    current_tag: String,
    cluster_swarm: ClusterSwarmKube,
}

impl ClusterBuilder {
    pub fn new(current_tag: String, cluster_swarm: ClusterSwarmKube) -> Self {
        Self {
            current_tag,
            cluster_swarm,
        }
    }

    pub async fn setup_cluster(&self, params: &ClusterBuilderParams) -> Result<Cluster> {
        self.cluster_swarm
            .cleanup()
            .await
            .map_err(|e| format_err!("cleanup on startup failed: {}", e))?;
        let current_tag = &self.current_tag;
        info!(
            "Deploying with {} tag for validators and fullnodes",
            current_tag
        );
        let asg_name = format!(
            "{}-k8s-testnet-validators",
            self.cluster_swarm
                .get_workspace()
                .await
                .expect("Failed to get workspace")
        );
        let mut instance_count =
            params.num_validators + (params.fullnodes_per_validator * params.num_validators);
        if params.enable_lsr {
            if params.lsr_backend == "vault" {
                instance_count += params.num_validators * 2;
            } else {
                instance_count += params.num_validators;
            }
        }
        // First scale down to zero instances and wait for it to complete so that we don't schedule pods on
        // instances which are going into termination state
        aws::set_asg_size(0, 0.0, &asg_name, true, true)
            .await
            .map_err(|err| format_err!("{} scale down failed: {}", asg_name, err))?;
        // Then scale up and bring up new instances
        aws::set_asg_size(instance_count as i64, 5.0, &asg_name, true, false)
            .await
            .map_err(|err| format_err!("{} scale up failed: {}", asg_name, err))?;
        let (validators, fullnodes) = self
            .cluster_swarm
            .spawn_validator_and_fullnode_set(
                params.num_validators,
                params.fullnodes_per_validator,
                params.enable_lsr,
                &params.lsr_backend,
                current_tag,
                params.cfg.as_slice(),
                true,
            )
            .await
            .map_err(|e| format_err!("Failed to spawn_validator_and_fullnode_set: {}", e))?;
        let cluster = Cluster::new(validators, fullnodes);

        info!(
            "Deployed {} validators and {} fns",
            cluster.validator_instances().len(),
            cluster.fullnode_instances().len(),
        );
        Ok(cluster)
    }
}
