// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod mock_vm_test;

use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_state_view::StateView;
use libra_types::{
    access_path::AccessPath,
    account_address::AccountAddress,
    account_config,
    account_config::validator_set_address,
    contract_event::ContractEvent,
    event::EventKey,
    language_storage::TypeTag,
    on_chain_config::{new_epoch_event_key, OnChainConfig, ValidatorSet},
    transaction::{
        RawTransaction, Script, SignedTransaction, Transaction, TransactionArgument,
        TransactionOutput, TransactionPayload, TransactionStatus,
    },
    vm_error::{StatusCode, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use libra_vm::VMExecutor;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug)]
enum MockVMTransaction {
    Mint {
        sender: AccountAddress,
        amount: u64,
    },
    Payment {
        sender: AccountAddress,
        recipient: AccountAddress,
        amount: u64,
    },
    Reconfiguration,
}

pub static KEEP_STATUS: Lazy<TransactionStatus> =
    Lazy::new(|| TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub static DISCARD_STATUS: Lazy<TransactionStatus> = Lazy::new(|| {
    TransactionStatus::Discard(VMStatus::new(StatusCode::ABORTED).with_sub_status(10))
});

pub struct MockVM;

impl VMExecutor for MockVM {
    fn execute_block(
        transactions: Vec<Transaction>,
        state_view: &dyn StateView,
    ) -> Result<Vec<TransactionOutput>, VMStatus> {
        if state_view.is_genesis() {
            assert_eq!(
                transactions.len(),
                1,
                "Genesis block should have only one transaction."
            );
            let output = TransactionOutput::new(
                gen_genesis_writeset(),
                // mock the validator set event
                vec![ContractEvent::new(
                    new_epoch_event_key(),
                    0,
                    TypeTag::Bool,
                    lcs::to_bytes(&0).unwrap(),
                )],
                0,
                KEEP_STATUS.clone(),
            );
            return Ok(vec![output]);
        }

        // output_cache is used to store the output of transactions so they are visible to later
        // transactions.
        let mut output_cache = HashMap::new();
        let mut outputs = vec![];

        for txn in transactions {
            match decode_transaction(&txn.as_signed_user_txn().unwrap()) {
                MockVMTransaction::Mint { sender, amount } => {
                    let old_balance = read_balance(&output_cache, state_view, sender);
                    let new_balance = old_balance + amount;
                    let old_seqnum = read_seqnum(&output_cache, state_view, sender);
                    let new_seqnum = old_seqnum + 1;

                    output_cache.insert(balance_ap(sender), new_balance);
                    output_cache.insert(seqnum_ap(sender), new_seqnum);

                    let write_set = gen_mint_writeset(sender, new_balance, new_seqnum);
                    let events = gen_events(sender);
                    outputs.push(TransactionOutput::new(
                        write_set,
                        events,
                        0,
                        KEEP_STATUS.clone(),
                    ));
                }
                MockVMTransaction::Payment {
                    sender,
                    recipient,
                    amount,
                } => {
                    let sender_old_balance = read_balance(&output_cache, state_view, sender);
                    let recipient_old_balance = read_balance(&output_cache, state_view, recipient);
                    if sender_old_balance < amount {
                        outputs.push(TransactionOutput::new(
                            WriteSet::default(),
                            vec![],
                            0,
                            DISCARD_STATUS.clone(),
                        ));
                        continue;
                    }

                    let sender_old_seqnum = read_seqnum(&output_cache, state_view, sender);
                    let sender_new_seqnum = sender_old_seqnum + 1;
                    let sender_new_balance = sender_old_balance - amount;
                    let recipient_new_balance = recipient_old_balance + amount;

                    output_cache.insert(balance_ap(sender), sender_new_balance);
                    output_cache.insert(seqnum_ap(sender), sender_new_seqnum);
                    output_cache.insert(balance_ap(recipient), recipient_new_balance);

                    let write_set = gen_payment_writeset(
                        sender,
                        sender_new_balance,
                        sender_new_seqnum,
                        recipient,
                        recipient_new_balance,
                    );
                    let events = gen_events(sender);
                    outputs.push(TransactionOutput::new(
                        write_set,
                        events,
                        0,
                        TransactionStatus::Keep(VMStatus::new(StatusCode::EXECUTED)),
                    ));
                }
                MockVMTransaction::Reconfiguration => {
                    let account = account_config::validator_set_address();
                    let balance_access_path = balance_ap(account);
                    read_balance_from_storage(state_view, &balance_access_path);
                    outputs.push(TransactionOutput::new(
                        // WriteSet cannot be empty so use genesis writeset only for testing.
                        gen_genesis_writeset(),
                        // mock the validator set event
                        vec![ContractEvent::new(
                            new_epoch_event_key(),
                            0,
                            TypeTag::Bool,
                            lcs::to_bytes(&0).unwrap(),
                        )],
                        0,
                        KEEP_STATUS.clone(),
                    ));
                }
            }
        }

        Ok(outputs)
    }
}

fn read_balance(
    output_cache: &HashMap<AccessPath, u64>,
    state_view: &dyn StateView,
    account: AccountAddress,
) -> u64 {
    let balance_access_path = balance_ap(account);
    match output_cache.get(&balance_access_path) {
        Some(balance) => *balance,
        None => read_balance_from_storage(state_view, &balance_access_path),
    }
}

fn read_seqnum(
    output_cache: &HashMap<AccessPath, u64>,
    state_view: &dyn StateView,
    account: AccountAddress,
) -> u64 {
    let seqnum_access_path = seqnum_ap(account);
    match output_cache.get(&seqnum_access_path) {
        Some(seqnum) => *seqnum,
        None => read_seqnum_from_storage(state_view, &seqnum_access_path),
    }
}

fn read_balance_from_storage(state_view: &dyn StateView, balance_access_path: &AccessPath) -> u64 {
    read_u64_from_storage(state_view, &balance_access_path)
}

fn read_seqnum_from_storage(state_view: &dyn StateView, seqnum_access_path: &AccessPath) -> u64 {
    read_u64_from_storage(state_view, &seqnum_access_path)
}

fn read_u64_from_storage(state_view: &dyn StateView, access_path: &AccessPath) -> u64 {
    state_view
        .get(&access_path)
        .expect("Failed to query storage.")
        .map_or(0, |bytes| decode_bytes(&bytes))
}

fn decode_bytes(bytes: &[u8]) -> u64 {
    let mut buf = [0; 8];
    buf.copy_from_slice(bytes);
    u64::from_le_bytes(buf)
}

fn balance_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"balance".to_vec())
}

