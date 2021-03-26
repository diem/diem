// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
    test_utils::KeyPair,
    Signature, SigningKey, Uniform, ValidCryptoMaterialStringExt,
};
use diem_transaction_builder::stdlib as transaction_builder;
use diem_types::{
    chain_id::ChainId,
    transaction::{
        authenticator::AuthenticationKey, RawTransaction, SignedTransaction, Transaction,
        TransactionPayload,
    },
};
use rand::{prelude::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use structopt::StructOpt;
use swiss_knife::helpers;

#[derive(Debug, StructOpt)]
enum Command {
    /// Generates and serializes a RawTransaction and its hash. The hash of this RawTransaction needs to be signed to generate a SignedTransaction.
    /// Takes the input json payload from stdin. Writes the output json payload to stdout.
    /// Refer to README.md for examples.
    GenerateRawTxn,
    /// Generates a SignedTransaction given the serialized RawTransaction, the public key and signature.
    /// It also includes the txn_hash which gets included in the chain
    /// Takes the input json payload from stdin. Writes the output json payload to stdout.
    /// Refer to README.md for examples.
    GenerateSignedTxn,
    /// Generates a Ed25519Keypair for testing from the given u64 seed.
    GenerateTestEd25519Keypair {
        #[structopt(long)]
        seed: Option<u64>,
    },
    /// Verifies the Ed25519 signature using the provided Ed25519 public
    /// key. Assumes the caller has a correct binary payload: this is thex
    /// Ed25519 signature verification you would find in an off-the-shelf
    /// Ed25519 library (RFC 8032), hence advised only for sanity-checking and
    /// testing.
    VerifyEd25519Signature,
    /// Generates a signature of a RawTransaction using the provided Ed25519
    /// private key. Handles producing the binary representation of that transaction.
    SignTransactionUsingEd25519,
    /// Verifies the Ed25519 signature using the provided Ed25519 public
    /// key. Handles producing the binary representation of that transaction.
    VerifyTransactionEd25519Signature,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "swiss-knife",
    about = "Tool for generating, serializing (BCS), hashing and signing Diem transactions. Additionally, contains tools for testing. Please refer to README.md for examples."
)]
struct Opt {
    #[structopt(subcommand)]
    pub cmd: Command,
}

