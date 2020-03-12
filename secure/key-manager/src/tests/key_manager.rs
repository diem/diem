// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::key_manager::{KeyManager, CONSENSUS_KEY, VALIDATOR_KEY};
use libra_crypto::ed25519::Ed25519PublicKey;
use libra_crypto::PrivateKey;
use libra_secure_storage::InMemoryStorage;

#[test]
fn ensure_keys_not_initialized_on_create() {
    let mut storage = InMemoryStorage::new_boxed_in_memory_storage();
    let mut key_manager = KeyManager::new(storage.as_mut());

    // Verify no keys have been initialized in storage
    assert!(key_manager.get_key_for_testing(VALIDATOR_KEY).is_err());
    assert!(key_manager.get_key_for_testing(CONSENSUS_KEY).is_err());
}

#[test]
fn initialize_keys_in_storage() {
    let mut storage = InMemoryStorage::new_boxed_in_memory_storage();
    let mut key_manager = KeyManager::new(storage.as_mut());
    initialize_key_manager(&mut key_manager);

    // Verify the keys have now been initialized in storage
    assert!(key_manager.get_key_for_testing(VALIDATOR_KEY).is_ok());
    assert!(key_manager.get_key_for_testing(CONSENSUS_KEY).is_ok());
}

#[test]
fn try_rotate_consensus_without_initialization() {
    let mut storage = InMemoryStorage::new_boxed_in_memory_storage();
    let mut key_manager = KeyManager::new(storage.as_mut());

    assert!(key_manager.rotate_consensus_key().is_err());
}

#[test]
fn rotate_consensus_keys() {
    let mut storage = InMemoryStorage::new_boxed_in_memory_storage();
    let mut key_manager = KeyManager::new(storage.as_mut());
    initialize_key_manager(&mut key_manager);

    let consensus_key_before_rotation = get_consensus_key(&mut key_manager);
    rotate_consensus_key(&mut key_manager);
    let consensus_key_after_rotation = get_consensus_key(&mut key_manager);

    // Verify that the consensus key has been rotated
    assert_ne!(consensus_key_before_rotation, consensus_key_after_rotation);

    // Verify that the old consensus key still remains in storage (under a previous version)
    let previous_consensus_key = storage
        .get_private_key_for_name_and_version(CONSENSUS_KEY, consensus_key_before_rotation.clone())
        .expect("Failed to get the consensus key before rotation!");
    assert_eq!(
        consensus_key_before_rotation,
        previous_consensus_key.public_key(),
    )
}

#[test]
fn fix_initialization_after_crash() {
    let mut storage = InMemoryStorage::new_boxed_in_memory_storage();
    let mut key_manager = KeyManager::new(storage.as_mut());

    /* Assume a failure happens during initialize_key_manager(..), one key is set, but the other is
    not. */
    let consensus_key = initialize_and_get_consensus_key(&mut key_manager);
    assert!(key_manager.get_key_for_testing(VALIDATOR_KEY).is_err());

    /* Key_manager is then restarted and a storage state fix is performed */
    let mut key_manager = KeyManager::new(storage.as_mut());
    check_and_fix_keys_after_failure(&mut key_manager);

    /* Verify both keys are now correctly initialized, and the consensus key has not changed */
    assert_eq!(get_consensus_key(&mut key_manager), consensus_key);
    assert!(key_manager.get_key_for_testing(VALIDATOR_KEY).is_ok());
}

fn initialize_key_manager(key_manager: &mut KeyManager) {
    key_manager
        .initialize_keys()
        .expect("Failed to initialize key manager!");
}

fn rotate_consensus_key(key_manager: &mut KeyManager) {
    key_manager
        .rotate_consensus_key()
        .expect("Failed to rotate the consensus key!");
}

fn get_consensus_key(key_manager: &mut KeyManager) -> Ed25519PublicKey {
    key_manager
        .get_key_for_testing(CONSENSUS_KEY)
        .expect("Failed to get the consensus key!")
        .public_key()
}

fn initialize_and_get_consensus_key(key_manager: &mut KeyManager) -> Ed25519PublicKey {
    key_manager
        .initialize_key_for_testing(CONSENSUS_KEY)
        .expect("Failed to initialize the consensus key!");
    get_consensus_key(key_manager)
}

fn check_and_fix_keys_after_failure(key_manager: &mut KeyManager) {
    key_manager
        .check_and_fix_keys_after_failure()
        .expect("Failed to check and fix keys after a failure!");
}
