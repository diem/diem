// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    dpn_txn_emitter::DPNTxnEmitter, instance::Instance, nft_txn_emitter::NFTTxnEmitter,
    query_sequence_numbers, Mode, TxnEmitter,
};
use anyhow::{bail, format_err, Result};
use async_trait::async_trait;
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
    PrivateKey, Uniform,
};
use diem_rest_client::Client as RestClient;
use diem_sdk::{
    move_types::account_address::AccountAddress,
    transaction_builder::TransactionFactory,
    types::{
        account_config::{
            diem_root_address, testnet_dd_account_address, treasury_compliance_account_address,
            xus_tag,
        },
        chain_id::ChainId,
        transaction::authenticator::AuthenticationKey,
        AccountKey, LocalAccount,
    },
};
use diem_transaction_builder::experimental_stdlib;
use futures::future::join_all;
use itertools::zip;
use rand::seq::SliceRandom;
use rand::{
    rngs::{OsRng, StdRng},
    Rng, SeedableRng,
};
use reqwest::Client;
use std::{
    cmp::min,
    convert::TryFrom,
    time::{Duration, Instant},
};
use termion::color;

const DD_KEY: &str = "dd.key";

pub fn dummy_key_pair() -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    Ed25519PrivateKey::generate_for_testing().into()
}

fn clone(key: &Ed25519PrivateKey) -> Ed25519PrivateKey {
    let serialized: &[u8] = &(key.to_bytes());
    Ed25519PrivateKey::try_from(serialized).unwrap()
}

#[async_trait]
pub trait Cluster: Sync {
    fn instances(&self) -> &Vec<Instance>;
    fn chain_id(&self) -> ChainId;
    fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey>;
    fn random_instance(&self) -> Instance {
        let mut rnd = rand::thread_rng();
        self.instances()
            .choose(&mut rnd)
            .expect("random_validator_instance requires non-empty validator_instances")
            .clone()
    }

    fn all_instances(&self) -> Box<dyn Iterator<Item = &Instance> + '_> {
        Box::new(self.instances().iter())
    }

    fn account_key(&self) -> AccountKey {
        AccountKey::from_private_key(clone(&self.mint_key_pair().private_key))
    }

    async fn diag(&self) -> Result<()>;

    async fn gen_txn_emitter(&self) -> Result<Box<dyn TxnEmitter>>;

    async fn load_account_with_mint_key(
        &self,
        client: &RestClient,
        address: AccountAddress,
    ) -> Result<LocalAccount> {
        let sequence_number = query_sequence_numbers(client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for account {} failed: {}",
                    client,
                    address,
                    e
                )
            })?[0];
        Ok(LocalAccount::new(
            address,
            self.account_key(),
            sequence_number,
        ))
    }

    async fn load_diem_root_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, diem_root_address())
            .await
    }
}

pub struct DPNCluster {
    chain_id: ChainId,
    instances: Vec<Instance>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
    vasp: bool,
}

impl DPNCluster {
    pub fn from_host_port(
        peers: Vec<(String, u32, Option<u32>)>,
        mint_file: &str,
        chain_id: ChainId,
        vasp: bool,
    ) -> Self {
        let http_client = Client::new();
        let instances: Vec<Instance> = peers
            .into_iter()
            .map(|host_port| {
                Instance::new(
                    format!("{}:{}", &host_port.0, host_port.1), /* short_hash */
                    host_port.0,
                    host_port.1,
                    host_port.2,
                    http_client.clone(),
                )
            })
            .collect();

        let mint_key_pair = if vasp {
            dummy_key_pair()
        } else {
            KeyPair::from(generate_key::load_key(mint_file))
        };

        Self {
            vasp,
            instances,
            mint_key_pair,
            chain_id,
        }
    }

