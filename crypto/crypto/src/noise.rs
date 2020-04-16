// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Noise is a [protocol framework](https://noiseprotocol.org/) which we use in Libra to
//! encrypt and authenticate communications between nodes of the network.
//!
//! This file implements a stripped-down version of Noise_IK_25519_AESGCM_SHA256.
//! This means that only the parts that we care about (the IK handshake) are implemented.
//!
//! Note that to benefit from hardware support for AES, you must build this crate with the following
//! flags: `RUSTFLAGS="-Ctarget-cpu=sandybridge -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"`.
//!
//! Usage example:
//!
//! ```
//! use libra_crypto::noise::NoiseConfig;
//! use x25519_dalek as x25519;
//! use rand::prelude::*;
//!
//! # fn main() -> Result<(), libra_crypto::noise::NoiseError> {
//! let mut rng = rand::thread_rng();
//! let initiator_static = x25519::StaticSecret::new(&mut rng);
//! let responder_static = x25519::StaticSecret::new(&mut rng);
//! let responder_public = x25519::PublicKey::from(&responder_static);
//!
//! let initiator = NoiseConfig::new(initiator_static);
//! let responder = NoiseConfig::new(responder_static);
//!
//! let (initiator_state, first_message) = initiator
//!   .initiate_connection(&mut rng, b"prologue", &responder_public, None)?;
//! let (second_message, remote_static, _, mut responder_session) = responder
//!   .respond_to_client_and_finalize(&mut rng, b"prologue", &first_message, None)?;
//! let (_, mut initiator_session) = initiator
//!   .finalize_connection(initiator_state, &second_message)?;
//!
//! let encrypted_message = initiator_session
//!   .write_message(b"hello world")
//!   .expect("session should not be closed");
//! let received_message = responder_session
//!   .read_message(&encrypted_message)
//!   .expect("session should not be closed");
//! # Ok(())
//! # }
//! ```
//!

use std::io::{Cursor, Read as _};

use crate::{hash::HashValue, hkdf::Hkdf};

use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, NewAead, Payload},
    Aes256Gcm,
};
use sha2::Digest;
use thiserror::Error;
use x25519_dalek as x25519;

//
// Useful constants
// ----------------
//

const PROTOCOL_NAME: &[u8] = b"Noise_IK_25519_AESGCM_SHA256\0\0\0\0";
const X25519_PUBLIC_KEY_LENGTH: usize = 32;
const AES_GCM_TAGLEN: usize = 16;
const AES_NONCE_SIZE: usize = 12;
const MAX_SIZE_NOISE_MSG: usize = 65535;

/// This implementation relies on the fact that the hash function used has a 256-bit output
#[rustfmt::skip]
const _: [(); 0 - !{ const ASSERT: bool = HashValue::LENGTH == 32; ASSERT } as usize] = [];

//
// Errors
// ------
//

/// A NoiseError enum represents the different types of error that noise can return to users of the crate
#[derive(Debug, Error)]
pub enum NoiseError {
    /// the received message is too short to contain the expected data
    #[error("the received message is too short to contain the expected data")]
    MsgTooShort,

    /// HKDF has failed (in practice there is no reason for HKDF to fail)
    #[error("HKDF has failed")]
    Hkdf,

    /// encryption has failed (in practice there is no reason for encryption to fail)
    #[error("encryption has failed")]
    Encrypt,

    /// could not decrypt the received data (most likely the data was tampered with
    #[error("could not decrypt the received data")]
    Decrypt,

    /// the public key received is of the wrong format
    #[error("the public key received is of the wrong format")]
    WrongPublicKeyReceived,

    /// session was closed due to decrypt error
    #[error("session was closed due to decrypt error")]
    SessionClosed,

    /// the payload that we are trying to send is too large
    #[error("the payload that we are trying to send is too large")]
    PayloadTooLarge,

    /// the message we received is too large
    #[error("the message we received is too large")]
    ReceivedMsgTooLarge,
}

//
// helpers
// -------
//

fn hash(data: &[u8]) -> Vec<u8> {
    sha2::Sha256::digest(data).to_vec()
}