fn main() {
    let opt = Opt::from_args();
    match opt.cmd {
        Command::GenerateRawTxn => {
            let input = helpers::read_stdin();
            let g: GenerateRawTxnRequest = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(generate_raw_txn(g));
        }
        Command::GenerateSignedTxn => {
            let input = helpers::read_stdin();
            let g: GenerateSignedTxnRequest = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(generate_signed_txn(g));
        }
        Command::GenerateTestEd25519Keypair { seed } => {
            helpers::exit_success_with_data(generate_key_pair(seed));
        }
        Command::VerifyEd25519Signature => {
            let input = helpers::read_stdin();
            let request: VerifyEd25519SignatureRequest = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(verify_signature_using_ed25519(request));
        }
        Command::SignTransactionUsingEd25519 => {
            let input = helpers::read_stdin();
            let request: SignTransactionUsingEd25519Request = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(sign_transaction_using_ed25519(request));
        }
        Command::VerifyTransactionEd25519Signature => {
            let input = helpers::read_stdin();
            let request: VerifyTransactionEd25519SignatureRequest = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(verify_transaction_signature_using_ed25519(request));
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
struct TxnParams {
    // Sender's address
    pub sender_address: String,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
    // Chain ID of the Diem network this transaction is intended for
    pub chain_id: String,
    // Maximal total gas specified by wallet to spend for this transaction.
    pub max_gas_amount: u64,
    // Maximal price can be paid per gas.
    pub gas_unit_price: u64,
    // identifier of the coin to be used as gas
    pub gas_currency_code: String,
    // Expiration time for this transaction in Unix Epoch Seconds. If storage
    // is queried and the time returned is greater than or equal to this time
    // and this transaction has not been included, you can be certain that it
    // will never be included.
    // A transaction that doesn't expire is represented by a very large value like
    // u64::max_value().
    pub expiration_timestamp_secs: u64,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum MoveScriptParams {
    Preburn {
        coin_tag: String,
        amount: u64,
    },
    PeerToPeerTransfer {
        coin_tag: String,
        recipient_address: String,
        amount: u64,
        metadata_hex_encoded: String,
        metadata_signature_hex_encoded: String,
    },
    RotateDualAttestationInfo {
        // "https://example.com/endpoint"
        new_url: String,
        // Hex encoded 32 byte Ed25519PublicKey, eg: "edd0f6de342a1e6a7236d6244f23d83eedfcecd059a386c85055701498e77033"
        new_key_hex_encoded: String,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateRawTxnRequest {
    pub txn_params: TxnParams,
    pub script_params: MoveScriptParams,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateRawTxnResponse {
    pub script: String,
    pub raw_txn: String,
}

fn generate_raw_txn(g: GenerateRawTxnRequest) -> GenerateRawTxnResponse {
    let script = match g.script_params {
        MoveScriptParams::Preburn { coin_tag, amount } => {
            let coin_tag = helpers::coin_tag_parser(&coin_tag);
            transaction_builder::encode_preburn_script(coin_tag, amount)
        }
        MoveScriptParams::PeerToPeerTransfer {
            coin_tag,
            recipient_address,
            amount,
            metadata_hex_encoded,
            metadata_signature_hex_encoded,
        } => {
            let coin_tag = helpers::coin_tag_parser(&coin_tag);
            let recipient_address = helpers::account_address_parser(&recipient_address);
            transaction_builder::encode_peer_to_peer_with_metadata_script(
                coin_tag,
                recipient_address,
                amount,
                helpers::hex_decode(&metadata_hex_encoded),
                helpers::hex_decode(&metadata_signature_hex_encoded),
            )
        }
        MoveScriptParams::RotateDualAttestationInfo {
            new_url,
            new_key_hex_encoded,
        } => {
            let new_url = new_url.into_bytes();
            let new_key = hex::decode(new_key_hex_encoded)
                .expect("Failed to hex decode new_key_hex_encoded field");
            transaction_builder::encode_rotate_dual_attestation_info_script(new_url, new_key)
        }
    };
    let payload = TransactionPayload::Script(script);
    let script_hex = hex::encode(bcs::to_bytes(&payload).unwrap());
    let raw_txn = RawTransaction::new(
        helpers::account_address_parser(&g.txn_params.sender_address),
        g.txn_params.sequence_number,
        payload,
        g.txn_params.max_gas_amount,
        g.txn_params.gas_unit_price,
        g.txn_params.gas_currency_code,
        g.txn_params.expiration_timestamp_secs,
        ChainId::from_str(&g.txn_params.chain_id).expect("Failed to convert str to ChainId"),
    );
    GenerateRawTxnResponse {
        script: script_hex,
        raw_txn: hex::encode(
            bcs::to_bytes(&raw_txn)
                .map_err(|err| {
                    helpers::exit_with_error(format!(
                        "bcs serialization failure of raw_txn : {}",
                        err
                    ))
                })
                .unwrap(),
        ),
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateSignedTxnRequest {
    pub raw_txn: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateSignedTxnResponse {
    pub signed_txn: String,
    pub txn_hash: String,
}

fn generate_signed_txn(request: GenerateSignedTxnRequest) -> GenerateSignedTxnResponse {
    let raw_txn: RawTransaction = bcs::from_bytes(
        &hex::decode(request.raw_txn.clone())
            .map_err(|err| {
                helpers::exit_with_error(format!("hex decode of raw_txn failed : {}", err))
            })
            .unwrap(),
    )
    .map_err(|err| {
        helpers::exit_with_error(format!("bcs deserialization failure of raw_txn : {}", err))
    })
    .unwrap();
    let signature = Ed25519Signature::from_encoded_string(&request.signature)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode signature {} : {}",
                request.signature, err
            ))
        })
        .unwrap();
    let public_key = Ed25519PublicKey::from_encoded_string(&request.public_key)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode public_key {} : {}",
                request.public_key, err
            ))
        })
        .unwrap();
    let signed_txn = SignedTransaction::new(raw_txn, public_key, signature);
    let txn_hash = CryptoHash::hash(&Transaction::UserTransaction(signed_txn.clone())).to_hex();
    let signed_txn = hex::encode(
        bcs::to_bytes(&signed_txn)
            .map_err(|err| {
                helpers::exit_with_error(format!(
                    "bcs serialization failure of signed_txn : {}",
                    err
                ))
            })
            .unwrap(),
    );
    GenerateSignedTxnResponse {
        signed_txn,
        txn_hash,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateKeypairResponse {
    pub private_key: String,
    pub public_key: String,
    pub diem_auth_key: String,
    pub diem_account_address: String,
}

fn generate_key_pair(seed: Option<u64>) -> GenerateKeypairResponse {
    let mut rng = StdRng::seed_from_u64(seed.unwrap_or_else(rand::random));
    let keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
        Ed25519PrivateKey::generate(&mut rng).into();
    let diem_auth_key = AuthenticationKey::ed25519(&keypair.public_key);
    let diem_account_address: String = diem_auth_key.derived_address().to_string();
    let diem_auth_key: String = diem_auth_key.to_string();
    GenerateKeypairResponse {
        private_key: keypair
            .private_key
            .to_encoded_string()
            .map_err(|err| {
                helpers::exit_with_error(format!("Failed to encode private key : {}", err))
            })
            .unwrap(),
        public_key: keypair
            .public_key
            .to_encoded_string()
            .map_err(|err| {
                helpers::exit_with_error(format!("Failed to encode public key : {}", err))
            })
            .unwrap(),
        diem_auth_key,
        diem_account_address,
    }
}

///////////////////////////
// Sign a RawTransaction //
///////////////////////////

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct SignTransactionUsingEd25519Request {
    pub raw_txn: String,
    pub private_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct SignTransactionUsingEd25519Response {
    pub signature: String,
}

fn sign_transaction_using_ed25519(
    request: SignTransactionUsingEd25519Request,
) -> SignTransactionUsingEd25519Response {
    let raw_txn: RawTransaction = bcs::from_bytes(
        &hex::decode(request.raw_txn.clone())
            .map_err(|err| {
                helpers::exit_with_error(format!("hex decode of raw_txn failed : {}", err))
            })
            .unwrap(),
    )
    .map_err(|err| {
        helpers::exit_with_error(format!("bcs deserialization failure of raw_txn : {}", err))
    })
    .unwrap();
    let private_key = Ed25519PrivateKey::from_encoded_string(&request.private_key)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode private_key {} : {}",
                request.private_key, err
            ))
        })
        .unwrap();
    let signature = private_key.sign(&raw_txn);
    SignTransactionUsingEd25519Response {
        signature: signature
            .to_encoded_string()
            .map_err(|err| {
                helpers::exit_with_error(format!("Failed to encode signature : {}", err))
            })
            .unwrap(),
    }
}

//////////////////////
// Verify raw bytes //
//////////////////////

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct VerifyEd25519SignatureRequest {
    pub payload: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct VerifyEd25519SignatureResponse {
    pub valid_signature: bool,
}

fn verify_signature_using_ed25519(
    request: VerifyEd25519SignatureRequest,
) -> VerifyEd25519SignatureResponse {
    let message = helpers::hex_decode(&request.payload);
    let signature = Ed25519Signature::from_encoded_string(&request.signature)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode signature {} : {}",
                request.signature, err
            ))
        })
        .unwrap();
    let public_key = Ed25519PublicKey::from_encoded_string(&request.public_key)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode public_key {} : {}",
                request.public_key, err
            ))
        })
        .unwrap();
    let valid_signature = signature
        .verify_arbitrary_msg(&message, &public_key)
        .is_ok();
    VerifyEd25519SignatureResponse { valid_signature }
}

