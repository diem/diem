script {
use 0x1::RecoveryAddress;

/// # Summary
/// Rotates the authentication key of a specified account that is part of a recovery address to a
/// new authentication key. Only used for accounts that are part of a recovery address (see
/// `Script::add_recovery_rotation_capability` for account restrictions).
///
/// # Technical Description
/// Rotates the authentication key of the `to_recover` account to `new_key` using the
/// `LibraAccount::KeyRotationCapability` stored in the `RecoveryAddress::RecoveryAddress` resource
/// published under `recovery_address`. This transaction can be sent either by the `to_recover`
/// account, or by the account where the `RecoveryAddress::RecoveryAddress` resource is published
/// that contains `to_recover`'s `LibraAccount::KeyRotationCapability`.
///
/// # Parameters
/// | Name               | Type         | Description                                                                                                                    |
/// | ------             | ------       | -------------                                                                                                                  |
/// | `account`          | `&signer`    | Signer reference of the sending account of the transaction.                                                                    |
/// | `recovery_address` | `address`    | Address where `RecoveryAddress::RecoveryAddress` that holds `to_recover`'s `LibraAccount::KeyRotationCapability` is published. |
/// | `to_recover`       | `address`    | The address of the account whose authentication key will be updated.                                                           |
/// | `new_key`          | `vector<u8>` | New ed25519 public key to be used for the account at the `to_recover` address.                                                 |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                                  | Description                                                                                                                                          |
/// | ----------------           | --------------                                | -------------                                                                                                                                        |
/// | `Errors::NOT_PUBLISHED`    | `RecoveryAddress::ERECOVERY_ADDRESS`          | `recovery_address` does not have a `RecoveryAddress::RecoveryAddress` resource published under it.                                                   |
/// | `Errors::INVALID_ARGUMENT` | `RecoveryAddress::ECANNOT_ROTATE_KEY`         | The address of `account` is not `recovery_address` or `to_recover`.                                                                                  |
/// | `Errors::INVALID_ARGUMENT` | `RecoveryAddress::EACCOUNT_NOT_RECOVERABLE`   | `to_recover`'s `LibraAccount::KeyRotationCapability`  is not in the `RecoveryAddress::RecoveryAddress`  resource published under `recovery_address`. |
/// | `Errors::INVALID_ARGUMENT` | `LibraAccount::EMALFORMED_AUTHENTICATION_KEY` | `new_key` was an invalid length.                                                                                                                     |
///
/// # Related Scripts
/// * `Script::rotate_authentication_key`
/// * `Script::rotate_authentication_key_with_nonce`
/// * `Script::rotate_authentication_key_with_nonce_admin`

fun rotate_authentication_key_with_recovery_address(
    account: &signer,
    recovery_address: address,
    to_recover: address,
    new_key: vector<u8>
) {
    RecoveryAddress::rotate_authentication_key(account, recovery_address, to_recover, new_key)
}
spec fun rotate_authentication_key_with_recovery_address {
    use 0x1::Errors;
    use 0x1::LibraAccount;
    use 0x1::Signer;

    include LibraAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    include RecoveryAddress::RotateAuthenticationKeyAbortsIf;
    include RecoveryAddress::RotateAuthenticationKeyEnsures;

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT;

    /// **Access Control:**
    /// The delegatee at the recovery address has to hold the key rotation capability for
    /// the address to recover. The address of the transaction signer has to be either
    /// the delegatee's address or the address to recover [[H17]][PERMISSION][[J17]][PERMISSION].
    let account_addr = Signer::spec_address_of(account);
    aborts_if !RecoveryAddress::spec_holds_key_rotation_cap_for(recovery_address, to_recover) with Errors::INVALID_ARGUMENT;
    aborts_if !(account_addr == recovery_address || account_addr == to_recover) with Errors::INVALID_ARGUMENT;
}
}
