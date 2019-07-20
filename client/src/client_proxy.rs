// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{commands::*, grpc_client::GRPCClient, AccountData, AccountStatus};
use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use config::trusted_peers::TrustedPeersConfig;
use crypto::signing::KeyPair;
use failure::prelude::*;
use futures::{future::Future, stream::Stream};
use hyper;
use libra_wallet::{io_utils, wallet_library::WalletLibrary};
use logger::prelude::*;
use nextgen_crypto::ed25519::Ed25519PublicKey;
use num_traits::{
    cast::{FromPrimitive, ToPrimitive},
    identities::Zero,
};
use proto_conv::{FromProtoBytes, IntoProto};
use rust_decimal::Decimal;
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt,
    fs::{self, File},
    io::{stdout, Read, Write},
    path::Path,
    process::Command,
    str::FromStr,
    sync::Arc,
    thread, time,
};
use tempfile::{NamedTempFile, TempPath};
use tokio::{self, runtime::Runtime};
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    account_config::{
        account_received_event_path, account_sent_event_path, association_address,
        get_account_resource_or_default, AccountResource,
    },
    account_state_blob::{AccountStateBlob, AccountStateWithProof},
    contract_event::{ContractEvent, EventWithProof},
    transaction::{Program, RawTransaction, SignedTransaction, Version},
    transaction_helpers::{create_signed_txn, TransactionSigner},
    validator_verifier::ValidatorVerifier,
};

const CLIENT_WALLET_MNEMONIC_FILE: &str = "client.mnemonic";
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 100_000;
const TX_EXPIRATION: i64 = 100;

/// Enum used for error formatting.
#[derive(Debug)]
enum InputType {
    Bool,
    UnsignedInt,
    Usize,
}

/// Account data is stored in a map and referenced by an index.
#[derive(Debug)]
pub struct AddressAndIndex {
    /// Address of the account.
    pub address: AccountAddress,
    /// The account_ref_id of this account in client.
    pub index: usize,
}

/// Account is represented either as an entry into accounts vector or as an address.
pub enum AccountEntry {
    /// Index into client.accounts
    Index(usize),
    /// Address of the account
    Address(AccountAddress),
}

/// Used to return the sequence and sender account index submitted for a transfer
pub struct IndexAndSequence {
    /// Index/key of the account in TestClient::accounts vector.
    pub account_index: AccountEntry,
    /// Sequence number of the account.
    pub sequence_number: u64,
}

/// Proxy handling CLI commands/inputs.
pub struct ClientProxy {
    /// client for admission control interface.
    pub client: GRPCClient,
    /// Created accounts.
    pub accounts: Vec<AccountData>,
    /// Address to account_ref_id map.
    address_to_ref_id: HashMap<AccountAddress, usize>,
    /// Host that operates a faucet service
    faucet_server: String,
    /// Account used for mint operations.
    pub faucet_account: Option<AccountData>,
    /// Wallet library managing user accounts.
    wallet: WalletLibrary,
    /// Whether to sync with validator on account creation.
    sync_on_wallet_recovery: bool,
    /// temp files (alive for duration of program)
    temp_files: Vec<TempPath>,
}

impl ClientProxy {
    /// Construct a new TestClient.
    pub fn new(
        host: &str,
        ac_port: &str,
        validator_set_file: &str,
        faucet_account_file: &str,
        sync_on_wallet_recovery: bool,
        faucet_server: Option<String>,
        mnemonic_file: Option<String>,
    ) -> Result<Self> {
        let validators_config = TrustedPeersConfig::load_config(Path::new(validator_set_file));
        let validators = validators_config.get_trusted_consensus_peers();
        ensure!(
            !validators.is_empty(),
            "Not able to load validators from trusted peers config!"
        );
        // Total 3f + 1 validators, 2f + 1 correct signatures are required.
        // If < 4 validators, all validators have to agree.
        let validator_pubkeys: HashMap<AccountAddress, Ed25519PublicKey> = validators
            .into_iter()
            .map(|(key, value)| (key, value.into()))
            .collect();
        let validator_verifier = Arc::new(ValidatorVerifier::new(validator_pubkeys));
        let client = GRPCClient::new(host, ac_port, validator_verifier)?;

        let accounts = vec![];

        // If we have a faucet account file, then load it to get the keypair
        let faucet_account = if faucet_account_file.is_empty() {
            None
        } else {
            let faucet_account_keypair: KeyPair =
                ClientProxy::load_faucet_account_file(faucet_account_file);
            let faucet_account_data = Self::get_account_data_from_address(
                &client,
                association_address(),
                true,
                Some(KeyPair::new(faucet_account_keypair.private_key().clone())),
            )?;
            // Load the keypair from file
            Some(faucet_account_data)
        };

        let faucet_server = match faucet_server {
            Some(server) => server.to_string(),
            None => host.replace("ac", "faucet"),
        };

        let address_to_ref_id = accounts
            .iter()
            .enumerate()
            .map(|(ref_id, acc_data): (usize, &AccountData)| (acc_data.address, ref_id))
            .collect::<HashMap<AccountAddress, usize>>();

        Ok(ClientProxy {
            client,
            accounts,
            address_to_ref_id,
            faucet_server,
            faucet_account,
            wallet: Self::get_libra_wallet(mnemonic_file)?,
            sync_on_wallet_recovery,
            temp_files: vec![],
        })
    }

