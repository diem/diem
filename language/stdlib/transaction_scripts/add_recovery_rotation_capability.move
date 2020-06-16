script {
use 0x1::RecoveryAddress;

/// Add the `KeyRotationCapability` for `to_recover_account` to the `RecoveryAddress`
/// resource under `recovery_address`.
/// Aborts if `to_recovery_account` and `to_recovery_address belong to different VASPs, if
/// `recovery_address` does not have a `RecoveryAddress` resource, or if
/// `to_recover_account` has already extracted its `KeyRotationCapability`.
fun add_recovery_rotation_capability(to_recover_account: &signer, recovery_address: address) {
    RecoveryAddress::add_rotation_capability(to_recover_account, recovery_address)
}
}
