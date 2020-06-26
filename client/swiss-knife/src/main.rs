// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature},
    hash::CryptoHash,
    test_utils::KeyPair,
    HashValue, Signature, SigningKey, Uniform, ValidCryptoMaterialStringExt,
};
use libra_types::transaction::{
    authenticator::AuthenticationKey, RawTransaction, SignedTransaction, TransactionPayload,
};
use rand::{prelude::StdRng, SeedableRng};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use swiss_knife::helpers;

#[derive(Debug, StructOpt)]
enum Command {
    /// Generates and serializes a RawTransaction and its hash. The hash of this RawTransaction needs to be signed to generate a SignedTransaction.
    /// Takes the input json payload from stdin. Writes the output json payload to stdout.
    /// Refer to README.md for examples.
    GenerateRawTxn,
    /// Generates a SignedTransaction given the serialized RawTransaction, the public key and signature.
    /// Takes the input json payload from stdin. Writes the output json payload to stdout.
    /// Refer to README.md for examples.
    GenerateSignedTxn,
    /// Generates a Ed25519Keypair for testing from the given u64 seed.
    GenerateTestEd25519Keypair {
        #[structopt(long)]
        seed: u64,
    },
    /// Generates a signature using the provided Ed25519 private key.
    SignPayloadUsingEd25519,
    /// Verifies the Ed25519 signature using the provided Ed25519 public key.
    VerifyEd25519Signature,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "swiss-knife",
    about = "Tool for generating, serializing (LCS), hashing and signing Libra transactions. Additionally, contains tools for testing. Please refer to README.md for examples."
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
        Command::SignPayloadUsingEd25519 => {
            let input = helpers::read_stdin();
            let request: SignPayloadUsingEd25519Request = serde_json::from_str(&input)
                .map_err(|err| {
                    helpers::exit_with_error(format!("Failed to deserialize json : {}", err))
                })
                .unwrap();
            helpers::exit_success_with_data(sign_payload_using_ed25519(request));
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
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(rename_all = "snake_case")]
struct TxnParams {
    // Sender's address
    pub sender_address: String,
    // Sequence number of this transaction corresponding to sender's account.
    pub sequence_number: u64,
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
    pub expiration_timestamp: u64,
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
    pub raw_txn: String,
    pub raw_txn_hash: String,
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
            transaction_builder::encode_transfer_with_metadata_script(
                coin_tag,
                recipient_address,
                amount,
                helpers::hex_decode(&metadata_hex_encoded),
                helpers::hex_decode(&metadata_signature_hex_encoded),
            )
        }
    };
    let raw_txn = RawTransaction::new(
        helpers::account_address_parser(&g.txn_params.sender_address),
        g.txn_params.sequence_number,
        TransactionPayload::Script(script),
        g.txn_params.max_gas_amount,
        g.txn_params.gas_unit_price,
        g.txn_params.gas_currency_code,
        std::time::Duration::new(g.txn_params.expiration_timestamp, 0),
    );
    GenerateRawTxnResponse {
        raw_txn: hex::encode(
            lcs::to_bytes(&raw_txn)
                .map_err(|err| {
                    helpers::exit_with_error(format!(
                        "lcs serialization failure of raw_txn : {}",
                        err
                    ))
                })
                .unwrap(),
        ),
        raw_txn_hash: CryptoHash::hash(&raw_txn).to_hex(),
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
}

fn generate_signed_txn(request: GenerateSignedTxnRequest) -> GenerateSignedTxnResponse {
    let raw_txn: RawTransaction = lcs::from_bytes(
        &hex::decode(request.raw_txn.clone())
            .map_err(|err| {
                helpers::exit_with_error(format!("hex decode of raw_txn failed : {}", err))
            })
            .unwrap(),
    )
    .map_err(|err| {
        helpers::exit_with_error(format!("lcs deserialization failure of raw_txn : {}", err))
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
    let signed_txn = hex::encode(
        lcs::to_bytes(&signed_txn)
            .map_err(|err| {
                helpers::exit_with_error(format!(
                    "lcs serialization failure of signed_txn : {}",
                    err
                ))
            })
            .unwrap(),
    );
    GenerateSignedTxnResponse { signed_txn }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct GenerateKeypairResponse {
    pub private_key: String,
    pub public_key: String,
    pub libra_auth_key: String,
    pub libra_account_address: String,
}

fn generate_key_pair(seed: u64) -> GenerateKeypairResponse {
    let mut rng = StdRng::seed_from_u64(seed);
    let keypair: KeyPair<Ed25519PrivateKey, Ed25519PublicKey> =
        Ed25519PrivateKey::generate(&mut rng).into();
    let libra_auth_key = AuthenticationKey::ed25519(&keypair.public_key);
    let libra_account_address: String = libra_auth_key.derived_address().to_string();
    let libra_auth_key: String = libra_auth_key.to_string();
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
        libra_auth_key,
        libra_account_address,
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct SignPayloadUsingEd25519Request {
    pub payload: String,
    pub private_key: String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
struct SignPayloadUsingEd25519Response {
    pub signature: String,
}

fn sign_payload_using_ed25519(
    request: SignPayloadUsingEd25519Request,
) -> SignPayloadUsingEd25519Response {
    let hash_value = HashValue::from_hex(&request.payload)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode payload into HashValue {} : {}",
                request.payload, err
            ))
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
    let signature = private_key.sign_message(&hash_value);
    SignPayloadUsingEd25519Response {
        signature: signature
            .to_encoded_string()
            .map_err(|err| {
                helpers::exit_with_error(format!("Failed to encode signature : {}", err))
            })
            .unwrap(),
    }
}

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
    let hash_value = HashValue::from_hex(&request.payload)
        .map_err(|err| {
            helpers::exit_with_error(format!(
                "Failed to hex decode payload into HashValue {} : {}",
                request.payload, err
            ))
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
    VerifyEd25519SignatureResponse {
        valid_signature: signature.verify(&hash_value, &public_key).is_ok(),
    }
}
