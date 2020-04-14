// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    commands::{is_address, is_authentication_key},
    libra_client::LibraClient,
    AccountData, AccountStatus,
};
use anyhow::{bail, ensure, format_err, Error, Result};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    test_utils::KeyPair,
    traits::ValidKey,
    x25519, ValidKeyStringExt,
};
use libra_json_rpc::views::{AccountView, BlockMetadata, EventView, TransactionView};
use libra_logger::prelude::*;
use libra_temppath::TempPath;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config::{
        association_address, lbr_type_tag, ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH,
        CORE_CODE_ADDRESS, LBR_NAME,
    },
    account_state::AccountState,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::VMPublishingOption,
    transaction::{
        authenticator::AuthenticationKey,
        helpers::{create_unsigned_txn, create_user_txn, TransactionSigner},
        parse_as_transaction_argument, Module, RawTransaction, Script, SignedTransaction,
        TransactionArgument, TransactionPayload, Version,
    },
    waypoint::Waypoint,
};
use libra_wallet::{io_utils, WalletLibrary};
use num_traits::{
    cast::{FromPrimitive, ToPrimitive},
    identities::Zero,
};
use parity_multiaddr::Multiaddr;
use reqwest::Url;
use rust_decimal::Decimal;
use serde_json;
use std::{
    collections::HashMap,
    convert::TryFrom,
    fmt, fs,
    io::{stdout, Write},
    path::{Display, Path, PathBuf},
    process::{Command, Stdio},
    str::{self, FromStr},
    thread, time,
};
use stdlib::transaction_scripts::StdlibScript;
use transaction_builder::encode_register_validator_script;

const CLIENT_WALLET_MNEMONIC_FILE: &str = "client.mnemonic";
const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 400_000;
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
    pub client: LibraClient,
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
    /// Whether to sync with validator on wallet recovery.
    sync_on_wallet_recovery: bool,
    /// temp files (alive for duration of program)
    temp_files: Vec<PathBuf>,
    // invariant self.address_to_ref_id.values().iter().all(|i| i < self.accounts.len())
}

