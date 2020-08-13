address 0x1 {

module DualAttestation {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LBR::LBR;
    use 0x1::LCS;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
    use 0x1::Signature;
    use 0x1::Signer;
    use 0x1::VASP;
    use 0x1::Vector;

    /// This resource holds an entity's globally unique name and all of the metadata it needs to
    /// participate in off-chain protocols.
    resource struct Credential {
        /// The human readable name of this entity. Immutable.
        human_name: vector<u8>,
        /// The base_url holds the URL to be used for off-chain communication. This contains the
        /// entire URL (e.g. https://...). Mutable.
        base_url: vector<u8>,
        /// 32 byte Ed25519 public key whose counterpart must be used to sign
        /// (1) the payment metadata for on-chain transactions that require dual attestation (e.g.,
        ///     transactions subject to the travel rule)
        /// (2) information exchanged in the off-chain protocols (e.g., KYC info in the travel rule
        ///     protocol)
        /// Note that this is different than `authentication_key` used in LibraAccount, which is
        /// a hash of a public key + signature scheme identifier, not a public key. Mutable.
        compliance_public_key: vector<u8>,
        /// Expiration date in microseconds from unix epoch. For V1, it is always set to
        /// U64_MAX. Mutable, but only by LibraRoot.
        expiration_date: u64,
    }

    /// Struct to store the limit on-chain
    resource struct Limit {
        micro_lbr_limit: u64,
    }

    const MAX_U64: u128 = 18446744073709551615;

    // Error codes
    /// A credential is not or already published.
    const ECREDENTIAL: u64 = 0;
    /// A limit is not or already published.
    const ELIMIT: u64 = 1;
    /// Cannot parse this as an ed25519 public key
    const EINVALID_PUBLIC_KEY: u64 = 2;
    /// Cannot parse this as an ed25519 signature (e.g., != 64 bytes)
    const EMALFORMED_METADATA_SIGNATURE: u64 = 3;
    /// Signature does not match message and public key
    const EINVALID_METADATA_SIGNATURE: u64 = 4;
    /// The recipient of a dual attestation payment needs to set a compliance public key
    const EPAYEE_COMPLIANCE_KEY_NOT_SET: u64 = 5;

    /// Value of the dual attestation limit at genesis
    const INITIAL_DUAL_ATTESTATION_LIMIT: u64 = 1000;
    /// Suffix of every signed dual attestation message
    const DOMAIN_SEPARATOR: vector<u8> = b"@@$$LIBRA_ATTEST$$@@";
    /// A year in microseconds
    const ONE_YEAR: u64 = 31540000000000;
    const U64_MAX: u64 = 18446744073709551615;

    /// Publish a `Credential` resource with name `human_name` under `created` with an empty
    /// `base_url` and `compliance_public_key`. Before receiving any dual attestation payments,
    /// the `created` account must send a transaction that invokes `rotate_base_url` and
    /// `rotate_compliance_public_key` to set these fields to a valid URL/public key.
    public fun publish_credential(
        created: &signer,
        creator: &signer,
        human_name: vector<u8>,
    ) {
        Roles::assert_parent_vasp_or_designated_dealer(created);
        Roles::assert_libra_root_or_treasury_compliance(creator);
        assert(
            !exists<Credential>(Signer::address_of(created)),
            Errors::already_published(ECREDENTIAL)
        );
        move_to(created, Credential {
            human_name,
            base_url: Vector::empty(),
            compliance_public_key: Vector::empty(),
            // For testnet and V1, so it should never expire. So set to u64::MAX
            expiration_date: U64_MAX,
        })
    }
    spec fun publish_credential {
        /// The permission "RotateDualAttestationInfo" is granted to ParentVASP, DesignatedDealer [B26].
        include Roles::AbortsIfNotParentVaspOrDesignatedDealer{account: created};
        include Roles::AbortsIfNotLibraRootOrTreasuryCompliance{account: creator};
        aborts_if exists<Credential>(Signer::spec_address_of(created)) with Errors::ALREADY_PUBLISHED;
    }

    /// Rotate the base URL for `account` to `new_url`
    public fun rotate_base_url(account: &signer, new_url: vector<u8>) acquires Credential {
        let addr = Signer::address_of(account);
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        borrow_global_mut<Credential>(addr).base_url = new_url
    }
    spec fun rotate_base_url {
        include AbortsIfNoCredential{addr: Signer::spec_address_of(account)};
        ensures
            global<Credential>(Signer::spec_address_of(account)).base_url == new_url;
    }
    spec schema AbortsIfNoCredential {
        addr: address;
        aborts_if !exists<Credential>(addr) with Errors::NOT_PUBLISHED;
    }

    /// Rotate the compliance public key for `account` to `new_key`.
    public fun rotate_compliance_public_key(
        account: &signer,
        new_key: vector<u8>,
    ) acquires Credential {
        let addr = Signer::address_of(account);
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        assert(Signature::ed25519_validate_pubkey(copy new_key), Errors::invalid_argument(EINVALID_PUBLIC_KEY));
        borrow_global_mut<Credential>(addr).compliance_public_key = new_key
    }
    spec fun rotate_compliance_public_key {
        include AbortsIfNoCredential{addr: Signer::spec_address_of(account)};
        aborts_if !Signature::ed25519_validate_pubkey(new_key) with Errors::INVALID_ARGUMENT;
        ensures global<Credential>(Signer::spec_address_of(account)).compliance_public_key
             == new_key;
    }

    /// Return the human-readable name for the VASP account.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun human_name(addr: address): vector<u8> acquires Credential {
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        *&borrow_global<Credential>(addr).human_name
    }
    spec fun human_name {
        pragma opaque;
        include AbortsIfNoCredential;
        ensures result == global<Credential>(addr).human_name;
    }

    /// Return the base URL for `addr`.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun base_url(addr: address): vector<u8> acquires Credential {
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        *&borrow_global<Credential>(addr).base_url
    }
    spec fun base_url {
        pragma opaque;
        include AbortsIfNoCredential;
        ensures result == global<Credential>(addr).base_url;
    }

    /// Return the compliance public key for `addr`.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun compliance_public_key(addr: address): vector<u8> acquires Credential {
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        *&borrow_global<Credential>(addr).compliance_public_key
    }
    spec fun compliance_public_key {
        pragma opaque;
        include AbortsIfNoCredential;
        ensures result == spec_compliance_public_key(addr);
    }
    /// Spec version of `Self::compliance_public_key`.
    spec define spec_compliance_public_key(addr: address): vector<u8> {
        global<Credential>(addr).compliance_public_key
    }

    /// Return the expiration date `addr
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun expiration_date(addr: address): u64  acquires Credential {
        assert(exists<Credential>(addr), Errors::not_published(ECREDENTIAL));
        *&borrow_global<Credential>(addr).expiration_date
    }
    spec fun expiration_date {
        pragma opaque;
        include AbortsIfNoCredential;
        ensures result == global<Credential>(addr).expiration_date;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Dual attestation requirements and checking
    ///////////////////////////////////////////////////////////////////////////

    /// Return the address where the credentials for `addr` are stored
    fun credential_address(addr: address): address {
        if (VASP::is_child(addr)) VASP::parent_address(addr) else addr
    }
    spec fun credential_address {
        pragma opaque;
        aborts_if false;
        ensures result == spec_credential_address(addr);
    }
    spec define spec_credential_address(addr: address): address {
        if (VASP::is_child(addr)) {
            VASP::spec_parent_address(addr)
        } else {
            addr
        }
    }

    /// Helper which returns true if dual attestion is required for a deposit.
    fun dual_attestation_required<Token>(
        payer: address, payee: address, deposit_value: u64
    ): bool acquires Limit {
        // travel rule applies for payments over a limit
        let travel_rule_limit_microlibra = get_cur_microlibra_limit();
        let approx_lbr_microlibra_value = Libra::approx_lbr_for_value<Token>(deposit_value);
        let above_limit = approx_lbr_microlibra_value >= travel_rule_limit_microlibra;
        if (!above_limit) {
            return false
        };
        // self-deposits never require dual attestation
        if (payer == payee) {
            return false
        };
        // dual attestation is required if the amount is above the limit AND between distinct
        // VASPs
        VASP::is_vasp(payer) && VASP::is_vasp(payee) &&
            VASP::parent_address(payer) != VASP::parent_address(payee)
    }
    spec fun dual_attestation_required {
        pragma opaque = true;
        include DualAttestationRequiredAbortsIf<Token>;
        ensures result == spec_dual_attestation_required<Token>(payer, payee, deposit_value);
    }
    spec schema DualAttestationRequiredAbortsIf<Token> {
        deposit_value: num;
        include Libra::ApproxLbrForValueAbortsIf<Token>{from_value: deposit_value};
        aborts_if !spec_is_published() with Errors::NOT_PUBLISHED;
    }
    spec module {
        define spec_is_inter_vasp(payer: address, payee: address): bool {
            VASP::is_vasp(payer) && VASP::is_vasp(payee)
                && VASP::spec_parent_address(payer) != VASP::spec_parent_address(payee)
        }
        /// Helper functions which simulates `Self::dual_attestation_required`.
        define spec_dual_attestation_required<Token>(
            payer: address, payee: address, deposit_value: u64
        ): bool {
            Libra::spec_approx_lbr_for_value<Token>(deposit_value)
                    >= spec_get_cur_microlibra_limit() &&
            payer != payee &&
            spec_is_inter_vasp(payer, payee)
        }
    }

    /// Helper to construct a message for dual attestation.
    /// Message is `metadata` | `payer` | `amount` | `DOMAIN_SEPARATOR`.
    fun dual_attestation_message(
        payer: address, metadata: vector<u8>, deposit_value: u64
    ): vector<u8> {
        let message = metadata;
        Vector::append(&mut message, LCS::to_bytes(&payer));
        Vector::append(&mut message, LCS::to_bytes(&deposit_value));
        Vector::append(&mut message, DOMAIN_SEPARATOR);
        message
    }
    spec fun dual_attestation_message {
        /// Abstract from construction of message for the prover. Concatenation of results from `LCS::to_bytes`
        /// are difficult to reason about, so we avoid doing it. This is possible because the actual value of this
        /// message is not important for the verification problem, as long as the prover considers both
        /// messages which fail verification and which do not.
        pragma opaque = true, verify = false;
        aborts_if false;
        ensures result == spec_dual_attestation_message(payer, metadata, deposit_value);
    }
    /// Uninterpreted function for `Self::dual_attestation_message`.
    spec define spec_dual_attestation_message(payer: address, metadata: vector<u8>, deposit_value: u64): vector<u8>;

    /// Helper function to check validity of a signature when dual attestion is required.
    fun assert_signature_is_valid(
        payer: address,
        payee: address,
        metadata_signature: vector<u8>,
        metadata: vector<u8>,
        deposit_value: u64
    ) acquires Credential {
        // sanity check of signature validity
        assert(
            Vector::length(&metadata_signature) == 64,
            Errors::invalid_argument(EMALFORMED_METADATA_SIGNATURE)
        );
        // sanity check of payee compliance key validity
        let payee_compliance_key = compliance_public_key(credential_address(payee));
        assert(
            !Vector::is_empty(&payee_compliance_key),
            Errors::invalid_state(EPAYEE_COMPLIANCE_KEY_NOT_SET)
        );
        // cryptographic check of signature validity
        let message = dual_attestation_message(payer, metadata, deposit_value);
        assert(
            Signature::ed25519_verify(metadata_signature, payee_compliance_key, message),
            Errors::invalid_argument(EINVALID_METADATA_SIGNATURE),
        );
    }
    spec fun assert_signature_is_valid {
        pragma opaque = true;
        include AssertSignatureValidAbortsIf;
    }
    spec schema AssertSignatureValidAbortsIf {
        payer: address;
        payee: address;
        metadata_signature: vector<u8>;
        metadata: vector<u8>;
        deposit_value: u64;
        aborts_if !exists<Credential>(spec_credential_address(payee)) with Errors::NOT_PUBLISHED;
        aborts_if Vector::is_empty(spec_compliance_public_key(spec_credential_address(payee))) with Errors::INVALID_STATE;
        aborts_if !spec_signature_is_valid(payer, payee, metadata_signature, metadata, deposit_value)
            with Errors::INVALID_ARGUMENT;
    }

    /// Returns true if signature is valid.
    spec define spec_signature_is_valid(
        payer: address,
        payee: address,
        metadata_signature: vector<u8>,
        metadata: vector<u8>,
        deposit_value: u64
    ): bool {
        let payee_compliance_key = spec_compliance_public_key(spec_credential_address(payee));
        len(metadata_signature) == 64 &&
            !Vector::is_empty(payee_compliance_key) &&
            Signature::ed25519_verify(
                metadata_signature,
                payee_compliance_key,
                spec_dual_attestation_message(payer, metadata, deposit_value)
            )
    }

    /// Public API for checking whether a payment of `value` coins of type `Currency`
    /// from `payer` to `payee` has a valid dual attestation. This returns without aborting if
    /// (1) dual attestation is not required for this payment, or
    /// (2) dual attestation is required, and `metadata_signature` can be verified on the message
    ///     `metadata` | `payer` | `value` | `DOMAIN_SEPARATOR` using the `compliance_public_key`
    ///     published in `payee`'s `Credential` resource
    /// It aborts with an appropriate error code if dual attestation is required, but one or more of
    /// the conditions in (2) is not met.
    public fun assert_payment_ok<Currency>(
        payer: address,
        payee: address,
        value: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires Credential, Limit {
        if (!Vector::is_empty(&metadata_signature) || // allow opt-in dual attestation
            dual_attestation_required<Currency>(payer, payee, value)
        ) {
          assert_signature_is_valid(payer, payee, metadata_signature, metadata, value)
        }
    }
    spec fun assert_payment_ok {
        pragma opaque;
        include AssertPaymentOkAbortsIf<Currency>;
    }
    spec schema AssertPaymentOkAbortsIf<Currency> {
        payer: address;
        payee: address;
        value: u64;
        metadata: vector<u8>;
        metadata_signature: vector<u8>;
        include len(metadata_signature) == 0 ==> DualAttestationRequiredAbortsIf<Currency>{deposit_value: value};
        include (len(metadata_signature) != 0 || spec_dual_attestation_required<Currency>(payer, payee, value))
            ==> AssertSignatureValidAbortsIf{deposit_value: value};
    }

    ///////////////////////////////////////////////////////////////////////////
    // Creating and updating dual attestation limit
    ///////////////////////////////////////////////////////////////////////////

    /// Travel rule limit set during genesis
    public fun initialize(lr_account: &signer) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        assert(!exists<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::already_published(ELIMIT));
        let initial_limit = (INITIAL_DUAL_ATTESTATION_LIMIT as u128) * (Libra::scaling_factor<LBR>() as u128);
        assert(initial_limit <= MAX_U64, Errors::limit_exceeded(ELIMIT));
        move_to(
            lr_account,
            Limit {
                micro_lbr_limit: (initial_limit as u64)
            }
        )
    }
    spec fun initialize {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        aborts_if exists<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()) with Errors::ALREADY_PUBLISHED;
        let initial_limit = INITIAL_DUAL_ATTESTATION_LIMIT * Libra::spec_scaling_factor<LBR>();
        aborts_if initial_limit > MAX_U64 with Errors::LIMIT_EXCEEDED;
        include Libra::AbortsIfNoCurrency<LBR>; // for scaling_factor.
    }

    /// Return the current dual attestation limit in microlibra
    public fun get_cur_microlibra_limit(): u64 acquires Limit {
        assert(exists<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::not_published(ELIMIT));
        borrow_global<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit
    }
    spec fun get_cur_microlibra_limit {
        pragma opaque;
        aborts_if !spec_is_published() with Errors::NOT_PUBLISHED;
        ensures result == spec_get_cur_microlibra_limit();
    }

    /// Set the dual attestation limit to `micro_libra_limit`.
    /// Aborts if `tc_account` does not have the TreasuryCompliance role
    public fun set_microlibra_limit(tc_account: &signer, micro_lbr_limit: u64) acquires Limit {
        Roles::assert_treasury_compliance(tc_account);
        assert(exists<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()), Errors::not_published(ELIMIT));
        borrow_global_mut<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit = micro_lbr_limit;
    }
    spec fun set_microlibra_limit {
        include Roles::AbortsIfNotTreasuryCompliance{account: tc_account};
        aborts_if !spec_is_published() with Errors::NOT_PUBLISHED;
        ensures global<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit == micro_lbr_limit;
    }

    // **************************** SPECIFICATION ********************************

    /// The Limit resource should be published after genesis
    spec module {
        invariant [global] LibraTimestamp::is_operating() ==> spec_is_published();
    }

    spec module {
        pragma verify = true;

        /// Helper function to determine whether the Limit is published.
        define spec_is_published(): bool {
            exists<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS())
        }

        /// Mirrors `Self::get_cur_microlibra_limit`.
        define spec_get_cur_microlibra_limit(): u64 {
            global<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit
        }
    }
}
}
