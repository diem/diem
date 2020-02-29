// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::AccountData, executor::test_all_genesis, gas_costs};
use libra_config::config::VMPublishingOption;
use libra_types::{
    account_address::AccountAddress, account_address::ADDRESS_LENGTH,
    transaction::TransactionStatus, vm_error::StatusCode,
};
use move_core_types::identifier::Identifier;
use vm::file_format::{
    empty_script, AddressPoolIndex, Bytecode, FunctionHandle, FunctionHandleIndex,
    FunctionSignatureIndex, IdentifierIndex, LocalsSignatureIndex, ModuleHandle, ModuleHandleIndex,
};

#[test]
fn script_code_unverifiable() {
    test_all_genesis(Some(VMPublishingOption::Open), |mut executor| {
        // create and publish sender
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);

        // create a bogus script
        let mut script = empty_script();
        script.main.code.code = vec![Bytecode::LdU8(0), Bytecode::Add, Bytecode::Ret];
        let mut blob = vec![];
        script.serialize(&mut blob).expect("script must serialize");
        let txn = sender.account().create_signed_txn_with_args(
            blob,
            vec![],
            10,
            gas_costs::TXN_RESERVED,
            1,
        );

        // execute transaction
        let output = &executor.execute_transaction(txn);
        let status = output.status();
        match status {
            TransactionStatus::Keep(_) => (),
            _ => panic!("TransactionStatus must be Keep"),
        }
        assert_eq!(
            status.vm_status().major_status,
            StatusCode::NEGATIVE_STACK_SIZE_WITHIN_BLOCK,
        );
        executor.apply_write_set(output.write_set());

        // Check that numbers in store are correct.
        let gas = output.gas_used();
        let balance = 1_000_000 - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        assert_eq!(balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());
    });
}

