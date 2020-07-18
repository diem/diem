script {
use 0x1::LibraAccount;

// imports for the prover
use 0x1::Signer;
use 0x1::Option;

/// Rotate the sender's authentication key to `new_key`.
/// `new_key` should be a 256 bit sha3 hash of an ed25519 public key.
/// * Aborts with `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if the `KeyRotationCapability` for `account` has already been extracted.
/// * Aborts with `0` if the key rotation capability held by the account doesn't match the sender's address.
/// * Aborts with `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` if the length of `new_key` != 32.
fun rotate_authentication_key(account: &signer, new_key: vector<u8>) {
  let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
  assert(*LibraAccount::key_rotation_capability_address(&key_rotation_capability) == Signer::address_of(account), 0);
  LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
  LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
spec fun rotate_authentication_key {
    pragma verify = true;
    /// This rotates the authentication key of `account` to `new_key`
    ensures LibraAccount::spec_rotate_authentication_key(Signer::spec_address_of(account), new_key);

    /// If the sending account doesn't exist this will abort
    aborts_if !exists<LibraAccount::LibraAccount>(Signer::spec_address_of(account));
    /// `account` must not have delegated its rotation capability
    aborts_if LibraAccount::spec_delegated_key_rotation_capability(Signer::spec_address_of(account));
    /// `account` must hold its own rotation capability
    aborts_if LibraAccount::spec_key_rotation_capability_address(
                Option::spec_get(
                    LibraAccount::spec_get_key_rotation_cap(Signer::spec_address_of(account))
              )) != Signer::spec_address_of(account);
    /// `new_key`'s length must be `32`.
    aborts_if len(new_key) != 32;
}
}
