// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

//! Noise is a [protocol framework](https://noiseprotocol.org/) which we use in Diem to
//! encrypt and authenticate communications between nodes of the network.
//!
//! This file implements a stripped-down version of Noise_IK_25519_AESGCM_SHA256.
//! This means that only the parts that we care about (the IK handshake) are implemented.
//!
//! Note that to benefit from hardware support for AES, you must build this crate with the following
//! flags: `RUSTFLAGS="-Ctarget-cpu=skylake -Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3"`.
//!
//! Usage example:
//!
//! ```
//! use diem_crypto::{noise, x25519, traits::*};
//! use rand::prelude::*;
//!
//! # fn main() -> Result<(), diem_crypto::noise::NoiseError> {
//! let mut rng = rand::thread_rng();
//! let initiator_static = x25519::PrivateKey::generate(&mut rng);
//! let responder_static = x25519::PrivateKey::generate(&mut rng);
//! let responder_public = responder_static.public_key();
//!
//! let initiator = noise::NoiseConfig::new(initiator_static);
//! let responder = noise::NoiseConfig::new(responder_static);
//!
//! let payload1 = b"the client can send an optional payload in the first message";
//! let mut buffer = vec![0u8; noise::handshake_init_msg_len(payload1.len())];
//! let initiator_state = initiator
//!   .initiate_connection(&mut rng, b"prologue", responder_public, Some(payload1), &mut buffer)?;
//!
//! let payload2 = b"the server can send an optional payload as well as part of the handshake";
//! let mut buffer2 = vec![0u8; noise::handshake_resp_msg_len(payload2.len())];
//! let (received_payload, mut responder_session) = responder
//!   .respond_to_client_and_finalize(&mut rng, b"prologue", &buffer, Some(payload2), &mut buffer2)?;
//! assert_eq!(received_payload.as_slice(), &payload1[..]);
//!
//! let (received_payload, mut initiator_session) = initiator
//!   .finalize_connection(initiator_state, &buffer2)?;
//! assert_eq!(received_payload.as_slice(), &payload2[..]);
//!
//! let message_sent = b"hello world".to_vec();
//! let mut buffer = message_sent.clone();
//! let auth_tag = initiator_session
//!   .write_message_in_place(&mut buffer)?;
//! buffer.extend_from_slice(&auth_tag);
//!
//! let received_message = responder_session
//!   .read_message_in_place(&mut buffer)?;
//!
//! assert_eq!(received_message, message_sent.as_slice());
//!
//! # Ok(())
//! # }
//! ```
//!
#![allow(clippy::integer_arithmetic)]

use crate::{hash::HashValue, hkdf::Hkdf, traits::Uniform as _, x25519};
use aes_gcm::{
    aead::{generic_array::GenericArray, Aead, AeadInPlace, NewAead, Payload},
    Aes256Gcm,
};
use sha2::Digest;
use std::{
    convert::TryFrom as _,
    io::{Cursor, Read as _, Write as _},
};
use thiserror::Error;

//
// Useful constants
// ----------------
//

/// A noise message cannot be larger than 65535 bytes as per the specification.
pub const MAX_SIZE_NOISE_MSG: usize = 65535;

/// The authentication tag length of AES-GCM.
pub const AES_GCM_TAGLEN: usize = 16;

/// The only Noise handshake protocol that we implement in this file.
const PROTOCOL_NAME: &[u8] = b"Noise_IK_25519_AESGCM_SHA256\0\0\0\0";

/// The nonce size we use for AES-GCM.
const AES_NONCE_SIZE: usize = 12;

/// A handy const fn to get the expanded size of a plaintext after encryption
pub const fn encrypted_len(plaintext_len: usize) -> usize {
    plaintext_len + AES_GCM_TAGLEN
}

/// A handy const fn to get the size of a plaintext from a ciphertext size
pub const fn decrypted_len(ciphertext_len: usize) -> usize {
    ciphertext_len - AES_GCM_TAGLEN
}

/// A handy const fn to get the size of the first handshake message
pub const fn handshake_init_msg_len(payload_len: usize) -> usize {
    // e
    let e_len = x25519::PUBLIC_KEY_SIZE;
    // encrypted s
    let enc_s_len = encrypted_len(x25519::PUBLIC_KEY_SIZE);
    // encrypted payload
    let enc_payload_len = encrypted_len(payload_len);
    //
    e_len + enc_s_len + enc_payload_len
}

