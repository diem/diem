address 0x1 {

module DualAttestation {
    use 0x1::CoreAddresses;
    use 0x1::DesignatedDealer;
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

    // Error codes
    /// Attempting to initialize the dual attestation limit after genesis
    const ENOT_GENESIS: u64 = 0;
    /// Calling a privileged function from an account without the LibraRoot role
    const EACCOUNT_NOT_LIBRA_ROOT: u64 = 1;
    /// Attempting to publish a `Credential` resource under an account without the ParentVASP or
    /// DesignatedDealer role
    const ENOT_PARENT_VASP_OR_DD: u64 = 2;
    /// Transaction not signed by LibraRoot or TreasuryCompliance
    const ENOT_LIBRA_ROOT_OR_TREASURY_COMPLIANCE: u64 = 3;
    /// Attempting to update the limit from an account that does not contain the
    /// `ModifyLimitCapability` resource
    const ECANNOT_UPDATE_LIMIT: u64 = 4;
    /// Cannot parse this as an ed25519 public key
    const EINVALID_PUBLIC_KEY: u64 = 5;
    /// Cannot parse this as an ed25519 signature (e.g., != 64 bytes)
    const EMALFORMED_METADATA_SIGNATURE: u64 = 6;
    /// Signature does not match message and public key
    const EINVALID_METADATA_SIGNATURE: u64 = 7;

    /// Value of the dual attestation limit at genesis
    const INITIAL_DUAL_ATTESTATION_LIMIT: u64 = 1000;
    /// Suffix of every signed dual attestation message
    const DOMAIN_SEPARATOR: vector<u8> = b"@@$$LIBRA_ATTEST$$@@";
    /// A year in microseconds
    const ONE_YEAR: u64 = 31540000000000;
    const U64_MAX: u64 = 18446744073709551615;

    public fun publish_credential(
        created: &signer,
        creator: &signer,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>,
    ) {
        assert(
            Roles::has_parent_VASP_role(created) || Roles::has_designated_dealer_role(created),
            ENOT_PARENT_VASP_OR_DD
        );
        assert(
            Roles::has_libra_root_role(creator) || Roles::has_treasury_compliance_role(creator),
            ENOT_LIBRA_ROOT_OR_TREASURY_COMPLIANCE
        );
        assert(Signature::ed25519_validate_pubkey(copy compliance_public_key), EINVALID_PUBLIC_KEY);
        move_to(created, Credential {
            human_name,
            base_url,
            compliance_public_key,
            // For testnet and V1, so it should never expire. So set to u64::MAX
            expiration_date: U64_MAX,
        })
    }

    /// Rotate the base URL for `account` to `new_url`
    public fun rotate_base_url(account: &signer, new_url: vector<u8>) acquires Credential {
        borrow_global_mut<Credential>(Signer::address_of(account)).base_url = new_url
    }
    spec fun rotate_base_url {
        aborts_if !exists<Credential>(Signer::spec_address_of(account));
        ensures
            global<Credential>(Signer::spec_address_of(account)).base_url == new_url;
    }

    /// Rotate the compliance public key for `account` to `new_key`.
    public fun rotate_compliance_public_key(
        account: &signer,
        new_key: vector<u8>,
    ) acquires Credential {
        assert(Signature::ed25519_validate_pubkey(copy new_key), EINVALID_PUBLIC_KEY);
        let parent_addr = Signer::address_of(account);
        borrow_global_mut<Credential>(parent_addr).compliance_public_key = new_key
    }
    spec fun rotate_compliance_public_key {
        aborts_if !exists<Credential>(Signer::spec_address_of(account));
        aborts_if !Signature::spec_ed25519_validate_pubkey(new_key);
        ensures global<Credential>(Signer::spec_address_of(account)).compliance_public_key
             == new_key;
    }

    /// Return the human-readable name for the VASP account.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun human_name(addr: address): vector<u8> acquires Credential {
        *&borrow_global<Credential>(addr).human_name
    }

    /// Return the base URL for `addr`.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun base_url(addr: address): vector<u8> acquires Credential {
        *&borrow_global<Credential>(addr).base_url
    }

    /// Return the compliance public key for `addr`.
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun compliance_public_key(addr: address): vector<u8> acquires Credential {
        *&borrow_global<Credential>(addr).compliance_public_key
    }
    spec fun compliance_public_key {
        aborts_if !exists<Credential>(addr);
        ensures result == spec_compliance_public_key(addr);
    }
    spec module {
        /// Spec version of `Self::compliance_public_key`.
        define spec_compliance_public_key(addr: address): vector<u8> {
            global<Credential>(addr).compliance_public_key
        }
    }

    /// Return the expiration date `addr
    /// Aborts if `addr` does not have a `Credential` resource.
    public fun expiration_date(addr: address): u64  acquires Credential {
        *&borrow_global<Credential>(addr).expiration_date
    }

    ///////////////////////////////////////////////////////////////////////////
    // Certification and expiration
    ///////////////////////////////////////////////////////////////////////////


    /// Renew's `credential`'s certificate
    public fun recertify(credential: &mut Credential) {
        credential.expiration_date = LibraTimestamp::now_microseconds() + ONE_YEAR;
    }
    spec fun recertify {
        // TODO: use ONE_YEAR below once the spec language supports constants
        aborts_if !LibraTimestamp::root_ctm_initialized();
        aborts_if LibraTimestamp::spec_now_microseconds() + 31540000000000 > max_u64();
        ensures credential.expiration_date
             == LibraTimestamp::spec_now_microseconds() + 31540000000000;
    }

    /// Non-destructively decertify `credential`. Can be recertified later on via `recertify`.
    public fun decertify(credential: &mut Credential) {
        // Expire the parent credential.
        credential.expiration_date = 0;
    }
    spec fun decertify {
        aborts_if false;
        ensures credential.expiration_date == 0;
    }

    ///////////////////////////////////////////////////////////////////////////
    // Dual attestation requirements and checking
    ///////////////////////////////////////////////////////////////////////////

    /// Return the address where the credentials for `addr` are stored
    fun credential_address(addr: address): address {
        if (VASP::is_child(addr)) VASP::parent_address(addr) else addr
    }
    spec fun credential_address {
        aborts_if false;
        ensures result == spec_credential_address(addr);
    }
    spec module {
        define spec_credential_address(addr: address): address {
            if (VASP::spec_is_child_vasp(addr)) {
                VASP::spec_parent_address(addr)
            } else {
                addr
            }
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
        // entities. E.g.:
        // (1) inter-VASP
        // (2) inter-DD
        // (3) VASP -> DD
        // (4) DD -> VASP
        // We assume that any DD <-> DD payment is inter-DD because each DD has a single account
        let is_payer_vasp = VASP::is_vasp(payer);
        let is_payee_vasp = VASP::is_vasp(payee);
        let is_payer_dd = DesignatedDealer::exists_at(payer);
        let is_payee_dd = DesignatedDealer::exists_at(payee);
        let is_inter_vasp =
            is_payer_vasp && is_payee_vasp &&
            VASP::parent_address(payer) != VASP::parent_address(payee);
        is_inter_vasp || // (1) inter-VASP
            (is_payer_dd && is_payee_dd) || // (2) inter-DD
            (is_payer_vasp && is_payee_dd) || // (3) VASP -> DD
            (is_payer_dd && is_payee_vasp) // (4) DD -> VASP
    }
    spec fun dual_attestation_required {
        pragma opaque = true;
        include TravelRuleAppliesAbortsIf<Token>;
        ensures result == spec_dual_attestation_required<Token>(payer, payee, deposit_value);
    }
    spec schema TravelRuleAppliesAbortsIf<Token> {
        aborts_if !Libra::spec_is_currency<Token>();
        aborts_if !spec_is_published();
    }
    spec module {
        define spec_is_inter_vasp(payer: address, payee: address): bool {
            VASP::spec_is_vasp(payer) && VASP::spec_is_vasp(payee)
                && VASP::spec_parent_address(payer) != VASP::spec_parent_address(payee)
        }
        define spec_is_inter_dd(payer: address, payee: address): bool {
            DesignatedDealer::spec_exists_at(payer) && DesignatedDealer::spec_exists_at(payee)
        }
        define spec_is_vasp_to_dd(payer: address, payee: address): bool {
            VASP::spec_is_vasp(payer) && DesignatedDealer::spec_exists_at(payee)
        }
        define spec_is_dd_to_vasp(payer: address, payee: address): bool {
            DesignatedDealer::spec_exists_at(payer) && VASP::spec_is_vasp(payee)
        }
        /// Helper functions which simulates `Self::dual_attestation_required`.
        define spec_dual_attestation_required<Token>(
            payer: address, payee: address, deposit_value: u64
        ): bool {
            Libra::spec_approx_lbr_for_value<Token>(deposit_value)
                    >= spec_get_cur_microlibra_limit() &&
            payer != payee &&
            (spec_is_inter_vasp(payer, payee) ||
             spec_is_inter_dd(payer, payee) ||
             spec_is_vasp_to_dd(payer, payee) ||
             spec_is_dd_to_vasp(payer, payee))
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
        ensures result == spec_dual_attestation_message(payer, metadata, deposit_value);
    }
    spec module {
        /// Uninterpreted function for `Self::dual_attestation_message`.
        define spec_dual_attestation_message(payer: address, metadata: vector<u8>, deposit_value: u64): vector<u8>;
    }

    /// Helper function to check validity of a signature when dual attestion is required.
    fun assert_signature_is_valid(
        payer: address,
        payee: address,
        metadata_signature: vector<u8>,
        metadata: vector<u8>,
        deposit_value: u64
    ) acquires Credential {
        // sanity check of signature validity
        assert(Vector::length(&metadata_signature) == 64, EMALFORMED_METADATA_SIGNATURE);
        // cryptographic check of signature validity
        let message = dual_attestation_message(payer, metadata, deposit_value);
        assert(
            Signature::ed25519_verify(
                metadata_signature,
                compliance_public_key(credential_address(payee)),
                message
            ),
            EINVALID_METADATA_SIGNATURE
        );
    }
    spec fun assert_signature_is_valid {
        pragma opaque = true;
        aborts_if !exists<Credential>(spec_credential_address(payee));
        aborts_if !signature_is_valid(payer, payee, metadata_signature, metadata, deposit_value);
    }
    spec module {
        /// Returns true if signature is valid.
        define signature_is_valid(
            payer: address,
            payee: address,
            metadata_signature: vector<u8>,
            metadata: vector<u8>,
            deposit_value: u64
        ): bool {
            len(metadata_signature) == 64
                && Signature::spec_ed25519_verify(
                        metadata_signature,
                        spec_compliance_public_key(spec_credential_address(payee)),
                        spec_dual_attestation_message(payer, metadata, deposit_value)
                   )
        }
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
        if (dual_attestation_required<Currency>(payer, payee, value)) {
          assert_signature_is_valid(payer, payee, metadata_signature, metadata, value)
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Creating and updating dual attestation limit
    ///////////////////////////////////////////////////////////////////////////

    /// Travel rule limit set during genesis
    public fun initialize(lr_account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(
            Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(),
            EACCOUNT_NOT_LIBRA_ROOT
        );
        move_to(
            lr_account,
            Limit {
                micro_lbr_limit: INITIAL_DUAL_ATTESTATION_LIMIT * Libra::scaling_factor<LBR>()
            }
        )
    }

    /// Return the current dual attestation limit in microlibra
    public fun get_cur_microlibra_limit(): u64 acquires Limit {
        borrow_global<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit
    }
    spec fun get_cur_microlibra_limit {
        aborts_if !exists<Limit>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS());
        ensures result == spec_get_cur_microlibra_limit();
    }

    /// Set the dual attestation limit to `micro_libra_limit`.
    /// Aborts if `tc_account` does not have the TreasuryCompliance role
    public fun set_microlibra_limit(tc_account: &signer, micro_lbr_limit: u64) acquires Limit {
        assert(
            Roles::has_update_dual_attestation_limit_privilege(tc_account),
            ECANNOT_UPDATE_LIMIT
        );
        borrow_global_mut<Limit>(CoreAddresses::LIBRA_ROOT_ADDRESS()).micro_lbr_limit = micro_lbr_limit;
    }

    // **************************** SPECIFICATION ********************************

    /// The Limit resource should be published after genesis
    spec schema LimitExists {
        invariant !LibraTimestamp::spec_is_genesis() ==> spec_is_published();
    }

    spec module {
        pragma verify = true;

        /// Helper function to determine whether the Limit is published.
        define spec_is_published(): bool {
            exists<Limit>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS())
        }

        /// Mirrors `Self::get_cur_microlibra_limit`.
        define spec_get_cur_microlibra_limit(): u64 {
            global<Limit>(CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS()).micro_lbr_limit
        }

        // TODO(shb): why does verification of LimitExists fail for only these two procedures?
        apply LimitExists to * except get_cur_microlibra_limit, compliance_public_key;
    }

}
}
