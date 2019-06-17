// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! ConsensusKeyManager gives a simple interface for consensus to interact with the secret service.
//! This simple key manager will become more complicated in future versions,
//! now it asks the secret service to generate an ed25519 key on creation,
//! it can then transfer to the secret service the requests to get consensus public key and to sign
//! a consensus message.

use crate::{
    crypto_wrappers::{GenericPublicKey, GenericSignature, KeyID},
    proto::{
        secret_service::{GenerateKeyRequest, KeyType, PublicKeyRequest, SignRequest},
        secret_service_grpc::SecretServiceClient,
    },
};
use crypto::hash::HashValue;
use failure::prelude::*;
use nextgen_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use std::{convert::TryFrom, sync::Arc};

/// A consensus key manager - interface between consensus and the secret service.
pub struct ConsensusKeyManager {
    secret_service: Arc<SecretServiceClient>,
    signing_keyid: KeyID,
}

impl ConsensusKeyManager {
    /// Saves a reference to the secret service and asks it to generate a new signing key.
    pub fn new(secret_service: Arc<SecretServiceClient>) -> Result<Self> {
        Ok(Self {
            secret_service: Arc::clone(&secret_service),
            signing_keyid: {
                // generating consensus key: for simplicity it's assumed we only have one key and it
                // is generated here we will have to modify this later
                let mut gen_req: GenerateKeyRequest = GenerateKeyRequest::new();
                gen_req.set_spec(KeyType::Ed25519);

                let response = secret_service.generate_key(&gen_req)?;
                KeyID(HashValue::from_slice(response.get_key_id())?)
            },
        })
    }

    /// Asks the secret service for the public key and returns it.
    pub fn get_consensus_public_key(&self) -> Result<GenericPublicKey> {
        let mut pk_req: PublicKeyRequest = PublicKeyRequest::new();
        pk_req.set_key_id(self.signing_keyid.to_vec());
        let response = self.secret_service.get_public_key(&pk_req)?;
        let public_key: &[u8] = response.get_public_key();

        Ok(GenericPublicKey::Ed(Ed25519PublicKey::try_from(
            public_key,
        )?))
    }

    /// Asks the secret service to sign a hash of the consensus message.
    pub fn sign_consensus_message(&self, message: &HashValue) -> Result<(GenericSignature)> {
        let mut sig_req: SignRequest = SignRequest::new();
        sig_req.set_key_id(self.signing_keyid.to_vec());
        sig_req.set_message_hash(message.to_vec());
        let response = self.secret_service.sign(&sig_req)?;
        let signature = response.get_signature();

        Ok(GenericSignature::Ed(Ed25519Signature::try_from(signature)?))
    }
}
