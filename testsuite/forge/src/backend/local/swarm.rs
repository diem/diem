// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    ChainInfo, FullNode, HealthCheckError, LocalNode, LocalVersion, Node, NodeExt, Swarm, SwarmExt,
    Validator, Version,
};
use anyhow::{anyhow, bail, Result};
use diem_config::config::NodeConfig;
use diem_genesis_tool::{fullnode_builder::FullnodeConfig, validator_builder::ValidatorBuilder};
use diem_sdk::{
    crypto::ed25519::Ed25519PrivateKey,
    types::{
        chain_id::ChainId, on_chain_config::experimental_version_access_path,
        transaction::Transaction, waypoint::Waypoint, AccountKey, LocalAccount, PeerId,
    },
};
use std::{
    collections::HashMap,
    convert::TryFrom,
    fs, mem,
    num::NonZeroUsize,
    ops,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};
use tempfile::TempDir;

#[derive(Debug)]
pub enum SwarmDirectory {
    Persistent(PathBuf),
    Temporary(TempDir),
}

impl SwarmDirectory {
    pub fn persist(&mut self) {
        match self {
            SwarmDirectory::Persistent(_) => {}
            SwarmDirectory::Temporary(_) => {
                let mut temp = SwarmDirectory::Persistent(PathBuf::new());
                mem::swap(self, &mut temp);
                let _ = mem::replace(self, temp.into_persistent());
            }
        }
    }

    pub fn into_persistent(self) -> Self {
        match self {
            SwarmDirectory::Temporary(tempdir) => SwarmDirectory::Persistent(tempdir.into_path()),
            SwarmDirectory::Persistent(dir) => SwarmDirectory::Persistent(dir),
        }
    }
}

impl ops::Deref for SwarmDirectory {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        match self {
            SwarmDirectory::Persistent(dir) => dir.deref(),
            SwarmDirectory::Temporary(dir) => dir.path(),
        }
    }
}

impl AsRef<Path> for SwarmDirectory {
    fn as_ref(&self) -> &Path {
        match self {
            SwarmDirectory::Persistent(dir) => dir.as_ref(),
            SwarmDirectory::Temporary(dir) => dir.as_ref(),
        }
    }
}

#[derive(Debug)]
struct ValidatorNetworkAddressEncryptionKey {
    key: diem_sdk::types::network_address::encrypted::Key,
    version: diem_sdk::types::network_address::encrypted::KeyVersion,
}

pub struct LocalSwarmBuilder {
    versions: Arc<HashMap<Version, LocalVersion>>,
    initial_version: Option<Version>,
    template: NodeConfig,
    number_of_validators: NonZeroUsize,
    dir: Option<PathBuf>,
    genesis_modules: Option<Vec<Vec<u8>>>,
}

impl LocalSwarmBuilder {
    pub fn new(versions: Arc<HashMap<Version, LocalVersion>>) -> Self {
        Self {
            versions,
            initial_version: None,
            template: NodeConfig::default_for_validator(),
            number_of_validators: NonZeroUsize::new(1).unwrap(),
            dir: None,
            genesis_modules: None,
        }
    }

    pub fn initial_version(mut self, initial_version: Version) -> Self {
        self.initial_version = Some(initial_version);
        self
    }

    pub fn template(mut self, template: NodeConfig) -> Self {
        self.template = template;
        self
    }

    pub fn number_of_validators(mut self, number_of_validators: NonZeroUsize) -> Self {
        self.number_of_validators = number_of_validators;
        self
    }

    pub fn dir<T: AsRef<Path>>(mut self, dir: T) -> Self {
        self.dir = Some(dir.as_ref().into());
        self
    }

    pub fn genesis_modules(mut self, genesis_modules: Vec<Vec<u8>>) -> Self {
        self.genesis_modules = Some(genesis_modules);
        self
    }

