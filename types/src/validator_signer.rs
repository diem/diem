// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::{AccountAddress, ADDRESS_LENGTH};
use crypto::{signing, HashValue, PrivateKey, PublicKey, Signature};
use failure::Error;
use proptest::{prelude::*, sample, strategy::LazyJust};
use std::convert::TryFrom;

/// ValidatorSigner associates an author with public and private keys with helpers for signing and
/// validating. This struct can be used for all signing operations including block and network
/// signing, respectively.
#[derive(Debug, Clone)]
pub struct ValidatorSigner {
    author: AccountAddress,
    public_key: PublicKey,
    private_key: PrivateKey,
}

impl ValidatorSigner {
    pub fn new(
        account_address: AccountAddress,
        public_key: PublicKey,
        private_key: PrivateKey,
    ) -> Self {
        ValidatorSigner {
            author: account_address,
            public_key,
            private_key,
        }
    }

    /// Generate the genesis block signer information.
    pub fn genesis() -> Self {
        let (private_key, public_key) = signing::generate_genesis_keypair();
        Self::new(AccountAddress::from(public_key), public_key, private_key)
    }

    /// Generate a random set of public and private keys and author information.
    pub fn random() -> Self {
        let (private_key, public_key) = signing::generate_keypair();
        ValidatorSigner {
            author: AccountAddress::from(public_key),
            public_key,
            private_key,
        }
    }

    /// For test only - makes signer with nicely looking account address that has specified integer
    /// as fist byte, and rest are zeroes
    pub fn from_int(num: u8) -> Self {
        let mut address = [0; ADDRESS_LENGTH];
        address[0] = num;
        let (private_key, public_key) = signing::generate_keypair();
        ValidatorSigner {
            author: AccountAddress::try_from(&address[..]).unwrap(),
            public_key,
            private_key,
        }
    }

    /// Constructs a signature for `message` using `private_key`.
    pub fn sign_message(&self, message: HashValue) -> Result<Signature, Error> {
        signing::sign_message(message, &self.private_key)
    }

    /// Checks that `signature` is valid for `message` using `public_key`.
    pub fn verify_message(&self, message: HashValue, signature: &Signature) -> Result<(), Error> {
        signing::verify_message(message, signature, &self.public_key)
    }

    /// Returns the author associated with this signer.
    pub fn author(&self) -> AccountAddress {
        self.author
    }

    /// Returns the public key associated with this signer.
    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }
}

#[allow(clippy::redundant_closure)]
pub fn arb_keypair() -> impl Strategy<Value = (PrivateKey, PublicKey)> {
    prop_oneof![
        // The no_shrink here reflects that particular keypair choices out
        // of random options are irrelevant.
        LazyJust::new(|| signing::generate_keypair()).no_shrink(),
        LazyJust::new(|| signing::generate_genesis_keypair()),
    ]
}

prop_compose! {
    pub fn signer_strategy(key_pair_strategy: impl Strategy<Value = (PrivateKey, PublicKey)>)(
        keypair in key_pair_strategy) -> ValidatorSigner {
        let (private_key, public_key) = keypair;
        let account_address = AccountAddress::from(public_key);
        ValidatorSigner::new(account_address, public_key, private_key)
    }
}

#[allow(clippy::redundant_closure)]
pub fn rand_signer() -> impl Strategy<Value = ValidatorSigner> {
    // random signers warrant no shrinkage.
    signer_strategy(arb_keypair()).no_shrink()
}

#[allow(clippy::redundant_closure)]
pub fn arb_signer() -> impl Strategy<Value = ValidatorSigner> {
    prop_oneof![rand_signer(), LazyJust::new(|| ValidatorSigner::genesis()),]
}

fn select_keypair(
    key_pairs: Vec<(PrivateKey, PublicKey)>,
) -> impl Strategy<Value = (PrivateKey, PublicKey)> {
    // no_shrink() => shrinking is not relevant as signers are equivalent.
    sample::select(key_pairs).no_shrink()
}

pub fn mostly_in_keypair_pool(
    key_pairs: Vec<(PrivateKey, PublicKey)>,
) -> impl Strategy<Value = ValidatorSigner> {
    prop::strategy::Union::new_weighted(vec![
        (9, signer_strategy(select_keypair(key_pairs)).boxed()),
        (1, arb_signer().boxed()),
    ])
}

#[cfg(test)]
mod tests {
    use crate::{
        account_address::AccountAddress,
        validator_signer::{arb_keypair, arb_signer, ValidatorSigner},
    };
    use crypto::HashValue;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_new_signer(keypair in arb_keypair()){
            let (private_key, public_key) = keypair;
            let signer = ValidatorSigner::new(AccountAddress::from(public_key), public_key, private_key);
            prop_assert_eq!(public_key, signer.public_key());
        }

        #[test]
        fn test_signer(signer in arb_signer(), message in HashValue::arbitrary()) {
            let signature = signer.sign_message(message).unwrap();
            prop_assert!(signer
                         .verify_message(message, &signature)
                         .is_ok());
        }
    }
}
