// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crate::validator_set::ValidatorSet;
use anyhow::{ensure, Result};
use libra_crypto::*;
use mirai_annotations::*;
use std::collections::BTreeMap;
use std::fmt;
use thiserror::Error;

/// Errors possible during signature verification.
#[derive(Debug, Error, PartialEq)]
pub enum VerifyError {
    #[error("Author is unknown")]
    /// The author for this signature is unknown by this validator.
    UnknownAuthor,
    #[error(
        "The voting power ({}) is less than quorum voting power ({})",
        voting_power,
        quorum_voting_power
    )]
    TooLittleVotingPower {
        voting_power: u64,
        quorum_voting_power: u64,
    },
    #[error(
        "The number of signatures ({}) is greater than total number of authors ({})",
        num_of_signatures,
        num_of_authors
    )]
    TooManySignatures {
        num_of_signatures: usize,
        num_of_authors: usize,
    },
    #[error("Signature is invalid")]
    /// The signature does not match the hash.
    InvalidSignature,
}

/// Helper struct to manage validator information for validation
#[derive(Clone)]
pub struct ValidatorInfo<PublicKey> {
    public_key: PublicKey,
    voting_power: u64,
}

impl<PublicKey: VerifyingKey> ValidatorInfo<PublicKey> {
    pub fn new(public_key: PublicKey, voting_power: u64) -> Self {
        ValidatorInfo {
            public_key,
            voting_power,
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn voting_power(&self) -> u64 {
        self.voting_power
    }
}

/// Supports validation of signatures for known authors with individual voting powers. This struct
/// can be used for all signature verification operations including block and network signature
/// verification, respectively.
#[derive(Clone)]
pub struct ValidatorVerifier<PublicKey> {
    /// An ordered map of each validator's on-chain account address to its pubkeys
    /// and voting power.
    address_to_validator_info: BTreeMap<AccountAddress, ValidatorInfo<PublicKey>>,
    /// The minimum voting power required to achieve a quorum
    quorum_voting_power: u64,
    /// Total voting power of all validators (cached from address_to_validator_info)
    total_voting_power: u64,
}

impl<PublicKey: VerifyingKey> ValidatorVerifier<PublicKey> {
    /// Initialize with a map of account address to validator info and set quorum size to
    /// default (`2f + 1`) or zero if `address_to_validator_info` is empty.
    pub fn new(
        address_to_validator_info: BTreeMap<AccountAddress, ValidatorInfo<PublicKey>>,
    ) -> Self {
        let total_voting_power = address_to_validator_info
            .values()
            .map(|x| x.voting_power)
            .sum();
        let quorum_voting_power = if address_to_validator_info.is_empty() {
            0
        } else {
            total_voting_power * 2 / 3 + 1
        };
        ValidatorVerifier {
            address_to_validator_info,
            quorum_voting_power,
            total_voting_power,
        }
    }

    /// Initializes a validator verifier with a specified quorum voting power.
    pub fn new_with_quorum_voting_power(
        address_to_validator_info: BTreeMap<AccountAddress, ValidatorInfo<PublicKey>>,
        quorum_voting_power: u64,
    ) -> Result<Self> {
        let total_voting_power = address_to_validator_info.values().fold(0, |sum, x| {
            // The voting power of any node is assumed to be small relative to u64::max_value()
            assume!(sum <= u64::max_value() - x.voting_power);
            sum + x.voting_power
        });
        ensure!(
            quorum_voting_power <= total_voting_power,
            "Quorum voting power is greater than the sum of all voting power of authors: {}, \
             quorum_size: {}.",
            quorum_voting_power,
            total_voting_power
        );
        Ok(ValidatorVerifier {
            address_to_validator_info,
            quorum_voting_power,
            total_voting_power,
        })
    }

    /// Helper method to initialize with a single author and public key with quorum voting power 1.
    pub fn new_single(author: AccountAddress, public_key: PublicKey) -> Self {
        let mut author_to_validator_info = BTreeMap::new();
        author_to_validator_info.insert(author, ValidatorInfo::new(public_key, 1));
        Self::new(author_to_validator_info)
    }

    /// Verify the correctness of a signature of a hash by a known author.
    pub fn verify_signature(
        &self,
        author: AccountAddress,
        hash: HashValue,
        signature: &PublicKey::SignatureMaterial,
    ) -> std::result::Result<(), VerifyError> {
        match self.get_public_key(&author) {
            Some(public_key) => {
                if public_key.verify_signature(&hash, signature).is_err() {
                    Err(VerifyError::InvalidSignature)
                } else {
                    Ok(())
                }
            }
            None => Err(VerifyError::UnknownAuthor),
        }
    }

    /// This function will successfully return when at least quorum_size signatures of known authors
    /// are successfully verified. Also, an aggregated signature is considered invalid if any of the
    /// attached signatures is invalid or it does not correspond to a known author. The latter is to
    /// prevent malicious users from adding arbitrary content to the signature payload that would go
    /// unnoticed.
    pub fn verify_aggregated_signature<T>(
        &self,
        hash: HashValue,
        aggregated_signature: &BTreeMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        self.check_num_of_signatures(&aggregated_signature)?;
        self.check_voting_power(aggregated_signature.keys())?;
        for (author, signature) in aggregated_signature {
            self.verify_signature(*author, hash, &signature.clone().into())?;
        }
        Ok(())
    }

    /// This function will try batch signature verification and falls back to normal
    /// iterated verification if batching fails.
    pub fn batch_verify_aggregated_signature<T>(
        &self,
        hash: HashValue,
        aggregated_signature: &BTreeMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        self.check_num_of_signatures(aggregated_signature)?;
        self.check_voting_power(aggregated_signature.keys())?;
        let keys_and_signatures: Vec<(PublicKey, PublicKey::SignatureMaterial)> =
            aggregated_signature
                .iter()
                .flat_map(|(address, signature)| {
                    let sig: PublicKey::SignatureMaterial = signature.clone().into();
                    self.get_public_key(&address).map(|pub_key| (pub_key, sig))
                })
                .collect();
        // Fallback is required to identify the source of the problem if batching fails.
        if PublicKey::batch_verify_signatures(&hash, keys_and_signatures).is_err() {
            self.verify_aggregated_signature(hash, aggregated_signature)?
        }
        Ok(())
    }

    /// Ensure there are not more than the maximum expected signatures (all possible signatures).
    fn check_num_of_signatures<T>(
        &self,
        aggregated_signature: &BTreeMap<AccountAddress, T>,
    ) -> std::result::Result<(), VerifyError>
    where
        T: Into<PublicKey::SignatureMaterial> + Clone,
    {
        let num_of_signatures = aggregated_signature.len();
        if num_of_signatures > self.len() {
            return Err(VerifyError::TooManySignatures {
                num_of_signatures,
                num_of_authors: self.len(),
            });
        }
        Ok(())
    }
    /// Ensure there is at least quorum_voting_power in the provided signatures and there
    /// are only known authors. According to the threshold verification policy,
    /// invalid public keys are not allowed.
    pub fn check_voting_power<'a>(
        &self,
        authors: impl Iterator<Item = &'a AccountAddress>,
    ) -> std::result::Result<(), VerifyError> {
        // Add voting power for valid accounts, exiting early for unknown authors
        let mut aggregated_voting_power = 0;
        for account_address in authors {
            match self.get_voting_power(&account_address) {
                Some(voting_power) => aggregated_voting_power += voting_power,
                None => return Err(VerifyError::UnknownAuthor),
            }
        }

        if aggregated_voting_power < self.quorum_voting_power {
            return Err(VerifyError::TooLittleVotingPower {
                voting_power: aggregated_voting_power,
                quorum_voting_power: self.quorum_voting_power,
            });
        }
        Ok(())
    }

    /// Returns the public key for this address.
    pub fn get_public_key(&self, author: &AccountAddress) -> Option<PublicKey> {
        self.address_to_validator_info
            .get(&author)
            .map(|validator_info| validator_info.public_key.clone())
    }

    /// Returns the voting power for this address.
    pub fn get_voting_power(&self, author: &AccountAddress) -> Option<u64> {
        self.address_to_validator_info
            .get(&author)
            .map(|validator_info| validator_info.voting_power)
    }

    /// Returns an ordered list of account addresses as an `Iterator`.
    pub fn get_ordered_account_addresses_iter(&self) -> impl Iterator<Item = AccountAddress> + '_ {
        // Since `address_to_validator_info` is a `BTreeMap`, the `.keys()` iterator
        // is guaranteed to be sorted.
        self.address_to_validator_info.keys().copied()
    }

