script {
use 0x1::RecoveryAddress;

/// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
/// `RecoveryAddress` resource under  `recovery_account`.
/// Aborts if `recovery_account` has delegated its `KeyRotationCapability`, already has a
/// `RecoveryAddress` resource, or is not a VASP.
fun rotate_authentication_key_with_recovery_address(account: &signer, recovery_address: address, to_recover: address, new_key: vector<u8>) {
    RecoveryAddress::rotate_authentication_key(account, recovery_address, to_recover, new_key)
}
}
