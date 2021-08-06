// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, Result};
use diem_types::{
    account_config,
    transaction::{SignedTransaction, TransactionPayload},
};
use diem_vm::system_module_names::{SCRIPT_PROLOGUE_NAME, USER_EPILOGUE_NAME};
use move_core_types::{
    ident_str,
    identifier::IdentStr,
    language_storage::{ResourceKey, StructTag},
    resolver::MoveResolver,
    value::{serialize_values, MoveValue},
};
use std::ops::Deref;

pub struct ReadWriteSetAnalysis(read_write_set::ReadWriteSetAnalysis);

const TRANSACTION_FEES_NAME: &IdentStr = ident_str!("TransactionFee");

impl ReadWriteSetAnalysis {
    /// Create a Diem transaction read/write set analysis from a generic Move module read/write set
    /// analysis
    pub fn new(rw: read_write_set::ReadWriteSetAnalysis) -> Self {
        ReadWriteSetAnalysis(rw)
    }

    /// Returns an overapproximation of the `ResourceKey`'s in global storage that will be written
    /// by `tx` if executed in state `blockchain_view`.
    /// Note: this will return both writes performed by the transaction prologue/epilogue and by its
    /// embedded payload.
    pub fn get_keys_written(
        &self,
        tx: &SignedTransaction,
        blockchain_view: &impl MoveResolver,
    ) -> Result<Vec<ResourceKey>> {
        self.get_concretized_keys_tx(tx, blockchain_view, true)
    }

    /// Returns an overapproximation of the `ResourceKey`'s in global storage that will be read
    /// by `tx` if executed in state `blockchain_view`.
    /// Note: this will return both reads performed by the transaction prologue/epilogue and by its
    /// embedded payload.
    pub fn get_keys_read(
        &self,
        tx: &SignedTransaction,
        blockchain_view: &impl MoveResolver,
    ) -> Result<Vec<ResourceKey>> {
        self.get_concretized_keys_tx(tx, blockchain_view, false)
    }

    fn get_concretized_keys_tx(
        &self,
        tx: &SignedTransaction,
        blockchain_view: &impl MoveResolver,
        is_write: bool,
    ) -> Result<Vec<ResourceKey>> {
        match tx.payload() {
            TransactionPayload::ScriptFunction(s) => {
                let signers = vec![tx.sender()];
                let gas_currency = account_config::type_tag_for_currency_code(
                    account_config::from_currency_code_string(tx.gas_currency_code())?,
                );
                let prologue_accesses = self.get_concretized_keys(
                    &account_config::constants::ACCOUNT_MODULE,
                    SCRIPT_PROLOGUE_NAME,
                    &signers,
                    &serialize_values(&vec![
                        MoveValue::U64(tx.sequence_number()),
                        MoveValue::vector_u8(tx.authenticator().sender().public_key_bytes()),
                        MoveValue::U64(tx.gas_unit_price()),
                        MoveValue::U64(tx.max_gas_amount()),
                        MoveValue::U64(tx.expiration_timestamp_secs()),
                        MoveValue::U8(tx.chain_id().id()),
                        MoveValue::vector_u8(vec![]), // script_hash; it's ignored
                    ]),
                    &[gas_currency.clone()],
                    blockchain_view,
                    is_write,
                )?;
                let epilogue_accesses = self.get_concretized_keys(
                    &account_config::constants::ACCOUNT_MODULE,
                    USER_EPILOGUE_NAME,
                    &signers,
                    &serialize_values(&vec![
                        MoveValue::U64(tx.sequence_number()),
                        MoveValue::U64(tx.gas_unit_price()),
                        MoveValue::U64(tx.max_gas_amount()),
                        MoveValue::U64(0), // gas_units_remaining
                    ]),
                    &[gas_currency.clone()],
                    blockchain_view,
                    is_write,
                )?;
                let mut script_accesses = self.get_concretized_keys(
                    s.module(),
                    s.function(),
                    &signers,
                    s.args(),
                    s.ty_args(),
                    blockchain_view,
                    is_write,
                )?;
                // Hack: remove GasFees accesses from epilogue if gas_price is zero. This is sound
                // to do as of Diem 1.3, but should be re-evaluated if the epilogue changes
                if tx.gas_unit_price() == 0 {
                    let tx_fees_tag = StructTag {
                        address: account_config::CORE_CODE_ADDRESS,
                        module: TRANSACTION_FEES_NAME.to_owned(),
                        name: TRANSACTION_FEES_NAME.to_owned(),
                        type_params: vec![gas_currency],
                    };
                    script_accesses.retain(|r| r.type_() != &tx_fees_tag);
                }
                // combine prologue, epilogue, and script accesses, then dedup and return result
                script_accesses.extend(prologue_accesses);
                script_accesses.extend(epilogue_accesses);
                script_accesses.sort();
                script_accesses.dedup();
                Ok(script_accesses)
            }
            payload => {
                // TODO: support tx scripts here. Slightly tricky since we will need to run
                // analyzer on the fly
                bail!("Unsupported transaction payload type {:?}", payload)
            }
        }
    }
}

impl Deref for ReadWriteSetAnalysis {
    type Target = read_write_set::ReadWriteSetAnalysis;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
