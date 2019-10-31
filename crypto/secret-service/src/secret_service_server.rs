// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! The Secret service server stores the secret key and performs operations on these keys.
//! Right now the service supports requests to generate the secret key (of Ed25519 or BLS12-381
//! type), return the corresponding public key and sign.

use crate::{
    crypto_wrappers::{GenericPrivateKey, GenericPublicKey, GenericSignature, KeyID},
    proto::{
        ErrorCode, GenerateKeyRequest, GenerateKeyResponse, KeyType, PublicKeyRequest,
        PublicKeyResponse, SecretService, SignRequest, SignResponse,
    },
};
use failure::prelude::*;
use grpc_helpers::provide_grpc_response;
use libra_crypto::{
    bls12381::BLS12381PrivateKey, ed25519::Ed25519PrivateKey, hash::HashValue, traits::*,
};
use rand::{rngs::EntropyRng, Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

#[cfg(test)]
#[path = "unit_tests/secret_service_test.rs"]
mod secret_service_test;

/// Secret service server that holds the secret keys and implements the necessary operations on
/// them.
#[derive(Clone, Default)]
pub struct SecretServiceServer {
    // RwLock is chosen over Mutex because the RwLock won't get poisoned if a panic occurs during
    // read
    keys: Arc<RwLock<HashMap<KeyID, GenericPrivateKey>>>, /* Arc for shared ownership by clones;
                                                           * RwLock for being write-accessible
                                                           * by one
                                                           * thread at a time */
}

/// SecretServiceServer matches the API of proto/secret_service.proto but operates on our own Crypto
/// API structures
impl SecretServiceServer {
    /// A fresh secret service creates an empty HashMap for the keys.
    pub fn new() -> Self {
        SecretServiceServer {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generates a new secret key (for now this is the code for testing).
    pub fn generate_key_inner(&mut self, spec: KeyType) -> Result<KeyID> {
        let seed: [u8; 32] = EntropyRng::new().gen();

        let private_key: GenericPrivateKey = {
            let mut rng = ChaChaRng::from_seed(seed);

            match spec {
                KeyType::Ed25519 => {
                    GenericPrivateKey::Ed(Ed25519PrivateKey::generate_for_testing(&mut rng))
                }
                KeyType::Bls12381 => {
                    GenericPrivateKey::BLS(BLS12381PrivateKey::generate_for_testing(&mut rng))
                }
            }
        };

        // For better security the keyid is a random string independent of the keys
        let keyid = KeyID(HashValue::random());

        // Alternatively (but less secure): keyid can be the hash of the public key.
        // The problem here is that someone who knows the public key (e.g. from the chain) will know
        // the how to make a signature request to the secret service
        /*
        let ed25519_public_key: ed25519_dalek::PublicKey = (&new_ed25519_key).into();
        let keyid = KeyID(HashValue::from_slice(ed25519_public_key.as_bytes()).unwrap());
         */

        let result = keyid.clone();
        let mut keys = self
            .keys
            .write()
            .expect("[generating new key] acquire keys lock");
        keys.insert(keyid, private_key);
        Ok(result)
    }

    /// Computes and returns the public key of the corresponding secret key.
    pub fn get_public_key_inner(&self, keyid: &KeyID) -> Option<GenericPublicKey> {
        let keys = self
            .keys
            .read()
            .expect("[getting public key] acquire keys lock");
        keys.get(keyid).map(GenericPublicKey::from)
    }

    /// Signs a hash value and returns the signature.
    pub fn sign_inner(&self, keyid: &KeyID, message: &HashValue) -> Option<GenericSignature> {
        let keys = self
            .keys
            .read()
            .expect("[obtaining signature] acquire keys lock");
        keys.get(keyid)
            .map(|secret_key| secret_key.sign_message(message))
    }
}

/// SecretServiceServer implements the proto trait SecretService.
/// The methods below wrap around inner methods of SecretServiceServer and operate on grpc's
/// requests/responses.
impl SecretService for SecretServiceServer {
    /// Generates a new key answering a GenerateKeyRequest with a GenerateKeyResponse.
    fn generate_key(
        &mut self,
        ctx: ::grpcio::RpcContext,
        req: GenerateKeyRequest,
        sink: ::grpcio::UnarySink<GenerateKeyResponse>,
    ) {
        let mut response = GenerateKeyResponse::default();
        let spec = req.spec();
        let keyid = self.generate_key_inner(spec);
        if let Ok(key_identity) = keyid {
            response.set_code(ErrorCode::Success);
            response.key_id = key_identity.to_vec();
        } else {
            response.set_code(ErrorCode::Unspecified);
        }
        provide_grpc_response(Ok(response), ctx, sink);
    }

    /// Returns a corresponding public key answering a PublicKeyRequest with a PublicKeyResponse.
    fn get_public_key(
        &mut self,
        ctx: ::grpcio::RpcContext,
        req: PublicKeyRequest,
        sink: ::grpcio::UnarySink<PublicKeyResponse>,
    ) {
        let mut response = PublicKeyResponse::default();
        let keyid_raw_bytes = req.key_id;
        if keyid_raw_bytes.len() != HashValue::LENGTH {
            response.set_code(ErrorCode::WrongLength);
        } else if let Ok(keyid) = HashValue::from_slice(&keyid_raw_bytes) {
            let keyid = KeyID(keyid);
            let public_key = self.get_public_key_inner(&keyid);
            if let Some(pkey) = public_key {
                response.set_code(ErrorCode::Success);
                response.public_key = pkey.to_bytes().to_vec();
            } else {
                response.set_code(ErrorCode::KeyIdNotFound);
            }
        } else {
            response.set_code(ErrorCode::Unspecified);
        }
        provide_grpc_response(Ok(response), ctx, sink);
    }

    /// Returns a signature on a given hash-value with a corresponding signing key answering a
    /// SignRequest with a SignResponse.
    fn sign(
        &mut self,
        ctx: ::grpcio::RpcContext,
        req: SignRequest,
        sink: ::grpcio::UnarySink<SignResponse>,
    ) {
        let mut response = SignResponse::default();
        let keyid_raw_bytes = req.key_id;
        let message_raw_bytes = req.message_hash;
        if keyid_raw_bytes.len() != HashValue::LENGTH
            || message_raw_bytes.len() != HashValue::LENGTH
        {
            response.set_code(ErrorCode::WrongLength);
        } else if let Ok(keyid) = HashValue::from_slice(&keyid_raw_bytes) {
            let keyid = KeyID(keyid);
            if let Ok(message) = HashValue::from_slice(&message_raw_bytes) {
                let signature = self.sign_inner(&keyid, &message);
                if let Some(sig) = signature {
                    response.set_code(ErrorCode::Success);
                    response.signature = sig.to_bytes().to_vec();
                } else {
                    response.set_code(ErrorCode::KeyIdNotFound);
                }
            } else {
                response.set_code(ErrorCode::Unspecified);
            }
        } else {
            response.set_code(ErrorCode::Unspecified);
        }
        provide_grpc_response(Ok(response), ctx, sink);
    }
}