    fn get_account_ref_id(&self, sender_account_address: &AccountAddress) -> Result<usize> {
        Ok(*self
            .address_to_ref_id
            .get(&sender_account_address)
            .ok_or_else(|| {
                format_err!(
                    "Unable to find existing managing account by address: {}, to see all existing \
                     accounts, run: 'account list'",
                    sender_account_address
                )
            })?)
    }

    /// Returns the account index that should be used by user to reference this account
    pub fn create_next_account(&mut self, sync_with_validator: bool) -> Result<AddressAndIndex> {
        let (address, _) = self.wallet.new_address()?;

        let account_data =
            Self::get_account_data_from_address(&self.client, address, sync_with_validator, None)?;

        Ok(self.insert_account_data(account_data))
    }

    /// Print index and address of all accounts.
    pub fn print_all_accounts(&self) {
        if self.accounts.is_empty() {
            println!("No user accounts");
        } else {
            for (ref index, ref account) in self.accounts.iter().enumerate() {
                println!(
                    "User account index: {}, address: {}, sequence number: {}, status: {:?}",
                    index,
                    hex::encode(&account.address),
                    account.sequence_number,
                    account.status,
                );
            }
        }

        if let Some(faucet_account) = &self.faucet_account {
            println!(
                "Faucet account address: {}, sequence_number: {}, status: {:?}",
                hex::encode(&faucet_account.address),
                faucet_account.sequence_number,
                faucet_account.status,
            );
        }
    }

    /// Clone all accounts held in the client.
    pub fn copy_all_accounts(&self) -> Vec<AccountData> {
        self.accounts.clone()
    }

    /// Set the account of this client instance.
    pub fn set_accounts(&mut self, accounts: Vec<AccountData>) -> Vec<AddressAndIndex> {
        self.accounts.clear();
        self.address_to_ref_id.clear();
        let mut ret = vec![];
        for data in accounts {
            ret.push(self.insert_account_data(data));
        }
        ret
    }