impl ClientProxy {
    /// Construct a new TestClient.
    pub fn new(
        url: &str,
        faucet_account_file: &str,
        sync_on_wallet_recovery: bool,
        faucet_server: Option<String>,
        mnemonic_file: Option<String>,
        waypoint: Option<Waypoint>,
    ) -> Result<Self> {
        // fail fast if url is not valid
        let url = Url::parse(url)?;
        let mut client = LibraClient::new(url.clone(), waypoint)?;

        let accounts = vec![];

        // If we have a faucet account file, then load it to get the keypair
        let faucet_account = if faucet_account_file.is_empty() {
            None
        } else {
            let faucet_account_keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
                ClientProxy::load_faucet_account_file(faucet_account_file);
            let faucet_account_data = Self::get_account_data_from_address(
                &mut client,
                association_address(),
                true,
                Some(KeyPair::<Ed25519PrivateKey, _>::from(
                    faucet_account_keypair.private_key,
                )),
                None,
            )?;
            // Load the keypair from file
            Some(faucet_account_data)
        };

        let faucet_server = match faucet_server {
            Some(server) => server,
            None => url
                .host_str()
                .ok_or_else(|| format_err!("Missing host in URL"))?
                .replace("client", "faucet"),
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
        let (auth_key, _) = self.wallet.new_address()?;
        let account_data = Self::get_account_data_from_address(
            &mut self.client,
            auth_key.derived_address(),
            sync_with_validator,
            None,
            Some(auth_key.to_vec()),
        )?;

        Ok(self.insert_account_data(account_data))
    }

    /// Returns the ledger info corresonding to the latest epoch change
    /// (could further be used for e.g., generating a waypoint)
    pub fn latest_epoch_change_li(&self) -> Option<&LedgerInfoWithSignatures> {
        self.client.latest_epoch_change_li()
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
    #[cfg(any(test, feature = "fuzzing"))]
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
        let (address, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        self.get_account_resource_and_update(address).map(|res| {
            let whole_num = res.balance / 1_000_000;
            let remainder = res.balance % 1_000_000;
            format!("{}.{:0>6}", whole_num.to_string(), remainder.to_string())
        })
    }

    /// Get the latest sequence number from validator for the account specified.
    pub fn get_sequence_number(&mut self, space_delim_strings: &[&str]) -> Result<u64> {
        ensure!(
            space_delim_strings.len() == 2 || space_delim_strings.len() == 3,
            "Invalid number of arguments for getting sequence number"
        );
        let (address, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let sequence_number = self
            .get_account_resource_and_update(address)?
            .sequence_number;

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
            if let Some(faucet_account) = &mut self.faucet_account {
                if faucet_account.address == address {
                    faucet_account.sequence_number = sequence_number;
                    return Ok(sequence_number);
                }
            }
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
        let (receiver, receiver_auth_key_opt) =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        let receiver_auth_key = receiver_auth_key_opt.ok_or_else(|| {
            format_err!("Need authentication key to create new account via minting")
        })?;
        let num_coins = Self::convert_to_micro_libras(space_delim_strings[2])?;

        ensure!(num_coins > 0, "Invalid number of coins to mint.");

        match self.faucet_account {
            Some(_) => self.association_transaction_with_local_faucet_account(
                transaction_builder::encode_mint_script(
                    &receiver,
                    receiver_auth_key.prefix().to_vec(),
                    num_coins,
                ),
                is_blocking,
            ),
            None => self.mint_coins_with_faucet_service(receiver_auth_key, num_coins, is_blocking),
        }
    }

    /// Allow executing arbitrary script in the network.
    pub fn enable_custom_script(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "enable_custom_script",
            "inconsistent command '{}' for enable_custom_script",
            space_delim_strings[0]
        );
        ensure!(
            space_delim_strings.len() == 1,
            "Invalid number of arguments for setting publishing option"
        );
        match self.faucet_account {
            Some(_) => self.association_transaction_with_local_faucet_account(
                transaction_builder::encode_publishing_option_script(
                    VMPublishingOption::CustomScripts,
                ),
                is_blocking,
            ),
            None => unimplemented!(),
        }
    }

    /// Only allow executing predefined script in the Move standard library in the network.
    pub fn disable_custom_script(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "disable_custom_script",
            "inconsistent command '{}' for disable_custom_script",
            space_delim_strings[0]
        );
        ensure!(
            space_delim_strings.len() == 1,
            "Invalid number of arguments for setting publishing option"
        );
        match self.faucet_account {
            Some(_) => self.association_transaction_with_local_faucet_account(
                transaction_builder::encode_publishing_option_script(VMPublishingOption::Locked(
                    StdlibScript::whitelist(),
                )),
                is_blocking,
            ),
            None => unimplemented!(),
        }
    }

    /// Remove a existing validator.
    pub fn remove_validator(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "remove_validator",
            "inconsistent command '{}' for remove_validator",
            space_delim_strings[0]
        );
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for removing validator"
        );
        let (account_address, _) =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        match self.faucet_account {
            Some(_) => self.association_transaction_with_local_faucet_account(
                transaction_builder::encode_remove_validator_script(&account_address),
                is_blocking,
            ),
            None => unimplemented!(),
        }
    }

    /// Add a new validator.
    pub fn add_validator(&mut self, space_delim_strings: &[&str], is_blocking: bool) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "add_validator",
            "inconsistent command '{}' for add_validator",
            space_delim_strings[0]
        );
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for adding validator"
        );
        let (account_address, _) =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        match self.faucet_account {
            Some(_) => self.association_transaction_with_local_faucet_account(
                transaction_builder::encode_add_validator_script(&account_address),
                is_blocking,
            ),
            None => unimplemented!(),
        }
    }

    /// Register an account as validator candidate with ValidatorConfig
    pub fn register_validator(
        &mut self,
        space_delim_strings: &[&str],
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "register_validator",
            "inconsistent command '{}' for register_validator",
            space_delim_strings[0]
        );
        ensure!(
            space_delim_strings.len() == 9,
            "Invalid number of arguments for registering validator"
        );
        let (address, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let private_key = Ed25519PrivateKey::from_encoded_string(space_delim_strings[2])?;
        let consensus_public_key = Ed25519PublicKey::from_encoded_string(space_delim_strings[3])?;
        let network_signing_key = Ed25519PublicKey::from_encoded_string(space_delim_strings[4])?;
        let network_identity_key = x25519::PublicKey::from_encoded_string(space_delim_strings[5])?;
        let network_address = Multiaddr::from_str(space_delim_strings[6])?;
        let fullnode_identity_key = x25519::PublicKey::from_encoded_string(space_delim_strings[7])?;
        let fullnode_network_address = Multiaddr::from_str(space_delim_strings[8])?;
        let mut sender = Self::get_account_data_from_address(
            &mut self.client,
            address,
            true,
            Some(KeyPair::from(private_key)),
            None,
        )?;
        let program = encode_register_validator_script(
            consensus_public_key.to_bytes().to_vec(),
            network_signing_key.to_bytes().to_vec(),
            network_identity_key.to_bytes(),
            network_address.to_vec(),
            fullnode_identity_key.to_bytes(),
            fullnode_network_address.to_vec(),
        );
        let txn =
            self.create_txn_to_submit(TransactionPayload::Script(program), &sender, None, None)?;
        self.client.submit_transaction(Some(&mut sender), txn)?;
        if is_blocking {
            self.wait_for_transaction(sender.address, sender.sequence_number);
        }
        Ok(())
    }

    /// Waits for the next transaction for a specific address and prints it
    pub fn wait_for_transaction(&mut self, account: AccountAddress, sequence_number: u64) {
        let mut max_iterations = 5000;
        print!(
            "waiting for {} with sequence number {}",
            account, sequence_number
        );
        loop {
            stdout().flush().unwrap();

            match self
                .client
                .get_txn_by_acc_seq(account, sequence_number - 1, true)
            {
                Ok(Some(txn_view)) => {
                    println!("transaction is stored!");
                    if txn_view.events.is_empty() {
                        println!("no events emitted");
                    }
                    break;
                }
                Err(e) => {
                    println!("Response with error: {:?}", e);
                }
                _ => {
                    print!(".");
                }
            }
            max_iterations -= 1;
            if max_iterations == 0 {
                panic!("wait_for_transaction timeout");
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
        receiver_auth_key_prefix: Vec<u8>,
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
            let program = transaction_builder::encode_transfer_with_metadata_script(
                lbr_type_tag(),
                &receiver_address,
                receiver_auth_key_prefix,
                num_coins,
                vec![],
            );
            let txn = self.create_txn_to_submit(
                TransactionPayload::Script(program),
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
            self.client.submit_transaction(Some(sender_mut), txn)?;
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

    /// Prepare a transfer transaction: return the unsigned raw transaction
    pub fn prepare_transfer_coins(
        &mut self,
        sender_address: AccountAddress,
        sender_sequence_number: u64,
        receiver_address: AccountAddress,
        receiver_auth_key_prefix: Vec<u8>,
        num_coins: u64,
        gas_unit_price: Option<u64>,
        max_gas_amount: Option<u64>,
    ) -> Result<RawTransaction> {
        let program = transaction_builder::encode_transfer_with_metadata_script(
            lbr_type_tag(),
            &receiver_address,
            receiver_auth_key_prefix,
            num_coins,
            vec![],
        );

        Ok(create_unsigned_txn(
            TransactionPayload::Script(program),
            sender_address,
            sender_sequence_number,
            max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
            gas_unit_price.unwrap_or(GAS_UNIT_PRICE),
            LBR_NAME.to_string(),
            TX_EXPIRATION,
        ))
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

        let (sender_account_address, _) =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        let (receiver_address, receiver_auth_key_opt) =
            self.get_account_address_from_parameter(space_delim_strings[2])?;

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
        let receiver_auth_key_prefix = receiver_auth_key_opt.map_or(Vec::new(), |auth_key| {
            AuthenticationKey::prefix(&auth_key).to_vec()
        });

        self.transfer_coins_int(
            sender_account_ref_id,
            &receiver_address,
            receiver_auth_key_prefix,
            num_coins,
            gas_unit_price,
            max_gas_amount,
            is_blocking,
        )
    }

    /// Compile Move program
    pub fn compile_program(&mut self, space_delim_strings: &[&str]) -> Result<String> {
        ensure!(
            space_delim_strings[0] == "compile",
            "inconsistent command '{}' for compile_program",
            space_delim_strings[0]
        );
        let (address, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let file_path = space_delim_strings[2];
        let is_module = match space_delim_strings[3] {
            "module" => true,
            "script" => false,
            _ => bail!(
                "Invalid program type: {}. Available options: module, script",
                space_delim_strings[3]
            ),
        };

        let tmp_source_path = TempPath::new().as_ref().with_extension("mvir");
        let output_path = &tmp_source_path.with_extension("mv");
        let mut tmp_source_file = std::fs::File::create(tmp_source_path.clone())?;
        let mut code = fs::read_to_string(file_path)?;
        code = code.replace("{{sender}}", &format!("0x{}", address));
        writeln!(tmp_source_file, "{}", code)?;
        self.temp_files.push(output_path.to_path_buf());
        let dependencies_file = self.handle_dependencies(tmp_source_path.display(), is_module)?;

        let mut args = format!(
            "run -p compiler -- {} -a {}{}",
            tmp_source_path.display(),
            address,
            if is_module { " -m" } else { "" },
        );
        if let Some(file) = &dependencies_file {
            args.push_str(&format!(" --deps={}", file.as_ref().display()));
        }

        let status = Command::new("cargo")
            .args(args.split(' '))
            .spawn()?
            .wait()?;
        if !status.success() {
            return Err(format_err!("compilation failed"));
        }
        Ok(output_path
            .to_str()
            .expect(
                "TempPath::new() should always generate a path that can be converted to a string",
            )
            .to_string())
    }

    fn handle_dependencies(
        &mut self,
        source_path: Display,
        is_module: bool,
    ) -> Result<Option<TempPath>> {
        let mut args = format!("run -p compiler -- -l {}", source_path);
        if is_module {
            args.push_str(" -m");
        }
        let child = Command::new("cargo")
            .args(args.split(' '))
            .stdout(Stdio::piped())
            .spawn()?;
        let output = child.wait_with_output()?;
        let paths: Vec<AccessPath> = serde_json::from_str(str::from_utf8(&output.stdout)?)?;
        let mut dependencies = vec![];
        for path in paths {
            if path.address != CORE_CODE_ADDRESS {
                if let Some(blob) = self.client.get_account_blob(path.address)? {
                    let account_state = AccountState::try_from(&blob)?;
                    if let Some(code) = account_state.get(&path.path) {
                        dependencies.push(code.clone());
                    }
                }
            }
        }
        if dependencies.is_empty() {
            return Ok(None);
        }
        let path = TempPath::new();
        let mut file = std::fs::File::create(path.as_ref())?;
        file.write_all(&serde_json::to_vec(&dependencies)?)?;
        Ok(Some(path))
    }

    /// Submit a transaction to the network given the unsigned raw transaction, sender public key
    /// and signature
    pub fn submit_signed_transaction(
        &mut self,
        raw_txn: RawTransaction,
        public_key: Ed25519PublicKey,
        signature: Ed25519Signature,
    ) -> Result<()> {
        let transaction = SignedTransaction::new(raw_txn, public_key, signature);

        let sender_address = transaction.sender();
        let sender_sequence = transaction.sequence_number();

        self.client.submit_transaction(None, transaction)?;
        // blocking by default (until transaction completion)
        self.wait_for_transaction(sender_address, sender_sequence + 1);

        Ok(())
    }

    fn submit_program(
        &mut self,
        space_delim_strings: &[&str],
        program: TransactionPayload,
    ) -> Result<()> {
        let (sender_address, _) =
            self.get_account_address_from_parameter(space_delim_strings[1])?;
        let sender_ref_id = self.get_account_ref_id(&sender_address)?;
        let sender = self.accounts.get(sender_ref_id).unwrap();
        let sequence_number = sender.sequence_number;

        let txn = self.create_txn_to_submit(program, &sender, None, None)?;

        self.client
            .submit_transaction(self.accounts.get_mut(sender_ref_id), txn)?;
        self.wait_for_transaction(sender_address, sequence_number + 1);

        Ok(())
    }

    /// Publish Move module
    pub fn publish_module(&mut self, space_delim_strings: &[&str]) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "publish",
            "inconsistent command '{}' for publish_module",
            space_delim_strings[0]
        );
        let module_bytes = fs::read(space_delim_strings[2])?;
        self.submit_program(
            space_delim_strings,
            TransactionPayload::Module(Module::new(module_bytes)),
        )
    }

    /// Execute custom script
    pub fn execute_script(&mut self, space_delim_strings: &[&str]) -> Result<()> {
        ensure!(
            space_delim_strings[0] == "execute",
            "inconsistent command '{}' for execute_script",
            space_delim_strings[0]
        );
        let script_bytes = fs::read(space_delim_strings[2])?;
        let arguments: Vec<_> = space_delim_strings[3..]
            .iter()
            .filter_map(|arg| parse_as_transaction_argument_for_client(arg).ok())
            .collect();
        // TODO: support type arguments in the client.
        self.submit_program(
            space_delim_strings,
            TransactionPayload::Script(Script::new(script_bytes, vec![], arguments)),
        )
    }

    /// Get the latest account state from validator.
    pub fn get_latest_account_state(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<(Option<AccountView>, Version)> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments to get latest account state"
        );
        let (account, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        self.get_account_state_and_update(account)
    }

    /// Get committed txn by account and sequence number.
    pub fn get_committed_txn_by_acc_seq(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<Option<TransactionView>> {
        ensure!(
            space_delim_strings.len() == 4,
            "Invalid number of arguments to get transaction by account and sequence number"
        );
        let (account, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
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
    ) -> Result<Vec<TransactionView>> {
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

    /// Get account address and (if applicable) authentication key from parameter. If the parameter
    /// is string of address, try to convert it to address, otherwise, try to convert to u64 and
    /// looking at TestClient::accounts.
    pub fn get_account_address_from_parameter(
        &self,
        para: &str,
    ) -> Result<(AccountAddress, Option<AuthenticationKey>)> {
        if is_authentication_key(para) {
            let auth_key = ClientProxy::authentication_key_from_string(para)?;
            Ok((auth_key.derived_address(), Some(auth_key)))
        } else if is_address(para) {
            Ok((ClientProxy::address_from_strings(para)?, None))
        } else {
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
            Ok((
                account_data.address,
                account_data
                    .authentication_key
                    .clone()
                    .map(|bytes| AuthenticationKey::try_from(bytes).ok())
                    .flatten(),
            ))
        }
    }

    /// Get events by account and event type with start sequence number and limit.
    pub fn get_events_by_account_and_type(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<(Vec<EventView>, AccountView)> {
        ensure!(
            space_delim_strings.len() == 5,
            "Invalid number of arguments to get events by access path"
        );
        let (account, _) = self.get_account_address_from_parameter(space_delim_strings[1])?;
        let path = match space_delim_strings[2] {
            "sent" => ACCOUNT_SENT_EVENT_PATH.to_vec(),
            "received" => ACCOUNT_RECEIVED_EVENT_PATH.to_vec(),
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
        let limit = space_delim_strings[4].parse::<u64>().map_err(|error| {
            format_parse_data_error(
                "start_seq_number",
                InputType::UnsignedInt,
                space_delim_strings[4],
                error,
            )
        })?;
        self.client
            .get_events_by_access_path(access_path, start_seq_number, limit)
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

    /// Recover wallet accounts from command 'recover <file>' and return vec<(account_address, index)>.
    pub fn recover_wallet_accounts(
        &mut self,
        space_delim_strings: &[&str],
    ) -> Result<Vec<AddressAndIndex>> {
        ensure!(
            space_delim_strings.len() == 2,
            "Invalid number of arguments for recovering wallets"
        );
        let wallet = WalletLibrary::recover(&Path::new(space_delim_strings[1]))?;
        self.set_wallet(wallet);
        self.recover_accounts_in_wallet()
    }

    /// Recover accounts in wallets and sync state if sync_on_wallet_recovery is true.
    pub fn recover_accounts_in_wallet(&mut self) -> Result<Vec<AddressAndIndex>> {
        let wallet_addresses = self.wallet.get_addresses()?;
        let mut account_data = Vec::new();
        for address in wallet_addresses {
            account_data.push(Self::get_account_data_from_address(
                &mut self.client,
                address,
                self.sync_on_wallet_recovery,
                None,
                None,
            )?);
        }
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

    /// Test JSON RPC client connection with validator.
    pub fn test_validator_connection(&mut self) -> Result<BlockMetadata> {
        self.client.get_metadata()
    }

    /// Test client's connection to validator with proof.
    pub fn test_trusted_connection(&mut self) -> Result<()> {
        self.client.get_state_proof()
    }

    /// Get account state from validator and update status of account if it is cached locally.
    fn get_account_state_and_update(
        &mut self,
        address: AccountAddress,
    ) -> Result<(Option<AccountView>, Version)> {
        let account_state = self.client.get_account_state(address, true)?;
        if self.address_to_ref_id.contains_key(&address) {
            let account_ref_id = self
                .address_to_ref_id
                .get(&address)
                .expect("Should have the key");
            // assumption follows from invariant
            let mut account_data: &mut AccountData =
                self.accounts.get_mut(*account_ref_id).unwrap_or_else(|| unreachable!("Local cache not consistent, reference id {} not available in local accounts", account_ref_id));
            if account_state.0.is_some() {
                account_data.status = AccountStatus::Persisted;
            }
        };
        Ok(account_state)
    }

    /// Get account resource from validator and update status of account if it is cached locally.
    fn get_account_resource_and_update(&mut self, address: AccountAddress) -> Result<AccountView> {
        let account_state = self.get_account_state_and_update(address)?;
        if let Some(view) = account_state.0 {
            Ok(view)
        } else {
            bail!("No account exists at {:?}", address)
        }
    }

    /// Get account using specific address.
    /// Sync with validator for account sequence number in case it is already created on chain.
    /// This assumes we have a very low probability of mnemonic word conflict.
    fn get_account_data_from_address(
        client: &mut LibraClient,
        address: AccountAddress,
        sync_with_validator: bool,
        key_pair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
        authentication_key_opt: Option<Vec<u8>>,
    ) -> Result<AccountData> {
        let (sequence_number, authentication_key, status) = if sync_with_validator {
            match client.get_account_state(address, true) {
                Ok(resp) => match resp.0 {
                    Some(account_view) => (
                        account_view.sequence_number,
                        Some(account_view.authentication_key.into_bytes()?),
                        AccountStatus::Persisted,
                    ),
                    None => (0, authentication_key_opt, AccountStatus::Local),
                },
                Err(e) => {
                    error!("Failed to get account state from validator, error: {:?}", e);
                    (0, authentication_key_opt, AccountStatus::Unknown)
                }
            }
        } else {
            (0, authentication_key_opt, AccountStatus::Local)
        };
        Ok(AccountData {
            address,
            authentication_key,
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

    fn load_faucet_account_file(
        faucet_account_file: &str,
    ) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
        match fs::read(faucet_account_file) {
            Ok(data) => {
                lcs::from_bytes(&data[..]).expect("Unable to deserialize faucet account file")
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
            account_vec.len() == AccountAddress::LENGTH,
            "The address {:?} is of invalid length. Addresses must be 16-bytes long"
        );
        let account = AccountAddress::try_from(&account_vec[..]).map_err(|error| {
            format_err!(
                "The address {:?} is invalid, error: {:?}",
                &account_vec,
                error,
            )
        })?;
        Ok(account)
    }

    fn authentication_key_from_string(data: &str) -> Result<AuthenticationKey> {
        let bytes_vec: Vec<u8> = hex::decode(data.parse::<String>()?)?;
        ensure!(
            bytes_vec.len() == AuthenticationKey::LENGTH,
            "The authentication key string {:?} is of invalid length. Authentication keys must be 32-bytes long"
        );

        let auth_key = AuthenticationKey::try_from(&bytes_vec[..]).map_err(|error| {
            format_err!(
                "The authentication key {:?} is invalid, error: {:?}",
                &bytes_vec,
                error,
            )
        })?;
        Ok(auth_key)
    }

    fn association_transaction_with_local_faucet_account(
        &mut self,
        program: Script,
        is_blocking: bool,
    ) -> Result<()> {
        ensure!(self.faucet_account.is_some(), "No faucet account loaded");
        let sender = self.faucet_account.as_ref().unwrap();
        let sender_address = sender.address;
        let txn =
            self.create_txn_to_submit(TransactionPayload::Script(program), sender, None, None)?;
        let mut sender_mut = self.faucet_account.as_mut().unwrap();
        let resp = self.client.submit_transaction(Some(&mut sender_mut), txn);
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
        receiver: AuthenticationKey,
        num_coins: u64,
        is_blocking: bool,
    ) -> Result<()> {
        let client = reqwest::blocking::ClientBuilder::new().build()?;

        let url = Url::parse_with_params(
            format!("http://{}", self.faucet_server).as_str(),
            &[
                ("amount", num_coins.to_string().as_str()),
                ("auth_key", &hex::encode(receiver)),
            ],
        )?;

        let response = client.post(url).send()?;
        let status_code = response.status();
        let body = response.text()?;
        if !status_code.is_success() {
            return Err(format_err!(
                "Failed to query remote faucet server[status={}]: {:?}",
                status_code.as_str(),
                body,
            ));
        }
        let sequence_number = body.parse::<u64>()?;
        if is_blocking {
            self.wait_for_transaction(association_address(), sequence_number);
        }

        Ok(())
    }

    /// convert number of Libras (main unit) given as string to number of micro Libras
    pub fn convert_to_micro_libras(input: &str) -> Result<u64> {
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

    /// Craft a transaction to be submitted.
    fn create_txn_to_submit(
        &self,
        program: TransactionPayload,
        sender_account: &AccountData,
        max_gas_amount: Option<u64>,
        gas_unit_price: Option<u64>,
    ) -> Result<SignedTransaction> {
        let signer: Box<&dyn TransactionSigner> = match &sender_account.key_pair {
            Some(key_pair) => Box::new(key_pair),
            None => Box::new(&self.wallet),
        };
        create_user_txn(
            *signer,
            program,
            sender_account.address,
            sender_account.sequence_number,
            max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
            gas_unit_price.unwrap_or(GAS_UNIT_PRICE),
            LBR_NAME.to_string(),
            TX_EXPIRATION,
        )
    }

    fn mut_account_from_parameter(&mut self, para: &str) -> Result<&mut AccountData> {
        let account_ref_id = if is_address(para) {
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
        } else {
            para.parse::<usize>()?
        };
        let account_data = self
            .accounts
            .get_mut(account_ref_id)
            .ok_or_else(|| format_err!("Unable to find account by ref id: {}", account_ref_id))?;
        Ok(account_data)
    }
}

fn parse_as_transaction_argument_for_client(s: &str) -> Result<TransactionArgument> {
    if is_address(s) {
        let account_address = ClientProxy::address_from_strings(s)?;
        return Ok(TransactionArgument::Address(account_address));
    }
    parse_as_transaction_argument(s)
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
    use libra_temppath::TempPath;
    use libra_wallet::io_utils;
    use proptest::prelude::*;

    fn generate_accounts_from_wallet(count: usize) -> (ClientProxy, Vec<AddressAndIndex>) {
        let mut accounts = Vec::new();
        accounts.reserve(count);
        let file = TempPath::new();
        let mnemonic_path = file.path().to_str().unwrap().to_string();

        // Note: `client_proxy` won't actually connect to URL - it will be used only to
        // generate random accounts
        let mut client_proxy = ClientProxy::new(
            "http://localhost:8080",
            &"",
            false,
            None,
            Some(mnemonic_path),
            None,
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

        let file = TempPath::new();
        let path = file.path();
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