fn hkdf(ck: &[u8], dh_output: Option<&[u8]>) -> Result<(Vec<u8>, Vec<u8>), NoiseError> {
    let dh_output = dh_output.unwrap_or_else(|| &[]);
    let hkdf_output = Hkdf::<sha2::Sha256>::extract_then_expand(Some(ck), dh_output, None, 64);

    let hkdf_output = hkdf_output.map_err(|_| NoiseError::Hkdf)?;
    let (k1, k2) = hkdf_output.split_at(32);
    Ok((k1.to_vec(), k2.to_vec()))
}

fn mix_hash(h: &mut Vec<u8>, data: &[u8]) {
    h.extend_from_slice(data);
    *h = hash(h);
}

fn mix_key(ck: &mut Vec<u8>, dh_output: &[u8]) -> Result<Vec<u8>, NoiseError> {
    let (new_ck, k) = hkdf(ck, Some(dh_output))?;
    *ck = new_ck;
    Ok(k)
}

fn checked_vec_to_public_key(bytes: &[u8]) -> Result<x25519::PublicKey, NoiseError> {
    if bytes.len() != X25519_PUBLIC_KEY_LENGTH {
        return Err(NoiseError::WrongPublicKeyReceived);
    }
    let mut array = [0u8; X25519_PUBLIC_KEY_LENGTH];
    array.copy_from_slice(bytes);
    Ok(array.into())
}

//
// Noise implementation
// --------------------
//

/// A key holder structure used for both initiators and responders.
pub struct NoiseConfig {
    private_key: x25519::StaticSecret,
    public_key: [u8; X25519_PUBLIC_KEY_LENGTH],
}

/// Refer to the Noise protocol framework specification in order to understand these fields.
pub struct InitiatorHandshakeState {
    /// rolling hash
    h: Vec<u8>,
    /// chaining key
    ck: Vec<u8>,
    /// ephemeral key
    e: x25519::StaticSecret,
}

impl NoiseConfig {
    /// A peer must create a NoiseConfig through this function before being able to connect with other peers.
    pub fn new(private_key: x25519::StaticSecret) -> Self {
        let public_key = x25519::PublicKey::from(&private_key);
        Self {
            private_key,
            public_key: public_key.as_bytes().to_owned(),
        }
    }

    //
    // Initiator
    // ---------

    /// An initiator can use this function to initiate a handshake with a known responder.
    pub fn initiate_connection(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        prologue: &[u8],
        remote_public: &x25519::PublicKey,
        payload: Option<&[u8]>,
    ) -> Result<(InitiatorHandshakeState, Vec<u8>), NoiseError> {
        // checks
        if let Some(payload) = payload {
            if payload.len()
                > MAX_SIZE_NOISE_MSG - 2 * X25519_PUBLIC_KEY_LENGTH - 2 * AES_GCM_TAGLEN
            {
                return Err(NoiseError::PayloadTooLarge);
            }
        }

        // initialize
        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        mix_hash(&mut h, &prologue);
        mix_hash(&mut h, remote_public.as_bytes());

        // -> e
        let e = if cfg!(test) {
            let mut ephemeral_private = [0u8; X25519_PUBLIC_KEY_LENGTH];
            rng.fill_bytes(&mut ephemeral_private);
            x25519::StaticSecret::from(ephemeral_private)
        } else {
            x25519::StaticSecret::new(rng)
        };
        let e_pub = x25519::PublicKey::from(&e);
        mix_hash(&mut h, e_pub.as_bytes());
        let mut msg = e_pub.as_bytes().to_vec();

        // -> es
        let dh_output = e.diffie_hellman(remote_public);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> s
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let msg_and_ad = Payload {
            msg: &self.public_key,
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_static = aead.encrypt(nonce, msg_and_ad).unwrap(); // this API cannot fail
        mix_hash(&mut h, &encrypted_static);
        msg.extend_from_slice(&encrypted_static);

        // -> ss
        let dh_output = self.private_key.diffie_hellman(remote_public);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> payload
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let msg_and_ad = Payload {
            msg: payload.unwrap_or_else(|| &[]),
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_payload = aead
            .encrypt(nonce, msg_and_ad)
            .map_err(|_| NoiseError::Encrypt)?;
        mix_hash(&mut h, &encrypted_payload);
        msg.extend_from_slice(&encrypted_payload);

        // return
        let handshake_state = InitiatorHandshakeState { h, ck, e };
        Ok((handshake_state, msg))
    }

    /// A client can call this to finalize a connection, after receiving an answer from a server.
    pub fn finalize_connection(
        &self,
        handshake_state: InitiatorHandshakeState,
        received_message: &[u8],
    ) -> Result<(Vec<u8>, NoiseSession), NoiseError> {
        // checks
        if received_message.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }
        // retrieve handshake state
        let InitiatorHandshakeState { mut h, mut ck, e } = handshake_state;

        // <- e
        let mut remote_ephemeral = [0u8; X25519_PUBLIC_KEY_LENGTH];
        let mut cursor = Cursor::new(received_message);
        cursor
            .read_exact(&mut remote_ephemeral)
            .map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &remote_ephemeral);
        let remote_ephemeral = x25519::PublicKey::from(remote_ephemeral);

        // <- ee
        let dh_output = e.diffie_hellman(&remote_ephemeral);
        mix_key(&mut ck, dh_output.as_bytes())?;

        // <- se
        let dh_output = self.private_key.diffie_hellman(&remote_ephemeral);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // <- payload
        let offset = cursor.position() as usize;
        let received_encrypted_payload = &cursor.into_inner()[offset..];

        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: received_encrypted_payload,
            aad: &h,
        };
        let received_payload = match aead.decrypt(nonce, ct_and_ad) {
            Ok(res) => res,
            Err(_) if cfg!(feature = "fuzzing") => Vec::new(),
            Err(_) => {
                return Err(NoiseError::Decrypt);
            }
        };

