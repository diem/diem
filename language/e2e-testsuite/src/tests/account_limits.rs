// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use compiler::Compiler;
use diem_types::{
    account_address::AccountAddress,
    account_config,
    transaction::{Script, TransactionArgument, TransactionOutput, WriteSetPayload},
    vm_status::KeptVMStatus,
};
use language_e2e_tests::{
    account::{self, Account},
    current_function_name,
    executor::FakeExecutor,
};
use transaction_builder::*;

fn assert_aborted_with(output: TransactionOutput, error_code: u64) {
    assert!(matches!(
        output.status().status(),
        Ok(KeptVMStatus::MoveAbort(_, code)) if code == error_code
    ));
}

fn encode_add_account_limits_admin_script(execute_as: AccountAddress) -> WriteSetPayload {
    let add_account_limits_admin_script = {
        let code = "
    import 0x1.AccountLimits;
    import 0x1.XUS;
    import 0x1.Signer;

    main(dr_account: &signer, vasp: &signer) {
        AccountLimits.publish_unrestricted_limits<XUS.XUS>(copy(vasp));
        AccountLimits.publish_window<XUS.XUS>(
            move(dr_account),
            copy(vasp),
            Signer.address_of(move(vasp))
        );
        return;
    }
";
        let compiler = Compiler {
            address: account_config::CORE_CODE_ADDRESS,
            extra_deps: vec![],
            ..Compiler::default()
        };
        compiler
            .into_script_blob("file_name", code)
            .expect("Failed to compile")
    };

    WriteSetPayload::Script {
        script: Script::new(add_account_limits_admin_script, vec![], vec![]),
        execute_as,
    }
}

fn encode_update_account_limit_definition_script(
    limit_addr: AccountAddress,
    new_max_inflow: u64,
    new_max_outflow: u64,
    new_max_holding_balance: u64,
    new_time_period: u64,
) -> Script {
    let script_body = {
        let code = "
    import 0x1.AccountLimits;
    import 0x1.XUS;

    main(
        account: &signer,
        limit_addr: address,
        new_max_inflow: u64,
        new_max_outflow: u64,
        new_max_holding_balance: u64,
        new_time_period: u64
    ) {
        AccountLimits.update_limits_definition<XUS.XUS>(
            move(account),
            move(limit_addr),
            move(new_max_inflow),
            move(new_max_outflow),
            move(new_max_holding_balance),
            move(new_time_period),
        );
        return;
    }
";
        let compiler = Compiler {
            address: account_config::CORE_CODE_ADDRESS,
            extra_deps: vec![],
            ..Compiler::default()
        };
        compiler
            .into_script_blob("file_name", code)
            .expect("Failed to compile")
    };

    Script::new(
        script_body,
        vec![],
        vec![
            TransactionArgument::Address(limit_addr),
            TransactionArgument::U64(new_max_inflow),
            TransactionArgument::U64(new_max_outflow),
            TransactionArgument::U64(new_max_holding_balance),
            TransactionArgument::U64(new_time_period),
        ],
    )
}

fn encode_update_account_limit_window_info_script(
    window_addr: AccountAddress,
    aggregate_balance: u64,
    new_limit_address: AccountAddress,
) -> Script {
    let script_body = {
        let code = "
    import 0x1.AccountLimits;
    import 0x1.XUS;

    main(account: &signer,
        window_addr: address,
        aggregate_balance: u64,
        new_limit_address: address
    ) {
        AccountLimits.update_window_info<XUS.XUS>(
            move(account),
            move(window_addr),
            move(aggregate_balance),
            move(new_limit_address),
        );
        return;
    }
";
        let compiler = Compiler {
            address: account_config::CORE_CODE_ADDRESS,
            extra_deps: vec![],
            ..Compiler::default()
        };
        compiler
            .into_script_blob("file_name", code)
            .expect("Failed to compile")
    };

    Script::new(
        script_body,
        vec![],
        vec![
            TransactionArgument::Address(window_addr),
            TransactionArgument::U64(aggregate_balance),
            TransactionArgument::Address(new_limit_address),
        ],
    )
}