    /// Get balance from validator for the account specified.
    pub fn get_balance(&mut self, space_delim_strings: &[&str]) -> Result<String> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for getting balance"
        );
        let address = self.get_account_address_from_parameter(space_delim_strings[1])?;
        self.get_account_resource_and_update(address).map(|res| {
            let whole_num = res.balance() / 1_000_000;
            let remainder = res.balance() % 1_000_000;
            format!("{}.{:0>6}", whole_num.to_string(), remainder.to_string())
        })
    }

    /// Get the latest sequence number from validator for the account specified.
    pub fn get_sequence_number(&mut self, space_delim_strings: &[&str]) -> Result<u64> {
        ensure!(
            space_delim_strings.len() == 2 || space_delim_strings.len() == 3,
            "Invalid number of arguments for getting sequence number"
        );
        let address = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let sequence_number = self
            .get_account_resource_and_update(address)?
            .sequence_number();

        let reset_sequence_number = if space_delim_strings.len() == 3 {
            parse_bool(space_delim_strings[2]).map_err(|error| {
                format_parse_data_error(
                    "reset_sequence_number",
                    InputType::Bool,
                    space_delim_strings[2],
                    error,
                )
            })?
        } else {
            false
        };
        if reset_sequence_number {
            let mut account = self.mut_account_from_parameter(space_delim_strings[1])?;
            // Set sequence_number to latest one.
            account.sequence_number = sequence_number;
        }
        Ok(sequence_number)
    }

    /// Mints coins for the receiver specified.
    pub fn mint_coins(&mut self, space_delim_strings: &[&str], is_blocking: bool) -> Result<()> {
        ensure!(
            space_delim_strings.len() == 3,
            "Invalid number of arguments for mint"
        );
        let receiver = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let num_coins = Self::convert_to_micro_libras(space_delim_strings[2])?;

        match self.faucet_account {
            Some(_) => self.mint_coins_with_local_faucet_account(&receiver, num_coins, is_blocking),
            None => self.mint_coins_with_faucet_service(&receiver, num_coins, is_blocking),
        }
    }

    /// Waits for the next transaction for a specific address and prints it
    pub fn wait_for_transaction(&mut self, account: AccountAddress, sequence_number: u64) {
        let mut max_iterations = 5000;
        print!("[waiting ");
        loop {
            stdout().flush().unwrap();
            max_iterations -= 1;

            if let Ok(Some((_, Some(events)))) =
                self.client
                    .get_txn_by_acc_seq(account, sequence_number - 1, true)
            {
                println!("transaction is stored!");
                if events.is_empty() {
                    println!("but it didn't emit any events (failed execution)");
                }
                break;
            } else if max_iterations == 0 {
                panic!("wait_for_transaction timeout");
            } else {
                print!(".");
            }
            thread::sleep(time::Duration::from_millis(10));
        }
    }

    /// Transfer num_coins from sender account to receiver. If is_blocking = true,
    /// it will keep querying validator till the sequence number is bumped up in validator.
    pub fn transfer_coins_int(
        &mut self,
        sender_account_ref_id: usize,
        receiver_address: &AccountAddress,
        num_coins: u64,
        gas_unit_price: Option<u64>,
        max_gas_amount: Option<u64>,
        is_blocking: bool,
    ) -> Result<IndexAndSequence> {
        let sender_address;
        let sender_sequence;
        {
            let sender = self.accounts.get(sender_account_ref_id).ok_or_else(|| {
                format_err!("Unable to find sender account: {}", sender_account_ref_id)
            })?;

            let program = vm_genesis::encode_transfer_program(&receiver_address, num_coins);
            let req = self.create_submit_transaction_req(
                program,
                sender,
                max_gas_amount, /* max_gas_amount */
                gas_unit_price, /* gas_unit_price */
            )?;
            let sender_mut = self
                .accounts
                .get_mut(sender_account_ref_id)
                .ok_or_else(|| {
                    format_err!("Unable to find sender account: {}", sender_account_ref_id)
                })?;
            self.client.submit_transaction(Some(sender_mut), &req)?;
            sender_address = sender_mut.address;
            sender_sequence = sender_mut.sequence_number;
        }

        if is_blocking {
            self.wait_for_transaction(sender_address, sender_sequence);
        }

        Ok(IndexAndSequence {
            account_index: AccountEntry::Index(sender_account_ref_id),
            sequence_number: sender_sequence - 1,
        })
    }

    /// Transfers coins from sender to receiver.
    pub fn transfer_coins(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<IndexAndSequence> {
        ensure!(
            space_delim_strings.len() >= 4 && space_delim_strings.len() <= 6,
            "Invalid number of arguments for transfer"
        );

        let sender_account_address =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        let receiver_address = self.get_account_address_from_parameter(space_delim_strings[2])?;

        let num_coins = Self::convert_to_micro_libras(space_delim_strings[3])?;

        let gas_unit_price = if space_delim_strings.len() > 4 {
            Some(space_delim_strings[4].parse::<u64>().map_err(|error| {
                format_parse_data_error(
                    "gas_unit_price",
                    InputType::UnsignedInt,
                    space_delim_strings[4],
                    error,
                )
            })?)
        } else {
            None
        };

        let max_gas_amount = if space_delim_strings.len() > 5 {
            Some(space_delim_strings[5].parse::<u64>().map_err(|error| {
                format_parse_data_error(
                    "max_gas_amount",
                    InputType::UnsignedInt,
                    space_delim_strings[5],
                    error,
                )
            })?)
        } else {
            None
        };

        let sender_account_ref_id = self.get_account_ref_id(&sender_account_address)?;

        self.transfer_coins_int(
            sender_account_ref_id,
            &receiver_address,
            num_coins,
            gas_unit_price,
            max_gas_amount,
            is_blocking,
        )
    }

    /// Compile move program
    pub fn compile_program(&mut self, space_delim_strings: &[&str]) -> Result<String> {
        let file_path = space_delim_strings[1];
        let output_path = {
            if space_delim_strings.len() == 3 {
                space_delim_strings[2].to_string()
            } else {
                let tmp_path = NamedTempFile::new()?.into_temp_path();
                let path = tmp_path.to_str().unwrap().to_string();
                self.temp_files.push(tmp_path);
                path
            }
        };
        let args = format!("run -p compiler -- -o {} {}", output_path, file_path);
        let status = Command::new("cargo")
            .args(args.split(' '))
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(format_err!("compilation failed"));
        }
        Ok(output_path)
    }

    /// Submit a transaction to the network.
    pub fn submit_transaction_from_disk(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<IndexAndSequence> {
        let signer_account_address =
            self.get_account_address_from_parameter(space_delim_strings[1])?;

        let txn = {
            let mut file = File::open(space_delim_strings[2]).map_err(|_| {
                format_err!("Cannot open file located at {}", space_delim_strings[2])
            })?;
            let mut buf = vec![];
            file.read_to_end(&mut buf).map_err(|_| {
                format_err!("Cannot read file located at {}", space_delim_strings[2])
            })?;
            RawTransaction::from_proto_bytes(&buf).map_err(|_| {
                format_err!(
                    "Cannot deserialize file located at {} as RawTransaction",
                    space_delim_strings[2]
                )
            })?
        };
        self.submit_custom_transaction(signer_account_address, txn, is_blocking)
    }

    fn submit_custom_transaction(
        &mut self,
        signer_address: AccountAddress,
        txn: RawTransaction,
        is_blocking: bool,
    ) -> Result<IndexAndSequence> {
        let sender_address;
        let sender_sequence;
        {
            let signer_account_ref_id = self.get_account_ref_id(&signer_address)?;
            let signer_account = self.accounts.get(signer_account_ref_id).ok_or_else(|| {
                format_err!("Unable to find sender account: {}", signer_account_ref_id)
            })?;
            let signer: Box<&dyn TransactionSigner> = match &signer_account.key_pair {
                Some(key_pair) => Box::new(key_pair),
                None => Box::new(&self.wallet),
            };
            let mut req = SubmitTransactionRequest::new();
            let txn = signer.sign_txn(txn).map_err(|_| {
                format_err!(
                    "Account #{} failed to sign transaction",
                    signer_account_ref_id
                )
            })?;
            sender_address = txn.sender();
            sender_sequence = txn.sequence_number();

            req.set_signed_txn(txn.into_proto());
            self.client.submit_transaction(None, &req)?;
        }

        if is_blocking {
            self.wait_for_transaction(sender_address, sender_sequence);
        }

        Ok(IndexAndSequence {
            account_index: AccountEntry::Address(sender_address),
            // The signer has nothing to do with the sequence here. The sequence number that we are
            // looking for should just be the sequence number in the sent transaction.
            sequence_number: sender_sequence,
        })
    }

    /// Get the latest account state from validator.
    pub fn get_latest_account_state(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<(Option<AccountStateBlob>, Version)> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments to get latest account state"
        );
        let account = self.get_account_address_from_parameter(space_delim_strings[1])?;
        self.get_account_state_and_update(account)
    }

    /// Get committed txn by account and sequence number.
    pub fn get_committed_txn_by_acc_seq(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<Option<(SignedTransaction, Option<Vec<ContractEvent>>)>> {
        ensure!(
            space_delim_strings.len() == 4,
            "Invalid number of arguments to get transaction by account and sequence number"
        );
        let account = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let sequence_number = space_delim_strings[2].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "account_sequence_number",
                InputType::UnsignedInt,
                space_delim_strings[2],
                error,
            )
        })?;

        let fetch_events = parse_bool(space_delim_strings[3]).map_err(|error| {
            format_parse_data_error(
                "fetch_events",
                InputType::Bool,
                space_delim_strings[3],
                error,
            )
        })?;

        self.client
            .get_txn_by_acc_seq(account, sequence_number, fetch_events)
    }

    /// Get committed txn by account and sequence number
    pub fn get_committed_txn_by_range(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<Vec<(SignedTransaction, Option<Vec<ContractEvent>>)>> {
        ensure!(
            space_delim_strings.len() == 4,
            "Invalid number of arguments to get transaction by range"
        );
        let start_version = space_delim_strings[1].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "start_version",
                InputType::UnsignedInt,
                space_delim_strings[1],
                error,
            )
        })?;
        let limit = space_delim_strings[2].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "limit",
                InputType::UnsignedInt,
                space_delim_strings[2],
                error,
            )
        })?;
        let fetch_events = parse_bool(space_delim_strings[3]).map_err(|error| {
            format_parse_data_error(
                "fetch_events",
                InputType::Bool,
                space_delim_strings[3],
                error,
            )
        })?;

        self.client
            .get_txn_by_range(start_version, limit, fetch_events)
    }

    /// Get account address from parameter. If the parameter is string of address, try to convert
    /// it to address, otherwise, try to convert to u64 and looking at TestClient::accounts.
    pub fn get_account_address_from_parameter(&self, para: &str) -> Result<AccountAddress> {
        match is_address(para) {
            true => ClientProxy::address_from_strings(para),
            false => {
                let account_ref_id = para.parse::<usize>().map_err(|error| {
                    format_parse_data_error(
                        "account_reference_id/account_address",
                        InputType::Usize,
                        para,
                        error,
                    )
                })?;
                let account_data = self.accounts.get(account_ref_id).ok_or_else(|| {
                    format_err!(
                        "Unable to find account by account reference id: {}, to see all existing \
                         accounts, run: 'account list'",
                        account_ref_id
                    )
                })?;
                Ok(account_data.address)
            }
        }
    }

    /// Get events by account and event type with start sequence number and limit.
    pub fn get_events_by_account_and_type(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<(Vec<EventWithProof>, Option<AccountStateWithProof>)> {
        ensure!(
            space_delim_strings.len() == 6,
            "Invalid number of arguments to get events by access path"
        );
        let account = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let path = match space_delim_strings[2] {
            "sent" => account_sent_event_path(),
            "received" => account_received_event_path(),
            _ => bail!(
                "Unknown event type: {:?}, only sent and received are supported",
                space_delim_strings[2]
            ),
        };
        let access_path = AccessPath::new(account, path);
        let start_seq_number = space_delim_strings[3].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "start_seq_number",
                InputType::UnsignedInt,
                space_delim_strings[3],
                error,
            )
        })?;
        let ascending = parse_bool(space_delim_strings[4]).map_err(|error| {
            format_parse_data_error("ascending", InputType::Bool, space_delim_strings[4], error)
        })?;
        let limit = space_delim_strings[5].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "start_seq_number",
                InputType::UnsignedInt,
                space_delim_strings[3],
                error,
            )
        })?;
        self.client
            .get_events_by_access_path(access_path, start_seq_number, ascending, limit)
    }

    /// Write mnemonic recover to the file specified.
    pub fn write_recovery(&self, space_delim_strings: &[&str]) -> Result<()> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for writing recovery"
        );

        self.wallet
            .write_recovery(&Path::new(space_delim_strings[1]))?;
        Ok(())
    }

    /// Recover wallet accounts from file and return vec<(account_address, index)>.
    pub fn recover_wallet_accounts(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<Vec<AddressAndIndex>> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for recovering wallets"
        );

        let wallet = WalletLibrary::recover(&Path::new(space_delim_strings[1]))?;
        let wallet_addresses = wallet.get_addresses()?;
        let mut account_data = Vec::new();
        for address in wallet_addresses {
            account_data.push(Self::get_account_data_from_address(
                &self.client,
                address,
                self.sync_on_wallet_recovery,
                None,
            )?);
        }
        self.set_wallet(wallet);
        // Clear current cached AccountData as we always swap the entire wallet completely.
        Ok(self.set_accounts(account_data))
    }

    /// Insert the account data to Client::accounts and return its address and index.s
    pub fn insert_account_data(&mut self, account_data: AccountData) -> AddressAndIndex {
        let address = account_data.address;

        self.accounts.push(account_data);
        self.address_to_ref_id
            .insert(address, self.accounts.len() - 1);

        AddressAndIndex {
            address,
            index: self.accounts.len() - 1,
        }
    }

    /// Test gRPC client connection with validator.
    pub fn test_validator_connection(&self) -> Result<()> {
        self.client.get_with_proof_sync(vec![])?;
        Ok(())
    }

    /// Get account state from validator and update status of account if it is cached locally.
    fn get_account_state_and_update(
        &mut self,
        address: AccountAddress,
    ) -> Result<(Option<AccountStateBlob>, Version)> {
        let account_state = self.client.get_account_blob(address)?;
        if self.address_to_ref_id.contains_key(&address) {
            let account_ref_id = self
                .address_to_ref_id
                .get(&address)
                .expect("Should have the key");
            let mut account_data: &mut AccountData =
                self.accounts.get_mut(*account_ref_id).unwrap_or_else(|| panic!("Local cache not consistent, reference id {} not available in local accounts", account_ref_id));
            if account_state.0.is_some() {
                account_data.status = AccountStatus::Persisted;
            }
        };
        Ok(account_state)
    }

    /// Get account resource from validator and update status of account if it is cached locally.
    fn get_account_resource_and_update(
        &mut self,
        address: AccountAddress,
    ) -> Result<AccountResource> {
        let account_state = self.get_account_state_and_update(address)?;
        get_account_resource_or_default(&account_state.0)
    }

    /// Get account using specific address.
    /// Sync with validator for account sequence number in case it is already created on chain.
    /// This assumes we have a very low probability of mnemonic word conflict.
    fn get_account_data_from_address(
        client: &GRPCClient,
        address: AccountAddress,
        sync_with_validator: bool,
        key_pair: Option<KeyPair>,
    ) -> Result<AccountData> {
        let (sequence_number, status) = match sync_with_validator {
            true => match client.get_account_blob(address) {
                Ok(resp) => match resp.0 {
                    Some(account_state_blob) => (
                        get_account_resource_or_default(&Some(account_state_blob))?
                            .sequence_number(),
                        AccountStatus::Persisted,
                    ),
                    None => (0, AccountStatus::Local),
                },
                Err(e) => {
                    error!("Failed to get account state from validator, error: {:?}", e);
                    (0, AccountStatus::Unknown)
                }
            },
            false => (0, AccountStatus::Local),
        };
        Ok(AccountData {
            address,
            key_pair,
            sequence_number,
            status,
        })
    }

    fn get_libra_wallet(mnemonic_file: Option<String>) -> Result<WalletLibrary> {
        let wallet_recovery_file_path = if let Some(input_mnemonic_word) = mnemonic_file {
            Path::new(&input_mnemonic_word).to_path_buf()
        } else {
            let mut file_path = std::env::current_dir()?;
            file_path.push(CLIENT_WALLET_MNEMONIC_FILE);
            file_path
        };

        let wallet = if let Ok(recovered_wallet) = io_utils::recover(&wallet_recovery_file_path) {
            recovered_wallet
        } else {
            let new_wallet = WalletLibrary::new();
            new_wallet.write_recovery(&wallet_recovery_file_path)?;
            new_wallet
        };
        Ok(wallet)
    }

    /// Set wallet instance used by this client.
    fn set_wallet(&mut self, wallet: WalletLibrary) {
        self.wallet = wallet;
    }

    fn load_faucet_account_file(faucet_account_file: &str) -> KeyPair {
        match fs::read(faucet_account_file) {
            Ok(data) => {
                bincode::deserialize(&data[..]).expect("Unable to deserialize faucet account file")
            }
            Err(e) => {
                panic!(
                    "Unable to read faucet account file: {}, {}",
                    faucet_account_file, e
                );
            }
        }
    }

    fn address_from_strings(data: &str) -> Result<AccountAddress> {
        let account_vec: Vec<u8> = hex::decode(data.parse::<String>()?)?;
        ensure!(
            account_vec.len() == ADDRESS_LENGTH,
            "The address {:?} is of invalid length. Addresses must be 32-bytes long"
        );
        let account = match AccountAddress::try_from(&account_vec[..]) {
            Ok(address) => address,
            Err(error) => bail!(
                "The address {:?} is invalid, error: {:?}",
                &account_vec,
                error,
            ),
        };
        Ok(account)
    }

    fn mint_coins_with_local_faucet_account(
        &mut self,
        receiver: &AccountAddress,
        num_coins: u64,
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(self.faucet_account.is_some(), "No faucet account loaded");
        let sender = self.faucet_account.as_ref().unwrap();
        let sender_address = sender.address;
        let program = vm_genesis::encode_mint_program(&receiver, num_coins);
        let req = self.create_submit_transaction_req(
            program, sender, None, /* max_gas_amount */
            None, /* gas_unit_price */
        )?;
        let mut sender_mut = self.faucet_account.as_mut().unwrap();
        let resp = self.client.submit_transaction(Some(&mut sender_mut), &req);
        if is_blocking {
            self.wait_for_transaction(
                sender_address,
                self.faucet_account.as_ref().unwrap().sequence_number,
            );
        }
        resp
    }

    fn mint_coins_with_faucet_service(
        &mut self,
        receiver: &AccountAddress,
        num_coins: u64,
        is_blocking: bool,
    ) -> Result<()> {
        let mut runtime = Runtime::new().unwrap();
        let client = hyper::Client::new();

        let url = format!(
            "http://{}?amount={}&address={:?}",
            self.faucet_server, num_coins, receiver
        )
        .parse::<hyper::Uri>()?;

        let response = runtime.block_on(client.get(url))?;
        let status_code = response.status();
        let body = response.into_body().concat2().wait()?;
        let raw_data = std::str::from_utf8(&body)?;

        if status_code != 200 {
            return Err(format_err!(
                "Failed to query remote faucet server[status={}]: {:?}",
                status_code,
                raw_data,
            ));
        }
        let sequence_number = raw_data.parse::<u64>()?;
        if is_blocking {
            self.wait_for_transaction(AccountAddress::new([0; 32]), sequence_number);
        }
        Ok(())
    }

    fn convert_to_micro_libras(input: &str) -> Result<u64> {
        ensure!(!input.is_empty(), "Empty input not allowed for libra unit");
        // This is not supposed to panic as it is used as constant here.
        let max_value = Decimal::from_u64(std::u64::MAX).unwrap() / Decimal::new(1_000_000, 0);
        let scale = input.find('.').unwrap_or(input.len() - 1);
        ensure!(
            scale <= 14,
            "Input value is too big: {:?}, max: {:?}",
            input,
            max_value
        );
        let original = Decimal::from_str(input)?;
        ensure!(
            original <= max_value,
            "Input value is too big: {:?}, max: {:?}",
            input,
            max_value
        );
        let value = original * Decimal::new(1_000_000, 0);
        ensure!(value.fract().is_zero(), "invalid value");
        value.to_u64().ok_or_else(|| format_err!("invalid value"))
    }

    /// Craft a transaction request.
    fn create_submit_transaction_req(
        &self,
        program: Program,
        sender_account: &AccountData,
        max_gas_amount: Option<u64>,
        gas_unit_price: Option<u64>,
    ) -> Result<SubmitTransactionRequest> {
        let signer: Box<&dyn TransactionSigner> = match &sender_account.key_pair {
            Some(key_pair) => Box::new(key_pair),
            None => Box::new(&self.wallet),
        };
        let signed_txn = create_signed_txn(
            *signer,
            program,
            sender_account.address,
            sender_account.sequence_number,
            max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
            gas_unit_price.unwrap_or(GAS_UNIT_PRICE),
            TX_EXPIRATION,
        )
        .unwrap();
        let mut req = SubmitTransactionRequest::new();
        req.set_signed_txn(signed_txn.into_proto());
        Ok(req)
    }

    fn mut_account_from_parameter(&mut self, para: &str) -> Result<&mut AccountData> {
        let account_ref_id = match is_address(para) {
            true => {
                let account_address = ClientProxy::address_from_strings(para)?;
                *self
                    .address_to_ref_id
                    .get(&account_address)
                    .ok_or_else(|| {
                        format_err!(
                            "Unable to find local account by address: {:?}",
                            account_address
                        )
                    })?
            }
            false => para.parse::<usize>()?,
        };
        let account_data = self
            .accounts
            .get_mut(account_ref_id)
            .ok_or_else(|| format_err!("Unable to find account by ref id: {}", account_ref_id))?;
        Ok(account_data)
    }
}

