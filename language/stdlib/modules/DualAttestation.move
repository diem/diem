address 0x1 {

module DualAttestation {
    use 0x1::LibraTimestamp;
    use 0x1::Roles;
    use 0x1::Signature;
    use 0x1::Signer;
//    use 0x1::VASP;

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

    const ENOT_PARENT_VASP_OR_DD: u64 = 0;
    const ENOT_LIBRA_ROOT: u64 = 1;
    const EINVALID_PUBLIC_KEY: u64 = 2;

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


}
}
