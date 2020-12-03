// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::test_utils::setup_swarm_and_client_proxy;
use diem_crypto::{ed25519::Ed25519PrivateKey, PrivateKey, SigningKey, Uniform};
use diem_types::{account_config::XUS_NAME, transaction::authenticator::AuthenticationKey};

#[test]
fn test_external_transaction_signer() {
    let (_env, mut client) = setup_swarm_and_client_proxy(1, 0);

    // generate key pair
    let private_key = Ed25519PrivateKey::generate_for_testing();
    let public_key = private_key.public_key();

    // create transfer parameters
    let sender_auth_key = AuthenticationKey::ed25519(&public_key);
    let sender_address = sender_auth_key.derived_address();
    let (receiver_address, receiver_auth_key) = client
        .get_account_address_from_parameter(
            "1bfb3b36384dabd29e38b4a0eafd9797b75141bb007cea7943f8a4714d3d784a",
        )
        .unwrap();
    let amount = 1_000_000;
    let gas_unit_price = 1;
    let max_gas_amount = 1_000_000;

    // mint to the sender address
    client
        .mint_coins(
            &["mintb", &format!("{}", sender_auth_key), "10", "XUS"],
            true,
        )
        .unwrap();
    // mint to the recipient address
    client
        .mint_coins(
            &[
                "mintb",
                &format!("{}", receiver_auth_key.unwrap()),
                "1",
                "XUS",
            ],
            true,
        )
        .unwrap();

    // prepare transfer transaction
    let sequence_number = client
        .get_sequence_number(&["sequence", &format!("{}", sender_address)])
        .unwrap();

    let currency_code = XUS_NAME;

    let unsigned_txn = client
        .prepare_transfer_coins(
            sender_address,
            sequence_number,
            receiver_address,
            amount,
            currency_code.to_owned(),
            Some(gas_unit_price),
            Some(max_gas_amount),
            Some(currency_code.to_owned()),
        )
        .unwrap();

    assert_eq!(unsigned_txn.sender(), sender_address);

    // sign the transaction with the private key
    let signature = private_key.sign(&unsigned_txn);

    // submit the transaction
    let submit_txn_result = client.submit_signed_transaction(unsigned_txn, public_key, signature);

    assert!(submit_txn_result.is_ok());

    // query the transaction and check it contains the same values as requested
    let txn = client
        .get_committed_txn_by_acc_seq(&[
            "txn_acc_seq",
            &format!("{}", sender_address),
            &sequence_number.to_string(),
            "false",
        ])
        .unwrap()
        .unwrap();

    match txn.transaction {
        Some(data) => {
            assert_eq!("user", data.r#type);

            let p_sender = data.sender;
            let p_sequence_number = data.sequence_number;
            let p_gas_unit_price = data.gas_unit_price;
            let p_max_gas_amount = data.max_gas_amount;
            let p_gas_currency = data.gas_currency;
            let script = data.script.unwrap();

            assert_eq!(p_sender, sender_address.to_string().to_lowercase());
            assert_eq!(p_sequence_number, sequence_number);
            assert_eq!(p_gas_unit_price, gas_unit_price);
            assert_eq!(p_gas_currency, currency_code.to_string());
            assert_eq!(p_max_gas_amount, max_gas_amount);

            assert_eq!(script.r#type, "peer_to_peer_with_metadata");
            assert_eq!(script.type_arguments, vec!["XUS"]);
            assert_eq!(
                script.arguments,
                vec![
                    format!("{{ADDRESS: {:?}}}", &receiver_address),
                    format!("{{U64: {}}}", amount),
                    "{U8Vector: 0x}".to_string(),
                    "{U8Vector: 0x}".to_string()
                ]
            );
            // legacy fields
            assert_eq!(script.receiver, receiver_address.to_string().to_lowercase());
            assert_eq!(script.amount, amount);
            assert_eq!(script.currency, currency_code.to_string());
            assert_eq!(script.metadata, "");
            assert_eq!(script.metadata_signature, "");
        }
        _ => panic!("Query should get user transaction"),
    }
}
