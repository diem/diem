use crate::write_set::WriteSet;
use anyhow::{ensure, Result};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_crypto::hash::CryptoHash;
use libra_crypto::{ed25519::Ed25519Signature, hash::HashValue, VerifyingKey};
use libra_crypto_derive::CryptoHasher;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct WitnessData {
    channel_sequence_number: u64,
    write_set: WriteSet,
}
impl WitnessData {
    pub fn new(channel_sequence_number: u64, write_set: WriteSet) -> Self {
        Self {
            channel_sequence_number,
            write_set,
        }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.channel_sequence_number
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }
}

impl CryptoHash for WitnessData {
    type Hasher = WitnessDataHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        libra_crypto::hash::CryptoHasher::write(
            &mut state,
            &lcs::to_bytes(self).expect("Serialization should work."),
        );
        libra_crypto::hash::CryptoHasher::finish(state)
    }
}

#[derive(Clone, Debug, Default, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct Witness {
    data: WitnessData,
    /// Channel participant's signatures.
    signatures: Vec<Ed25519Signature>,
}

impl Witness {
    pub fn new(data: WitnessData, signatures: Vec<Ed25519Signature>) -> Self {
        Self { data, signatures }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.data.channel_sequence_number
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.data.write_set
    }

    pub fn into_write_set(self) -> WriteSet {
        self.data.write_set
    }

    pub fn signatures(&self) -> &[Ed25519Signature] {
        self.signatures.as_slice()
    }

    pub fn verify(&self, public_keys: &[Ed25519PublicKey]) -> Result<()> {
        // only validate signature when write_set is not empty.
        if !self.data.write_set.is_empty() {
            ensure!(
                self.signatures.len() == public_keys.len(),
                "witness signature len must equals public_keys."
            );
            let hash = self.data.hash();
            for (public_key, signature) in public_keys.iter().zip(self.signatures.iter()) {
                public_key.verify_signature(&hash, signature)?;
            }
        }

        return Ok(());
    }
}
