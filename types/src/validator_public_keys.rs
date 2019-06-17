// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

use crate::{
    account_address::AccountAddress,
    proto::validator_public_keys::ValidatorPublicKeys as ProtoValidatorPublicKeys,
};
use canonical_serialization::{
    CanonicalDeserialize, CanonicalDeserializer, CanonicalSerialize, CanonicalSerializer,
};
use crypto::{x25519::X25519PublicKey, PublicKey};
use failure::Result;
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};

/// After executing a special transaction that sets the validators that should be used for the
/// next epoch, consensus and networking get the new list of validators.  Consensus will have a
/// public key to validate signed messages and networking will have a TBD public key for
/// creating secure channels of communication between validators.  The validators and their
/// public keys may or may not change between epochs.
#[derive(Arbitrary, Clone, Debug, Eq, PartialEq)]
pub struct ValidatorPublicKeys {
    // Hash value of the current public key of the account address
    account_address: AccountAddress,
    // This key can validate messages sent from this validator
    consensus_public_key: PublicKey,
    // This key can validate signed messages at the network layer
    network_signing_public_key: PublicKey,
    // This key establishes the corresponding PrivateKey holder's eligibility to join the p2p
    // network
    network_identity_public_key: X25519PublicKey,
}

impl ValidatorPublicKeys {
    pub fn new(
        account_address: AccountAddress,
        consensus_public_key: PublicKey,
        network_signing_public_key: PublicKey,
        network_identity_public_key: X25519PublicKey,
    ) -> Self {
        ValidatorPublicKeys {
            account_address,
            consensus_public_key,
            network_signing_public_key,
            network_identity_public_key,
        }
    }

    /// Returns the id of this validator (hash of the current public key of the
    /// validator associated account address)
    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    /// Returns the key for validating signed messages from this validator
    pub fn consensus_public_key(&self) -> &PublicKey {
        &self.consensus_public_key
    }

    /// Returns the key for validating signed messages at the network layers
    pub fn network_signing_public_key(&self) -> &PublicKey {
        &self.network_signing_public_key
    }

    /// Returns the key that establishes a validator's identity in the p2p network
    pub fn network_identity_public_key(&self) -> &X25519PublicKey {
        &self.network_identity_public_key
    }
}

impl FromProto for ValidatorPublicKeys {
    type ProtoType = ProtoValidatorPublicKeys;

    fn from_proto(object: Self::ProtoType) -> Result<Self> {
        let account_address = AccountAddress::from_proto(object.get_account_address().to_vec())?;
        let consensus_public_key = PublicKey::from_slice(object.get_consensus_public_key())?;
        let network_signing_public_key =
            PublicKey::from_slice(object.get_network_signing_public_key())?;
        let network_identity_public_key =
            X25519PublicKey::from_slice(object.get_network_identity_public_key())?;
        Ok(Self::new(
            account_address,
            consensus_public_key,
            network_signing_public_key,
            network_identity_public_key,
        ))
    }
}

impl IntoProto for ValidatorPublicKeys {
    type ProtoType = ProtoValidatorPublicKeys;

    fn into_proto(self) -> Self::ProtoType {
        let mut proto = Self::ProtoType::new();
        proto.set_account_address(AccountAddress::into_proto(self.account_address));
        proto.set_consensus_public_key(PublicKey::to_slice(&self.consensus_public_key).to_vec());
        proto.set_network_signing_public_key(
            PublicKey::to_slice(&self.network_signing_public_key).to_vec(),
        );
        proto.set_network_identity_public_key(
            X25519PublicKey::to_slice(&self.network_identity_public_key).to_vec(),
        );
        proto
    }
}

impl CanonicalSerialize for ValidatorPublicKeys {
    fn serialize(&self, serializer: &mut impl CanonicalSerializer) -> Result<()> {
        serializer
            .encode_struct(&self.account_address)?
            .encode_variable_length_bytes(&self.consensus_public_key.to_slice())?
            .encode_variable_length_bytes(&self.network_identity_public_key.to_slice())?
            .encode_variable_length_bytes(&self.network_signing_public_key.to_slice())?;
        Ok(())
    }
}

impl CanonicalDeserialize for ValidatorPublicKeys {
    fn deserialize(deserializer: &mut impl CanonicalDeserializer) -> Result<Self> {
        let account_address = deserializer.decode_struct::<AccountAddress>()?;
        let concensus_public_key =
            PublicKey::from_slice(&deserializer.decode_variable_length_bytes()?)?;
        let network_identity_public_key =
            X25519PublicKey::from_slice(&deserializer.decode_variable_length_bytes()?)?;
        let network_signing_public_key =
            PublicKey::from_slice(&deserializer.decode_variable_length_bytes()?)?;
        Ok(ValidatorPublicKeys::new(
            account_address,
            concensus_public_key,
            network_signing_public_key,
            network_identity_public_key,
        ))
    }
}
