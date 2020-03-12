// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[cfg(test)]
use libra_crypto::ed25519::Ed25519PrivateKey;
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_secure_storage::{Error, Policy, Storage};

/// KeyManager represents the entity that is responsible for: (i) initializing cryptographic
/// keys in secure storage (e.g., the validator key and the consensus key); and (ii) periodically
/// rotating keys (e.g., the consensus key) and updating the on-blockchain key mapping for each
/// rotation.
///
/// Note: it is assumed that KeyManager may fail during execution (e.g., during initialization and
/// rotation). To overcome this, subsequent initialization and key rotation calls must be able to
/// handle incorrect or inconsistent state written to either secure storage or the blockchain (i.e.,
/// due to a previous failure).
pub struct KeyManager<'a> {
    storage: &'a mut dyn Storage,
}

// TODO(joshlind): we need to have a single, global definition for these key names, otherwise
// multiple definitions will exist (e.g., one already exists in persistent_safety_storage.rs).
// It might make most sense for the global definition to live here? Or perhaps in secure storage?
pub const CONSENSUS_KEY: &str = "consensus_key";
pub const VALIDATOR_KEY: &str = "validator_key";

impl<'a> KeyManager<'a> {
    pub fn new(storage: &'a mut dyn Storage) -> Self {
        Self { storage }
    }

    /// Initializes the consensus and validator keys for each validator node in secure storage. If
    /// any of these keys already exist, this method returns an error.
    pub fn initialize_keys(&mut self) -> Result<(), Error> {
        // TODO(joshlind): generate the correct policies for these keys (i.e., policies that respect
        // the LSR and KeyManagement access rules)
        let consensus_key_policy = Policy::public();
        let validator_key_policy = Policy::public();

        self.initialize_key(VALIDATOR_KEY, &validator_key_policy)?;
        self.initialize_key(CONSENSUS_KEY, &consensus_key_policy)
    }

    /// Initializes a cryptographic key in storage using a given 'key_name' and 'policy'. This
    /// method verifies the key doesn't already exist and returns an error if the key has already
    /// been initialized.
    fn initialize_key(&mut self, key_name: &str, policy: &Policy) -> Result<(), Error> {
        self.verify_key_not_created(key_name)?;
        self.storage
            .generate_new_ed25519_key_pair(key_name, policy)?;
        Ok(())
    }

    /// Verifies that a named key has not already been created and stored in secure storage. If the
    /// key does exist, an error is returned.
    fn verify_key_not_created(&mut self, key_name: &str) -> Result<(), Error> {
        match self.storage.get_public_key_for_name(key_name) {
            Err(Error::KeyNotSet(_)) => Ok(()),
            _ => Err(Error::KeyAlreadyExists(key_name.to_string())),
        }
    }

    /// Rotates the consensus key by: (i) generating and storing a new consensus key in the
    /// secure store; and (ii) generating and signing a transaction that updates the consensus key
    /// for this validator node on the blockchain. If a failure occurs during the process (e.g.,
    /// the consensus key does not exist, or a new transaction could not be generated), an error is
    /// returned.
    pub fn rotate_consensus_key(&mut self) -> Result<(), Error> {
        // Ensure key has already been initialized before rotating
        self.verify_key_correctly_initialized(CONSENSUS_KEY)?;

        // Rotate the key in storage
        let new_public_consensus_key = self.storage.rotate_key_pair(CONSENSUS_KEY)?;

        // Update the on-chain validator config to reflect the new_public_consensus_key
        let _ = self.update_on_chain_consensus_key(new_public_consensus_key);

        Ok(())
    }

    /// Verifies that a named key has already been initialized correctly in the secure store.
    /// Returns an error if the key doesn't exist, or if the value related to the key is not
    /// correct.
    fn verify_key_correctly_initialized(&mut self, key_name: &str) -> Result<(), Error> {
        match self.storage.get_public_key_for_name(key_name) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// This method is called after KeyManager has failed but has now been restarted.
    /// It verifies the internal state of secure storage and fixes any potential inconsistencies due
    /// to the failure. For this, we: (i) verify that all keys in secure storage have already been
    /// initialized. If not, we initialize these keys for the first time; and (ii) verify the
    /// current consensus key registered on-chain reflects the state of secure storage. If not, we
    /// broadcasts the current consensus key on-chain. On the next Validator reconfiguration event,
    /// the consensus key will be registered and the node can begin participating in consensus once
    /// again.
    pub fn check_and_fix_keys_after_failure(&mut self) -> Result<(), Error> {
        // TODO(joshlind): generate the correct policies for these keys (i.e., policies that respect
        // the LSR and KeyManagement access rules)
        let consensus_key_policy = Policy::public();
        let validator_key_policy = Policy::public();

        // Initialize both keys -- these calls  will fail if the keys have already been initialized!
        let _ = self.initialize_key(CONSENSUS_KEY, &consensus_key_policy);
        let _ = self.initialize_key(VALIDATOR_KEY, &validator_key_policy);

        // TODO(joshlind): check the registered on-chain consensus key for this validator.
        // If it doesn't match the key held in secure storage, update the on-chain state.
        let validator_public_key = self.storage.get_public_key_for_name(VALIDATOR_KEY)?;
        let _consensus_public_key = self.storage.get_public_key_for_name(CONSENSUS_KEY)?;
        let _on_chain_consensus_key = self.get_on_chain_consensus_key(validator_public_key);
        // compare the on_chain_consensus_key to consensus_public_key and call
        // update_on_chain_consensus_key if they don't match!

        Ok(())
    }

    /// Retrieves the current consensus key registered on-chain in the Validator config of the
    /// given 'validator_public_key'. If a consensus key is not registered on-chain, or the
    /// validator config isn't found, an error is returned.
    fn get_on_chain_consensus_key(
        &mut self,
        _validator_public_key: Ed25519PublicKey,
    ) -> Result<Ed25519PublicKey, Error> {
        // TODO(joshlind): implement this method to interface with the Validator Config!
        Err(Error::InternalError(
            "get_on_chain_consensus_key() is not yet supported!".to_string(),
        ))
    }

    /// Updates the registered (i.e., on-chain) consensus key for this validator node after a
    /// key rotation has occurred. To achieve this, the on-chain validator config is modified to
    /// include the new rotated 'public_key', and submitted to the blockchain for processing. If
    /// any part of this method fails, an error is returned.
    fn update_on_chain_consensus_key(
        &mut self,
        _public_key: Ed25519PublicKey,
    ) -> Result<(), Error> {
        // TODO(joshlind): implement this method to interface with the Validator Config!
        Err(Error::InternalError(
            "update_on_chain_consensus_key() is not yet supported!".to_string(),
        ))
    }

    #[cfg(test)]
    /// Returns the private key for the given key_name as held in storage. This is required to
    /// test the internal state of secure storage.
    pub fn get_key_for_testing(&mut self, key_name: &str) -> Result<Ed25519PrivateKey, Error> {
        self.storage.get_private_key_for_name(key_name)
    }

    #[cfg(test)]
    /// Creates and initializes the given key_name in storage. This is required to
    /// manipulate the internal state of storage for testing.
    pub fn initialize_key_for_testing(&mut self, key_name: &str) -> Result<(), Error> {
        self.initialize_key(key_name, &Policy::public())
    }
}
