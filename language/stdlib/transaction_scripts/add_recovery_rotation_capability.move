script {
use 0x1::LibraAccount;
use 0x1::RecoveryAddress;

/// Add the `KeyRotationCapability` for `to_recover_account` to the `RecoveryAddress` resource under `recovery_address`.
///
/// ## Aborts
/// * Aborts with `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` if `account` has already delegated its `KeyRotationCapability`.
/// * Aborts with `RecoveryAddress:ENOT_A_RECOVERY_ADDRESS` if `recovery_address` does not have a `RecoveryAddress` resource.
/// * Aborts with `RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION` if `to_recover_account` and `recovery_address` do not belong to the same VASP.
fun add_recovery_rotation_capability(to_recover_account: &signer, recovery_address: address) {
    RecoveryAddress::add_rotation_capability(
        LibraAccount::extract_key_rotation_capability(to_recover_account), recovery_address
    )
}
}