    pub async fn load_faucet_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, testnet_dd_account_address())
            .await
    }

    pub async fn load_tc_account(&self, client: &RestClient) -> Result<LocalAccount> {
        self.load_account_with_mint_key(client, treasury_compliance_account_address())
            .await
    }

    pub async fn load_dd_account(&self, client: &RestClient) -> Result<LocalAccount> {
        let mint_key: Ed25519PrivateKey = generate_key::load_key(DD_KEY);
        let account_key = AccountKey::from_private_key(mint_key);
        let address = account_key.authentication_key().derived_address();
        let sequence_number = query_sequence_numbers(client, &[address])
            .await
            .map_err(|e| {
                format_err!(
                    "query_sequence_numbers on {:?} for dd account failed: {}",
                    client,
                    e
                )
            })?[0];
        Ok(LocalAccount::new(address, account_key, sequence_number))
    }
}

#[async_trait]
impl Cluster for DPNCluster {
    fn instances(&self) -> &Vec<Instance> {
        &self.instances
    }

    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        &self.mint_key_pair
    }

    async fn diag(&self) -> Result<()> {
        let client = self.random_instance().rest_client();
        let (treasury_compliance_account, designated_dealer_account) = if self.vasp {
            (
                LocalAccount::generate(&mut rand::rngs::OsRng),
                self.load_dd_account(&client)
                    .await
                    .map_err(|e| format_err!("Failed to get dd account: {}", e))?,
            )
        } else {
            (
                self.load_tc_account(&client).await?,
                self.load_faucet_account(&client).await?,
            )
        };
        let emitter = DPNTxnEmitter::new(
            treasury_compliance_account,
            designated_dealer_account,
            client,
            TransactionFactory::new(self.chain_id()),
            StdRng::from_seed(OsRng.gen()),
        );
        let mut faucet_account: Option<LocalAccount> = None;
        let instances: Vec<_> = self.all_instances().collect();
        for instance in &instances {
            let client = instance.rest_client();
            print!("Getting faucet account sequence number on {}...", instance);
            let account = if self.vasp {
                self.load_dd_account(&client)
                    .await
                    .map_err(|e| format_err!("Failed to get dd account: {}", e))?
            } else {
                self.load_faucet_account(&client).await.map_err(|e| {
                    format_err!("Failed to get faucet account sequence number: {}", e)
                })?
            };
            println!("seq={}", account.sequence_number());
            if let Some(faucet_account) = &faucet_account {
                if account.sequence_number() != faucet_account.sequence_number() {
                    bail!(
                        "Loaded sequence number {}, which is different from seen before {}",
                        account.sequence_number(),
                        faucet_account.sequence_number()
                    );
                }
            } else {
                faucet_account = Some(account);
            }
        }
        let mut faucet_account =
            faucet_account.expect("There is no faucet account set (not expected)");
        let faucet_account_address = faucet_account.address();
        let mode = Mode::DPN {
            gas_price: 0,
            vasp: self.vasp,
        };
        for instance in &instances {
            print!("Submitting txn through {}...", instance);
            let deadline = emitter
                .submit_single_transaction(
                    &instance.rest_client(),
                    &mut faucet_account,
                    &faucet_account_address,
                    10,
                    mode,
                )
                .await
                .map_err(|e| format_err!("Failed to submit txn through {}: {}", instance, e))?;
            println!("seq={}", faucet_account.sequence_number());
            println!(
                "Waiting all full nodes to get to seq {}",
                faucet_account.sequence_number()
            );
            loop {
                let addresses = &[faucet_account_address];
                let clients = instances
                    .iter()
                    .map(|instance| instance.rest_client())
                    .collect::<Vec<_>>();
                let futures = clients
                    .iter()
                    .map(|client| query_sequence_numbers(client, addresses));
                let results = join_all(futures).await;
                let mut all_good = true;
                for (instance, result) in zip(instances.iter(), results) {
                    let seq = result.map_err(|e| {
                        format_err!("Failed to query sequence number from {}: {}", instance, e)
                    })?[0];
                    let ip = instance.ip();
                    let color = if seq != faucet_account.sequence_number() {
                        all_good = false;
                        color::Fg(color::Red).to_string()
                    } else {
                        color::Fg(color::Green).to_string()
                    };
                    print!(
                        "[{}{}:{}{}]  ",
                        color,
                        &ip[..min(ip.len(), 10)],
                        seq,
                        color::Fg(color::Reset)
                    );
                }
                println!();
                if all_good {
                    break;
                }
                if Instant::now() > deadline {
                    bail!("Not all full nodes were updated and transaction expired");
                }
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        }
        println!("Looks like all full nodes are healthy!");
        Ok(())
    }

    async fn gen_txn_emitter(&self) -> Result<Box<dyn TxnEmitter>> {
        let client = self.random_instance().rest_client();
        let (treasury_compliance_account, designated_dealer_account) = if self.vasp {
            (
                LocalAccount::generate(&mut rand::rngs::OsRng),
                self.load_dd_account(&client)
                    .await
                    .map_err(|e| format_err!("Failed to get dd account: {}", e))?,
            )
        } else {
            (
                self.load_tc_account(&client).await?,
                self.load_faucet_account(&client).await?,
            )
        };
        let emitter = DPNTxnEmitter::new(
            treasury_compliance_account,
            designated_dealer_account,
            client,
            TransactionFactory::new(self.chain_id()),
            StdRng::from_seed(OsRng.gen()),
        );
        Ok(Box::new(emitter))
    }
}

