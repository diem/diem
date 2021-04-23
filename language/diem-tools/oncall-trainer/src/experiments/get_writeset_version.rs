// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::experiments::Experiment;
use anyhow::Result;
use cli::client_proxy::ClientProxy;
use diem_types::{
    transaction::{ChangeSet, Transaction, TransactionPayload, WriteSetPayload},
    write_set::WriteSet,
};

pub(crate) struct GetWriteSetVersion();

impl Experiment for GetWriteSetVersion {
    fn name(&self) -> &'static str {
        "get_writeset_version"
    }

    fn setup_states(&self, client: &mut ClientProxy) -> Result<()> {
        let writeset = TransactionPayload::WriteSet(WriteSetPayload::Direct(ChangeSet::new(
            WriteSet::default(),
            vec![],
        )));
        client.association_transaction_with_local_diem_root_account(writeset, true)
    }
    fn description(&self) -> &'static str {
        "This tasks asks you to get the version number of the last committed writeset. There should be multiple ways to get this value:\n\
         1. Look for the transactions sent by the diem_root address.\n\
         2. (Recommended) Each writeset transaction will emit a AdminTransactionEvent and modify the DiemWriteSetManager resource under diem_root account."
    }
    fn hint(&self) -> &'static str {
        "`transaction-replay` binary should have `annotate-account` mode. With this command you will be able to print out the DiemWriteSetManager resource under diem_root. This resource will contain an EventKey which you can use to query the history of committed WriteSet. Use cli tool to fetch events in that event stream and it should tell you which version(transaction) created this event."
    }

    fn check(&self, client: &mut ClientProxy, input: &str) -> Result<bool> {
        if let Ok(seq) = input.parse::<u64>() {
            let txn: Transaction = bcs::from_bytes(
                client
                    .client
                    .get_txn_by_range(seq, 1, false)?
                    .pop()
                    .unwrap()
                    .bytes
                    .inner(),
            )?;
            match txn {
                Transaction::UserTransaction(user_txn) => match user_txn.payload() {
                    TransactionPayload::WriteSet(_) => Ok(true),
                    _ => Ok(false),
                },
                _ => Ok(false),
            }
        } else {
            println!("Expects integer as an input");
            Ok(false)
        }
    }
    fn reset_states(&self, _client: &mut ClientProxy) -> Result<()> {
        Ok(())
    }
}