    pub fn build<R>(mut self, rng: R) -> Result<LocalSwarm>
    where
        R: ::rand::RngCore + ::rand::CryptoRng,
    {
        let dir = if let Some(dir) = self.dir {
            if dir.exists() {
                fs::remove_dir_all(&dir)?;
            }
            fs::create_dir_all(&dir)?;
            SwarmDirectory::Persistent(dir)
        } else {
            SwarmDirectory::Temporary(TempDir::new()?)
        };

        // Single node orders blocks too fast which would trigger backpressure and stall for 1 sec
        // which cause flakiness in tests.
        if self.number_of_validators.get() == 1 {
            // this delays empty block by (30-1) * 30ms
            self.template.consensus.mempool_poll_count = 30;
        }

        let (root_keys, genesis, genesis_waypoint, validators) = ValidatorBuilder::new(
            &dir,
            self.genesis_modules
                .unwrap_or_else(|| diem_framework_releases::current_module_blobs().to_vec()),
        )
        .num_validators(self.number_of_validators)
        .template(self.template)
        .diem_version_access_path(experimental_version_access_path())
        .build(rng)?;

        // Get the initial version to start the nodes with, either the one provided or fallback to
        // using the the latest version
        let versions = self.versions;
        let initial_version = self.initial_version.unwrap_or_else(|| {
            versions
                .iter()
                .max_by(|v1, v2| v1.0.cmp(v2.0))
                .unwrap()
                .0
                .clone()
        });
        let version = versions.get(&initial_version).unwrap();

        let validators = validators
            .into_iter()
            .map(|v| {
                let node = LocalNode::new(version.to_owned(), v.name, v.directory)?;
                Ok((node.peer_id(), node))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        let validator_network_address_encryption_key = ValidatorNetworkAddressEncryptionKey {
            key: root_keys.validator_network_address_encryption_key,
            version: root_keys.validator_network_address_encryption_key_version,
        };

        let root_account = LocalAccount::new(
            diem_sdk::types::account_config::diem_root_address(),
            AccountKey::from_private_key(root_keys.root_key),
            0,
        );

        // Designated dealer account is built to have the same private key as the TC account
        let designated_dealer_account = LocalAccount::new(
            diem_sdk::types::account_config::testnet_dd_account_address(),
            AccountKey::from_private_key(
                Ed25519PrivateKey::try_from(root_keys.treasury_compliance_key.to_bytes().as_ref())
                    .unwrap(),
            ),
            0,
        );

        let treasury_compliance_account = LocalAccount::new(
            diem_sdk::types::account_config::treasury_compliance_account_address(),
            AccountKey::from_private_key(root_keys.treasury_compliance_key),
            0,
        );

        Ok(LocalSwarm {
            node_name_counter: validators.len() as u64,
            genesis,
            genesis_waypoint,
            versions,
            validators,
            fullnodes: HashMap::new(),
            dir,
            validator_network_address_encryption_key,
            root_account,
            treasury_compliance_account,
            designated_dealer_account,
            chain_id: ChainId::test(),
        })
    }
}

#[derive(Debug)]
pub struct LocalSwarm {
    node_name_counter: u64,
    genesis: Transaction,
    genesis_waypoint: Waypoint,
    versions: Arc<HashMap<Version, LocalVersion>>,
    validators: HashMap<PeerId, LocalNode>,
    fullnodes: HashMap<PeerId, LocalNode>,
    dir: SwarmDirectory,
    validator_network_address_encryption_key: ValidatorNetworkAddressEncryptionKey,
    root_account: LocalAccount,
    treasury_compliance_account: LocalAccount,
    designated_dealer_account: LocalAccount,
    chain_id: ChainId,
}

impl LocalSwarm {
    pub fn builder(versions: Arc<HashMap<Version, LocalVersion>>) -> LocalSwarmBuilder {
        LocalSwarmBuilder::new(versions)
    }

    pub fn launch(&mut self) -> Result<()> {
        // Start all the validators
        for validator in self.validators.values_mut() {
            validator.start()?;
        }

        // Wait for all of them to startup
        let deadline = Instant::now() + Duration::from_secs(60);
        self.wait_for_startup()?;
        self.wait_for_connectivity(deadline)?;
        self.liveness_check(deadline)?;

        Ok(())
    }

    fn wait_for_startup(&mut self) -> Result<()> {
        let num_attempts = 10;
        let mut done = vec![false; self.validators.len()];
        for i in 0..num_attempts {
            println!("Wait for startup attempt: {} of {}", i, num_attempts);
            for (node, done) in self.validators.values_mut().zip(done.iter_mut()) {
                if *done {
                    continue;
                }
                match node.health_check() {
                    Ok(()) => *done = true,

                    Err(HealthCheckError::Unknown(e)) => {
                        return Err(anyhow!("Diem node '{}' is not running: {}", node.name(), e));
                    }
                    Err(HealthCheckError::NotRunning) => {
                        return Err(anyhow!("Diem node '{}' is not running", node.name()));
                    }
                    Err(HealthCheckError::Failure(e)) => {
                        println!("health check failure: {}", e);
                        continue;
                    }
                }
            }

            // Check if all the nodes have been successfully launched
            if done.iter().all(|status| *status) {
                return Ok(());
            }

            ::std::thread::sleep(::std::time::Duration::from_millis(1000));
        }

        Err(anyhow!("Launching Swarm timed out"))
    }

    pub fn add_validator_fullnode(
        &mut self,
        version: &Version,
        template: NodeConfig,
        validator_peer_id: PeerId,
    ) -> Result<PeerId> {
        let validator = self
            .validators
            .get_mut(&validator_peer_id)
            .ok_or_else(|| anyhow!("no validator with peer_id: {}", validator_peer_id))?;

        if self.fullnodes.contains_key(&validator_peer_id) {
            bail!("VFN for validator {} already configured", validator_peer_id);
        }

        let mut validator_config = validator.config().clone();
        let name = self.node_name_counter.to_string();
        self.node_name_counter += 1;
        let fullnode_config = FullnodeConfig::validator_fullnode(
            name,
            self.dir.as_ref(),
            template,
            &mut validator_config,
            &self.genesis_waypoint,
            &self.genesis,
        )?;

        // Since the validator's config has changed we need to save it
        validator_config.save(validator.config_path())?;
        *validator.config_mut() = validator_config;
        validator.restart()?;

        let version = self.versions.get(version).unwrap();
        let mut fullnode = LocalNode::new(
            version.to_owned(),
            fullnode_config.name,
            fullnode_config.directory,
        )?;

        let peer_id = fullnode.peer_id();
        assert_eq!(peer_id, validator_peer_id);
        fullnode.start()?;

        self.fullnodes.insert(peer_id, fullnode);

        Ok(peer_id)
    }

    fn add_fullnode(&mut self, version: &Version, template: NodeConfig) -> Result<PeerId> {
        let name = self.node_name_counter.to_string();
        self.node_name_counter += 1;
        let fullnode_config = FullnodeConfig::public_fullnode(
            name,
            self.dir.as_ref(),
            template,
            &self.genesis_waypoint,
            &self.genesis,
        )?;

        let version = self.versions.get(version).unwrap();
        let mut fullnode = LocalNode::new(
            version.to_owned(),
            fullnode_config.name,
            fullnode_config.directory,
        )?;

        let peer_id = fullnode.peer_id();
        fullnode.start()?;

        self.fullnodes.insert(peer_id, fullnode);

        Ok(peer_id)
    }

    pub fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    pub fn validator(&self, peer_id: PeerId) -> Option<&LocalNode> {
        self.validators.get(&peer_id)
    }

    pub fn validator_mut(&mut self, peer_id: PeerId) -> Option<&mut LocalNode> {
        self.validators.get_mut(&peer_id)
    }

    pub fn validators(&self) -> impl Iterator<Item = &LocalNode> {
        self.validators.values()
    }

    pub fn validators_mut(&mut self) -> impl Iterator<Item = &mut LocalNode> {
        self.validators.values_mut()
    }

    pub fn fullnode(&self, peer_id: PeerId) -> Option<&LocalNode> {
        self.fullnodes.get(&peer_id)
    }

    pub fn fullnode_mut(&mut self, peer_id: PeerId) -> Option<&mut LocalNode> {
        self.fullnodes.get_mut(&peer_id)
    }

    pub fn dir(&self) -> &Path {
        self.dir.as_ref()
    }
}

impl Drop for LocalSwarm {
    fn drop(&mut self) {
        // If panicking, persist logs
        if std::thread::panicking() {
            eprintln!("Logs located at {}", self.logs_location());
        }
    }
}

impl Swarm for LocalSwarm {
    fn health_check(&mut self) -> Result<()> {
        Ok(())
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

    fn upgrade_validator(&mut self, id: PeerId, version: &Version) -> Result<()> {
        let version = self
            .versions
            .get(version)
            .cloned()
            .ok_or_else(|| anyhow!("Invalid version: {:?}", version))?;
        let validator = self
            .validators
            .get_mut(&id)
            .ok_or_else(|| anyhow!("Invalid id: {}", id))?;
        validator.upgrade(version)
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

    fn add_validator(&mut self, _version: &Version, _template: NodeConfig) -> Result<PeerId> {
        todo!()
    }

    fn remove_validator(&mut self, _id: PeerId) -> Result<()> {
        todo!()
    }

    fn add_full_node(&mut self, version: &Version, template: NodeConfig) -> Result<PeerId> {
        self.add_fullnode(version, template)
    }

    fn remove_full_node(&mut self, id: PeerId) -> Result<()> {
        if let Some(mut fullnode) = self.fullnodes.remove(&id) {
            fullnode.stop();
        }

        Ok(())
    }

    fn versions<'a>(&'a self) -> Box<dyn Iterator<Item = Version> + 'a> {
        Box::new(self.versions.keys().cloned())
    }

    fn chain_info(&mut self) -> ChainInfo<'_> {
        ChainInfo::new(
            &mut self.root_account,
            &mut self.treasury_compliance_account,
            &mut self.designated_dealer_account,
            self.validators
                .values()
                .map(|v| v.json_rpc_endpoint().to_string())
                .next()
                .unwrap(),
            self.validators
                .values()
                .map(|v| v.rest_api_endpoint().to_string())
                .next()
                .unwrap(),
            self.chain_id,
        )
    }

    fn logs_location(&mut self) -> String {
        self.dir.persist();
        self.dir.display().to_string()
    }
}