pub struct NFTCluster {
    chain_id: ChainId,
    instances: Vec<Instance>,
    mint_key_pair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey>,
}

impl NFTCluster {
    pub fn from_host_port(
        peers: Vec<(String, u32, Option<u32>)>,
        mint_file: &str,
        chain_id: ChainId,
    ) -> Self {
        let http_client = Client::new();
        let instances: Vec<Instance> = peers
            .into_iter()
            .map(|host_port| {
                Instance::new(
                    format!("{}:{}", &host_port.0, host_port.1), /* short_hash */
                    host_port.0,
                    host_port.1,
                    host_port.2,
                    http_client.clone(),
                )
            })
            .collect();

        Self {
            instances,
            chain_id,
            mint_key_pair: KeyPair::from(generate_key::load_key(mint_file)),
        }
    }
}

#[async_trait]
impl Cluster for NFTCluster {
    fn instances(&self) -> &Vec<Instance> {
        self.instances.as_ref()
    }

    fn chain_id(&self) -> ChainId {
        self.chain_id
    }

    fn mint_key_pair(&self) -> &KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        &self.mint_key_pair
    }

    async fn diag(&self) -> Result<()> {
        Ok(())
    }

    async fn gen_txn_emitter(&self) -> Result<Box<dyn TxnEmitter>> {
        let client = self.random_instance().rest_client();
        let mut root_account = self.load_diem_root_account(&client).await?;

        // send a transaction to initialize nft
        let init_nft_txn = root_account.sign_with_transaction_builder(
            TransactionFactory::new(self.chain_id())
                .payload(experimental_stdlib::encode_nft_initialize_script_function()),
        );
        client.submit_and_wait(&init_nft_txn).await?;

        let bars_key = Ed25519PrivateKey::generate(&mut StdRng::from_seed([1; 32]));
        let bars_address = AuthenticationKey::ed25519(&bars_key.public_key()).derived_address();

        let bars_account = LocalAccount::new(bars_address, bars_key, 0);
        // create an account for bars
        let bars_account_creation_txn = root_account.sign_with_transaction_builder(
            TransactionFactory::new(self.chain_id()).payload(
                experimental_stdlib::encode_create_account_script_function(
                    xus_tag(),
                    bars_account.address(),
                    bars_account.authentication_key().prefix().to_vec(),
                ),
            ),
        );

        client.submit_and_wait(&bars_account_creation_txn).await?;

        let emitter = NFTTxnEmitter::new(
            client,
            TransactionFactory::new(self.chain_id()),
            bars_account,
            StdRng::from_seed(OsRng.gen()),
        );
        Ok(Box::new(emitter))
    }
}
