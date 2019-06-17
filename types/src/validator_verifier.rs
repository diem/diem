// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::account_address::AccountAddress;
use crypto::{signing, HashValue, PublicKey, Signature};
use failure::Fail;
use std::collections::HashMap;

/// Errors possible during signature verification.
#[derive(Debug, Fail, PartialEq)]
pub enum VerifyError {
    #[fail(display = "Author is unknown")]
    /// The author for this signature is unknown by this validator.
    UnknownAuthor,
    #[fail(
        display = "The number of signatures ({}) is smaller than quorum size ({})",
        num_of_signatures, quorum_size
    )]
    TooFewSignatures {
        num_of_signatures: usize,
        quorum_size: usize,
    },
    #[fail(
        display = "The number of signatures ({}) is greater than total number of authors ({})",
        num_of_signatures, num_of_authors
    )]
    TooManySignatures {
        num_of_signatures: usize,
        num_of_authors: usize,
    },
    #[fail(display = "Signature is invalid")]
    /// The signature does not match the hash.
    InvalidSignature,
}

/// Supports validation of signatures for known authors. This struct can be used for all signature
/// verification operations including block and network signature verification, respectively.
#[derive(Clone)]
pub struct ValidatorVerifier {
    author_to_public_keys: HashMap<AccountAddress, PublicKey>,
    quorum_size: usize,
}

impl ValidatorVerifier {
    /// Initialize with a map of author to public key.
    pub fn new(
        author_to_public_keys: HashMap<AccountAddress, PublicKey>,
        quorum_size: usize,
    ) -> Self {
        ValidatorVerifier {
            author_to_public_keys,
            quorum_size,
        }
    }

    /// Helper method to initialize with a single author and public key.
    pub fn new_single(author: AccountAddress, public_key: PublicKey) -> Self {
        let mut author_to_public_keys = HashMap::new();
        author_to_public_keys.insert(author, public_key);
        ValidatorVerifier {
            author_to_public_keys,
            quorum_size: 1,
        }
    }

    /// Helper method to initialize with an empty validator set.
    pub fn new_empty() -> Self {
        ValidatorVerifier {
            author_to_public_keys: HashMap::new(),
            quorum_size: 0,
        }
    }

    /// Verify the correctness of a signature of a hash by a known author.
    pub fn verify_signature(
        &self,
        author: AccountAddress,
        hash: HashValue,
        signature: &Signature,
    ) -> Result<(), VerifyError> {
        let public_key = self.author_to_public_keys.get(&author);
        match public_key {
            None => Err(VerifyError::UnknownAuthor),
            Some(public_key) => {
                if signing::verify_message(hash, signature, public_key).is_err() {
                    Err(VerifyError::InvalidSignature)
                } else {
                    Ok(())
                }
            }
        }
    }

    /// This function will successfully return when at least quorum_size signatures of known authors
    /// are successfully verified. Also, an aggregated signature is considered invalid if any of the
    /// attached signatures is invalid or it does not correspond to a known author. The latter is to
    /// prevent malicious users from adding arbitrary content to the signature payload that would go
    /// unnoticed.
    pub fn verify_aggregated_signature(
        &self,
        hash: HashValue,
        aggregated_signature: &HashMap<AccountAddress, Signature>,
    ) -> Result<(), VerifyError> {
        let num_of_signatures = aggregated_signature.len();
        if num_of_signatures < self.quorum_size {
            return Err(VerifyError::TooFewSignatures {
                num_of_signatures,
                quorum_size: self.quorum_size,
            });
        }
        if num_of_signatures > self.len() {
            return Err(VerifyError::TooManySignatures {
                num_of_signatures,
                num_of_authors: self.len(),
            });
        }
        for (author, signature) in aggregated_signature {
            if let Err(err) = self.verify_signature(*author, hash, signature) {
                return Err(err);
            }
        }
        Ok(())
    }

    pub fn get_public_key(&self, author: AccountAddress) -> Option<PublicKey> {
        self.author_to_public_keys.get(&author).cloned()
    }

    /// Returns a ordered list of account addresses from smallest to largest.
    pub fn get_ordered_account_addresses(&self) -> Vec<AccountAddress> {
        let mut account_addresses: Vec<AccountAddress> = self
            .author_to_public_keys
            .keys()
            .into_iter()
            .cloned()
            .collect();
        account_addresses.sort();
        account_addresses
    }

    /// Returns the number of authors to be validated.
    pub fn len(&self) -> usize {
        self.author_to_public_keys.len()
    }

    /// Is there at least one author?
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns quorum_size.
    pub fn quorum_size(&self) -> usize {
        self.quorum_size
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        account_address::AccountAddress,
        validator_signer::ValidatorSigner,
        validator_verifier::{ValidatorVerifier, VerifyError},
    };
    use crypto::{HashValue, PublicKey, Signature};
    use std::collections::HashMap;

    #[test]
    fn test_validator() {
        let validator_signer = ValidatorSigner::random();
        let random_hash = HashValue::random();
        let signature = validator_signer.sign_message(random_hash).unwrap();
        let validator =
            ValidatorVerifier::new_single(validator_signer.author(), validator_signer.public_key());
        assert_eq!(
            validator.verify_signature(validator_signer.author(), random_hash, &signature),
            Ok(())
        );
        let unknown_validator_signer = ValidatorSigner::random();
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
    fn test_quorum_validators() {
        // Generate 7 random signers.
        let validator_signers: Vec<ValidatorSigner> =
            (0..7).map(|_| ValidatorSigner::random()).collect();
        let random_hash = HashValue::random();

        // Create a map from authors to public keys.
        let mut author_to_public_key_map: HashMap<AccountAddress, PublicKey> = HashMap::new();
        for validator in validator_signers.iter() {
            author_to_public_key_map.insert(validator.author(), validator.public_key());
        }

        // Create a map from author to signatures.
        let mut author_to_signature_map: HashMap<AccountAddress, Signature> = HashMap::new();
        for validator in validator_signers.iter() {
            author_to_signature_map.insert(
                validator.author(),
                validator.sign_message(random_hash).unwrap(),
            );
        }

        // Let's assume our verifier needs to satisfy at least 5 signatures from the original 7.
        let validator_verifier = ValidatorVerifier::new(author_to_public_key_map, 5);

        // Check against signatures == N; this will pass.
        assert_eq!(
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an extra unknown signer, signatures > N; this will fail.
        let unknown_validator_signer = ValidatorSigner::random();
        let unknown_signature = unknown_validator_signer.sign_message(random_hash).unwrap();
        author_to_signature_map.insert(unknown_validator_signer.author(), unknown_signature);
        assert_eq!(
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
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
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
            Ok(())
        );

        // Add an unknown signer, but quorum is satisfied and signatures <= N; this will fail as we
        // don't tolerate invalid signatures.
        author_to_signature_map.insert(unknown_validator_signer.author(), unknown_signature);
        assert_eq!(
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
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
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::TooFewSignatures {
                num_of_signatures: 4,
                quorum_size: 5
            })
        );

        // Add an unknown signer, we have 5 signers, but one of them is invalid; this will fail.
        author_to_signature_map.insert(unknown_validator_signer.author(), unknown_signature);
        assert_eq!(
            validator_verifier.verify_aggregated_signature(random_hash, &author_to_signature_map),
            Err(VerifyError::UnknownAuthor)
        );
    }
}
