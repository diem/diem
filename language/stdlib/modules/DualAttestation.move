address 0x1 {

module DualAttestation {
    use 0x1::DualAttestationLimit;
    use 0x1::LCS;
    use 0x1::Libra;
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
    use 0x1::Signature;
    use 0x1::Signer;
    use 0x1::VASP;
    use 0x1::Vector;

    /// This resource holds an entity's globally unique name and all of the metadata it needs to
    /// participate if off-chain protocols.
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
        /// Note that this is different than `authentication_key` used in LibraAccount::T, which is
        /// a hash of a public key + signature scheme identifier, not a public key. Mutable.
        compliance_public_key: vector<u8>,
        /// Expiration date in microseconds from unix epoch. For V1, it is always set to
        /// U64_MAX. Mutable, but only by LibraRoot.
        expiration_date: u64,
    }

    // Error codes
    const ENOT_PARENT_VASP_OR_DD: u64 = 0;
    /// Transaction not signed by LIBRA_ROOT
    const ENOT_LIBRA_ROOT: u64 = 1;
    /// Cannot parse this as an ed25519 public key
    const EINVALID_PUBLIC_KEY: u64 = 2;
    /// Cannot parse this as an ed25519 signature (e.g., != 64 bytes)
    const EMALFORMED_METADATA_SIGNATURE: u64 = 4;
    /// Signature does not match message and public key
    const EINVALID_METADATA_SIGNATURE: u64 = 5;

    /// suffix of every signed dual attestation message
    const DOMAIN_SEPARATOR: vector<u8> = b"@@$$LIBRA_ATTEST$$@@";
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
        assert(Roles::has_libra_root_role(creator), ENOT_LIBRA_ROOT);
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
        credential.expiration_date = LibraTimestamp::now_microseconds() + cert_lifetime();
    }
    spec fun recertify {
        aborts_if !LibraTimestamp::root_ctm_initialized();
        aborts_if LibraTimestamp::spec_now_microseconds() + spec_cert_lifetime() > max_u64();
        ensures credential.expiration_date
             == LibraTimestamp::spec_now_microseconds() + spec_cert_lifetime();
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

    /// A year in microseconds
    fun cert_lifetime(): u64 {
        31540000000000
    }
    spec module {
        define spec_cert_lifetime(): u64 {
            31540000000000
        }
    }



    ///////////////////////////////////////////////////////////////////////////
    // Dual attestation requirements and checking
    ///////////////////////////////////////////////////////////////////////////

    /// Helper which returns true if dual attestion is required for a deposit.
    public fun dual_attestation_required<Token>(
        payer: address, payee: address, deposit_value: u64
    ): bool {
        // travel rule applies for payments over a threshold
        let travel_rule_limit_microlibra = DualAttestationLimit::get_cur_microlibra_limit();
        let approx_lbr_microlibra_value = Libra::approx_lbr_for_value<Token>(deposit_value);
        let above_threshold = approx_lbr_microlibra_value >= travel_rule_limit_microlibra;
        // travel rule applies if the payer and payee are both VASP, but not for
        // intra-VASP transactions
        above_threshold
            && VASP::is_vasp(payer) && VASP::is_vasp(payee)
            && VASP::parent_address(payer) != VASP::parent_address(payee)
    }
    spec fun dual_attestation_required {
        pragma verify = true, opaque = true;
        include TravelRuleAppliesAbortsIf<Token>;
        ensures result == spec_dual_attestation_required<Token>(payer, payee, deposit_value);
    }
    spec schema TravelRuleAppliesAbortsIf<Token> {
        aborts_if !Libra::spec_is_currency<Token>();
        aborts_if !DualAttestationLimit::spec_is_published();
    }
    spec module {
        /// Helper functions which simulates `Self::dual_attestation_required`.
        define spec_dual_attestation_required<Token>(payer: address, payee: address, deposit_value: u64): bool {
            Libra::spec_approx_lbr_for_value<Token>(deposit_value)
                    >= DualAttestationLimit::spec_get_cur_microlibra_limit()
            && VASP::spec_is_vasp(payer) && VASP::spec_is_vasp(payee)
            && VASP::spec_parent_address(payer) != VASP::spec_parent_address(payee)
        }
    }

    /// Helper to construct a message for dual attestation.
    /// Message is `metadata` | `payer` | `amount` | `DOMAIN_SEPARATOR`.
    public fun dual_attestation_message(
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
    public fun assert_signature_is_valid(
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
                compliance_public_key(payee),
                message
            ),
            EINVALID_METADATA_SIGNATURE
        );
    }
    spec fun assert_signature_is_valid {
        pragma verify = true, opaque = true;
        aborts_if !exists<Credential>(payee);
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
                        spec_compliance_public_key(payee),
                        spec_dual_attestation_message(payer, metadata, deposit_value)
                   )
        }
    }

    /// Public API for checking whether a payment of `value_microlibra` coins of type `Currency`
    /// from `payer` to `payee` has a valid dual attestation. This returns without aborting if
    /// (1) dual attestation is not required for this payment, or
    /// (2) dual attestation is required, and `metadata_signature` can be verified on the message
    ///     `metadata` | `payer` | `amount` | `DOMAIN_SEPARATOR` using the `compliance_public_key`
    ///     published in `payee`'s `Credential` resource
    /// It aborts with an appropriate error code if dual attestation is required, but one or more of
    /// the conditions in (2) is not met.
    public fun assert_payment_ok<Currency>(
        payer: address,
        payee: address,
        value_microlibra: u64,
        metadata: vector<u8>,
        metadata_signature: vector<u8>
    ) acquires Credential {
        if (dual_attestation_required<Currency>(payer, payee, value_microlibra)) {
          assert_signature_is_valid(payer, payee, metadata_signature, metadata, value_microlibra)
        }
    }

}
}
