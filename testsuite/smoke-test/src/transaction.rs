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
    let test_gas_unit_price = 1;
    let test_max_gas_amount = 1_000_000;

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
    let test_sequence_number = client
        .get_sequence_number(&["sequence", &format!("{}", sender_address)])
        .unwrap();

    let currency_code = XUS_NAME;

    let unsigned_txn = client
        .prepare_transfer_coins(
            sender_address,
            test_sequence_number,
            receiver_address,
            amount,
            currency_code.to_owned(),
            Some(test_gas_unit_price),
            Some(test_max_gas_amount),
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
            &test_sequence_number.to_string(),
            "false",
        ])
        .unwrap()
        .unwrap();

    match txn.transaction {
        diem_client::views::TransactionDataView::UserTransaction {
            sender,
            sequence_number,
            max_gas_amount,
            gas_unit_price,
            gas_currency,
            script,
            ..
        } => {
            assert_eq!(sender.0, sender_address.to_string().to_lowercase());
            assert_eq!(sequence_number, test_sequence_number);
            assert_eq!(gas_unit_price, test_gas_unit_price);
            assert_eq!(gas_currency, currency_code.to_string());
            assert_eq!(max_gas_amount, test_max_gas_amount);

            assert_eq!(script.r#type, "peer_to_peer_with_metadata");
            assert_eq!(script.type_arguments.unwrap(), vec!["XUS"]);
            assert_eq!(
                script.arguments.unwrap(),
                vec![
                    format!("{{ADDRESS: {:?}}}", &receiver_address),
                    format!("{{U64: {}}}", amount),
                    "{U8Vector: 0x}".to_string(),
                    "{U8Vector: 0x}".to_string()
                ]
            );
            // legacy fields
            assert_eq!(
                script.receiver.unwrap().0,
                receiver_address.to_string().to_lowercase()
            );
            assert_eq!(script.amount.unwrap(), amount);
            assert_eq!(script.currency.unwrap(), currency_code.to_string());
            assert_eq!(script.metadata.unwrap().0, "");
            assert_eq!(script.metadata_signature.unwrap().0, "");
        }
        _ => panic!("Query should get user transaction"),
    }
}
