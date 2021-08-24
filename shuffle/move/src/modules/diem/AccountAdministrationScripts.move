/// This module holds transactions that can be used to administer accounts in the Diem Framework.
module DiemFramework::AccountAdministrationScripts {
    use DiemFramework::DiemAccount;
    use DiemFramework::SlidingNonce;
    use DiemFramework::DualAttestation;

    /// # Summary
    /// Adds a zero `Currency` balance to the sending `account`. This will enable `account` to
    /// send, receive, and hold `Diem::Diem<Currency>` coins. This transaction can be
    /// successfully sent by any account that is allowed to hold balances
    /// (e.g., VASP, Designated Dealer).
    ///
    /// # Technical Description
    /// After the successful execution of this transaction the sending account will have a
    /// `DiemAccount::Balance<Currency>` resource with zero balance published under it. Only
    /// accounts that can hold balances can send this transaction, the sending account cannot
    /// already have a `DiemAccount::Balance<Currency>` published under it.
    ///
    /// # Parameters
    /// | Name       | Type     | Description                                                                                                                                         |
    /// | ------     | ------   | -------------                                                                                                                                       |
    /// | `Currency` | Type     | The Move type for the `Currency` being added to the sending account of the transaction. `Currency` must be an already-registered currency on-chain. |
    /// | `account`  | `signer` | The signer of the sending account of the transaction.                                                                                               |
    ///
    /// # Common Abort Conditions
    /// | Error Category              | Error Reason                             | Description                                                                |
    /// | ----------------            | --------------                           | -------------                                                              |
    /// | `Errors::NOT_PUBLISHED`     | `Diem::ECURRENCY_INFO`                  | The `Currency` is not a registered currency on-chain.                      |
    /// | `Errors::INVALID_ARGUMENT`  | `DiemAccount::EROLE_CANT_STORE_BALANCE` | The sending `account`'s role does not permit balances.                     |
    /// | `Errors::ALREADY_PUBLISHED` | `DiemAccount::EADD_EXISTING_CURRENCY`   | A balance for `Currency` is already published under the sending `account`. |
    ///
    /// # Related Scripts
    /// * `AccountCreationScripts::create_child_vasp_account`
    /// * `AccountCreationScripts::create_parent_vasp_account`
    /// * `PaymentScripts::peer_to_peer_with_metadata`

    public(script) fun add_currency_to_account<Currency>(account: signer) {
        DiemAccount::add_currency<Currency>(&account);
    }
    spec add_currency_to_account {
        use Std::Errors;
        use Std::Signer;
        use DiemFramework::Roles;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        include DiemAccount::AddCurrencyAbortsIf<Currency>;
        include DiemAccount::AddCurrencyEnsures<Currency>{addr: Signer::spec_address_of(account)};

        aborts_with [check]
            Errors::NOT_PUBLISHED,
            Errors::INVALID_ARGUMENT,
            Errors::ALREADY_PUBLISHED;

        /// **Access Control:**
        /// The account must be allowed to hold balances. Only Designated Dealers, Parent VASPs,
        /// and Child VASPs can hold balances [[D1]][ROLE][[D2]][ROLE][[D3]][ROLE][[D4]][ROLE][[D5]][ROLE][[D6]][ROLE][[D7]][ROLE].
        aborts_if !Roles::can_hold_balance(account) with Errors::INVALID_ARGUMENT;
    }

