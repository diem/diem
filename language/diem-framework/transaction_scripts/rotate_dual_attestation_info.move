script {
use 0x1::DualAttestation;

/// # Summary
/// Updates the url used for off-chain communication, and the public key used to verify dual
/// attestation on-chain. Transaction can be sent by any account that has dual attestation
/// information published under it. In practice the only such accounts are Designated Dealers and
/// Parent VASPs.
///
/// # Technical Description
/// Updates the `base_url` and `compliance_public_key` fields of the `DualAttestation::Credential`
/// resource published under `account`. The `new_key` must be a valid ed25519 public key.
///
/// ## Events
/// Successful execution of this transaction emits two events:
/// * A `DualAttestation::ComplianceKeyRotationEvent` containing the new compliance public key, and
/// the blockchain time at which the key was updated emitted on the `DualAttestation::Credential`
/// `compliance_key_rotation_events` handle published under `account`; and
/// * A `DualAttestation::BaseUrlRotationEvent` containing the new base url to be used for
/// off-chain communication, and the blockchain time at which the url was updated emitted on the
/// `DualAttestation::Credential` `base_url_rotation_events` handle published under `account`.
///
/// # Parameters
/// | Name      | Type         | Description                                                               |
/// | ------    | ------       | -------------                                                             |
/// | `account` | `&signer`    | Signer reference of the sending account of the transaction.               |
/// | `new_url` | `vector<u8>` | ASCII-encoded url to be used for off-chain communication with `account`.  |
/// | `new_key` | `vector<u8>` | New ed25519 public key to be used for on-chain dual attestation checking. |
///
/// # Common Abort Conditions
/// | Error Category             | Error Reason                           | Description                                                                |
/// | ----------------           | --------------                         | -------------                                                              |
/// | `Errors::NOT_PUBLISHED`    | `DualAttestation::ECREDENTIAL`         | A `DualAttestation::Credential` resource is not published under `account`. |
/// | `Errors::INVALID_ARGUMENT` | `DualAttestation::EINVALID_PUBLIC_KEY` | `new_key` is not a valid ed25519 public key.                               |
///
/// # Related Scripts
/// * `Script::create_parent_vasp_account`
/// * `Script::create_designated_dealer`
/// * `Script::rotate_dual_attestation_info`

fun rotate_dual_attestation_info(account: &signer, new_url: vector<u8>, new_key: vector<u8>) {
    DualAttestation::rotate_base_url(account, new_url);
    DualAttestation::rotate_compliance_public_key(account, new_key)
}
spec fun rotate_dual_attestation_info {
    use 0x1::Errors;
    use 0x1::DiemAccount;
    use 0x1::Signer;

    include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
    include DualAttestation::RotateBaseUrlAbortsIf;
    include DualAttestation::RotateBaseUrlEnsures;
    include DualAttestation::RotateCompliancePublicKeyAbortsIf;
    include DualAttestation::RotateCompliancePublicKeyEnsures;

    aborts_with [check]
        Errors::NOT_PUBLISHED,
        Errors::INVALID_ARGUMENT;

    /// **Access Control:**
    /// Only the account having Credential can rotate the info.
    /// Credential is granted to either a Parent VASP or a designated dealer [[H16]][PERMISSION].
    include DualAttestation::AbortsIfNoCredential{addr: Signer::spec_address_of(account)};
}
}