/// A handy const fn to get the size of the second handshake message
pub const fn handshake_resp_msg_len(payload_len: usize) -> usize {
    // e
    let e_len = x25519::PUBLIC_KEY_SIZE;
    // encrypted payload
    let enc_payload_len = encrypted_len(payload_len);
    //
    e_len + enc_payload_len
}

/// This implementation relies on the fact that the hash function used has a 256-bit output
#[rustfmt::skip]
const _: [(); 32] = [(); HashValue::LENGTH];

//
// Errors
// ------
//

/// A NoiseError enum represents the different types of error that noise can return to users of the crate
#[derive(Debug, Error)]
pub enum NoiseError {
    /// the received message is too short to contain the expected data
    #[error("noise: the received message is too short to contain the expected data")]
    MsgTooShort,

    /// HKDF has failed (in practice there is no reason for HKDF to fail)
    #[error("noise: HKDF has failed")]
    Hkdf,

    /// encryption has failed (in practice there is no reason for encryption to fail)
    #[error("noise: encryption has failed")]
    Encrypt,

    /// could not decrypt the received data (most likely the data was tampered with
    #[error("noise: could not decrypt the received data")]
    Decrypt,

    /// the public key received is of the wrong format
    #[error("noise: the public key received is of the wrong format")]
    WrongPublicKeyReceived,

    /// session was closed due to decrypt error
    #[error("noise: session was closed due to decrypt error")]
    SessionClosed,

    /// the payload that we are trying to send is too large
    #[error("noise: the payload that we are trying to send is too large")]
    PayloadTooLarge,

    /// the message we received is too large
    #[error("noise: the message we received is too large")]
    ReceivedMsgTooLarge,

    /// the response buffer passed as argument is too small
    #[error("noise: the response buffer passed as argument is too small")]
    ResponseBufferTooSmall,

    /// the nonce exceeds the maximum u64 value (in practice this should not happen)
    #[error("noise: the nonce exceeds the maximum u64 value")]
    NonceOverflow,
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
    let hkdf_output = if dh_output.is_empty() {
        Hkdf::<sha2::Sha256>::extract_then_expand_no_ikm(Some(ck), None, 64)
    } else {
        Hkdf::<sha2::Sha256>::extract_then_expand(Some(ck), dh_output, None, 64)
    };

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

//
// Noise implementation
// --------------------
//

/// A key holder structure used for both initiators and responders.
#[derive(Debug)]
pub struct NoiseConfig {
    private_key: x25519::PrivateKey,
    public_key: x25519::PublicKey,
}

/// Refer to the Noise protocol framework specification in order to understand these fields.
#[cfg_attr(test, derive(Clone))]
pub struct InitiatorHandshakeState {
    /// rolling hash
    h: Vec<u8>,
    /// chaining key
    ck: Vec<u8>,
    /// ephemeral key
    e: x25519::PrivateKey,
    /// remote static key used
    rs: x25519::PublicKey,
}

/// Refer to the Noise protocol framework specification in order to understand these fields.
#[cfg_attr(test, derive(Clone))]
pub struct ResponderHandshakeState {
    /// rolling hash
    h: Vec<u8>,
    /// chaining key
    ck: Vec<u8>,
    /// remote static key received
    rs: x25519::PublicKey,
    /// remote ephemeral key receiced
    re: x25519::PublicKey,
}

impl NoiseConfig {
    /// A peer must create a NoiseConfig through this function before being able to connect with other peers.
    pub fn new(private_key: x25519::PrivateKey) -> Self {
        // we could take a public key as argument, and it would be faster, but this is cleaner
        let public_key = private_key.public_key();
        Self {
            private_key,
            public_key,
        }
    }

    /// Handy getter to access the configuration's public key
    pub fn public_key(&self) -> x25519::PublicKey {
        self.public_key
    }

    //
    // Initiator
    // ---------

