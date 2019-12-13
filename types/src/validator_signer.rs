// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::{AccountAddress, ADDRESS_LENGTH};
use anyhow::Error;
use libra_crypto::{test_utils::TEST_SEED, HashValue, *};
use rand::{rngs::StdRng, SeedableRng};
use std::convert::TryFrom;

/// ValidatorSigner associates an author with public and private keys with helpers for signing and
/// validating. This struct can be used for all signing operations including block and network
/// signing, respectively.
#[derive(Debug)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Clone))]
pub struct ValidatorSigner<PrivateKey: SigningKey> {
    author: AccountAddress,
    public_key: PrivateKey::VerifyingKeyMaterial,
    private_key: PrivateKey,
}

impl<PrivateKey: SigningKey> ValidatorSigner<PrivateKey> {
    pub fn new(
        opt_account_address: impl Into<Option<AccountAddress>>,
        private_key: PrivateKey,
    ) -> Self {
        let public_key: PrivateKey::VerifyingKeyMaterial = private_key.public_key();

        let account_address = opt_account_address
            .into()
            .unwrap_or_else(|| AccountAddress::from_public_key(&public_key));
        ValidatorSigner {
            author: account_address,
            public_key,
            private_key,
        }
    }

    /// Constructs a signature for `message` using `private_key`.
    pub fn sign_message(&self, message: HashValue) -> Result<PrivateKey::SignatureMaterial, Error> {
        Ok(self.private_key.sign_message(&message))
    }

    /// Returns the author associated with this signer.
    pub fn author(&self) -> AccountAddress {
        self.author
    }

    /// Returns the public key associated with this signer.
    pub fn public_key(&self) -> PrivateKey::VerifyingKeyMaterial {
        self.public_key.clone()
    }
}

impl<PrivateKey: SigningKey + Genesis> ValidatorSigner<PrivateKey> {
    /// Generate the genesis block signer information.
    pub fn genesis() -> Self {
        let genesis_key = PrivateKey::genesis();
        Self::new(None, genesis_key)
    }
}

impl<PrivateKey: SigningKey + Uniform> ValidatorSigner<PrivateKey> {
    /// Generate a random set of public and private keys and author
    /// information.
    /// This takes an optional seed, which it initializes to
    /// `test_utils::TEST_SEED` if passed `None`
    pub fn random(opt_rng_seed: impl for<'a> Into<Option<[u8; 32]>>) -> Self {
        let mut rng = StdRng::from_seed(opt_rng_seed.into().unwrap_or(TEST_SEED));
        let private_key = PrivateKey::generate_for_testing(&mut rng);
        Self::new(None, private_key)
    }

    /// For test only - makes signer with nicely looking account address that has specified integer
    /// as fist byte, and rest are zeroes
    pub fn from_int(num: u8) -> Self {
        let mut address = [0; ADDRESS_LENGTH];
        address[0] = num;
        let mut rng = StdRng::from_seed(TEST_SEED);
        let private_key = PrivateKey::generate_for_testing(&mut rng);
        Self::new(AccountAddress::try_from(&address[..]).unwrap(), private_key)
    }
}

#[cfg(any(test, feature = "fuzzing"))]
pub mod proptests {
    use super::*;
    #[cfg(test)]
    use libra_crypto::ed25519::*;
    use proptest::{prelude::*, sample, strategy::LazyJust};

    #[allow(clippy::redundant_closure)]
    pub fn arb_signing_key<PrivateKey: SigningKey + Uniform + Genesis + 'static>(
    ) -> impl Strategy<Value = PrivateKey> {
        prop_oneof![
            // The no_shrink here reflects that particular keypair choices out
            // of random options are irrelevant.
            LazyJust::new(|| PrivateKey::generate_for_testing(&mut StdRng::from_seed(TEST_SEED))),
            LazyJust::new(|| PrivateKey::genesis()),
        ]
    }

    pub fn signer_strategy<PrivateKey: SigningKey + Uniform + Genesis>(
        signing_key_strategy: impl Strategy<Value = PrivateKey>,
    ) -> impl Strategy<Value = ValidatorSigner<PrivateKey>> {
        signing_key_strategy.prop_map(|signing_key| ValidatorSigner::new(None, signing_key))
    }

    #[allow(clippy::redundant_closure)]
    pub fn rand_signer<PrivateKey: SigningKey + Uniform + Genesis + 'static>(
    ) -> impl Strategy<Value = ValidatorSigner<PrivateKey>> {
        signer_strategy(arb_signing_key())
    }

    #[allow(clippy::redundant_closure)]
    pub fn arb_signer<PrivateKey: SigningKey + Uniform + Genesis + 'static>(
    ) -> impl Strategy<Value = ValidatorSigner<PrivateKey>> {
        prop_oneof![rand_signer(), LazyJust::new(|| ValidatorSigner::genesis()),]
    }

    fn select_keypair<PrivateKey: SigningKey + Uniform + Genesis + Clone + 'static>(
        keys: Vec<PrivateKey>,
    ) -> impl Strategy<Value = PrivateKey> {
        sample::select(keys)
    }

    pub fn mostly_in_keypair_pool<PrivateKey: SigningKey + Uniform + Genesis + Clone + 'static>(
        keys: Vec<PrivateKey>,
    ) -> impl Strategy<Value = ValidatorSigner<PrivateKey>> {
        prop::strategy::Union::new_weighted(vec![
            (9, signer_strategy(select_keypair(keys)).boxed()),
            (1, arb_signer().boxed()),
        ])
    }

    proptest! {
        #[test]
        fn test_new_signer(signing_key in arb_signing_key::<Ed25519PrivateKey>()){
            let public_key = signing_key.public_key();
            let signer = ValidatorSigner::new(None, signing_key);
            prop_assert_eq!(public_key, signer.public_key());
        }

    }
}
