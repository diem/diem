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
spec fun create_recovery_address {
    use 0x1::Signer;
    use 0x1::Errors;

    include LibraAccount::ExtractKeyRotationCapabilityAbortsIf;
    include LibraAccount::ExtractKeyRotationCapabilityEnsures;

    let account_addr = Signer::spec_address_of(account);
    let rotation_cap = LibraAccount::spec_get_key_rotation_cap(account_addr);

    include RecoveryAddress::PublishAbortsIf{
        recovery_account: account,
        rotation_cap: rotation_cap
    };

    // TODO: Commented out due to the unsupported feature below.
    // include RecoveryAddress::PublishEnsures{
    //     recovery_account: account,
    //     rotation_cap: old(rotation_cap) //! error: `old(..)` expression not allowed in this context
    // };
    // Instead, the postconditions in RecoveryAddress::PublishEnsures are expanded here.
    ensures RecoveryAddress::spec_is_recovery_address(account_addr);
    ensures len(RecoveryAddress::spec_get_rotation_caps(account_addr)) == 1;
    ensures RecoveryAddress::spec_get_rotation_caps(account_addr)[0] == old(rotation_cap);

    // TODO: The following line is added to help Prover verifying the
    // "aborts_with [check]" spec. This fact can be inferred from the successful
    // termination of the prologue functions, but Prover cannot figure it out currently.
    requires LibraAccount::exists_at(account_addr);

    aborts_with [check]
        Errors::INVALID_STATE,
        Errors::INVALID_ARGUMENT,
        Errors::ALREADY_PUBLISHED;
}
}
