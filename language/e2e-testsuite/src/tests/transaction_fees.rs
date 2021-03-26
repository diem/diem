// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, Uniform};
use diem_framework_releases::legacy::transaction_scripts::LegacyStdlibScript;
use diem_transaction_builder::stdlib::*;
use diem_types::{
    account_config::{self, BurnEvent, XUS_NAME},
    transaction::{authenticator::AuthenticationKey, Script, TransactionArgument},
    vm_status::KeptVMStatus,
};
use language_e2e_tests::{test_with_different_versions, versioning::CURRENT_RELEASE_VERSIONS};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, TypeTag},
};
use std::convert::TryFrom;

#[test]
fn burn_txn_fees() {
    test_with_different_versions! {CURRENT_RELEASE_VERSIONS, |test_env| {
        let mut executor = test_env.executor;

        let sender = executor.create_raw_account();
        let dd = test_env.dd_account;
        let blessed = test_env.tc_account;

        executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_create_parent_vasp_account_script(
                    account_config::xus_tag(),
                    0,
                    *sender.address(),
                    sender.auth_key_prefix(),
                    vec![],
                    false,
                ))
                .sequence_number(test_env.tc_sequence_number)
                .sign(),
        );

        executor.execute_and_apply(
            dd.transaction()
                .script(encode_peer_to_peer_with_metadata_script(
                    account_config::xus_tag(),
                    *sender.address(),
                    10_000_000,
                    vec![],
                    vec![],
                ))
                .sequence_number(test_env.dd_sequence_number)
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
                        LegacyStdlibScript::RotateAuthenticationKey
                            .compiled_bytes()
                            .into_vec(),
                        vec![],
                        args,
                    ))
                    .sequence_number(0)
                    .gas_unit_price(1)
                    .gas_currency_code(XUS_NAME)
                    .sign(),
            );
            assert_eq!(status.status().status(), Ok(KeptVMStatus::Executed));
            status.gas_used()
        };

        let xus_ty = TypeTag::Struct(StructTag {
            address: account_config::CORE_CODE_ADDRESS,
            module: Identifier::new("XUS").unwrap(),
            name: Identifier::new("XUS").unwrap(),
            type_params: vec![],
        });

        let output = executor.execute_and_apply(
            blessed
                .transaction()
                .script(encode_burn_txn_fees_script(xus_ty))
                .sequence_number(test_env.tc_sequence_number.checked_add(1).unwrap())
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
            .any(|event| event.currency_code().as_str() == "XUS"));
        burn_events
            .iter()
            .for_each(|event| assert_eq!(event.amount(), gas_used));
    }
    }
}