fn format_parse_data_error<T: std::fmt::Debug>(
    field: &str,
    input_type: InputType,
    value: &str,
    error: T,
) -> Error {
    format_err!(
        "Unable to parse input for {} - \
         please enter an {:?}.  Input was: {}, error: {:?}",
        field,
        input_type,
        value,
        error
    )
}

fn parse_bool(para: &str) -> Result<bool> {
    Ok(para.to_lowercase().parse::<bool>()?)
}

impl fmt::Display for AccountEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountEntry::Index(i) => write!(f, "{}", i),
            AccountEntry::Address(addr) => write!(f, "{}", addr),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::client_proxy::{parse_bool, AddressAndIndex, ClientProxy};
    use config::trusted_peers::TrustedPeersConfigHelpers;
    use libra_wallet::io_utils;
    use proptest::prelude::*;
    use tempfile::NamedTempFile;

    fn generate_accounts_from_wallet(count: usize) -> (ClientProxy, Vec<AddressAndIndex>) {
        let mut accounts = Vec::new();
        accounts.reserve(count);
        let file = NamedTempFile::new().unwrap();
        let mnemonic_path = file.into_temp_path().to_str().unwrap().to_string();
        let trust_peer_file = NamedTempFile::new().unwrap();
        let (_, trust_peer_config) = TrustedPeersConfigHelpers::get_test_config(1, None);
        let trust_peer_path = trust_peer_file.into_temp_path();
        trust_peer_config.save_config(&trust_peer_path);

        let val_set_file = trust_peer_path.to_str().unwrap().to_string();

        // We don't need to specify host/port since the client won't be used to connect, only to
        // generate random accounts
        let mut client_proxy = ClientProxy::new(
            "", /* host */
            "", /* port */
            &val_set_file,
            &"",
            false,
            None,
            Some(mnemonic_path),
        )
        .unwrap();
        for _ in 0..count {
            accounts.push(client_proxy.create_next_account(false).unwrap());
        }

        (client_proxy, accounts)
    }

    #[test]
    fn test_parse_bool() {
        assert!(parse_bool("true").unwrap());
        assert!(parse_bool("True").unwrap());
        assert!(parse_bool("TRue").unwrap());
        assert!(parse_bool("TRUE").unwrap());
        assert!(!parse_bool("false").unwrap());
        assert!(!parse_bool("False").unwrap());
        assert!(!parse_bool("FaLSe").unwrap());
        assert!(!parse_bool("FALSE").unwrap());
        assert!(parse_bool("1").is_err());
        assert!(parse_bool("0").is_err());
        assert!(parse_bool("2").is_err());
        assert!(parse_bool("1adf").is_err());
        assert!(parse_bool("ad13").is_err());
        assert!(parse_bool("ad1f").is_err());
    }

    #[test]
    fn test_micro_libra_conversion() {
        assert!(ClientProxy::convert_to_micro_libras("").is_err());
        assert!(ClientProxy::convert_to_micro_libras("-11").is_err());
        assert!(ClientProxy::convert_to_micro_libras("abc").is_err());
        assert!(ClientProxy::convert_to_micro_libras("11111112312321312321321321").is_err());
        assert!(ClientProxy::convert_to_micro_libras("0").is_ok());
        assert!(ClientProxy::convert_to_micro_libras("1").is_ok());
        assert!(ClientProxy::convert_to_micro_libras("0.1").is_ok());
        assert!(ClientProxy::convert_to_micro_libras("1.1").is_ok());
        // Max of micro libra is u64::MAX (18446744073709551615).
        assert!(ClientProxy::convert_to_micro_libras("18446744073709.551615").is_ok());
        assert!(ClientProxy::convert_to_micro_libras("184467440737095.51615").is_err());
        assert!(ClientProxy::convert_to_micro_libras("18446744073709.551616").is_err());
    }

    #[test]
    fn test_generate() {
        let num = 1;
        let (_, accounts) = generate_accounts_from_wallet(num);
        assert_eq!(accounts.len(), num);
    }

    #[test]
    fn test_write_recover() {
        let num = 100;
        let (client, accounts) = generate_accounts_from_wallet(num);
        assert_eq!(accounts.len(), num);

        let file = NamedTempFile::new().unwrap();
        let path = file.into_temp_path();
        io_utils::write_recovery(&client.wallet, &path).expect("failed to write to file");

        let wallet = io_utils::recover(&path).expect("failed to load from file");

        assert_eq!(client.wallet.mnemonic(), wallet.mnemonic());
    }

    proptest! {
        // Proptest is used to verify that the conversion will not panic with random input.
        #[test]
        fn test_micro_libra_conversion_random_string(req in any::<String>()) {
            let _res = ClientProxy::convert_to_micro_libras(&req);
        }
        #[test]
        fn test_micro_libra_conversion_random_f64(req in any::<f64>()) {
            let req_str = req.to_string();
            let _res = ClientProxy::convert_to_micro_libras(&req_str);
        }
        #[test]
        fn test_micro_libra_conversion_random_u64(req in any::<u64>()) {
            let req_str = req.to_string();
            let _res = ClientProxy::convert_to_micro_libras(&req_str);
        }
    }
}
