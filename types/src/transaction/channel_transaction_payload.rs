use serde::{Deserialize, Serialize};

use failure::prelude::*;
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher, TestOnlyHasher},
    HashValue, SigningKey, VerifyingKey,
};

use crate::transaction::script_action::ScriptAction;
use crate::{account_address::AccountAddress, transaction::Script, write_set::WriteSet};
use libra_crypto::ed25519::{Ed25519PrivateKey, Ed25519PublicKey, Ed25519Signature};

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ChannelTransactionPayloadBody {
    WriteSet(ChannelWriteSetBody),
    Script(ChannelScriptBody),
    Action(ChannelActionBody),
}

impl ChannelTransactionPayloadBody {
    // TODO: refactor this two methods, seems very old to use.
    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = match &self {
            ChannelTransactionPayloadBody::WriteSet(write_set_body) => write_set_body.hash(),
            ChannelTransactionPayloadBody::Script(script_body) => script_body.hash(),
            ChannelTransactionPayloadBody::Action(action_body) => action_body.hash(),
        };
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(self, public_key, signature)
    }
    pub fn verify(
        &self,
        public_key: &Ed25519PublicKey,
        signature: &Ed25519Signature,
    ) -> Result<()> {
        let hash = match self {
            ChannelTransactionPayloadBody::WriteSet(write_set_body) => write_set_body.hash(),
            ChannelTransactionPayloadBody::Script(script_body) => script_body.hash(),
            ChannelTransactionPayloadBody::Action(action_body) => action_body.hash(),
        };
        public_key.verify_signature(&hash, &signature)
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelTransactionPayload {
    pub body: ChannelTransactionPayloadBody,
    pub receiver_public_key: Ed25519PublicKey,
    /// signature for body.
    pub receiver_signature: Ed25519Signature,
}

impl ChannelTransactionPayload {
    pub fn new(
        body: ChannelTransactionPayloadBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body,
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn new_with_write_set(
        body: ChannelWriteSetBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body: ChannelTransactionPayloadBody::WriteSet(body),
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn new_with_script(
        body: ChannelScriptBody,
        receiver_public_key: Ed25519PublicKey,
        receiver_signature: Ed25519Signature,
    ) -> Self {
        Self {
            body: ChannelTransactionPayloadBody::Script(body),
            receiver_public_key,
            receiver_signature,
        }
    }

    pub fn receiver(&self) -> AccountAddress {
        match self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody { receiver, .. })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody { receiver, .. })
            | ChannelTransactionPayloadBody::Action(ChannelActionBody { receiver, .. }) => receiver,
        }
    }

    pub fn channel_sequence_number(&self) -> u64 {
        match self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody {
                channel_sequence_number,
                ..
            })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody {
                channel_sequence_number,
                ..
            })
            | ChannelTransactionPayloadBody::Action(ChannelActionBody {
                channel_sequence_number,
                ..
            }) => channel_sequence_number,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        match &self.body {
            ChannelTransactionPayloadBody::Script(ChannelScriptBody { write_set, .. })
            | ChannelTransactionPayloadBody::WriteSet(ChannelWriteSetBody { write_set, .. })
            | ChannelTransactionPayloadBody::Action(ChannelActionBody { write_set, .. }) => {
                write_set
            }
        }
    }
}

impl CryptoHash for ChannelTransactionPayload {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize ChannelTransactionPayload")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelWriteSetBody {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
}

impl ChannelWriteSetBody {
    pub fn new(
        channel_sequence_number: u64,
        write_set: WriteSet,
        receiver: AccountAddress,
    ) -> Self {
        Self {
            channel_sequence_number,
            write_set,
            receiver,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = self.hash();
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(
            ChannelTransactionPayloadBody::WriteSet(self),
            public_key,
            signature,
        )
    }
}

impl CryptoHash for ChannelWriteSetBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize ChannelWriteSetBody")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelScriptBody {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
    pub script: Script,
}

impl ChannelScriptBody {
    pub fn new(
        channel_sequence_number: u64,
        write_set: WriteSet,
        receiver: AccountAddress,
        script: Script,
    ) -> Self {
        Self {
            channel_sequence_number,
            write_set,
            receiver,
            script,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn script(&self) -> &Script {
        &self.script
    }

    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = self.hash();
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(
            ChannelTransactionPayloadBody::Script(self),
            public_key,
            signature,
        )
    }
}

impl CryptoHash for ChannelScriptBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize ChannelScriptBody")
                .as_slice(),
        );
        state.finish()
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ChannelActionBody {
    pub channel_sequence_number: u64,
    pub write_set: WriteSet,
    pub receiver: AccountAddress,
    pub action: ScriptAction,
}

impl ChannelActionBody {
    pub fn new(
        channel_sequence_number: u64,
        write_set: WriteSet,
        receiver: AccountAddress,
        action: ScriptAction,
    ) -> Self {
        Self {
            channel_sequence_number,
            write_set,
            receiver,
            action,
        }
    }

    pub fn write_set(&self) -> &WriteSet {
        &self.write_set
    }

    pub fn action(&self) -> &ScriptAction {
        &self.action
    }

    pub fn sign(
        self,
        private_key: &Ed25519PrivateKey,
        public_key: Ed25519PublicKey,
    ) -> ChannelTransactionPayload {
        let hash = self.hash();
        let signature = private_key.sign_message(&hash);
        ChannelTransactionPayload::new(
            ChannelTransactionPayloadBody::Action(self),
            public_key,
            signature,
        )
    }
}

impl CryptoHash for ChannelActionBody {
    //TODO use special hasher
    type Hasher = TestOnlyHasher;

    fn hash(&self) -> HashValue {
        let mut state = Self::Hasher::default();
        state.write(
            lcs::to_bytes(self)
                .expect("Failed to serialize ChannelScriptBody")
                .as_slice(),
        );
        state.finish()
    }
}
