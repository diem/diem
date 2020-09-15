script {
use 0x1::LibraAccount;

/// Rotate the sender's authentication key to `new_key`.
/// `new_key` should be a 256 bit sha3 hash of an ed25519 public key.
/// * Aborts with `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if the `KeyRotationCapability` for `account` has already been extracted.
/// * Aborts with `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` if the length of `new_key` != 32.
fun rotate_authentication_key(account: &signer, new_key: vector<u8>) {
    let key_rotation_capability = LibraAccount::extract_key_rotation_capability(account);
    LibraAccount::rotate_authentication_key(&key_rotation_capability, new_key);
    LibraAccount::restore_key_rotation_capability(key_rotation_capability);
}
spec fun rotate_authentication_key {
    use 0x1::Signer;
    let account_addr = Signer::spec_address_of(account);
    include LibraAccount::ExtractKeyRotationCapabilityAbortsIf;
    let key_rotation_capability = LibraAccount::spec_get_key_rotation_cap(account_addr);
    include LibraAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

    /// This rotates the authentication key of `account` to `new_key`
    include LibraAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};
}
}
