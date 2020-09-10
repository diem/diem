script {
use 0x1::LibraAccount;
use 0x1::RecoveryAddress;

/// Extract the `KeyRotationCapability` for `recovery_account` and publish it in a
/// `RecoveryAddress` resource under  `account`.
/// ## Aborts
/// * Aborts with `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if `account` has already delegated its `KeyRotationCapability`.
/// * Aborts with `RecoveryAddress::ENOT_A_VASP` if `account` is not a ParentVASP or ChildVASP
fun create_recovery_address(account: &signer) {
    RecoveryAddress::publish(account, LibraAccount::extract_key_rotation_capability(account))
}
}