#[test]
fn account_limits() {
    let mut executor = FakeExecutor::from_genesis_file();
    executor.set_golden_file(current_function_name!());

    let vasp_a = executor.create_raw_account();
    let vasp_b = executor.create_raw_account();
    let vasp_a_child = executor.create_raw_account();
    let vasp_b_child = executor.create_raw_account();
    let diem_root = Account::new_diem_root();
    let blessed = Account::new_blessed_tc();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());

    let mint_amount = 1_000_000;
    let window_micros = 86400000000;
    let ttl = window_micros;

    // Create vasp accounts
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *vasp_a.address(),
                vasp_a.auth_key_prefix(),
                vec![],
                true,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::xus_tag(),
                0,
                *vasp_b.address(),
                vasp_b.auth_key_prefix(),
                vec![],
                true,
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    // Create child vasp accounts
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
                *vasp_a_child.address(),
                vasp_a_child.auth_key_prefix(),
                true,
                0,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_create_child_vasp_account_script(
                account_config::xus_tag(),
                *vasp_b_child.address(),
                vasp_b_child.auth_key_prefix(),
                true,
                0,
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    executor.execute_and_apply(
        diem_root
            .transaction()
            .write_set(encode_add_account_limits_admin_script(*vasp_a.address()))
            .sequence_number(1)
            .sign(),
    );

    executor.execute_and_apply(
        diem_root
            .transaction()
            .write_set(encode_add_account_limits_admin_script(*vasp_b.address()))
            .sequence_number(2)
            .sign(),
    );

    // mint money to both vasp A & B
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a.address(),
                2 * mint_amount,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );
    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_b.address(),
                2 * mint_amount,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_update_account_limit_window_info_script(
                *vasp_a.address(),
                0,
                *vasp_a.address(),
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    ///////////////////////////////////////////////////////////////////////////
    // Inflow tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's inflow limit to half of what we just minted them
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_update_account_limit_definition_script(
                *vasp_a.address(),
                mint_amount,
                0,
                0,
                0,
            ))
            .sequence_number(3)
            .ttl(ttl)
            .sign(),
    );

    {
        // Now try and pay in to vasp A; fails since inflow is exceeded
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a.address(),
                    mint_amount + 1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);
    }

    {
        // Now try and pay in to child of vasp A; fails since inflow is exceeded
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    mint_amount + 1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);
    }

    // Intra-vasp transfer isn't limited
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a_child.address(),
                mint_amount + 1,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    // Only inflow is limited; can send from vasp a still
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_b_child.address(),
                mint_amount + 1,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    // The previous mints don't count in this window since it wasn't a vasp->vasp transfer
    executor.execute_and_apply(
        vasp_b_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a_child.address(),
                mint_amount,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .ttl(ttl)
            .sign(),
    );

    {
        // DD deposit fails since vasp A is at inflow limit
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);

        // Reset the window
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        // DD deposit now succeeds since window is reset
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed));
    }

    ///////////////////////////////////////////////////////////////////////////
    // Outflow tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's outflow to 1000
    executor.execute_and_apply(
        blessed
            .transaction()
            .script(encode_update_account_limit_definition_script(
                *vasp_a.address(),
                std::u64::MAX, // unlimit inflow
                1000,          // set outflow to 1000
                0,
                0,
            ))
            .sequence_number(4)
            .ttl(ttl)
            .sign(),
    );

    // Intra-vasp transfer isn't limited
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a_child.address(),
                1001,
                vec![],
                vec![],
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    // Can send up to the limit inter-vasp:
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_b_child.address(),
                1000,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    {
        // Inter-vasp transfer is limited
        let output = executor.execute_transaction(
            vasp_a
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_b.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(3)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 1544);
    }

    {
        // Inter-vasp transfer is limited; holds between children too
        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_b_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 1544);
    }

    {
        // vasp->anything transfer is limited
        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *dd.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 1544);

        // update block time
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        let output = executor.execute_transaction(
            vasp_a_child
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *dd.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(window_micros)
                .ttl(ttl)
                .sign(),
        );
        assert_eq!(output.status().status(), Ok(KeptVMStatus::Executed),);
    }

    ///////////////////////////////////////////////////////////////////////////
    // Holding tests
    /////////////////////////////////////////////////////////////////////////////

    // Set vasp A's max holding to its current balance across all accounts
    {
        let a_parent_balance = executor
            .read_balance_resource(&vasp_a, account::xus_currency_code())
            .unwrap()
            .coin();
        let a_child_balance = executor
            .read_balance_resource(&vasp_a_child, account::xus_currency_code())
            .unwrap()
            .coin();
        let a_balance = a_parent_balance + a_child_balance;
        executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_update_account_limit_definition_script(
                    *vasp_a.address(),
                    0,
                    std::u64::MAX, // unlimit outflow
                    a_balance,     // set max holding to the current balance of A
                    0,
                ))
                .sequence_number(5)
                .ttl(ttl)
                .sign(),
        );
        // TC needs to set the current aggregate balance for vasp a's window
        executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_update_account_limit_window_info_script(
                    *vasp_a.address(),
                    a_balance,
                    *vasp_a.address(),
                ))
                .sequence_number(6)
                .ttl(ttl)
                .sign(),
        );
    }

    // inter-vasp: fails since limit is set at A's current balance
    {
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(1)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);
    }

    // Fine since A can still send
    executor.execute_and_apply(
        vasp_a
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_b_child.address(),
                10,
                vec![],
                vec![],
            ))
            .sequence_number(3)
            .ttl(ttl)
            .sign(),
    );

    // inter-vasp: OK since A's total balance = limit - 10
    executor.execute_and_apply(
        vasp_b
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a_child.address(),
                10,
                vec![],
                vec![],
            ))
            .sequence_number(1)
            .ttl(ttl)
            .sign(),
    );

    {
        // inter-vasp: should now fail again
        let output = executor.execute_transaction(
            vasp_b
                .transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);
    }

    // intra-vasp: OK since it isn't checked/contributes to the total balance
    executor.execute_and_apply(
        vasp_a_child
            .transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::xus_tag(),
                *vasp_a.address(),
                1100,
                vec![],
                vec![],
            ))
            .sequence_number(2)
            .ttl(ttl)
            .sign(),
    );

    {
        // DD deposit fails since vasp A is at holding limit
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);

        // Reset window
        let prev_block_time = executor.get_block_time();
        executor.set_block_time(prev_block_time + window_micros);
        executor.new_block();

        // DD deposit fails since vasp A is at holding limit
        // and because holdings are not reset from one window to the next.
        let output = executor.execute_transaction(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *vasp_a_child.address(),
                    1,
                    vec![],
                    vec![],
                ))
                .sequence_number(2)
                .ttl(ttl)
                .sign(),
        );
        assert_aborted_with(output, 776);
    }
}
