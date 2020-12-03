script {
use 0x1::DiemAccount;
use 0x1::RecoveryAddress;

/// # Summary
/// Stores the sending accounts ability to rotate its authentication key with a designated recovery
/// account. Both the sending and recovery accounts need to belong to the same VASP and
/// both be VASP accounts. After this transaction both the sending account and the
/// specified recovery account can rotate the sender account's authentication key.
///
/// # Technical Description
/// Adds the `DiemAccount::KeyRotationCapability` for the sending account
/// (`to_recover_account`) to the `RecoveryAddress::RecoveryAddress` resource under
/// `recovery_address`. After this transaction has been executed successfully the account at
/// `recovery_address` and the `to_recover_account` may rotate the authentication key of
/// `to_recover_account` (the sender of this transaction).
///
/// The sending account of this transaction (`to_recover_account`) must not have previously given away its unique key
/// rotation capability, and must be a VASP account. The account at `recovery_address`
/// must also be a VASP account belonging to the same VASP as the `to_recover_account`.
/// Additionally the account at `recovery_address` must have already initialized itself as
/// a recovery account address using the `Script::create_recovery_address` transaction script.
///
/// The sending account's (`to_recover_account`) key rotation capability is
/// removed in this transaction and stored in the `RecoveryAddress::RecoveryAddress`
/// resource stored under the account at `recovery_address`.
///
/// # Parameters
/// | Name                 | Type      | Description                                                                                                |
/// | ------               | ------    | -------------                                                                                              |
/// | `to_recover_account` | `&signer` | The signer reference of the sending account of this transaction.                                           |
/// | `recovery_address`   | `address` | The account address where the `to_recover_account`'s `DiemAccount::KeyRotationCapability` will be stored. |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                               | Description                                                                                     |
/// | ----------------           | --------------                                             | -------------                                                                                   |
/// | `Errors::INVALID_STATE`    | `DiemAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `to_recover_account` has already delegated/extracted its `DiemAccount::KeyRotationCapability`. |
/// | `Errors::NOT_PUBLISHED`    | `RecoveryAddress::ERECOVERY_ADDRESS`                       | `recovery_address` does not have a `RecoveryAddress` resource published under it.               |
/// | `Errors::INVALID_ARGUMENT` | `RecoveryAddress::EINVALID_KEY_ROTATION_DELEGATION`        | `to_recover_account` and `recovery_address` do not belong to the same VASP.                     |
///
/// # Related Scripts
/// * `Script::create_recovery_address`
/// * `Script::rotate_authentication_key_with_recovery_address`

fun add_recovery_rotation_capability(to_recover_account: &signer, recovery_address: address) {
    RecoveryAddress::add_rotation_capability(
        DiemAccount::extract_key_rotation_capability(to_recover_account), recovery_address
    )
}
spec fun add_recovery_rotation_capability {
    use 0x1::Signer;
    use 0x1::Errors;

    include DiemAccount::TransactionChecks{sender: to_recover_account}; // properties checked by the prologue.
    include DiemAccount::ExtractKeyRotationCapabilityAbortsIf{account: to_recover_account};
    include DiemAccount::ExtractKeyRotationCapabilityEnsures{account: to_recover_account};

    let addr = Signer::spec_address_of(to_recover_account);
    let rotation_cap = DiemAccount::spec_get_key_rotation_cap(addr);

    include RecoveryAddress::AddRotationCapabilityAbortsIf{
        to_recover: rotation_cap
    };

    ensures RecoveryAddress::spec_get_rotation_caps(recovery_address)[
        len(RecoveryAddress::spec_get_rotation_caps(recovery_address)) - 1] == old(rotation_cap);

    aborts_with [check]
        Errors::INVALID_STATE,
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT;
}
}
