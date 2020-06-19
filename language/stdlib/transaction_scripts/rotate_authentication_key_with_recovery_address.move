script {
use 0x1::RecoveryAddress;

/// Rotate the authentication key of `to_recover` to `new_key`. Can be invoked by either
/// `recovery_address` or `to_recover`. Aborts if `recovery_address` does not have the
/// `KeyRotationCapability` for `to_recover`.
fun rotate_authentication_key_with_recovery_address(account: &signer, recovery_address: address, to_recover: address, new_key: vector<u8>) {
    RecoveryAddress::rotate_authentication_key(account, recovery_address, to_recover, new_key)
}
}