//////////////////////////////////////////
// verify signature of a RawTransaction //
//////////////////////////////////////////

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct VerifyTransactionEd25519SignatureRequest {
    pub raw_txn: String,
    pub signature: String,
    pub public_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct VerifyTransactionEd25519SignatureResponse {
    pub valid_signature: bool,
}

fn verify_transaction_signature_using_ed25519(
    request: VerifyTransactionEd25519SignatureRequest,
) -> VerifyTransactionEd25519SignatureResponse {
    let raw_txn: RawTransaction = bcs::from_bytes(
        &hex::decode(request.raw_txn.clone())
            .map_err(|err| {
                helpers::exit_with_error(format!("hex decode of raw_txn failed : {}", err))
            })
            .unwrap(),
    )
    .map_err(|err| {
        helpers::exit_with_error(format!("bcs deserialization failure of raw_txn : {}", err))
    })
    .unwrap();
    let signature = Ed25519Signature::from_encoded_string(&request.signature)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode signature {} : {}",
                request.signature, err
            ))
        })
        .unwrap();
    let public_key = Ed25519PublicKey::from_encoded_string(&request.public_key)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode public_key {} : {}",
                request.public_key, err
            ))
        })
        .unwrap();
    let valid_signature = signature.verify(&raw_txn, &public_key).is_ok();
    VerifyTransactionEd25519SignatureResponse { valid_signature }
}
