script {
use 0x1::RecoveryAddress;

/// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
/// `RecoveryAddress` resource under  `recovery_account`.
/// Aborts if `recovery_account` has delegated its `KeyRotationCapability`, already has a
/// `RecoveryAddress` resource, or is not a VASP.
fun create_recovery_address(account: &signer) {
    RecoveryAddress::publish(account)
}
}