        // split
        let (k1, k2) = hkdf(&ck, None)?;
        let session = NoiseSession::new(k1, k2);

        //
        Ok((received_payload, session))
    }

    //
    // Responder
    // ---------

    /// The sole function the server can call to parse and respond to a client initiating a connection.
    pub fn respond_to_client_and_finalize(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        prologue: &[u8],
        received_message: &[u8],
        payload: Option<&[u8]>,
    ) -> Result<(Vec<u8>, x25519::PublicKey, Vec<u8>, NoiseSession), NoiseError> {
        // checks
        if let Some(payload) = payload {
            if payload.len() > MAX_SIZE_NOISE_MSG - X25519_PUBLIC_KEY_LENGTH - AES_GCM_TAGLEN {
                return Err(NoiseError::PayloadTooLarge);
            }
        }
        if received_message.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }
        // initialize
        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        mix_hash(&mut h, prologue);
        mix_hash(&mut h, &self.public_key);

        // buffer message received
        let mut cursor = Cursor::new(received_message);

        // <- e
        let mut remote_ephemeral = [0u8; X25519_PUBLIC_KEY_LENGTH];
        cursor
            .read_exact(&mut remote_ephemeral)
            .map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &remote_ephemeral);
        let remote_ephemeral = x25519::PublicKey::from(remote_ephemeral);

        // <- es
        let dh_output = self.private_key.diffie_hellman(&remote_ephemeral);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // <- s
        let mut encrypted_remote_static = [0u8; X25519_PUBLIC_KEY_LENGTH + AES_GCM_TAGLEN];
        cursor
            .read_exact(&mut encrypted_remote_static)
            .map_err(|_| NoiseError::MsgTooShort)?;

        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: &encrypted_remote_static,
            aad: &h,
        };
        let remote_static = match aead.decrypt(nonce, ct_and_ad) {
            Ok(res) => res,
            Err(_) if cfg!(feature = "fuzzing") => encrypted_remote_static[..32].to_vec(),
            Err(_) => {
                return Err(NoiseError::Decrypt);
            }
        };
        let remote_static = checked_vec_to_public_key(&remote_static)?;
        mix_hash(&mut h, &encrypted_remote_static);

        // <- ss
        let dh_output = self.private_key.diffie_hellman(&remote_static);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // <- payload
        let offset = cursor.position() as usize;
        let received_encrypted_payload = &cursor.into_inner()[offset..];

        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: received_encrypted_payload,
            aad: &h,
        };
        let received_payload = match aead.decrypt(nonce, ct_and_ad) {
            Ok(res) => res,
            Err(_) if cfg!(feature = "fuzzing") => Vec::new(),
            Err(_) => {
                return Err(NoiseError::Decrypt);
            }
        };
        mix_hash(&mut h, received_encrypted_payload);

        // -> e
        let e = if cfg!(test) {
            let mut ephemeral_private = [0u8; 32];
            rng.fill_bytes(&mut ephemeral_private);
            x25519::StaticSecret::from(ephemeral_private)
        } else {
            x25519::StaticSecret::new(rng)
        };
        let e_pub = x25519::PublicKey::from(&e);
        mix_hash(&mut h, e_pub.as_bytes());
        let mut msg = e_pub.as_bytes().to_vec();

        // -> ee
        let dh_output = e.diffie_hellman(&remote_ephemeral);
        mix_key(&mut ck, dh_output.as_bytes())?;

        // -> se
        let dh_output = e.diffie_hellman(&remote_static);
        let k = mix_key(&mut ck, dh_output.as_bytes())?;

        // -> payload
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&k));

        let msg_and_ad = Payload {
            msg: payload.unwrap_or_else(|| &[]),
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_payload = aead
            .encrypt(nonce, msg_and_ad)
            .map_err(|_| NoiseError::Encrypt)?;
        mix_hash(&mut h, &encrypted_payload);
        msg.extend_from_slice(&encrypted_payload);

        // split
        let (k1, k2) = hkdf(&ck, None)?;
        let session = NoiseSession::new(k2, k1);

        //
        Ok((msg, remote_static, received_payload, session))
    }
}