fn seqnum_ap(account: AccountAddress) -> AccessPath {
    AccessPath::new(account, b"seqnum".to_vec())
}

fn gen_genesis_writeset() -> WriteSet {
    let mut write_set = WriteSetMut::default();
    let path = ValidatorSet::CONFIG_ID.access_path();
    write_set.push((
        AccessPath {
            address: validator_set_address(),
            path: path.path,
        },
        WriteOp::Value(lcs::to_bytes(&ValidatorSet::new(vec![])).unwrap()),
    ));
    write_set
        .freeze()
        .expect("genesis writeset should be valid")
}

fn gen_mint_writeset(sender: AccountAddress, balance: u64, seqnum: u64) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(seqnum.to_le_bytes().to_vec()),
    ));
    write_set.freeze().expect("mint writeset should be valid")
}

fn gen_payment_writeset(
    sender: AccountAddress,
    sender_balance: u64,
    sender_seqnum: u64,
    recipient: AccountAddress,
    recipient_balance: u64,
) -> WriteSet {
    let mut write_set = WriteSetMut::default();
    write_set.push((
        balance_ap(sender),
        WriteOp::Value(sender_balance.to_le_bytes().to_vec()),
    ));
    write_set.push((
        seqnum_ap(sender),
        WriteOp::Value(sender_seqnum.to_le_bytes().to_vec()),
    ));
    write_set.push((
        balance_ap(recipient),
        WriteOp::Value(recipient_balance.to_le_bytes().to_vec()),
    ));
    write_set
        .freeze()
        .expect("payment write set should be valid")
}