    /// An initiator can use this function to initiate a handshake with a known responder.
    pub fn initiate_connection(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        prologue: &[u8],
        remote_public: x25519::PublicKey,
        payload: Option<&[u8]>,
        response_buffer: &mut [u8],
    ) -> Result<InitiatorHandshakeState, NoiseError> {
        // checks
        let payload_len = payload.map(<[u8]>::len).unwrap_or(0);
        let buffer_size_required = handshake_init_msg_len(payload_len);
        if buffer_size_required > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::PayloadTooLarge);
        }
        if response_buffer.len() < buffer_size_required {
            return Err(NoiseError::ResponseBufferTooSmall);
        }
        // initialize
        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        let rs = remote_public; // for naming consistency with the specification
        mix_hash(&mut h, &prologue);
        mix_hash(&mut h, rs.as_slice());

        // -> e
        let e = x25519::PrivateKey::generate(rng);
        let e_pub = e.public_key();

        mix_hash(&mut h, e_pub.as_slice());
        let mut response_buffer = Cursor::new(response_buffer);
        response_buffer
            .write(e_pub.as_slice())
            .map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> es
        let dh_output = e.diffie_hellman(&rs);
        let k = mix_key(&mut ck, &dh_output)?;

        // -> s
        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let msg_and_ad = Payload {
            msg: self.public_key.as_slice(),
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_static = aead
            .encrypt(nonce, msg_and_ad)
            .map_err(|_| NoiseError::Encrypt)?;

        mix_hash(&mut h, &encrypted_static);
        response_buffer
            .write(&encrypted_static)
            .map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> ss
        let dh_output = self.private_key.diffie_hellman(&rs);
        let k = mix_key(&mut ck, &dh_output)?;

        // -> payload
        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let msg_and_ad = Payload {
            msg: payload.unwrap_or_else(|| &[]),
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_payload = aead
            .encrypt(nonce, msg_and_ad)
            .map_err(|_| NoiseError::Encrypt)?;

        mix_hash(&mut h, &encrypted_payload);

        response_buffer
            .write(&encrypted_payload)
            .map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // return
        let handshake_state = InitiatorHandshakeState { h, ck, e, rs };
        Ok(handshake_state)
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
        let InitiatorHandshakeState {
            mut h,
            mut ck,
            e,
            rs,
        } = handshake_state;

        // <- e
        let mut re = [0u8; x25519::PUBLIC_KEY_SIZE];
        let mut cursor = Cursor::new(received_message);
        cursor
            .read_exact(&mut re)
            .map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &re);
        let re = x25519::PublicKey::from(re);

        // <- ee
        let dh_output = e.diffie_hellman(&re);
        mix_key(&mut ck, &dh_output)?;

        // <- se
        let dh_output = self.private_key.diffie_hellman(&re);
        let k = mix_key(&mut ck, &dh_output)?;

        // <- payload
        let offset = cursor.position() as usize;
        let received_encrypted_payload = &cursor.into_inner()[offset..];

        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: received_encrypted_payload,
            aad: &h,
        };
        let received_payload = aead
            .decrypt(nonce, ct_and_ad)
            .map_err(|_| NoiseError::Decrypt)?;

        // split
        let (k1, k2) = hkdf(&ck, None)?;
        let session = NoiseSession::new(k1, k2, rs);

