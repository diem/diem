// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::common::utils::{AccountData, AccountStatus};
use anyhow::{ensure, format_err, Result};
use diem_types::{chain_id::ChainId, waypoint::Waypoint};

use crate::diem_client::DiemClient;
use rust_decimal::{
    prelude::{FromPrimitive, ToPrimitive, Zero},
    Decimal,
};

use diem_client::{views, WaitForTransactionError};
use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};

use diem_logger::prelude::{error, info};
use diem_transaction_builder::stdlib as transaction_builder;

use diem_types::{
    account_address::AccountAddress,
    account_config::{
        diem_root_address, from_currency_code_string, testnet_dd_account_address,
        treasury_compliance_account_address, type_tag_for_currency_code, XDX_NAME, XUS_NAME,
    },
    transaction::{
        authenticator::AuthenticationKey,
        helpers::{create_user_txn, TransactionSigner},
        SignedTransaction, TransactionPayload,
    },
};
use reqwest::Url;

use std::{
    convert::TryFrom,
    io::{stdout, Write},
    panic,
    str::{self, FromStr},
    time,
};

const GAS_UNIT_PRICE: u64 = 0;
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const TX_EXPIRATION: i64 = 100;
const DEFAULT_WAIT_TIMEOUT: time::Duration = time::Duration::from_secs(120);

/// Proxy handling CLI commands/inputs.
pub struct ClientProxy {
    /// chain ID of the Diem network this client is interacting with
    pub chain_id: ChainId,
    /// client for admission control interface.
    pub client: DiemClient,
    /// Host that operates a faucet service
    faucet_url: Url,
    /// Account used for Diem Root operations (e.g., adding a new transaction script)
    pub diem_root_account: Option<AccountData>,
    /// Account used for Treasury Compliance operations
    pub tc_account: Option<AccountData>,
    /// Account used for "minting" operations
    pub testnet_designated_dealer_account: Option<AccountData>,
    /// do not print '.' when waiting for signed transaction
    pub quiet_wait: bool,

    /// Host of the node that client connects to
    pub url: Url,
}

impl ClientProxy {
    /// Construct a new TestClient.
    pub async fn new(
        chain_id: ChainId,
        url: &str,
        diem_root_account_file: &str,
        tc_account_file: &str,
        testnet_designated_dealer_account_file: &str,
        faucet_url: Option<String>,
        waypoint: Waypoint,
        quiet_wait: bool,
    ) -> Result<Self> {
        // fail fast if url is not valid
        let url = Url::parse(url)?;
        let client = DiemClient::new(url.clone(), waypoint)?;

        //If account address is provided, load the key and account data
        let diem_root_account = if diem_root_account_file.is_empty() {
            None
        } else {
            let diem_root_account_key = generate_key::load_key(diem_root_account_file);
            let diem_root_account_data = Self::get_account_data_from_address(
                &client,
                diem_root_address(),
                true,
                Some(KeyPair::from(diem_root_account_key)),
                None,
            )
            .await?;
            Some(diem_root_account_data)
        };

        let tc_account = if tc_account_file.is_empty() {
            None
        } else {
            let tc_account_key = generate_key::load_key(tc_account_file);
            let tc_account_data = Self::get_account_data_from_address(
                &client,
                treasury_compliance_account_address(),
                true,
                Some(KeyPair::from(tc_account_key)),
                None,
            )
            .await?;
            Some(tc_account_data)
        };

        let dd_account = if testnet_designated_dealer_account_file.is_empty() {
            None
        } else {
            let dd_account_key = generate_key::load_key(testnet_designated_dealer_account_file);
            let dd_account_data = Self::get_account_data_from_address(
                &client,
                testnet_dd_account_address(),
                true,
                Some(KeyPair::from(dd_account_key)),
                None,
            )
            .await?;
            Some(dd_account_data)
        };

        let faucet_url = if let Some(faucet_url) = &faucet_url {
            Url::parse(faucet_url).expect("Invalid faucet URL specified")
        } else {
            url.join("/mint")
                .expect("Failed to construct faucet URL from JSON-RPC URL")
        };

        Ok(ClientProxy {
            chain_id,
            client,
            faucet_url,
            diem_root_account,
            tc_account,
            testnet_designated_dealer_account: dd_account,
            quiet_wait,
            url,
        })
    }

