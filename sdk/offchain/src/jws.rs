// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use diem_sdk::crypto::ed25519::ed25519_dalek::{self, Signature, Signer, Verifier};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{convert::TryFrom, str};

#[derive(Deserialize, Serialize)]
enum JwsAlgorithm {
    EdDSA,
}

#[derive(Deserialize, Serialize)]
struct Header {
    alg: JwsAlgorithm,
}

pub fn serialize<T: Serialize, S: Signer<Signature>>(
    t: &T,
    signer: &S,
) -> Result<String, JwsError> {
    let mut msg = String::new();
    base64::encode_config_buf(
        serde_json::to_string(&Header {
            alg: JwsAlgorithm::EdDSA,
        })
        .map_err(JwsError::json)?,
        base64::URL_SAFE,
        &mut msg,
    );
    msg.push('.');
    base64::encode_config_buf(
        serde_json::to_string(t).map_err(JwsError::json)?,
        base64::URL_SAFE,
        &mut msg,
    );

    // Sign msg
    let signature = signer
        .try_sign(msg.as_bytes())
        .map_err(JwsError::signature)?;
    msg.push('.');
    base64::encode_config_buf(signature, base64::URL_SAFE, &mut msg);

    Ok(msg)
}

pub fn deserialize<T: DeserializeOwned, V: Verifier<Signature>>(
    msg: &[u8],
    verifier: &V,
) -> Result<T, JwsError> {
    let body = deserialize_payload(msg, verifier)?;
    serde_json::from_slice(&body).map_err(JwsError::json)
}

fn deserialize_payload<V: Verifier<Signature>>(
    msg: &[u8],
    verifier: &V,
) -> Result<Vec<u8>, JwsError> {
    let msg = str::from_utf8(msg).map_err(JwsError::jws)?;
    let (signing_msg, signature) = rsplit_at_period(msg)?;
    let (header, body) = rsplit_at_period(signing_msg)?;

    // Verify that the header only is '{ "alg": "EdDSA" }'
    let header = base64::decode_config(header, base64::URL_SAFE).map_err(JwsError::jws)?;
    serde_json::from_slice::<Header>(&header).map_err(JwsError::json)?;

    // Verify Signature
    let signature = Signature::try_from(
        base64::decode_config(signature, base64::URL_SAFE)
            .map_err(JwsError::jws)?
            .as_slice(),
    )
    .map_err(JwsError::signature)?;
    verifier
        .verify(signing_msg.as_bytes(), &signature)
        .map_err(JwsError::signature)?;

    base64::decode_config(body, base64::URL_SAFE).map_err(JwsError::jws)
}

fn rsplit_at_period(msg: &str) -> Result<(&str, &str), JwsError> {
    let index = msg
        .rfind('.')
        .ok_or_else(|| JwsError::jws("missing section separator '.'"))?;
    let (a, b) = msg.split_at(index);

    Ok((a, &b[1..]))
}

type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
pub struct JwsError {
    inner: Box<InnerError>,
}

#[derive(Debug)]
struct InnerError {
    kind: ErrorKind,
    source: Option<BoxError>,
}

#[derive(PartialEq, Eq, Debug)]
enum ErrorKind {
    JwsCompact,
    Json,
    Signature,
}

impl JwsError {
    //
    // Constructors
    //
    fn new<E: Into<BoxError>>(kind: ErrorKind, source: Option<E>) -> Self {
        Self {
            inner: Box::new(InnerError {
                kind,
                source: source.map(Into::into),
            }),
        }
    }

    fn jws<E: Into<BoxError>>(e: E) -> Self {
        Self::new(ErrorKind::JwsCompact, Some(e))
    }

    fn json(e: serde_json::Error) -> Self {
        Self::new(ErrorKind::Json, Some(e))
    }

    fn signature(e: ed25519_dalek::SignatureError) -> Self {
        Self::new(ErrorKind::Signature, Some(e))
    }
}

impl std::fmt::Display for JwsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for JwsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source.as_ref().map(|e| &**e as _)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{CommandResponseObject, CommandStatus};
    use diem_sdk::crypto::ed25519::ed25519_dalek::{Keypair, PublicKey};

    #[test]
    fn deserialize_example() {
        let example = "eyJhbGciOiJFZERTQSJ9.U2FtcGxlIHNpZ25lZCBwYXlsb2FkLg.dZvbycl2Jkl3H7NmQzL6P0_lDEW42s9FrZ8z-hXkLqYyxNq8yOlDjlP9wh3wyop5MU2sIOYvay-laBmpdW6OBQ";
        let public_key = PublicKey::from_bytes(
            &hex::decode("bd47e3e7afb94debbd82e10ab7d410a885b589db49138628562ac2ec85726129")
                .unwrap(),
        )
        .unwrap();

        let payload = deserialize_payload(example.as_bytes(), &public_key).unwrap();

        assert_eq!(payload, b"Sample signed payload.");
    }

    #[test]
    fn serialize_deserialize_round_trip() {
        let keypair = Keypair::generate(&mut rand_core::OsRng);
        let expected = CommandResponseObject::new(CommandStatus::Failure);

        let s = serialize(&expected, &keypair).unwrap();
        let actual: CommandResponseObject = deserialize(s.as_bytes(), &keypair).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn invalid_signature_error() {
        let keypair = Keypair::generate(&mut rand_core::OsRng);
        let expected = CommandResponseObject::new(CommandStatus::Failure);

        let s = serialize(&expected, &keypair).unwrap();

        let keypair2 = Keypair::generate(&mut rand_core::OsRng);
        let error = deserialize::<CommandResponseObject, _>(s.as_bytes(), &keypair2).unwrap_err();
        assert_eq!(error.inner.kind, ErrorKind::Signature);
    }
}