    /// Returns the number of authors to be validated.
    pub fn len(&self) -> usize {
        self.address_to_validator_info.len()
    }

    /// Is there at least one author?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns quorum voting power.
    pub fn quorum_voting_power(&self) -> u64 {
        self.quorum_voting_power
    }
}

impl<PublicKey> fmt::Display for ValidatorVerifier<PublicKey> {
    fn fmt(&self, f: &mut fmt::Formatter) -> std::fmt::Result {
        write!(f, "ValidatorSet: [")?;
        for (addr, info) in &self.address_to_validator_info {
            write!(f, "{}: {}, ", addr.short_str(), info.voting_power)?;
        }
        write!(f, "]")
    }
}

impl<PublicKey: VerifyingKey> From<&ValidatorSet<PublicKey>> for ValidatorVerifier<PublicKey> {
    fn from(validator_set: &ValidatorSet<PublicKey>) -> Self {
        ValidatorVerifier::new(validator_set.iter().fold(BTreeMap::new(), |mut map, key| {
            map.insert(
                key.account_address().clone(),
                ValidatorInfo::new(
                    key.consensus_public_key().clone(),
                    key.consensus_voting_power(),
                ),
            );
            map
        }))
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl<PublicKey: VerifyingKey> From<&ValidatorVerifier<PublicKey>> for ValidatorSet<PublicKey> {
    fn from(verifier: &ValidatorVerifier<PublicKey>) -> Self {
        use crate::validator_public_keys::ValidatorPublicKeys;
        ValidatorSet::new(
            verifier
                .get_ordered_account_addresses_iter()
                .map(|addr| {
                    ValidatorPublicKeys::new_with_random_network_keys(
                        addr,
                        verifier.get_public_key(&addr).unwrap(),
                        verifier.get_voting_power(&addr).unwrap(),
                    )
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto_proxies::random_validator_verifier;
    use crate::validator_verifier::VerifyError::TooLittleVotingPower;
    use crate::{
        validator_signer::ValidatorSigner,
        validator_verifier::{ValidatorInfo, ValidatorVerifier, VerifyError},
    };
    use libra_crypto::{ed25519::*, test_utils::TEST_SEED, HashValue};
    use std::collections::BTreeMap;

    #[test]
    fn test_check_voting_power() {
        let (validator_signers, validator_verifier) = random_validator_verifier(2, None, false);
        let mut author_to_signature_map = BTreeMap::new();

        assert_eq!(
            validator_verifier
                .check_voting_power(author_to_signature_map.keys())
                .unwrap_err(),
            TooLittleVotingPower {
                voting_power: 0,
                quorum_voting_power: 2,
            }
        );

        let random_hash = HashValue::random();
        for validator in validator_signers.iter() {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }

        assert_eq!(
            validator_verifier.check_voting_power(author_to_signature_map.keys()),
            Ok(())
        );
    }

    #[test]
    fn test_validator() {
        let validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random(TEST_SEED);
        let random_hash = HashValue::random();
        let signature = validator_signer.sign_message(random_hash).unwrap();
        let validator =
            ValidatorVerifier::new_single(validator_signer.author(), validator_signer.public_key());
        assert_eq!(
            validator.verify_signature(validator_signer.author(), random_hash, &signature),
            Ok(())
        );
        let unknown_validator_signer = ValidatorSigner::<Ed25519PrivateKey>::random([1; 32]);
        let unknown_signature = unknown_validator_signer.sign_message(random_hash).unwrap();
        assert_eq!(
            validator.verify_signature(
                unknown_validator_signer.author(),
                random_hash,
                &unknown_signature
            ),
            Err(VerifyError::UnknownAuthor)
        );
        assert_eq!(
            validator.verify_signature(validator_signer.author(), random_hash, &unknown_signature),
            Err(VerifyError::InvalidSignature)
        );
    }

    #[test]
    fn test_equal_vote_quorum_validators() {
        const NUM_SIGNERS: u8 = 7;
        // Generate NUM_SIGNERS random signers.
        let validator_signers: Vec<ValidatorSigner<Ed25519PrivateKey>> = (0..NUM_SIGNERS)
            .map(|i| ValidatorSigner::random([i; 32]))
            .collect();
        let random_hash = HashValue::random();

        // Create a map from authors to public keys with equal voting power.
        let mut author_to_public_key_map = BTreeMap::new();
        for validator in validator_signers.iter() {
            author_to_public_key_map.insert(
                validator.author(),
                ValidatorInfo::new(validator.public_key(), 1),
            );
        }

        // Create a map from author to signatures.
        let mut author_to_signature_map = BTreeMap::new();
        for validator in validator_signers.iter() {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }

        // Let's assume our verifier needs to satisfy at least 5 signatures from the original
        // NUM_SIGNERS.
        let validator_verifier =
            ValidatorVerifier::<Ed25519PublicKey>::new_with_quorum_voting_power(
                author_to_public_key_map,
                5,
            )
            .expect("Incorrect quorum size.");

        // Check against signatures == N; this will pass.
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an extra unknown signer, signatures > N; this will fail.
        let unknown_validator_signer =
            ValidatorSigner::<Ed25519PrivateKey>::random([NUM_SIGNERS + 1; 32]);
        let unknown_signature = unknown_validator_signer.sign_message(random_hash).unwrap();
        author_to_signature_map
            .insert(unknown_validator_signer.author(), unknown_signature.clone());
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::TooManySignatures {
                num_of_signatures: 8,
                num_of_authors: 7
            })
        );

        // Add 5 valid signers only (quorum threshold is met); this will pass.
        author_to_signature_map.clear();
        for validator in validator_signers.iter().take(5) {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an unknown signer, but quorum is satisfied and signatures <= N; this will fail as we
        // don't tolerate invalid signatures.
        author_to_signature_map
            .insert(unknown_validator_signer.author(), unknown_signature.clone());
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::UnknownAuthor)
        );

        // Add 4 valid signers only (quorum threshold is NOT met); this will fail.
        author_to_signature_map.clear();
        for validator in validator_signers.iter().take(4) {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::TooLittleVotingPower {
                voting_power: 4,
                quorum_voting_power: 5
            })
        );

        // Add an unknown signer, we have 5 signers, but one of them is invalid; this will fail.
        author_to_signature_map.insert(unknown_validator_signer.author(), unknown_signature);
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::UnknownAuthor)
        );
    }