fn gen_events(sender: AccountAddress) -> Vec<ContractEvent> {
    vec![ContractEvent::new(
        EventKey::new_from_address(&sender, 0),
        0,
        TypeTag::Vector(Box::new(TypeTag::U8)),
        b"event_data".to_vec(),
    )]
}

pub fn encode_mint_program(amount: u64) -> Script {
    let argument = TransactionArgument::U64(amount);
    Script::new(vec![], vec![], vec![argument])
}

pub fn encode_transfer_program(recipient: AccountAddress, amount: u64) -> Script {
    let argument1 = TransactionArgument::Address(recipient);
    let argument2 = TransactionArgument::U64(amount);
    Script::new(vec![], vec![], vec![argument1, argument2])
}

pub fn encode_mint_transaction(sender: AccountAddress, amount: u64) -> Transaction {
    encode_transaction(sender, encode_mint_program(amount))
}

pub fn encode_transfer_transaction(
    sender: AccountAddress,
    recipient: AccountAddress,
    amount: u64,
) -> Transaction {
    encode_transaction(sender, encode_transfer_program(recipient, amount))
}

fn encode_transaction(sender: AccountAddress, program: Script) -> Transaction {
    let raw_transaction =
        RawTransaction::new_script(sender, 0, program, 0, 0, std::time::Duration::from_secs(0));

    let privkey = Ed25519PrivateKey::generate_for_testing();
    Transaction::UserTransaction(
        raw_transaction
            .sign(&privkey, privkey.public_key())
            .expect("Failed to sign raw transaction.")
            .into_inner(),
    )
}

pub fn encode_reconfiguration_transaction(sender: AccountAddress) -> Transaction {
    let raw_transaction = RawTransaction::new_write_set(sender, 0, WriteSet::default());

    let privkey = Ed25519PrivateKey::generate_for_testing();
    Transaction::UserTransaction(
        raw_transaction
            .sign(&privkey, privkey.public_key())
            .expect("Failed to sign raw transaction.")
            .into_inner(),
    )
}

fn decode_transaction(txn: &SignedTransaction) -> MockVMTransaction {
    let sender = txn.sender();
    match txn.payload() {
        TransactionPayload::Script(script) => {
            assert!(script.code().is_empty(), "Code should be empty.");
            match script.args().len() {
                1 => match script.args()[0] {
                    TransactionArgument::U64(amount) => MockVMTransaction::Mint { sender, amount },
                    _ => unimplemented!(
                        "Only one integer argument is allowed for mint transactions."
                    ),
                },
                2 => match (&script.args()[0], &script.args()[1]) {
                    (TransactionArgument::Address(recipient), TransactionArgument::U64(amount)) => {
                        MockVMTransaction::Payment {
                            sender,
                            recipient: *recipient,
                            amount: *amount,
                        }
                    }
                    _ => unimplemented!(
                        "The first argument for payment transaction must be recipient address \
                         and the second argument must be amount."
                    ),
                },
                _ => unimplemented!("Transaction must have one or two arguments."),
            }
        }
        TransactionPayload::WriteSet(_) => {
            // Use WriteSet for reconfig only for testing.
            MockVMTransaction::Reconfiguration
        }
        TransactionPayload::Program => {
            unimplemented!("MockVM does not support Program transaction payload.")
        }
        TransactionPayload::Module(_) => {
            unimplemented!("MockVM does not support Module transaction payload.")
        }
    }
}
