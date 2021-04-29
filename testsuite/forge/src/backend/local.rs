// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    AdminInfo, Coffer, Factory, FullNode, Node, NodeId, PublicInfo, Result, Swarm, Validator,
};
use anyhow::ensure;
use debug_interface::NodeDebugClient;
use diem_config::config::NodeConfig;
use diem_genesis_tool::config_builder::FullnodeType;
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::TransactionFactory,
    types::{chain_id::ChainId, AccountKey, LocalAccount, PeerId},
};
use diem_swarm::swarm::{DiemNode, DiemSwarm, HealthStatus};

struct LocalSwarm {
    validator_swarm: DiemSwarm,
    vfn_swarm: DiemSwarm,
    pfn_swarm: DiemSwarm,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    chain_id: ChainId,
    url: String,
}

impl Swarm for LocalSwarm {
    fn health_check(&mut self) -> Result<()> {
        ensure!(
            self.validator_swarm_health_check(),
            "validator swarm health check failed"
        );
        ensure!(
            self.vfn_swarm_health_check(),
            "vfn swarm health check failed"
        );
        ensure!(
            self.pfn_swarm_health_check(),
            "pfn swarm health check failed"
        );
        Ok(())
    }

    fn validators<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn Validator> + 'a> {
        Box::new(
            self.validator_swarm
                .nodes()
                .values()
                .map(|v| v as &'a dyn Validator),
        )
    }

    fn validators_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn Validator> + 'a> {
        Box::new(
            self.validator_swarm
                .nodes()
                .values_mut()
                .map(|v| v as &'a mut dyn Validator),
        )
    }

    fn validator(&self, id: NodeId) -> &dyn Validator {
        self.validator_swarm.get_node(id.as_inner()).unwrap()
    }

    fn validator_mut(&mut self, id: NodeId) -> &mut dyn Validator {
        self.validator_swarm.mut_node(id.as_inner()).unwrap()
    }

    fn full_nodes<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a dyn FullNode> + 'a> {
        Box::new(
            self.vfn_swarm
                .nodes()
                .values()
                .map(|v| v as &'a dyn FullNode)
                .chain(
                    self.pfn_swarm
                        .nodes()
                        .values()
                        .map(|v| v as &'a dyn FullNode),
                ),
        )
    }

    fn full_nodes_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item = &'a mut dyn FullNode> + 'a> {
        Box::new(
            self.vfn_swarm
                .nodes()
                .values_mut()
                .map(|v| v as &'a mut dyn FullNode)
                .chain(
                    self.pfn_swarm
                        .nodes()
                        .values_mut()
                        .map(|v| v as &'a mut dyn FullNode),
                ),
        )
    }

    fn full_node(&self, id: NodeId) -> &dyn FullNode {
        self.pfn_swarm.get_node(id.as_inner()).unwrap()
    }

    fn full_node_mut(&mut self, id: NodeId) -> &mut dyn FullNode {
        self.pfn_swarm.mut_node(id.as_inner()).unwrap()
    }

    fn add_validator(&mut self, _id: NodeId) -> Result<NodeId> {
        todo!()
    }

    fn remove_validator(&mut self, _id: NodeId) -> Result<()> {
        todo!()
    }

    fn add_full_node(&mut self, _id: NodeId) -> Result<()> {
        todo!()
    }

    fn remove_full_node(&mut self, _id: NodeId) -> Result<()> {
        todo!()
    }

    fn admin_info(&mut self) -> AdminInfo<'_> {
        AdminInfo::new(
            &mut self.root_account,
            &mut self.treasury_compliance_account,
            &self.url,
            self.chain_id,
        )
    }

    fn public_info(&mut self) -> PublicInfo<'_> {
        PublicInfo::new(
            &self.url,
            self.chain_id,
            Coffer::TreasuryCompliance {
                transaction_factory: TransactionFactory::new(ChainId::test()),
                json_rpc_client: BlockingClient::new(&self.url),
                treasury_compliance_account: &mut self.treasury_compliance_account,
                designated_dealer_account: &mut self.designated_dealer_account,
            },
        )
    }
}

impl LocalSwarm {
    // TODO need to define some status enum
    fn validator_swarm_health_check(&mut self) -> bool {
        for node in self.validator_swarm.nodes().values_mut() {
            match node.health_check() {
                HealthStatus::Healthy => continue,
                _ => return false,
            }
        }
        true
    }