    /// # Summary
    /// Rotates the `account`'s authentication key to the supplied new authentication key. May be sent by any account.
    ///
    /// # Technical Description
    /// Rotate the `account`'s `DiemAccount::DiemAccount` `authentication_key`
    /// field to `new_key`. `new_key` must be a valid authentication key that
    /// corresponds to an ed25519 public key as described [here](https://developers.diem.com/docs/core/accounts/#addresses-authentication-keys-and-cryptographic-keys),
    /// and `account` must not have previously delegated its `DiemAccount::KeyRotationCapability`.
    ///
    /// # Parameters
    /// | Name      | Type         | Description                                       |
    /// | ------    | ------       | -------------                                     |
    /// | `account` | `signer`     | Signer of the sending account of the transaction. |
    /// | `new_key` | `vector<u8>` | New authentication key to be used for `account`.  |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                              | Description                                                                         |
    /// | ----------------           | --------------                                            | -------------                                                                       |
    /// | `Errors::INVALID_STATE`    | `DiemAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `DiemAccount::KeyRotationCapability`. |
    /// | `Errors::INVALID_ARGUMENT` | `DiemAccount::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                    |
    ///
    /// # Related Scripts
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_nonce`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_nonce_admin`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_recovery_address`

    public(script) fun rotate_authentication_key(account: signer, new_key: vector<u8>) {
        let key_rotation_capability = DiemAccount::extract_key_rotation_capability(&account);
        DiemAccount::rotate_authentication_key(&key_rotation_capability, new_key);
        DiemAccount::restore_key_rotation_capability(key_rotation_capability);
    }
    spec rotate_authentication_key {
        use Std::Signer;
        use Std::Errors;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        let account_addr = Signer::spec_address_of(account);
        include DiemAccount::ExtractKeyRotationCapabilityAbortsIf;
        let key_rotation_capability = DiemAccount::spec_get_key_rotation_cap(account_addr);
        include DiemAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

        /// This rotates the authentication key of `account` to `new_key`
        include DiemAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};

        aborts_with [check]
            Errors::INVALID_STATE,
            Errors::INVALID_ARGUMENT;

        /// **Access Control:**
        /// The account can rotate its own authentication key unless
        /// it has delegrated the capability [[H18]][PERMISSION][[J18]][PERMISSION].
        include DiemAccount::AbortsIfDelegatedKeyRotationCapability;
    }

    /// # Summary
    /// Rotates the sender's authentication key to the supplied new authentication key. May be sent by
    /// any account that has a sliding nonce resource published under it (usually this is Treasury
    /// Compliance or Diem Root accounts).
    ///
    /// # Technical Description
    /// Rotates the `account`'s `DiemAccount::DiemAccount` `authentication_key`
    /// field to `new_key`. `new_key` must be a valid authentication key that
    /// corresponds to an ed25519 public key as described [here](https://developers.diem.com/docs/core/accounts/#addresses-authentication-keys-and-cryptographic-keys),
    /// and `account` must not have previously delegated its `DiemAccount::KeyRotationCapability`.
    ///
    /// # Parameters
    /// | Name            | Type         | Description                                                                |
    /// | ------          | ------       | -------------                                                              |
    /// | `account`       | `signer`     | Signer of the sending account of the transaction.                          |
    /// | `sliding_nonce` | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction. |
    /// | `new_key`       | `vector<u8>` | New authentication key to be used for `account`.                           |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                               | Description                                                                                |
    /// | ----------------           | --------------                                             | -------------                                                                              |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                             | A `SlidingNonce` resource is not published under `account`.                                |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                             | The `sliding_nonce` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                             | The `sliding_nonce` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`                    | The `sliding_nonce` has been previously recorded.                                          |
    /// | `Errors::INVALID_STATE`    | `DiemAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `DiemAccount::KeyRotationCapability`.       |
    /// | `Errors::INVALID_ARGUMENT` | `DiemAccount::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                           |
    ///
    /// # Related Scripts
    /// * `AccountAdministrationScripts::rotate_authentication_key`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_nonce_admin`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_recovery_address`

    public(script) fun rotate_authentication_key_with_nonce(account: signer, sliding_nonce: u64, new_key: vector<u8>) {
        SlidingNonce::record_nonce_or_abort(&account, sliding_nonce);
        let key_rotation_capability = DiemAccount::extract_key_rotation_capability(&account);
        DiemAccount::rotate_authentication_key(&key_rotation_capability, new_key);
        DiemAccount::restore_key_rotation_capability(key_rotation_capability);
    }
    spec rotate_authentication_key_with_nonce {
        use Std::Signer;
        use Std::Errors;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        let account_addr = Signer::spec_address_of(account);
        include SlidingNonce::RecordNonceAbortsIf{ seq_nonce: sliding_nonce };
        include DiemAccount::ExtractKeyRotationCapabilityAbortsIf;
        let key_rotation_capability = DiemAccount::spec_get_key_rotation_cap(account_addr);
        include DiemAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

        /// This rotates the authentication key of `account` to `new_key`
        include DiemAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};

        aborts_with [check]
            Errors::INVALID_ARGUMENT,
            Errors::INVALID_STATE,
            Errors::NOT_PUBLISHED;

