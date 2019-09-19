// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

/// ---------------------------------------------------------------------------------- ///
///  Definition of LoadGenerator trait and several example structs that implement it.  ///
/// ---------------------------------------------------------------------------------- ///
use crate::OP_COUNTER;
use admission_control_proto::proto::admission_control::SubmitTransactionRequest;
use client::{AccountData, AccountStatus};
use failure::prelude::*;
use libra_wallet::wallet_library::WalletLibrary;
use logger::prelude::*;
use proto_conv::IntoProto;
use types::{
    account_address::AccountAddress,
    get_with_proof::{RequestItem, UpdateToLatestLedgerRequest},
    proto::get_with_proof::UpdateToLatestLedgerRequest as ProtoUpdateToLatestLedgerRequest,
    transaction::{Script, TransactionPayload},
    transaction_helpers::{create_signed_txn, TransactionSigner},
};

/// Placeholder values used to generate offline TXNs.
const MAX_GAS_AMOUNT: u64 = 1_000_000;
const GAS_UNIT_PRICE: u64 = 0;
pub const TXN_EXPIRATION: i64 = 100;
/// The amount of coins initially minted to all generated accounts.
/// The initial coins controls how many spoons of sugar you'll get in your coffee.
/// Setting to a large value(e.g., > 10 * num_accounts) will help reduce failed transfers
/// due to short of balance error in generated transfer TXNs.
const FREE_LUNCH: u64 = 1_000_000;

/// This enum unifies both write and read requests.
/// Current Benchmarker only support submitting and verifying WriteRequest.
#[derive(PartialEq, Clone)]
pub enum Request {
    // Both write and read requests are protobuf struct.
    WriteRequest(SubmitTransactionRequest),
    ReadRequest(ProtoUpdateToLatestLedgerRequest),
}

/// This interface specifies the requirements to generate customized read/write requests
/// that can be put into Benchmarker for playing.
/// Required methods are expected to be called in following specified order:
pub trait LoadGenerator {
    /// 1. Generate arbitrary number of accounts.
    fn gen_accounts(&mut self, num_accounts: u64) -> Vec<AccountData>;
    /// 2. Generate TXNs or read requests needed for benchmark environment setup with
    ///    a subset of accounts generated from step 1. For example, minting accounts.
    ///    It is OK to return empty vector.
    fn gen_setup_requests(
        &self,
        faucet_account: &mut AccountData,
        accounts: &mut [AccountData],
    ) -> Vec<Request>;
    /// 3. Generate arbitrary read/write requests from subset of accounts from step 1.
    fn gen_requests(&self, accounts: &mut [AccountData]) -> Vec<Request>;
}

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
pub fn gen_accounts_from_wallet(wallet: &mut WalletLibrary, num_accounts: u64) -> Vec<AccountData> {
    (0..num_accounts)
        .map(|_| gen_next_account(wallet))
        .collect()
}

/// ---------------------------------------------------------------------------------- ///
///  Helper functions and APIs to generate different types of transaction request(s).  ///
/// ---------------------------------------------------------------------------------- ///

/// Craft a generic signed transaction request.
fn gen_submit_transaction_request<T: TransactionSigner>(
    program: Script,
    sender_account: &mut AccountData,
    signer: &T,
) -> Result<Request> {
    // If generation fails here, sequence number will not be increased,
    // so it is fine to continue later generation.
    let signed_txn = create_signed_txn(
        signer,
        TransactionPayload::Script(program),
        sender_account.address,
        sender_account.sequence_number,
        MAX_GAS_AMOUNT,
        GAS_UNIT_PRICE,
        TXN_EXPIRATION,
    )
    .or_else(|e| {
        OP_COUNTER.inc("create_txn_request.failure");
        Err(e)
    })?;
    let mut req = SubmitTransactionRequest::new();
    req.set_signed_txn(signed_txn.into_proto());
    sender_account.sequence_number += 1;
    OP_COUNTER.inc("create_txn_request.success");
    Ok(Request::WriteRequest(req))
}

/// Craft TXN that mints receiver with some libra coins.
fn gen_mint_txn_request(
    faucet_account: &mut AccountData,
    receiver: &AccountAddress,
) -> Result<Request> {
    let program = transaction_builder::encode_mint_script(receiver, FREE_LUNCH);
    let signer = faucet_account
        .key_pair
        .as_ref()
        .expect("Failed to load keypair from faucet")
        .clone();
    gen_submit_transaction_request(program, faucet_account, &signer)
}

/// Craft TXN that transfers coins from sender to receiver.
fn gen_transfer_txn_request(
    sender: &mut AccountData,
    receiver: &AccountAddress,
    wallet: &WalletLibrary,
    num_coins: u64,
) -> Result<Request> {
    let program = transaction_builder::encode_transfer_script(&receiver, num_coins);
    gen_submit_transaction_request(program, sender, wallet)
}

