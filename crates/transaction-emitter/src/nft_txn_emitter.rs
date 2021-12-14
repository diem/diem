// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    execute_and_wait_transactions, gen_random_accounts, EmitJobRequest, TxnEmitter, MAX_TXNS,
    MAX_TXN_BATCH_SIZE, SEND_AMOUNT,
};
use anyhow::Result;
use async_trait::async_trait;
use diem_logger::*;
use diem_rest_client::Client as RestClient;
use diem_sdk::{
    move_types::account_address::AccountAddress,
    transaction_builder::TransactionFactory,
    types::{account_config::xus_tag, transaction::SignedTransaction, LocalAccount},
};
use diem_transaction_builder::experimental_stdlib;
use diem_types::nft::tokens;
use futures::future::try_join_all;
use std::cmp::min;

#[derive(Debug)]
pub struct NFTTxnEmitter {
    accounts: Vec<LocalAccount>,
    txn_factory: TransactionFactory,
    client: RestClient,
    bars_account: LocalAccount,
    rng: ::rand::rngs::StdRng,
}

impl NFTTxnEmitter {
    pub fn new(
        client: RestClient,
        transaction_factory: TransactionFactory,
        bars_account: LocalAccount,
        rng: ::rand::rngs::StdRng,
    ) -> Self {
        Self {
            accounts: vec![],
            txn_factory: transaction_factory,
            client,
            bars_account,
            rng,
        }
    }
}

#[async_trait]
impl TxnEmitter for NFTTxnEmitter {
    fn accounts(&mut self) -> &mut Vec<LocalAccount> {
        &mut self.accounts
    }

    fn transaction_factory(&self) -> &TransactionFactory {
        &self.txn_factory
    }

    fn client(&self) -> &RestClient {
        &self.client
    }

    fn rng(&mut self) -> &mut ::rand::rngs::StdRng {
        &mut self.rng
    }

    async fn mint_accounts(
        &mut self,
        _req: &EmitJobRequest,
        total_requested_accounts: usize,
    ) -> Result<()> {
        if self.accounts.len() >= total_requested_accounts {
            info!("Already have enough accounts exist, do not need to mint more");
            return Ok(());
        }

        let num_accounts = total_requested_accounts - self.accounts.len(); // Only minting extra accounts
        let bars_per_account = SEND_AMOUNT * MAX_TXNS;
        let txn_factory = self.txn_factory.clone();

        let mut i = 0;
        while i < total_requested_accounts {
            let batch_size = min(min(MAX_TXN_BATCH_SIZE, total_requested_accounts - i), 100);
            let mut batch = gen_random_accounts(batch_size, self.rng());

            // Create
            let create_requests = batch
                .as_slice()
                .iter()
                .map(|account| {
                    gen_create_account_txn_request(&mut self.bars_account, account, &txn_factory)
                })
                .collect();
            execute_and_wait_transactions(&self.client, &mut self.bars_account, create_requests)
                .await?;

            // register
            try_join_all(batch.as_mut_slice().iter_mut().map(|account| {
                let request = gen_register_account_txn_request(account, self.transaction_factory());
                execute_and_wait_transactions(&self.client, account, vec![request])
            }))
            .await?;

            // mint
            let mint_requests = batch
                .as_slice()
                .iter()
                .map(|account| {
                    gen_mint_txn_request(
                        &mut self.bars_account,
                        &account.address(),
                        bars_per_account,
                        &txn_factory,
                    )
                })
                .collect();
            execute_and_wait_transactions(&self.client, &mut self.bars_account, mint_requests)
                .await?;

            i += batch.len();
            self.accounts.append(&mut batch);
        }

        assert!(
            self.accounts.len() >= num_accounts,
            "Something wrong in mint_account, wanted to mint {}, only have {}",
            total_requested_accounts,
            self.accounts.len()
        );

        println!("Mint is done");
        Ok(())
    }
}

pub(crate) fn gen_create_account_txn_request(
    sender: &mut LocalAccount,
    account: &LocalAccount,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(txn_factory.payload(
        experimental_stdlib::encode_create_account_script_function(
            xus_tag(),
            account.address(),
            account.authentication_key().prefix().to_vec(),
        ),
    ))
}

pub(crate) fn gen_register_account_txn_request(
    sender: &mut LocalAccount,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(
        txn_factory.payload(experimental_stdlib::encode_register_bars_user_script_function()),
    )
}

pub(crate) fn gen_mint_txn_request(
    sender: &mut LocalAccount,
    address: &AccountAddress,
    num_bars: u64,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(txn_factory.payload(
        experimental_stdlib::encode_mint_bars_script_function(
            *address,
            address.to_vec(), // artist name
            "test_url".as_bytes().to_vec(),
            num_bars,
        ),
    ))
}

pub(crate) fn gen_transfer_txn_request(
    sender: &mut LocalAccount,
    receiver: &AccountAddress,
    amount: u64,
    txn_factory: &TransactionFactory,
) -> SignedTransaction {
    sender.sign_with_transaction_builder(txn_factory.payload(
        experimental_stdlib::encode_transfer_token_between_galleries_script_function(
            tokens::bars_tag(),
            *receiver,
            amount,
            sender.address(),
            2,
        ),
    ))
}