        /// **Access Control:**
        /// The account can rotate its own authentication key unless
        /// it has delegrated the capability [[H18]][PERMISSION][[J18]][PERMISSION].
        include DiemAccount::AbortsIfDelegatedKeyRotationCapability;
    }

    /// # Summary
    /// Rotates the specified account's authentication key to the supplied new authentication key. May
    /// only be sent by the Diem Root account as a write set transaction.
    ///
    /// # Technical Description
    /// Rotate the `account`'s `DiemAccount::DiemAccount` `authentication_key` field to `new_key`.
    /// `new_key` must be a valid authentication key that corresponds to an ed25519
    /// public key as described [here](https://developers.diem.com/docs/core/accounts/#addresses-authentication-keys-and-cryptographic-keys),
    /// and `account` must not have previously delegated its `DiemAccount::KeyRotationCapability`.
    ///
    /// # Parameters
    /// | Name            | Type         | Description                                                                                       |
    /// | ------          | ------       | -------------                                                                                     |
    /// | `dr_account`    | `signer`     | The signer of the sending account of the write set transaction. May only be the Diem Root signer. |
    /// | `account`       | `signer`     | Signer of account specified in the `execute_as` field of the write set transaction.               |
    /// | `sliding_nonce` | `u64`        | The `sliding_nonce` (see: `SlidingNonce`) to be used for this transaction for Diem Root.          |
    /// | `new_key`       | `vector<u8>` | New authentication key to be used for `account`.                                                  |
    ///
    /// # Common Abort Conditions
    /// | Error Category             | Error Reason                                              | Description                                                                                                |
    /// | ----------------           | --------------                                            | -------------                                                                                              |
    /// | `Errors::NOT_PUBLISHED`    | `SlidingNonce::ESLIDING_NONCE`                            | A `SlidingNonce` resource is not published under `dr_account`.                                             |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_OLD`                            | The `sliding_nonce` in `dr_account` is too old and it's impossible to determine if it's duplicated or not. |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_TOO_NEW`                            | The `sliding_nonce` in `dr_account` is too far in the future.                                              |
    /// | `Errors::INVALID_ARGUMENT` | `SlidingNonce::ENONCE_ALREADY_RECORDED`                   | The `sliding_nonce` in` dr_account` has been previously recorded.                                          |
    /// | `Errors::INVALID_STATE`    | `DiemAccount::EKEY_ROTATION_CAPABILITY_ALREADY_EXTRACTED` | `account` has already delegated/extracted its `DiemAccount::KeyRotationCapability`.                        |
    /// | `Errors::INVALID_ARGUMENT` | `DiemAccount::EMALFORMED_AUTHENTICATION_KEY`              | `new_key` was an invalid length.                                                                           |
    ///
    /// # Related Scripts
    /// * `AccountAdministrationScripts::rotate_authentication_key`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_nonce`
    /// * `AccountAdministrationScripts::rotate_authentication_key_with_recovery_address`

    public(script) fun rotate_authentication_key_with_nonce_admin(dr_account: signer, account: signer, sliding_nonce: u64, new_key: vector<u8>) {
        SlidingNonce::record_nonce_or_abort(&dr_account, sliding_nonce);
        let key_rotation_capability = DiemAccount::extract_key_rotation_capability(&account);
        DiemAccount::rotate_authentication_key(&key_rotation_capability, new_key);
        DiemAccount::restore_key_rotation_capability(key_rotation_capability);
    }
    spec rotate_authentication_key_with_nonce_admin {
        use Std::Signer;
        use Std::Errors;
        use DiemFramework::Roles;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        let account_addr = Signer::spec_address_of(account);
        include SlidingNonce::RecordNonceAbortsIf{ account: dr_account, seq_nonce: sliding_nonce };
        include DiemAccount::ExtractKeyRotationCapabilityAbortsIf;
        let key_rotation_capability = DiemAccount::spec_get_key_rotation_cap(account_addr);
        include DiemAccount::RotateAuthenticationKeyAbortsIf{cap: key_rotation_capability, new_authentication_key: new_key};

        /// This rotates the authentication key of `account` to `new_key`
        include DiemAccount::RotateAuthenticationKeyEnsures{addr: account_addr, new_authentication_key: new_key};

        aborts_with [check]
            Errors::INVALID_ARGUMENT,
            Errors::INVALID_STATE,
            Errors::NOT_PUBLISHED;

        /// **Access Control:**
        /// Only the Diem Root account can process the admin scripts [[H9]][PERMISSION].
        requires Roles::has_diem_root_role(dr_account); /// This is ensured by DiemAccount::writeset_prologue.
        /// The account can rotate its own authentication key unless
        /// it has delegrated the capability [[H18]][PERMISSION][[J18]][PERMISSION].
        include DiemAccount::AbortsIfDelegatedKeyRotationCapability{account: account};
    }

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
    /// # Events
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
    /// | `account` | `signer`     | Signer of the sending account of the transaction.                         |
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
    /// * `AccountCreationScripts::create_parent_vasp_account`
    /// * `AccountCreationScripts::create_designated_dealer`
    /// * `AccountAdministrationScripts::rotate_dual_attestation_info`

    public(script) fun rotate_dual_attestation_info(account: signer, new_url: vector<u8>, new_key: vector<u8>) {
        DualAttestation::rotate_base_url(&account, new_url);
        DualAttestation::rotate_compliance_public_key(&account, new_key)
    }
    spec rotate_dual_attestation_info {
        use Std::Errors;
        use DiemFramework::DiemAccount;
        use Std::Signer;

        include DiemAccount::TransactionChecks{sender: account}; // properties checked by the prologue.
        include DualAttestation::RotateBaseUrlAbortsIf;
        include DualAttestation::RotateBaseUrlEnsures;
        include DualAttestation::RotateCompliancePublicKeyAbortsIf;
        include DualAttestation::RotateCompliancePublicKeyEnsures;

        aborts_with [check]
            Errors::NOT_PUBLISHED,
            Errors::INVALID_ARGUMENT;

        include DualAttestation::RotateBaseUrlEmits;
        include DualAttestation::RotateCompliancePublicKeyEmits;

        /// **Access Control:**
        /// Only the account having Credential can rotate the info.
        /// Credential is granted to either a Parent VASP or a designated dealer [[H17]][PERMISSION].
        include DualAttestation::AbortsIfNoCredential{addr: Signer::spec_address_of(account)};
    }
}