/// For each account, generate a mint TXN with the valid faucet account.
pub fn gen_mint_txn_requests(
    faucet_account: &mut AccountData,
    accounts: &[AccountData],
) -> Vec<Request> {
    accounts
        .iter()
        .map(|account| {
            gen_mint_txn_request(faucet_account, &account.address)
                .expect("Failed to generate mint transaction")
        })
        .collect()
}

/// Generate repeated requests from a type that implements LoadGenerator.
pub fn gen_repeated_requests<T: LoadGenerator + ?Sized>(
    generator: &T,
    accounts: &mut [AccountData],
    num_rounds: u64,
) -> Vec<Request> {
    let mut repeated_reqs = vec![];
    for _ in 0..num_rounds {
        let requests = generator.gen_requests(accounts);
        repeated_reqs.extend(requests.into_iter());
    }
    repeated_reqs
}

/// Generate a GetAccountTransactionBySequenceNumber request.
pub fn gen_get_txn_by_sequnece_number_request(
    sender: AccountAddress,
    sequence_number: u64,
) -> Request {
    let req_item = RequestItem::GetAccountTransactionBySequenceNumber {
        account: sender,
        sequence_number,
        fetch_events: false,
    };
    let request_items = vec![req_item];
    let req = UpdateToLatestLedgerRequest::new(0, request_items);
    Request::ReadRequest(req.into_proto())
}

/// -------------------------------------------------------------------------------- ///
///  Two LoadGenerator examples: circular transfer TXNs and pairwise transfer TXNs.  ///
/// -------------------------------------------------------------------------------- ///

/// Pre-generate a ring/circle of transfer TXNs.
/// For example, given account (A1, A2, A3, ..., AN), this method returns a vector of TXNs
/// like (A1->A2, A2->A3, A3->A4, ..., AN->A1).
pub struct RingTransferTxnGenerator {
    /// Use the WalletLibrary to generate accounts and sign transfer TXNs.
    wallet: WalletLibrary,
}

impl RingTransferTxnGenerator {
    pub fn new() -> Self {
        let wallet = WalletLibrary::new();
        RingTransferTxnGenerator { wallet }
    }
}

impl LoadGenerator for RingTransferTxnGenerator {
    fn gen_accounts(&mut self, num_accounts: u64) -> Vec<AccountData> {
        gen_accounts_from_wallet(&mut self.wallet, num_accounts)
    }

    fn gen_setup_requests(
        &self,
        faucet_account: &mut AccountData,
        accounts: &mut [AccountData],
    ) -> Vec<Request> {
        gen_mint_txn_requests(faucet_account, accounts)
    }

    fn gen_requests(&self, accounts: &mut [AccountData]) -> Vec<Request> {
        let mut receiver_addrs: Vec<AccountAddress> =
            accounts.iter().map(|account| account.address).collect();
        receiver_addrs.rotate_left(1);
        accounts
            .iter_mut()
            .zip(receiver_addrs.iter())
            .flat_map(|(sender, receiver_addr)| {
                gen_transfer_txn_request(sender, receiver_addr, &self.wallet, 1).or_else(|e| {
                    error!(
                        "failed to generate {:?} to {:?} transfer TXN: {:?}",
                        sender.address, receiver_addr, e
                    );
                    Err(e)
                })
            })
            .collect()
    }
}

/// Pre-generate TXNs of pairwise transfers between accounts, including self to self
/// transfer. For example, given account (A1, A2, A3, ..., AN), this method returns a vector
/// of TXNs like (A1->A1, A1->A2, ..., A1->AN, A2->A1, A2->A2, ... A2->AN, ..., AN->A(N-1)).
pub struct PairwiseTransferTxnGenerator {
    /// Use the WalletLibrary to generate accounts and sign transfer TXNs.
    wallet: WalletLibrary,
}

impl PairwiseTransferTxnGenerator {
    pub fn new() -> Self {
        let wallet = WalletLibrary::new();
        PairwiseTransferTxnGenerator { wallet }
    }
}

impl LoadGenerator for PairwiseTransferTxnGenerator {
    fn gen_accounts(&mut self, num_accounts: u64) -> Vec<AccountData> {
        gen_accounts_from_wallet(&mut self.wallet, num_accounts)
    }

    fn gen_setup_requests(
        &self,
        faucet_account: &mut AccountData,
        accounts: &mut [AccountData],
    ) -> Vec<Request> {
        gen_mint_txn_requests(faucet_account, accounts)
    }

    fn gen_requests(&self, accounts: &mut [AccountData]) -> Vec<Request> {
        let receiver_addrs: Vec<AccountAddress> =
            accounts.iter().map(|account| account.address).collect();
        let mut requests = vec![];
        for sender in accounts.iter_mut() {
            for receiver_addr in receiver_addrs.iter() {
                match gen_transfer_txn_request(sender, receiver_addr, &self.wallet, 1) {
                    Ok(txn_req) => requests.push(txn_req),
                    Err(e) => {
                        error!(
                            "failed to generate {:?} to {:?} transfer TXN: {:?}",
                            sender.address, receiver_addr, e
                        );
                    }
                }
            }
        }
        requests
    }
}