    fn vfn_swarm_health_check(&mut self) -> bool {
        for node in self.vfn_swarm.nodes().values_mut() {
            match node.health_check() {
                HealthStatus::Healthy => continue,
                _ => return false,
            }
        }
        true
    }

    fn pfn_swarm_health_check(&mut self) -> bool {
        for node in self.pfn_swarm.nodes().values_mut() {
            match node.health_check() {
                HealthStatus::Healthy => continue,
                _ => return false,
            }
        }
        true
    }
}

#[derive(Debug)]
pub enum HealthCheckError {
    Crashed,
    NotStarted,
    RpcFailure(anyhow::Error),
}

impl std::fmt::Display for HealthCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for HealthCheckError {}

impl Node for DiemNode {
    fn peer_id(&self) -> PeerId {
        self.peer_id()
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(self.node_id().parse::<usize>().unwrap())
    }

    fn json_rpc_endpoint(&self) -> String {
        self.config().json_rpc.address.to_string()
    }

    fn debug_client(&self) -> &NodeDebugClient {
        self.debug_client()
    }

    fn get_metric(&mut self, metric_name: &str) -> Option<i64> {
        self.get_metric(metric_name)
    }

    fn config(&self) -> &NodeConfig {
        self.config()
    }

    fn start(&mut self) -> Result<()> {
        self.start()
    }

    fn stop(&mut self) -> Result<()> {
        self.stop();
        Ok(())
    }

    fn clear_storage(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn health_check(&mut self) -> Result<(), HealthCheckError> {
        match self.health_check() {
            HealthStatus::Stopped => Err(HealthCheckError::NotStarted),
            HealthStatus::Crashed(_) => Err(HealthCheckError::Crashed),
            HealthStatus::RpcFailure(e) => Err(HealthCheckError::RpcFailure(e)),
            _ => Ok(()),
        }
    }
}

impl Validator for DiemNode {}

impl FullNode for DiemNode {}

pub struct LocalFactory {
    diem_node_bin: String,
}

impl LocalFactory {
    pub fn new(diem_node_bin: &str) -> Self {
        Self {
            diem_node_bin: diem_node_bin.into(),
        }
    }
}

impl Factory for LocalFactory {
    fn launch_swarm(&self, node_num: usize) -> Box<dyn Swarm> {
        let mut validator_swarm = DiemSwarm::configure_validator_swarm(
            self.diem_node_bin.as_ref(),
            node_num, // num_validators
            None,
            None,
        )
        .unwrap();
        let mut vfn_swarm = DiemSwarm::configure_fn_swarm(
            "ValidatorFullNode",
            self.diem_node_bin.as_ref(),
            None,
            None,
            &validator_swarm.config,
            FullnodeType::ValidatorFullnode,
        )
        .unwrap();
        let mut pfn_swarm = DiemSwarm::configure_fn_swarm(
            "public",
            self.diem_node_bin.as_ref(),
            None,
            None,
            &vfn_swarm.config,
            FullnodeType::PublicFullnode(1),
        )
        .unwrap();

        let key = generate_key::load_key(&validator_swarm.config.diem_root_key_path);
        let account_key = AccountKey::from_private_key(key);
        let root_account = LocalAccount::new(
            diem_sdk::types::account_config::diem_root_address(),
            account_key,
            0,
        );
        let key = generate_key::load_key(&validator_swarm.config.diem_root_key_path);
        let account_key = AccountKey::from_private_key(key);
        let treasury_compliance_account = LocalAccount::new(
            diem_sdk::types::account_config::treasury_compliance_account_address(),
            account_key,
            0,
        );
        let key = generate_key::load_key(&validator_swarm.config.diem_root_key_path);
        let account_key = AccountKey::from_private_key(key);
        let designated_dealer_account = LocalAccount::new(
            diem_sdk::types::account_config::testnet_dd_account_address(),
            account_key,
            0,
        );

        validator_swarm.launch();
        vfn_swarm.launch();
        pfn_swarm.launch();

        let url = format!("http://localhost:{}/v1", validator_swarm.get_client_port(0));

        Box::new(LocalSwarm {
            validator_swarm,
            vfn_swarm,
            pfn_swarm,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            chain_id: ChainId::test(),
            url,
        })
    }
}