//
// Post-Handshake
// --------------

/// A NoiseSession is produced after a successful Noise handshake, and can be use to encrypt and decrypt messages to the other peer.
pub struct NoiseSession {
    /// a session can be marked as invalid if it has seen a decryption failure
    valid: bool,
    /// key used to encrypt messages to the other peer
    write_key: Vec<u8>,
    /// associated nonce (in practice the maximum u64 value cannot be reached)
    write_nonce: u64,
    /// key used to decrypt messages received from the other peer
    read_key: Vec<u8>,
    /// associated nonce (in practice the maximum u64 value cannot be reached)
    read_nonce: u64,
}

impl NoiseSession {
    fn new(write_key: Vec<u8>, read_key: Vec<u8>) -> Self {
        Self {
            valid: true,
            write_key,
            write_nonce: 0,
            read_key,
            read_nonce: 0,
        }
    }

    /// encrypts a message for the other peers (post-handshake)
    pub fn write_message(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, NoiseError> {
        // checks
        if !self.valid {
            return Err(NoiseError::SessionClosed);
        }
        if plaintext.len() > MAX_SIZE_NOISE_MSG - AES_GCM_TAGLEN {
            return Err(NoiseError::PayloadTooLarge);
        }

        // encrypt
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&self.write_key));

        let msg_and_ad = Payload {
            msg: plaintext,
            aad: &[],
        };
        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.write_nonce.to_be_bytes());
        let nonce = GenericArray::from_slice(&nonce);
        let ciphertext = aead.encrypt(nonce, msg_and_ad).unwrap(); // this API cannot fail

        // increment nonce
        self.write_nonce += 1;

        //
        Ok(ciphertext)
    }

    /// decrypts a message from the other peer (post-handshake)
    pub fn read_message(&mut self, ciphertext: &[u8]) -> Result<Vec<u8>, NoiseError> {
        // checks
        if !self.valid {
            return Err(NoiseError::SessionClosed);
        }
        if ciphertext.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }

        // decrypt
        let aead = Aes256Gcm::new(GenericArray::clone_from_slice(&self.read_key));

        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.read_nonce.to_be_bytes());
        let nonce = GenericArray::from_slice(&nonce);
        let ct_and_ad = Payload {
            msg: &ciphertext,
            aad: &[],
        };
        let plaintext = aead.decrypt(nonce, ct_and_ad).map_err(|_| {
            self.valid = false;
            NoiseError::Decrypt
        })?;

        // increment nonce
        self.read_nonce += 1;

        //
        Ok(plaintext)
    }
}