    #[test]
    fn test_unequal_vote_quorum_validators() {
        const NUM_SIGNERS: u8 = 4;
        // Generate NUM_SIGNERS random signers.
        let validator_signers: Vec<ValidatorSigner<Ed25519PrivateKey>> = (0..NUM_SIGNERS)
            .map(|i| ValidatorSigner::random([i; 32]))
            .collect();
        let random_hash = HashValue::random();

        // Create a map from authors to public keys with increasing weights (0, 1, 2, 3) and
        // a map of author to signature.
        let mut author_to_public_key_map = BTreeMap::new();
        let mut author_to_signature_map = BTreeMap::new();
        for (i, validator_signer) in validator_signers.iter().enumerate() {
            author_to_public_key_map.insert(
                validator_signer.author(),
                ValidatorInfo::new(validator_signer.public_key(), i as u64),
            );
            author_to_signature_map.insert(
                validator_signer.author(),
                validator_signer.sign_message(random_hash).unwrap(),
            );
        }

        // Let's assume our verifier needs to satisfy at least 5 quorum voting power
        let validator_verifier =
            ValidatorVerifier::<Ed25519PublicKey>::new_with_quorum_voting_power(
                author_to_public_key_map,
                5,
            )
            .expect("Incorrect quorum size.");

        // Check against all signatures (6 voting power); this will pass.
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an extra unknown signer, signatures > N; this will fail.
        let unknown_validator_signer =
            ValidatorSigner::<Ed25519PrivateKey>::random([NUM_SIGNERS + 1; 32]);
        let unknown_signature = unknown_validator_signer.sign_message(random_hash).unwrap();
        author_to_signature_map
            .insert(unknown_validator_signer.author(), unknown_signature.clone());
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::TooManySignatures {
                num_of_signatures: 5,
                num_of_authors: 4
            })
        );

        // Add 5 voting power signers only (quorum threshold is met) with (2, 3) ; this will pass.
        author_to_signature_map.clear();
        for validator in validator_signers.iter().skip(2) {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an unknown signer, but quorum is satisfied and signatures <= N; this will fail as we
        // don't tolerate invalid signatures.
        author_to_signature_map
            .insert(unknown_validator_signer.author(), unknown_signature.clone());
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::UnknownAuthor)
        );

        // Add first 3 valid signers only (quorum threshold is NOT met); this will fail.
        author_to_signature_map.clear();
        for validator in validator_signers.iter().take(3) {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::TooLittleVotingPower {
                voting_power: 3,
                quorum_voting_power: 5
            })
        );

        // Add an unknown signer, we have 5 signers, but one of them is invalid; this will fail.
        author_to_signature_map.insert(unknown_validator_signer.author(), unknown_signature);
        assert_eq!(
            validator_verifier
                .batch_verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::UnknownAuthor)
        );
    }
}