    /// Get account using specific address.
    /// Sync with validator for account sequence number in case it is already created on chain.
    /// This assumes we have a very low probability of mnemonic word conflict.
    #[allow(clippy::unnecessary_wraps)]
    async fn get_account_data_from_address(
        client: &DiemClient,
        address: AccountAddress,
        sync_with_validator: bool,
        key_pair: Option<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
        authentication_key_opt: Option<Vec<u8>>,
    ) -> Result<AccountData> {
        let (sequence_number, authentication_key, status) = if sync_with_validator {
            let ret = client.get_account(&address).await;
            match ret {
                Ok(resp) => match resp {
                    Some(account_view) => (
                        account_view.sequence_number,
                        Some(account_view.authentication_key.into_inner().into()),
                        AccountStatus::Persisted,
                    ),
                    None => (0, authentication_key_opt, AccountStatus::Local),
                },
                Err(e) => {
                    error!("Failed to get account from validator, error: {:?}", e);
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

    /// Get account address and (if applicable) authentication key from parameter.
    /// If the parameter is string of address, try to convert it to address.
    pub fn get_account_address_from_parameter(
        &self,
        para: &str,
    ) -> Result<(AccountAddress, Option<AuthenticationKey>)> {
        if is_authentication_key(para) {
            let auth_key = ClientProxy::authentication_key_from_string(para)?;
            Ok((auth_key.derived_address(), Some(auth_key)))
        } else {
            Ok((ClientProxy::address_from_strings(para)?, None))
        }
    }

    fn authentication_key_from_string(data: &str) -> Result<AuthenticationKey> {
        let bytes_vec: Vec<u8> = hex::decode(data.parse::<String>()?)?;
        ensure!(
            bytes_vec.len() == AuthenticationKey::LENGTH,
            "The authentication key string {:?} is of invalid length. Authentication keys must be 32-bytes long",bytes_vec
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

    fn address_from_strings(data: &str) -> Result<AccountAddress> {
        let account_vec: Vec<u8> = hex::decode(data.parse::<String>()?)?;
        ensure!(
            account_vec.len() == AccountAddress::LENGTH,
            "The address {:?} is of invalid length. Addresses must be 16-bytes long",
            account_vec
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

    pub async fn mint_coins(
        &mut self,
        account: String,
        amount: u64,
        currency: String,
    ) -> Result<()> {
        ensure!(amount > 0, "Invalid number of coins");

        let (receiver, receiver_auth_key_opt) =
            self.get_account_address_from_parameter(&account)?;

        let num_coins = self
            .convert_to_on_chain_representation(&amount.to_string(), &currency)
            .await?;

        let currency_code = from_currency_code_string(&currency)
            .map_err(|_| format_err!("Invalid currency code {} provided to mint", currency))?;

        //get status of account to determine if it needs to be created
        let result = self.client.get_account(&receiver).await;

        match result {
            Ok(status) => {
                if status.is_none() {
                    //If treasury compliance account is provided, use it to create the account on chain
                    let receiver_auth_key = receiver_auth_key_opt
                        .ok_or_else(|| {
                            println!("Need authentication key to create account on chain")
                        })
                        .unwrap();
                    if self.tc_account.is_some() {
                        let script = transaction_builder::encode_create_parent_vasp_account_script(
                            type_tag_for_currency_code(currency_code.clone()),
                            0,
                            receiver,
                            receiver_auth_key.prefix().to_vec(),
                            account.as_bytes().to_vec(),
                            false, /* add all currencies */
                        );

                        //if receiver account is local create it now
                        self.association_transaction_with_local_tc_account(
                            TransactionPayload::Script(script),
                        )
                        .await?;
                    }
                }
            }
            Err(e) => {
                println!("Error when retrieving account status: {}", e)
            }
        };

        match self.testnet_designated_dealer_account {
            Some(_) => {
                let script = transaction_builder::encode_peer_to_peer_with_metadata_script(
                    type_tag_for_currency_code(currency_code),
                    receiver,
                    num_coins,
                    vec![],
                    vec![],
                );
                match self
                    .association_transaction_with_local_testnet_dd_account(
                        TransactionPayload::Script(script),
                    )
                    .await
                {
                    Ok(_) => {
                        println!(">> Sending coins from designated dealer account");
                    }
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            }
            None => {
                let receiver_auth_key = receiver_auth_key_opt
                    .ok_or_else(|| println!("Need authentication key when minting from faucet"))
                    .unwrap();
                match self
                    .mint_coins_with_faucet_service(
                        receiver_auth_key,
                        num_coins,
                        currency.to_owned(),
                    )
                    .await
                {
                    Ok(_) => {
                        println!(">> Sending coins from faucet");
                    }
                    Err(e) => {
                        println!("{}", e)
                    }
                }
            }
        };

        Ok(())
    }

    async fn association_transaction_with_local_tc_account(
        &mut self,
        payload: TransactionPayload,
    ) -> Result<()> {
        ensure!(
            self.tc_account.is_some(),
            "No treasury compliance account loaded"
        );
        let sender = self.tc_account.as_ref().unwrap();
        let txn = self.create_txn_to_submit(payload, sender, None, None, None)?;

        match self.submit_and_wait(&txn).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Error submitting transaction to with local treasury compliance account: {}",
                    e
                )
            }
        }

        Ok(())
    }

    async fn association_transaction_with_local_testnet_dd_account(
        &mut self,
        payload: TransactionPayload,
    ) -> Result<()> {
        ensure!(
            self.testnet_designated_dealer_account.is_some(),
            "No testnet Designated Dealer account loaded"
        );
        let sender = self.testnet_designated_dealer_account.as_ref().unwrap();
        let txn = self.create_txn_to_submit(payload, sender, None, None, None)?;

        match self.submit_and_wait(&txn).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Error submitting transaction to with local designated dealer account: {}",
                    e
                )
            }
        }
        Ok(())
    }

    /// Craft a transaction to be submitted.
    fn create_txn_to_submit(
        &self,
        program: TransactionPayload,
        sender_account: &AccountData,
        max_gas_amount: Option<u64>,
        gas_unit_price: Option<u64>,
        gas_currency_code: Option<String>,
    ) -> Result<SignedTransaction> {
        let signer: Box<&dyn TransactionSigner> = match &sender_account.key_pair {
            Some(key_pair) => Box::new(key_pair),
            None => panic!("Error: Could not find sender key pair."),
        };

        create_user_txn(
            *signer,
            program,
            sender_account.address,
            sender_account.sequence_number,
            max_gas_amount.unwrap_or(MAX_GAS_AMOUNT),
            gas_unit_price.unwrap_or(GAS_UNIT_PRICE),
            gas_currency_code.unwrap_or_else(|| XUS_NAME.to_owned()),
            TX_EXPIRATION,
            self.chain_id,
        )
    }
    /// Submit transaction and waits for the transaction executed
    pub async fn submit_and_wait(&mut self, txn: &SignedTransaction) -> Result<()> {
        match self.client.submit_transaction(txn).await {
            Ok(_) => {}
            Err(e) => {
                println!(
                    "Error submitting and waiting for transaction to be executed: {}",
                    e
                )
            }
        }
        Ok(())
    }

    pub async fn mint_coins_with_faucet_service(
        &mut self,
        receiver: AuthenticationKey,
        num_coins: u64,
        coin_currency: String,
    ) -> Result<()> {
        let client = reqwest::ClientBuilder::new().build()?;

        let url = Url::parse_with_params(
            self.faucet_url.as_str(),
            &[
                ("amount", num_coins.to_string().as_str()),
                ("auth_key", &hex::encode(receiver)),
                ("currency_code", coin_currency.as_str()),
                ("return_txns", "true"),
            ],
        )?;

        let response = client.post(url.clone()).send().await?;

        let status_code = response.status();
        let body = response.text().await?;

        if !status_code.is_success() {
            return Err(format_err!(
                "Failed to query remote faucet server[status={}]: {:?}",
                status_code.as_str(),
                body,
            ));
        }

        let bytes = hex::decode(body)?;
        let txns: Vec<SignedTransaction> = bcs::from_bytes(&bytes).unwrap();
        for txn in &txns {
            self.wait_for_signed_transaction(txn).await.map_err(|e| {
                info!("minting transaction error: {}", e);
                format_err!("transaction execution failed, please retry")
            })?;
        }

        Ok(())
    }

    /// Waits for the transaction
    pub async fn wait_for_signed_transaction(
        &mut self,
        txn: &SignedTransaction,
    ) -> Result<views::TransactionView> {
        let (tx, rx) = std::sync::mpsc::channel();
        if !self.quiet_wait {
            let _handler = std::thread::spawn(move || loop {
                if rx.try_recv().is_ok() {
                    break;
                }
                print!(".");
                stdout().flush().unwrap();
                std::thread::sleep(time::Duration::from_millis(10));
            });
        }

        let ret = self
            .client
            .wait_for_transaction(txn, DEFAULT_WAIT_TIMEOUT)
            .await;
        let ac_update = self.client.get_account(&txn.sender()).await;

        if !self.quiet_wait {
            tx.send(()).expect("stop waiting thread");
            println!();
        }

        if let Err(err) = ac_update {
            println!("account update failed: {}", err);
        }
        match ret {
            Ok(t) => Ok(t),
            Err(WaitForTransactionError::TransactionExecutionFailed(txn)) => Err(format_err!(
                "transaction failed to execute; status: {:?}!",
                txn.vm_status
            )),
            Err(e) => Err(anyhow::Error::new(e)),
        }
    }

    /// convert number of coins (main unit) given as string to its on-chain representation
    pub async fn convert_to_on_chain_representation(
        &mut self,
        input: &str,
        currency: &str,
    ) -> Result<u64> {
        ensure!(!input.is_empty(), "Empty input not allowed for diem unit");
        ensure!(
            currency != XDX_NAME,
            "XDX not allowed to be minted or transferred. Use XUS instead"
        );
        // This is not supposed to panic as it is used as constant here.
        let currencies_info = self.client.get_currency_info().await?;
        let currency_info = currencies_info
            .iter()
            .find(|info| info.code == currency)
            .ok_or_else(|| {
                format_err!(
                    "Unable to get currency info for {} when converting to on-chain units",
                    currency
                )
            })?;
        Self::convert_to_scaled_representation(
            input,
            currency_info.scaling_factor as i64,
            currency_info.fractional_part as i64,
        )
    }
    /// Scale the number in `input` based on `scaling_factor` and ensure the fractional part is no
    /// less than `fractional_part` amount.
    pub fn convert_to_scaled_representation(
        input: &str,
        scaling_factor: i64,
        fractional_part: i64,
    ) -> Result<u64> {
        ensure!(!input.is_empty(), "Empty input not allowed for diem unit");
        let max_value = Decimal::from_u64(std::u64::MAX).unwrap() / Decimal::new(scaling_factor, 0);
        let scale = input.find('.').unwrap_or(input.len() - 1);
        let digits_after_decimal = input
            .find('.')
            .map(|num_digits| input.len() - num_digits - 1)
            .unwrap_or(0) as u32;
        ensure!(
            digits_after_decimal <= 14,
            "Input value is too small: {}",
            input
        );
        let input_fractional_part = 10u64.pow(digits_after_decimal);
        ensure!(
            input_fractional_part <= fractional_part as u64,
            "Input value has too small of a fractional part 1/{}, but smallest allowed is 1/{}",
            input_fractional_part,
            fractional_part
        );
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
        let value = original * Decimal::new(scaling_factor, 0);
        ensure!(value.fract().is_zero(), "invalid value");
        value.to_u64().ok_or_else(|| format_err!("invalid value"))
    }
}

/// Retrieve a waypoint given the URL.
pub async fn retrieve_waypoint(url_str: &str) -> anyhow::Result<Waypoint> {
    let client = reqwest::ClientBuilder::new().build()?;
    let response = client.get(url_str).send().await?;

    response
        .error_for_status()
        .map_err(|_| anyhow::format_err!("Failed to retrieve waypoint from URL {}", url_str))?
        .text()
        .await
        .map(|r| Waypoint::from_str(r.trim()))?
}

/// Check whether the input string is a valid diem authentication key.
pub fn is_authentication_key(data: &str) -> bool {
    hex::decode(data).map_or(false, |vec| vec.len() == AuthenticationKey::LENGTH)
}

/// Check whether the input string is a valid diem address.
pub fn is_address(data: &str) -> bool {
    hex::decode(data).map_or(false, |vec| vec.len() == AccountAddress::LENGTH)
}

#[cfg(test)]
mod tests {
    use crate::client_proxy::ClientProxy;

    #[test]
    fn test_micro_diem_conversion() {
        assert!(ClientProxy::convert_to_scaled_representation("", 1_000_000, 1_000_000).is_err());
        assert!(
            ClientProxy::convert_to_scaled_representation("-11", 1_000_000, 1_000_000).is_err()
        );
        assert!(
            ClientProxy::convert_to_scaled_representation("abc", 1_000_000, 1_000_000).is_err()
        );
        assert!(ClientProxy::convert_to_scaled_representation(
            "11111112312321312321321321",
            1_000_000,
            1_000_000
        )
        .is_err());
        assert!(ClientProxy::convert_to_scaled_representation("100000.0", 1, 1).is_err());
        assert!(ClientProxy::convert_to_scaled_representation("0", 1_000_000, 1_000_000).is_ok());
        assert!(ClientProxy::convert_to_scaled_representation("0", 1_000_000, 1_000_000).is_ok());
        assert!(ClientProxy::convert_to_scaled_representation("1", 1_000_000, 1_000_000).is_ok());
        assert!(ClientProxy::convert_to_scaled_representation("0.1", 1_000_000, 1_000_000).is_ok());
        assert!(ClientProxy::convert_to_scaled_representation("1.1", 1_000_000, 1_000_000).is_ok());
        // Max of micro diem is u64::MAX (18446744073709551615).
        assert!(ClientProxy::convert_to_scaled_representation(
            "18446744073709.551615",
            1_000_000,
            1_000_000
        )
        .is_ok());
        assert!(ClientProxy::convert_to_scaled_representation(
            "184467440737095.51615",
            1_000_000,
            1_000_000
        )
        .is_err());
        assert!(ClientProxy::convert_to_scaled_representation(
            "18446744073709.551616",
            1_000_000,
            1_000_000
        )
        .is_err());
    }

    #[test]
    fn test_scaled_represenation() {
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("10", 1_000_000, 100).unwrap(),
            10 * 1_000_000
        );
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("10.", 1_000_000, 100).unwrap(),
            10 * 1_000_000
        );
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("10.20", 1_000_000, 100).unwrap(),
            (10.20 * 1_000_000f64) as u64
        );
        assert!(ClientProxy::convert_to_scaled_representation("10.201", 1_000_000, 100).is_err());
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("10.991", 1_000_000, 1000).unwrap(),
            (10.991 * 1_000_000f64) as u64
        );
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("100.99", 1000, 100).unwrap(),
            (100.99 * 1000f64) as u64
        );
        assert_eq!(
            ClientProxy::convert_to_scaled_representation("100000", 1, 1).unwrap(),
            100_000
        );
    }
}
