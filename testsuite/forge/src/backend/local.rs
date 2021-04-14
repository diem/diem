// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{AdminInfo, Coffer, Factory, FullNode, NodeId, PublicInfo, Result, Swarm, Validator};
use diem_config::config::NodeConfig;
use diem_sdk::{
    client::BlockingClient,
    transaction_builder::TransactionFactory,
    types::{chain_id::ChainId, AccountKey, LocalAccount},
};
use diem_swarm::swarm::DiemSwarm;

impl Swarm for LocalSwarm {
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

    fn health_check(&self) -> Result<()> {
        todo!()
    }

    fn validators(&self) -> Box<dyn Iterator<Item = &dyn Validator>> {
        todo!()
    }

    fn validators_mut(&self) -> Box<dyn Iterator<Item = &mut dyn Validator>> {
        todo!()
    }

    fn validator(&self, _id: NodeId) -> &dyn Validator {
        todo!()
    }

    fn validator_mut(&mut self, _id: NodeId) -> &mut dyn Validator {
        todo!()
    }

    fn full_nodes(&self) -> Box<dyn Iterator<Item = &dyn FullNode>> {
        todo!()
    }

    fn full_nodes_mut(&self) -> Box<dyn Iterator<Item = &mut dyn FullNode>> {
        todo!()
    }

    fn full_node(&self) -> &dyn FullNode {
        todo!()
    }

    fn full_node_mut(&mut self) -> &mut dyn FullNode {
        todo!()
    }

    fn add_validator(&mut self, _config: NodeConfig) -> Result<NodeId> {
        todo!()
    }

    fn remove_validator(&mut self, _id: NodeId) -> Result<()> {
        todo!()
    }

    fn add_full_node(&mut self, _config: NodeConfig) -> Result<()> {
        todo!()
    }

    fn remove_full_node(&mut self) -> Result<()> {
        todo!()
    }
}

struct LocalSwarm {
    _swarm: DiemSwarm,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    chain_id: ChainId,
    url: String,
}

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
    fn launch_swarm(&self) -> Box<dyn Swarm> {
        let mut validator_swarm = DiemSwarm::configure_validator_swarm(
            self.diem_node_bin.as_ref(),
            1, // num_validators
            None,
            None,
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

        let url = format!("http://localhost:{}/v1", validator_swarm.get_client_port(0));

        Box::new(LocalSwarm {
            _swarm: validator_swarm,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            chain_id: ChainId::test(),
            url,
        })
    }
}
