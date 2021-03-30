// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    client::AccountAddress,
    crypto::{
        ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
        traits::Uniform,
    },
    transaction_builder::TransactionBuilder,
    types::transaction::{authenticator::AuthenticationKey, RawTransaction, SignedTransaction},
};

pub use diem_types::*;

pub struct LocalAccount {
    /// Address of the account.
    address: AccountAddress,
    /// Authentication key of the account.
    key_ring: KeyRing,
    /// Latest known sequence number of the account, it can be different from validator.
    sequence_number: u64,
}

impl LocalAccount {
    pub fn new<T: Into<KeyRing>>(address: AccountAddress, key_ring: T) -> Self {
        Self {
            address,
            key_ring: key_ring.into(),
            sequence_number: 0,
        }
    }

    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand_core::RngCore + ::rand_core::CryptoRng,
    {
        let key_ring = KeyRing::generate(rng);
        let address = key_ring.authentication_key().derived_address();

        Self::new(address, key_ring)
    }

    pub fn sign_transaction(&self, txn: RawTransaction) -> SignedTransaction {
        txn.sign(self.private_key(), self.public_key().clone())
            .expect("Signing a txn can't fail")
            .into_inner()
    }

    pub fn sign_with_transaction_builder(
        &mut self,
        builder: TransactionBuilder,
    ) -> SignedTransaction {
        let raw_txn = builder
            .sender(self.address())
            .sequence_number(self.sequence_number())
            .build();
        *self.sequence_number_mut() += 1;
        self.sign_transaction(raw_txn)
    }

    pub fn address(&self) -> AccountAddress {
        self.address
    }

    pub fn private_key(&self) -> &Ed25519PrivateKey {
        self.key_ring.private_key()
    }

    pub fn public_key(&self) -> &Ed25519PublicKey {
        self.key_ring.public_key()
    }

    pub fn authentication_key(&self) -> AuthenticationKey {
        self.key_ring.authentication_key()
    }

    pub fn sequence_number(&self) -> u64 {
        self.sequence_number
    }

    pub fn sequence_number_mut(&mut self) -> &mut u64 {
        &mut self.sequence_number
    }

    pub fn rotate_key<T: Into<KeyRing>>(&mut self, new_key: T) -> KeyRing {
        std::mem::replace(&mut self.key_ring, new_key.into())
    }
}

pub struct KeyRing {
    private_key: Ed25519PrivateKey,
    public_key: Ed25519PublicKey,
    authentication_key: AuthenticationKey,
}

impl KeyRing {
    pub fn generate<R>(rng: &mut R) -> Self
    where
        R: ::rand_core::RngCore + ::rand_core::CryptoRng,
    {
        let private_key = Ed25519PrivateKey::generate(rng);
        Self::from_private_key(private_key)
    }

    pub fn from_private_key(private_key: Ed25519PrivateKey) -> Self {
        let public_key = Ed25519PublicKey::from(&private_key);
        let authentication_key = AuthenticationKey::ed25519(&public_key);

        Self {
            private_key,
            public_key,
            authentication_key,
        }
    }

    pub fn private_key(&self) -> &Ed25519PrivateKey {
        &self.private_key
    }

    pub fn public_key(&self) -> &Ed25519PublicKey {
        &self.public_key
    }

    pub fn authentication_key(&self) -> AuthenticationKey {
        self.authentication_key
    }
}

impl From<Ed25519PrivateKey> for KeyRing {
    fn from(private_key: Ed25519PrivateKey) -> Self {
        Self::from_private_key(private_key)
    }
}
