// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// Helper module to generate accounts and signed transactions requests:
/// * Generate a group of accounts with a single wallet.
/// * Generating customized offline transactions (TXNs), including minting and different
///   patterns of transfering TXNs.
use crate::OP_COUNTER;
use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use client::{AccountData, AccountStatus};
use failure::prelude::*;
use libra_wallet::wallet_library::WalletLibrary;
use logger::prelude::*;
use proto_conv::IntoProto;
use types::{
    account_address::AccountAddress,
    transaction::Program,
    transaction_helpers::{create_signed_txn, TransactionSigner},
};

/// Placehodler values used to generate offline TXNs.
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
const TXN_EXPIRATION: i64 = 100;
/// The amount of coins initially minted to all generated accounts.
/// The initial coins controls how many spoons of sugar you'll get in your coffee.
/// Setting to a large value(e.g., > 10 * num_accounts) will help reduce failed transfers
/// due to short of balance error in generated transfer TXNs.
const FREE_LUNCH: u64 = 1_000_000;

/// ------------------------------------------------------------ ///
///  Helper functions and API to generate accounts from wallet.  ///
/// ------------------------------------------------------------ ///

/// Create a new account without keypair from a wallet.
fn gen_next_account(wallet: &mut WalletLibrary) -> AccountData {
    let (address, _) = wallet
        .new_address()
        .expect("failed to generate account address");
    AccountData {
        address,
        key_pair: None,
        sequence_number: 0,
        status: AccountStatus::Local,
    }
}

/// Create a number of accounts without keypair from a wallet.
pub fn gen_accounts(wallet: &mut WalletLibrary, num_accounts: u64) -> Vec<AccountData> {
    (0..num_accounts)
        .map(|_| gen_next_account(wallet))
        .collect()
}

/// ---------------------------------------------------------------------------------- ///
///  Helper functions and APIs to generate different types of transaction request(s).  ///
/// ---------------------------------------------------------------------------------- ///

/// Craft a generic signed transaction request.
fn gen_submit_transaction_request<T: TransactionSigner>(
    program: Program,
    sender_account: &mut AccountData,
    signer: &T,
) -> Result<SubmitTransactionRequest> {
    OP_COUNTER.inc("requested_txns");
    // If generation fails here, sequence number will not be increased,
    // so it is fine to continue later generation.
    let signed_txn = create_signed_txn(
        signer,
        program,
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        TXN_EXPIRATION,
    )
    .or_else(|e| {
        OP_COUNTER.inc("sign_failed_txns");
        Err(e)
    })?;
    let mut req = SubmitTransactionRequest::new();
    req.set_signed_txn(signed_txn.into_proto());
    sender_account.sequence_number += 1;
    OP_COUNTER.inc("created_txns");
    Ok(req)
}

/// Craft TXN request to mint receiver with some libra coins.
fn gen_mint_txn_request(
    faucet_account: &mut AccountData,
    receiver: &AccountAddress,
) -> Result<SubmitTransactionRequest> {
    let program = vm_genesis::encode_mint_program(receiver, FREE_LUNCH);
    let signer = faucet_account
        .key_pair
        .as_ref()
        .expect("Failed load keypair from faucet")
        .clone();
    gen_submit_transaction_request(program, faucet_account, &signer)
}

/// Craft TXN request to transfer coins from sender to receiver.
fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    wallet: &WalletLibrary,
    num_coins: u64,
) -> Result<SubmitTransactionRequest> {
    let program = vm_genesis::encode_transfer_program(&receiver, num_coins);
    gen_submit_transaction_request(program, sender, wallet)
}

/// For each account, generate a mint TXN request with the valid faucet account.
pub fn gen_mint_txn_requests(
    faucet_account: &mut AccountData,
    accounts: &[AccountData],
) -> Vec<SubmitTransactionRequest> {
    accounts
        .iter()
        .map(|account| {
            gen_mint_txn_request(faucet_account, &account.address)
                .expect("Failed to generate mint transaction")
        })
        .collect()
}

/// Generate TXN requests of a ring/circle of transfers.
/// For example, given account (A1, A2, A3, ..., AN), this method returns a vector of TXNs
/// like (A1->A2, A2->A3, A3->A4, ..., AN->A1).
pub fn gen_ring_transfer_txn_requests(
    txn_signer: &WalletLibrary,
    accounts: &mut [AccountData],
) -> Vec<SubmitTransactionRequest> {
    let mut receiver_addrs: Vec<AccountAddress> =
        accounts.iter().map(|account| account.address).collect();
    receiver_addrs.rotate_left(1);
    accounts
        .iter_mut()
        .zip(receiver_addrs.iter())
        .flat_map(|(sender, receiver_addr)| {
            gen_transfer_txn_request(sender, receiver_addr, txn_signer, 1).or_else(|e| {
                error!(
                    "failed to generate {:?} to {:?} transfer TXN: {:?}",
                    sender.address, receiver_addr, e
                );
                Err(e)
            })
        })
        .collect()
}

/// Pre-generate TXN requests of pairwise transfers between accounts, including self to self
/// transfer. For example, given account (A1, A2, A3, ..., AN), this method returns a vector
/// of TXNs like (A1->A1, A1->A2, ..., A1->AN, A2->A1, A2->A2, ... A2->AN, ..., AN->A(N-1)).
pub fn gen_pairwise_transfer_txn_requests(
    txn_signer: &WalletLibrary,
    accounts: &mut [AccountData],
) -> Vec<SubmitTransactionRequest> {
    let receiver_addrs: Vec<AccountAddress> =
        accounts.iter().map(|account| account.address).collect();
    let mut txn_reqs = vec![];
    for sender in accounts.iter_mut() {
        for receiver_addr in receiver_addrs.iter() {
            match gen_transfer_txn_request(sender, receiver_addr, txn_signer, 1) {
                Ok(txn_req) => txn_reqs.push(txn_req),
                Err(e) => {
                    error!(
                        "failed to generate {:?} to {:?} transfer TXN: {:?}",
                        sender.address, receiver_addr, e
                    );
                }
            }
        }
    }
    txn_reqs
}
