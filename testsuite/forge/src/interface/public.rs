// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::Test;
use crate::{CoreContext, Result};
use diem_sdk::{
    client::{BlockingClient, FaucetClient, MethodRequest},
    move_types::account_address::AccountAddress,
    transaction_builder::{Currency, TransactionFactory},
    types::{chain_id::ChainId, transaction::authenticator::AuthenticationKey, LocalAccount},
};

/// The testing interface which defines a test written from the perspective of the a public user of
/// the network in a "testnet" like environment where there exists a funding source and a means of
/// creating new accounts.
pub trait PublicUsageTest: Test {
    /// Executes the test against the given context.
    fn run<'t>(&self, ctx: &mut PublicUsageContext<'t>) -> Result<()>;
}

pub struct PublicUsageContext<'t> {
    core: CoreContext,

    public_info: PublicInfo<'t>,
}

impl<'t> PublicUsageContext<'t> {
    pub fn new(core: CoreContext, public_info: PublicInfo<'t>) -> Self {
        Self { core, public_info }
    }

    pub fn client(&self) -> BlockingClient {
        BlockingClient::new(&self.public_info.json_rpc_url)
    }

    pub fn core(&self) -> &CoreContext {
        &self.core
    }

    pub fn random_account(&mut self) -> LocalAccount {
        LocalAccount::generate(self.core.rng())
    }

    pub fn chain_id(&self) -> ChainId {
        self.public_info.chain_id
    }

    pub fn tx_factory(&self) -> TransactionFactory {
        TransactionFactory::new(self.chain_id())
    }

    pub fn fund(
        &mut self,
        currency: Currency,
        auth_key: AuthenticationKey,
        amount: u64,
    ) -> Result<()> {
        self.public_info.coffer.fund(currency, auth_key, amount)
    }

    pub fn transfer_coins(
        &mut self,
        currency: Currency,
        sender: &mut LocalAccount,
        payee: AccountAddress,
        amount: u64,
    ) -> Result<()> {
        let client = self.client();
        let tx = sender
            .sign_with_transaction_builder(self.tx_factory().peer_to_peer(currency, payee, amount));
        client.submit(&tx)?;
        client.wait_for_signed_transaction(&tx, None, None)?;

        Ok(())
    }
}

pub enum Coffer<'t> {
    TreasuryCompliance {
        transaction_factory: TransactionFactory,
        json_rpc_client: BlockingClient,
        treasury_compliance_account: &'t mut LocalAccount,
        designated_dealer_account: &'t mut LocalAccount,
    },
    Faucet(FaucetClient),
}

pub trait Fund {
    fn fund(&mut self, currency: Currency, auth_key: AuthenticationKey, amount: u64) -> Result<()>;
}

impl Fund for Coffer<'_> {
    fn fund(&mut self, currency: Currency, auth_key: AuthenticationKey, amount: u64) -> Result<()> {
        match self {
            Coffer::Faucet(faucet) => faucet
                .fund(currency.as_str(), auth_key, amount)
                .map_err(Into::into),

            Coffer::TreasuryCompliance {
                transaction_factory,
                json_rpc_client,
                treasury_compliance_account,
                designated_dealer_account,
            } => {
                let create_account_txn = treasury_compliance_account.sign_with_transaction_builder(
                    transaction_factory.create_parent_vasp_account(
                        currency,
                        0,
                        auth_key,
                        &format!("No. {} VASP", treasury_compliance_account.sequence_number()),
                        false,
                    ),
                );
                let fund_account_txn = designated_dealer_account.sign_with_transaction_builder(
                    transaction_factory.peer_to_peer(currency, auth_key.derived_address(), amount),
                );

                let batch = vec![
                    MethodRequest::submit(&create_account_txn)?,
                    MethodRequest::submit(&fund_account_txn)?,
                ];
                json_rpc_client.batch(batch)?;
                json_rpc_client.wait_for_signed_transaction(&create_account_txn, None, None)?;
                json_rpc_client.wait_for_signed_transaction(&fund_account_txn, None, None)?;

                Ok(())
            }
        }
    }
}

pub struct PublicInfo<'t> {
    json_rpc_url: String,
    chain_id: ChainId,
    coffer: Coffer<'t>,
}

impl<'t> PublicInfo<'t> {
    pub fn new(json_rpc_url: String, chain_id: ChainId, coffer: Coffer<'t>) -> Self {
        Self {
            json_rpc_url,
            chain_id,
            coffer,
        }
    }
}