#[test]
fn script_non_existing_module_dep() {
    test_all_genesis(Some(VMPublishingOption::Open), |mut executor| {
        // create and publish sender
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);

        // create a bogus script
        let mut script = empty_script();
        // make a non existent external module
        script
            .address_pool
            .push(AccountAddress::new([1u8; ADDRESS_LENGTH]));
        script.identifiers.push(Identifier::new("module").unwrap());
        let module_handle = ModuleHandle {
            address: AddressPoolIndex((script.address_pool.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
        };
        script.module_handles.push(module_handle);
        // make a non existent function on the non existent external module
        script.identifiers.push(Identifier::new("foo").unwrap());
        let fun_handle = FunctionHandle {
            module: ModuleHandleIndex((script.module_handles.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
            signature: FunctionSignatureIndex(0),
        };
        script.function_handles.push(fun_handle);

        script.main.code.code = vec![
            Bytecode::Call(
                FunctionHandleIndex((script.function_handles.len() - 1) as u16),
                LocalsSignatureIndex(0),
            ),
            Bytecode::Ret,
        ];
        let mut blob = vec![];
        script.serialize(&mut blob).expect("script must serialize");
        let txn = sender.account().create_signed_txn_with_args(
            blob,
            vec![],
            10,
            gas_costs::TXN_RESERVED,
            1,
        );

        // execute transaction
        let output = &executor.execute_transaction(txn);
        let status = output.status();
        match status {
            TransactionStatus::Keep(_) => (),
            _ => panic!("TransactionStatus must be Keep"),
        }
        assert_eq!(status.vm_status().major_status, StatusCode::LINKER_ERROR,);
        executor.apply_write_set(output.write_set());

        // Check that numbers in store are correct.
        let gas = output.gas_used();
        let balance = 1_000_000 - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        assert_eq!(balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());
    });
}

#[test]
fn script_non_existing_function_dep() {
    test_all_genesis(Some(VMPublishingOption::Open), |mut executor| {
        // create and publish sender
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);

        // create a bogus script
        let mut script = empty_script();
        // AddressUtil module
        script
            .address_pool
            .push(AccountAddress::new([0u8; ADDRESS_LENGTH]));
        script
            .identifiers
            .push(Identifier::new("AddressUtil").unwrap());
        let module_handle = ModuleHandle {
            address: AddressPoolIndex((script.address_pool.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
        };
        script.module_handles.push(module_handle);
        // make a non existent function on AddressUtil
        script.identifiers.push(Identifier::new("foo").unwrap());
        let fun_handle = FunctionHandle {
            module: ModuleHandleIndex((script.module_handles.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
            signature: FunctionSignatureIndex(0),
        };
        script.function_handles.push(fun_handle);

        script.main.code.code = vec![
            Bytecode::Call(
                FunctionHandleIndex((script.function_handles.len() - 1) as u16),
                LocalsSignatureIndex(0),
            ),
            Bytecode::Ret,
        ];
        let mut blob = vec![];
        script.serialize(&mut blob).expect("script must serialize");
        let txn = sender.account().create_signed_txn_with_args(
            blob,
            vec![],
            10,
            gas_costs::TXN_RESERVED,
            1,
        );

        // execute transaction
        let output = &executor.execute_transaction(txn);
        let status = output.status();
        match status {
            TransactionStatus::Keep(_) => (),
            _ => panic!("TransactionStatus must be Keep"),
        }
        assert_eq!(status.vm_status().major_status, StatusCode::LOOKUP_FAILED,);
        executor.apply_write_set(output.write_set());

        // Check that numbers in store are correct.
        let gas = output.gas_used();
        let balance = 1_000_000 - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        assert_eq!(balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());
    });
}

#[test]
fn script_bad_sig_function_dep() {
    test_all_genesis(Some(VMPublishingOption::Open), |mut executor| {
        // create and publish sender
        let sender = AccountData::new(1_000_000, 10);
        executor.add_account_data(&sender);

        // create a bogus script
        let mut script = empty_script();
        // AddressUtil module
        script
            .address_pool
            .push(AccountAddress::new([0u8; ADDRESS_LENGTH]));
        script
            .identifiers
            .push(Identifier::new("AddressUtil").unwrap());
        let module_handle = ModuleHandle {
            address: AddressPoolIndex((script.address_pool.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
        };
        script.module_handles.push(module_handle);
        // AddressUtil::address_to_bytes with bad sig
        script
            .identifiers
            .push(Identifier::new("address_to_bytes").unwrap());
        let fun_handle = FunctionHandle {
            module: ModuleHandleIndex((script.module_handles.len() - 1) as u16),
            name: IdentifierIndex((script.identifiers.len() - 1) as u16),
            signature: FunctionSignatureIndex(0),
        };
        script.function_handles.push(fun_handle);

        script.main.code.code = vec![
            Bytecode::Call(
                FunctionHandleIndex((script.function_handles.len() - 1) as u16),
                LocalsSignatureIndex(0),
            ),
            Bytecode::Ret,
        ];
        let mut blob = vec![];
        script.serialize(&mut blob).expect("script must serialize");
        let txn = sender.account().create_signed_txn_with_args(
            blob,
            vec![],
            10,
            gas_costs::TXN_RESERVED,
            1,
        );

        // execute transaction
        let output = &executor.execute_transaction(txn);
        let status = output.status();
        match status {
            TransactionStatus::Keep(_) => (),
            _ => panic!("TransactionStatus must be Keep"),
        }
        assert_eq!(status.vm_status().major_status, StatusCode::TYPE_MISMATCH,);
        executor.apply_write_set(output.write_set());

        // Check that numbers in store are correct.
        let gas = output.gas_used();
        let balance = 1_000_000 - gas;
        let updated_sender = executor
            .read_account_resource(sender.account())
            .expect("sender must exist");
        assert_eq!(balance, updated_sender.balance());
        assert_eq!(11, updated_sender.sequence_number());
    });
}
