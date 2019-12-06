use serde::{Deserialize, Serialize};

use anyhow::Result;
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher, TestOnlyHasher},
    HashValue, VerifyingKey,
};

use crate::account_address::AccountAddress;
use crate::channel::Witness;
use crate::transaction::helpers::ChannelTransactionSigner;
use crate::transaction::script_action::ScriptAction;
use libra_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelTransactionPayloadBody {
    channel_address: AccountAddress,
    proposer: AccountAddress,
    witness: Witness,
    action: ScriptAction,
}

impl ChannelTransactionPayloadBody {
    pub fn new(
        channel_address: AccountAddress,
        proposer: AccountAddress,
        action: ScriptAction,
        witness: Witness,
    ) -> Self {
        Self {
            channel_address,
            proposer,
            action,
            witness,
        }
    }

    pub fn witness(&self) -> &Witness {
        &self.witness
    }
}

impl CryptoHash for ChannelTransactionPayloadBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize ChannelTransactionPayloadBody")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelTransactionPayload {
    body: ChannelTransactionPayloadBody,
    signatures: Vec<Option<Ed25519Signature>>,
    public_keys: Vec<Ed25519PublicKey>,
}

impl ChannelTransactionPayload {
    pub fn new(
        body: ChannelTransactionPayloadBody,
        public_keys: Vec<Ed25519PublicKey>,
        signatures: Vec<Option<Ed25519Signature>>,
    ) -> Self {
        Self {
            body,
            signatures,
            public_keys,
        }
    }

    /// Verify Witness and body's signature.
    pub fn verify(&self) -> Result<()> {
        self.body.witness.verify(self.public_keys())?;
        let hash = self.body.hash();
        for (public_key, signature) in self.public_keys.iter().zip(self.signatures.iter()) {
            match signature {
                Some(signature) => {
                    public_key.verify_signature(&hash, signature)?;
                }
                None => {}
            }
        }
        Ok(())
    }

    pub fn set_signature(&mut self, idx: usize, signature: Ed25519Signature) {
        self.signatures.insert(idx, Some(signature));
    }

    pub fn sign(&mut self, signer: Box<dyn ChannelTransactionSigner>) {
        let (idx, signature) = signer.sign(&self.body);
        self.set_signature(idx, signature);
    }

    pub fn channel_address(&self) -> AccountAddress {
        self.body.channel_address
    }

    pub fn proposer(&self) -> AccountAddress {
        self.body.proposer
    }

    pub fn witness(&self) -> &Witness {
        &self.body.witness
    }

    pub fn action(&self) -> &ScriptAction {
        &self.body.action
    }

    pub fn signatures(&self) -> &[Option<Ed25519Signature>] {
        self.signatures.as_slice()
    }

    pub fn public_keys(&self) -> &[Ed25519PublicKey] {
        self.public_keys.as_slice()
    }

    pub fn channel_sequence_number(&self) -> u64 {
        self.body.witness.channel_sequence_number()
    }

    pub fn is_authorized(&self) -> bool {
        let mut result = true;
        for signature in &self.signatures {
            match signature {
                Some(_signature) => {}
                None => result = false,
            }
        }
        return result;
    }
}
