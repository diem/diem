// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{account::Account, compile, executor::FakeExecutor};
use libra_types::{account_address::AccountAddress, transaction::WriteSetPayload};

pub fn add_currency_to_system(
    executor: &mut FakeExecutor,
    currency_code_to_register: &str,
    current_lr_sequence_number: u64,
) -> u64 {
    let lr_account = Account::new_libra_root();
    let mut lr_sequence_number = current_lr_sequence_number;

    {
        let compiled_script = {
            let script = "
            import 0x1.LibraTransactionPublishingOption;
            main(config: &signer) {
                LibraTransactionPublishingOption.set_open_module(move(config), false);
                return;
            }
            ";
            compile::compile_script_with_address(lr_account.address(), "file_name", script, vec![])
        };

        let txn = lr_account
            .transaction()
            .script(compiled_script)
            .sequence_number(lr_sequence_number)
            .sign();

        executor.execute_and_apply(txn);
    };

    executor.new_block();

    lr_sequence_number += 1;

    let (compiled_module, module) = {
        let module = format!(
            r#"
            module {} {{
                import 0x1.Libra;
                import 0x1.FixedPoint32;
                resource {currency_code} {{ x: bool }}
                public init(lr_account: &signer, tc_account: &signer) {{
                    Libra.register_SCS_currency<Self.{currency_code}>(
                        move(lr_account),
                        move(tc_account),
                        FixedPoint32.create_from_rational(1, 1),
                        100,
                        1000,
                        h"{currency_code_hex}"
                    );
                    return;
                }}
            }}
            "#,
            currency_code = currency_code_to_register,
            currency_code_hex = hex::encode(currency_code_to_register)
        );

        compile::compile_module_with_address(
            &AccountAddress::from_hex_literal("0x1").unwrap(),
            "this_is_a_filename",
            &module,
        )
    };

    let txn = lr_account
        .transaction()
        .module(module)
        .sequence_number(lr_sequence_number)
        .sign();

    executor.execute_and_apply(txn);

    lr_sequence_number += 1;

    {
        let compiled_script = {
            let script = format!(
                r#"
            import 0x1.{currency_code};
            main(lr_account: &signer, tc_account: &signer) {{
                {currency_code}.init(move(lr_account), move(tc_account));
                return;
            }}
            "#,
                currency_code = currency_code_to_register
            );
            compile::compile_script_with_address(
                lr_account.address(),
                "file_name",
                &script,
                vec![compiled_module],
            )
        };

        let write_set_payload = WriteSetPayload::Script {
            execute_as: *Account::new_blessed_tc().address(),
            script: compiled_script,
        };
        let txn = lr_account
            .transaction()
            .write_set(write_set_payload)
            .sequence_number(lr_sequence_number)
            .sign();
        executor.execute_and_apply(txn);
    };

    executor.new_block();

    lr_sequence_number + 1
}