        //
        Ok((received_payload, session))
    }

    //
    // Responder
    // ---------
    // There are two ways to use this API:
    // - either use `parse_client_init_message()` followed by `respond_to_client()`
    // - or use the all-in-one `respond_to_client_and_finalize()`
    //
    // the reason for the first deconstructed API is that we might want to do
    // some validation of the received initiator's public key which might
    //

    /// A responder can accept a connection by first parsing an initiator message.
    /// The function respond_to_client is usually called after this to respond to the initiator.
    pub fn parse_client_init_message(
        &self,
        prologue: &[u8],
        received_message: &[u8],
    ) -> Result<
        (
            x25519::PublicKey,       // initiator's public key
            ResponderHandshakeState, // state to be used in respond_to_client
            Vec<u8>,                 // payload received
        ),
        NoiseError,
    > {
        // checks
        if received_message.len() > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::ReceivedMsgTooLarge);
        }
        // initialize
        let mut h = PROTOCOL_NAME.to_vec();
        let mut ck = PROTOCOL_NAME.to_vec();
        mix_hash(&mut h, prologue);
        mix_hash(&mut h, self.public_key.as_slice());

        // buffer message received
        let mut cursor = Cursor::new(received_message);

        // <- e
        let mut re = [0u8; x25519::PUBLIC_KEY_SIZE];
        cursor
            .read_exact(&mut re)
            .map_err(|_| NoiseError::MsgTooShort)?;
        mix_hash(&mut h, &re);
        let re = x25519::PublicKey::from(re);

        // <- es
        let dh_output = self.private_key.diffie_hellman(&re);
        let k = mix_key(&mut ck, &dh_output)?;

        // <- s
        let mut encrypted_remote_static = [0u8; x25519::PUBLIC_KEY_SIZE + AES_GCM_TAGLEN];
        cursor
            .read_exact(&mut encrypted_remote_static)
            .map_err(|_| NoiseError::MsgTooShort)?;

        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: &encrypted_remote_static,
            aad: &h,
        };
        let rs = aead
            .decrypt(nonce, ct_and_ad)
            .map_err(|_| NoiseError::Decrypt)?;
        let rs = x25519::PublicKey::try_from(rs.as_slice())
            .map_err(|_| NoiseError::WrongPublicKeyReceived)?;
        mix_hash(&mut h, &encrypted_remote_static);

        // <- ss
        let dh_output = self.private_key.diffie_hellman(&rs);
        let k = mix_key(&mut ck, &dh_output)?;

        // <- payload
        let offset = cursor.position() as usize;
        let received_encrypted_payload = &cursor.into_inner()[offset..];

        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let ct_and_ad = Payload {
            msg: received_encrypted_payload,
            aad: &h,
        };
        let received_payload = aead
            .decrypt(nonce, ct_and_ad)
            .map_err(|_| NoiseError::Decrypt)?;
        mix_hash(&mut h, received_encrypted_payload);

        // return
        let handshake_state = ResponderHandshakeState { h, ck, rs, re };
        Ok((rs, handshake_state, received_payload))
    }

    /// A responder can respond to an initiator by calling this function with the state obtained,
    /// after calling parse_client_init_message
    pub fn respond_to_client(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        handshake_state: ResponderHandshakeState,
        payload: Option<&[u8]>,
        response_buffer: &mut [u8],
    ) -> Result<NoiseSession, NoiseError> {
        // checks
        let payload_len = payload.map(<[u8]>::len).unwrap_or(0);
        let buffer_size_required = handshake_resp_msg_len(payload_len);
        if buffer_size_required > MAX_SIZE_NOISE_MSG {
            return Err(NoiseError::PayloadTooLarge);
        }
        if response_buffer.len() < buffer_size_required {
            return Err(NoiseError::ResponseBufferTooSmall);
        }

        // retrieve handshake state
        let ResponderHandshakeState {
            mut h,
            mut ck,
            rs,
            re,
        } = handshake_state;

        // -> e
        let e = x25519::PrivateKey::generate(rng);
        let e_pub = e.public_key();

        mix_hash(&mut h, e_pub.as_slice());
        let mut response_buffer = Cursor::new(response_buffer);
        response_buffer
            .write(e_pub.as_slice())
            .map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // -> ee
        let dh_output = e.diffie_hellman(&re);
        mix_key(&mut ck, &dh_output)?;

        // -> se
        let dh_output = e.diffie_hellman(&rs);
        let k = mix_key(&mut ck, &dh_output)?;

        // -> payload
        let aead = Aes256Gcm::new(GenericArray::from_slice(&k));

        let msg_and_ad = Payload {
            msg: payload.unwrap_or_else(|| &[]),
            aad: &h,
        };
        let nonce = GenericArray::from_slice(&[0u8; AES_NONCE_SIZE]);
        let encrypted_payload = aead
            .encrypt(nonce, msg_and_ad)
            .map_err(|_| NoiseError::Encrypt)?;
        mix_hash(&mut h, &encrypted_payload);
        response_buffer
            .write(&encrypted_payload)
            .map_err(|_| NoiseError::ResponseBufferTooSmall)?;

        // split
        let (k1, k2) = hkdf(&ck, None)?;
        let session = NoiseSession::new(k2, k1, rs);

        //
        Ok(session)
    }

    /// This function is a one-call that replaces calling the two functions parse_client_init_message
    /// and respond_to_client consecutively
    pub fn respond_to_client_and_finalize(
        &self,
        rng: &mut (impl rand::RngCore + rand::CryptoRng),
        prologue: &[u8],
        received_message: &[u8],
        payload: Option<&[u8]>,
        response_buffer: &mut [u8],
    ) -> Result<
        (
            Vec<u8>,      // the payload the initiator sent
            NoiseSession, // The created session
        ),
        NoiseError,
    > {
        let (_, handshake_state, received_payload) =
            self.parse_client_init_message(prologue, received_message)?;
        let session = self.respond_to_client(rng, handshake_state, payload, response_buffer)?;
        Ok((received_payload, session))
    }
}

