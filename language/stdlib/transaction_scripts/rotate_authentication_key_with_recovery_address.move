script {
use 0x1::RecoveryAddress;

/// Rotate the authentication key of `to_recover` to `new_key` using the `KeyRotationCapability`
/// stored under `recovery_address`.
///
/// ## Aborts
/// * Aborts with `RecoveryAddress::ENOT_A_RECOVERY_ADDRESS` if `recovery_address` does not have a `RecoveryAddress` resource
/// * Aborts with `RecoveryAddress::ECANNOT_ROTATE_KEY` if `account` is not `recovery_address` or `to_recover`.
/// * Aborts with `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` if `new_key` is not 32 bytes.
/// * Aborts with `RecoveryAddress::ECANNOT_ROTATE_KEY` if `account` has not delegated its `KeyRotationCapability` to `recovery_address`.
fun rotate_authentication_key_with_recovery_address(
    account: &signer, recovery_address: address, to_recover: address, new_key: vector<u8>
) {
    RecoveryAddress::rotate_authentication_key(account, recovery_address, to_recover, new_key)
}
spec fun rotate_authentication_key_with_recovery_address {
    include RecoveryAddress::RotateAuthenticationKeyAbortsIf;
    include RecoveryAddress::RotateAuthenticationKeyEnsures;
}
}
