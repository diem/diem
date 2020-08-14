// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use compiled_stdlib::transaction_scripts::StdlibScript;
use language_e2e_tests::{account::Account, executor::FakeExecutor};
use libra_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use libra_types::{
    account_config::{self, BurnEvent, COIN1_NAME},
    transaction::{authenticator::AuthenticationKey, Script, TransactionArgument},
    vm_status::KeptVMStatus,
};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use std::convert::TryFrom;
use transaction_builder::*;

#[test]
fn burn_txn_fees() {
    let mut executor = FakeExecutor::from_genesis_file();
    let sender = Account::new();
    let dd = Account::new_genesis_account(account_config::testnet_dd_account_address());
    let tc = Account::new_blessed_tc();
    let libra_root = Account::new_libra_root();

    executor.execute_and_apply(
        libra_root
            .transaction()
            .script(encode_create_parent_vasp_account_script(
                account_config::coin1_tag(),
                0,
                *sender.address(),
                sender.auth_key_prefix(),
                vec![],
                false,
            ))
            .sequence_number(1)
            .sign(),
    );

    executor.execute_and_apply(
        dd.transaction()
            .script(encode_peer_to_peer_with_metadata_script(
                account_config::coin1_tag(),
                *sender.address(),
                10_000_000,
                vec![],
                vec![],
            ))
            .sequence_number(0)
            .sign(),
    );

    let gas_used = {
        let privkey = Ed25519PrivateKey::generate_for_testing();
        let pubkey = privkey.public_key();
        let new_key_hash = AuthenticationKey::ed25519(&pubkey).to_vec();
        let args = vec![TransactionArgument::U8Vector(new_key_hash)];
        let status = executor.execute_and_apply(
            sender
                .transaction()
                .script(Script::new(
                    StdlibScript::RotateAuthenticationKey
                        .compiled_bytes()
                        .into_vec(),
                    vec![],
                    args,
                ))
                .sequence_number(0)
                .gas_unit_price(1)
                .gas_currency_code(COIN1_NAME)
                .sign(),
        );
        assert_eq!(status.status().status(), Ok(KeptVMStatus::Executed));
        status.gas_used()
    };

    let coin1_ty = TypeTag::Struct(StructTag {
        address: account_config::CORE_CODE_ADDRESS,
        module: Identifier::new("Coin1").unwrap(),
        name: Identifier::new("Coin1").unwrap(),
        type_params: vec![],
    });

    let output = executor.execute_and_apply(
        tc.transaction()
            .script(encode_burn_txn_fees_script(coin1_ty))
            .sequence_number(0)
            .sign(),
    );

    let burn_events: Vec<_> = output
        .events()
        .iter()
        .filter_map(|event| BurnEvent::try_from(event).ok())
        .collect();

    assert_eq!(burn_events.len(), 1);
    assert!(burn_events
        .iter()
        .any(|event| event.currency_code().as_str() == "Coin1"));
    burn_events
        .iter()
        .for_each(|event| assert_eq!(event.amount(), gas_used));
}