//
// Post-Handshake
// --------------

/// A NoiseSession is produced after a successful Noise handshake, and can be use to encrypt and decrypt messages to the other peer.
#[cfg_attr(test, derive(Clone))]
pub struct NoiseSession {
    /// a session can be marked as invalid if it has seen a decryption failure
    valid: bool,
    /// the public key of the other peer
    remote_public_key: x25519::PublicKey,
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
    fn new(write_key: Vec<u8>, read_key: Vec<u8>, remote_public_key: x25519::PublicKey) -> Self {
        Self {
            valid: true,
            remote_public_key,
            write_key,
            write_nonce: 0,
            read_key,
            read_nonce: 0,
        }
    }

    /// create a dummy session with 0 keys
    #[cfg(any(test, feature = "fuzzing"))]
    pub fn new_for_testing() -> Self {
        Self::new(
            vec![0u8; 32],
            vec![0u8; 32],
            [0u8; x25519::PUBLIC_KEY_SIZE].into(),
        )
    }

    /// obtain remote static public key
    pub fn get_remote_static(&self) -> x25519::PublicKey {
        self.remote_public_key
    }

    /// encrypts a message for the other peers (post-handshake)
    /// the function encrypts in place, and returns the authentication tag as result
    pub fn write_message_in_place(&mut self, message: &mut [u8]) -> Result<Vec<u8>, NoiseError> {
        // checks
        if !self.valid {
            return Err(NoiseError::SessionClosed);
        }
        if message.len() > MAX_SIZE_NOISE_MSG - AES_GCM_TAGLEN {
            return Err(NoiseError::PayloadTooLarge);
        }

        // encrypt in place
        let aead = Aes256Gcm::new(GenericArray::from_slice(&self.write_key));
        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.write_nonce.to_be_bytes());
        let nonce = GenericArray::from_slice(&nonce);

        let authentication_tag = aead
            .encrypt_in_place_detached(nonce, b"", message)
            .map_err(|_| NoiseError::Encrypt)?;

        // increment nonce
        self.write_nonce = self
            .write_nonce
            .checked_add(1)
            .ok_or(NoiseError::NonceOverflow)?;

        // return a subslice without the authentication tag
        Ok(authentication_tag.to_vec())
    }

    /// decrypts a message from the other peer (post-handshake)
    /// the function decrypts in place, and returns a subslice without the auth tag
    pub fn read_message_in_place<'a>(
        &mut self,
        message: &'a mut [u8],
    ) -> Result<&'a [u8], NoiseError> {
        // checks
        if !self.valid {
            return Err(NoiseError::SessionClosed);
        }
        if message.len() > MAX_SIZE_NOISE_MSG {
            self.valid = false;
            return Err(NoiseError::ReceivedMsgTooLarge);
        }
        if message.len() < AES_GCM_TAGLEN {
            self.valid = false;
            return Err(NoiseError::ResponseBufferTooSmall);
        }

        // decrypt in place
        let aead = Aes256Gcm::new(GenericArray::from_slice(&self.read_key));

        let mut nonce = [0u8; 4].to_vec();
        nonce.extend_from_slice(&self.read_nonce.to_be_bytes());
        let nonce = GenericArray::from_slice(&nonce);

        let (buffer, authentication_tag) = message.split_at_mut(message.len() - AES_GCM_TAGLEN);
        let authentication_tag = GenericArray::from_slice(authentication_tag);
        aead.decrypt_in_place_detached(nonce, b"", buffer, authentication_tag)
            .map_err(|_| {
                self.valid = false;
                NoiseError::Decrypt
            })?;

        // increment nonce
        self.read_nonce = self
            .read_nonce
            .checked_add(1)
            .ok_or(NoiseError::NonceOverflow)?;

        // return a subslice of the buffer representing the decrypted plaintext
        Ok(buffer)
    }
}

impl std::fmt::Debug for NoiseSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoiseSession[...]")
    }
}
