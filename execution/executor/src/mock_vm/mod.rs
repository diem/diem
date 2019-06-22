// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
mod mock_vm_test;

use config::config::VMConfig;
use crypto::signing::generate_keypair;
use state_view::StateView;
use std::collections::HashMap;
use types::{
    access_path::AccessPath,
    account_address::{AccountAddress, ADDRESS_LENGTH},
    contract_event::ContractEvent,
    transaction::{
        Program, RawTransaction, SignedTransaction, TransactionArgument, TransactionOutput,
        TransactionPayload, TransactionStatus,
    },
    vm_error::{ExecutionStatus, VMStatus},
    write_set::{WriteOp, WriteSet, WriteSetMut},
};
use vm_runtime::VMExecutor;

#[derive(Debug)]
enum Transaction {
    Mint {
        sender: AccountAddress,
        amount: u64,
    },
    Payment {
        sender: AccountAddress,
        recipient: AccountAddress,
        amount: u64,
    },
}

pub const KEEP_STATUS: TransactionStatus =
    TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed));

// We use 10 as the assertion error code for insufficient balance within the Libra coin contract.
pub const DISCARD_STATUS: TransactionStatus =
    TransactionStatus::Discard(VMStatus::Execution(ExecutionStatus::AssertionFailure(10)));

pub struct MockVM;

impl VMExecutor for MockVM {
    fn execute_block(
        transactions: Vec<SignedTransaction>,
        _config: &VMConfig,
        state_view: &dyn StateView,
    ) -> Vec<TransactionOutput> {
        if state_view.is_genesis() {
            assert_eq!(
                transactions.len(),
                1,
                "Genesis block should have only one transaction."
            );
            let output = TransactionOutput::new(gen_genesis_writeset(), vec![], 0, KEEP_STATUS);
            return vec![output];
        }

        // output_cache is used to store the output of transactions so they are visible to later
        // transactions.
        let mut output_cache = HashMap::new();
        let mut outputs = vec![];

        for txn in transactions {
            match decode_transaction(&txn) {
                Transaction::Mint { sender, amount } => {
                    let old_balance = read_balance(&output_cache, state_view, sender);
                    let new_balance = old_balance + amount;
                    let old_seqnum = read_seqnum(&output_cache, state_view, sender);
                    let new_seqnum = old_seqnum + 1;

                    output_cache.insert(balance_ap(sender), new_balance);
                    output_cache.insert(seqnum_ap(sender), new_seqnum);

                    let write_set = gen_mint_writeset(sender, new_balance, new_seqnum);
                    let events = gen_events(sender);
                    outputs.push(TransactionOutput::new(write_set, events, 0, KEEP_STATUS));
                }
                Transaction::Payment {
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
                            DISCARD_STATUS,
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
                        TransactionStatus::Keep(VMStatus::Execution(ExecutionStatus::Executed)),
                    ));
                }
            }
        }

        outputs
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
    match state_view
        .get(&access_path)
        .expect("Failed to query storage.")
    {
        Some(bytes) => decode_bytes(&bytes),
        None => 0,
    }
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
    let address = AccountAddress::new([0xff; ADDRESS_LENGTH]);
    let path = b"hello".to_vec();
    let mut write_set = WriteSetMut::default();
    write_set.push((
        AccessPath::new(address, path),
        WriteOp::Value(b"world".to_vec()),
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
    let access_path = AccessPath::new(sender, b"event".to_vec());
    let event = ContractEvent::new(access_path, 0, b"event_data".to_vec());
    vec![event]
}

pub fn encode_mint_program(amount: u64) -> Program {
    let argument = TransactionArgument::U64(amount);
    Program::new(vec![], vec![], vec![argument])
}

pub fn encode_transfer_program(recipient: AccountAddress, amount: u64) -> Program {
    let argument1 = TransactionArgument::Address(recipient);
    let argument2 = TransactionArgument::U64(amount);
    Program::new(vec![], vec![], vec![argument1, argument2])
}

pub fn encode_mint_transaction(sender: AccountAddress, amount: u64) -> SignedTransaction {
    encode_transaction(sender, encode_mint_program(amount))
}

pub fn encode_transfer_transaction(
    sender: AccountAddress,
    recipient: AccountAddress,
    amount: u64,
) -> SignedTransaction {
    encode_transaction(sender, encode_transfer_program(recipient, amount))
}

fn encode_transaction(sender: AccountAddress, program: Program) -> SignedTransaction {
    let raw_transaction =
        RawTransaction::new(sender, 0, program, 0, 0, std::time::Duration::from_secs(0));

    let (privkey, pubkey) = generate_keypair();
    raw_transaction
        .sign(&privkey, pubkey)
        .expect("Failed to sign raw transaction.")
        .into_inner()
}

fn decode_transaction(txn: &SignedTransaction) -> Transaction {
    let sender = txn.sender();
    match txn.payload() {
        TransactionPayload::Program(program) => {
            assert!(program.code().is_empty(), "Code should be empty.");
            assert!(program.modules().is_empty(), "Modules should be empty.");
            match program.args().len() {
                1 => match program.args()[0] {
                    TransactionArgument::U64(amount) => Transaction::Mint { sender, amount },
                    _ => unimplemented!(
                        "Only one integer argument is allowed for mint transactions."
                    ),
                },
                2 => match (&program.args()[0], &program.args()[1]) {
                    (TransactionArgument::Address(recipient), TransactionArgument::U64(amount)) => {
                        Transaction::Payment {
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
            unimplemented!("MockVM does not support WriteSet transaction payload.")
        }
    }
}
