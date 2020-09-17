script {
use 0x1::LibraAccount;
use 0x1::RecoveryAddress;

/// # Summary
/// Initializes the sending account as a recovery address that may be used by
/// the VASP that it belongs to. The sending account must be a VASP account.
/// Multiple recovery addresses can exist for a single VASP, but accounts in
/// each must be disjoint.
///
/// # Technical Description
/// Publishes a `RecoveryAddress::RecoveryAddress` resource under `account`. It then
/// extracts the `LibraAccount::KeyRotationCapability` for `account` and adds
/// it to the resource. After the successful execution of this transaction
/// other accounts may add their key rotation to this resource so that `account`
/// may be used as a recovery account for those accounts.
///
/// # Parameters
/// | Name      | Type      | Description                                           |
/// | ------    | ------    | -------------                                         |
/// | `account` | `&signer` | The signer of the sending account of the transaction. |
///
/// # Common Abort Conditions
/// | Error Category              | Error Reason                                               | Description                                                                                   |
/// | ----------------            | --------------                                             | -------------                                                                                 |
/// | `Errors::INVALID_STATE`     | `LibraAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `LibraAccount::KeyRotationCapability`.          |
/// | `Errors::INVALID_ARGUMENT`  | `RecoveryAddress::ENOT_A_VASP`                             | `account` is not a VASP account.                                                              |
/// | `Errors::INVALID_ARGUMENT`  | `RecoveryAddress::EKEY_ROTATION_DEPENDENCY_CYCLE`          | A key rotation recovery cycle would be created by adding `account`'s key rotation capability. |
/// | `Errors::ALREADY_PUBLISHED` | `RecoveryAddress::ERECOVERY_ADDRESS`                       | A `RecoveryAddress::RecoveryAddress` resource has already been published under `account`.     |
///
/// # Related Scripts
/// * `Script::add_recovery_rotation_capability`
/// * `Script::rotate_authentication_key_with_recovery_address`

fun create_recovery_address(account: &signer) {
    RecoveryAddress::publish(account, LibraAccount::extract_key_rotation_capability(account))
}
}
