// Error codes:
// 7000 -> INSUFFICIENT_PRIVILEGES
// 7001 -> INVALID_PARENT_VASP_ACCOUNT
// 7002 -> INVALID_CHILD_VASP_ACCOUNT
// 7003 -> CHILD_ACCOUNT_STILL_PARENT
// 7004 -> INVALID_PUBLIC_KEY
address 0x1 {

module VASP {
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Signature;
    use 0x1::Roles::{Capability, AssociationRootRole, ParentVASPRole};

    /// Each VASP has a unique root account that holds a `ParentVASP` resource. This resource holds
    /// the VASP's globally unique name and all of the metadata that other VASPs need to perform
    /// off-chain protocols with this one.
    resource struct ParentVASP {
        /// The human readable name of this VASP. Immutable.
        human_name: vector<u8>,
        /// The base_url holds the URL to be used for off-chain communication. This contains the
        /// entire URL (e.g. https://...). Mutable.
        base_url: vector<u8>,
        /// Expiration date in microseconds from unix epoch. For V1 VASPs, it is always set to
        /// U64_MAX. Mutable, but only by the Association.
        expiration_date: u64,
        /// 32 byte Ed25519 public key whose counterpart must be used to sign
        /// (1) the payment metadata for on-chain travel rule transactions
        /// (2) the KYC information exchanged in the off-chain travel rule protocol.
        /// Note that this is different than `authentication_key` used in LibraAccount::T, which is
        /// a hash of a public key + signature scheme identifier, not a public key. Mutable.
        compliance_public_key: vector<u8>,
        /// Number of child accounts this parent has created.
        num_children: u64
    }

    /// A resource that represents a child account of the parent VASP account at `parent_vasp_addr`
    resource struct ChildVASP { parent_vasp_addr: address }

    ///////////////////////////////////////////////////////////////////////////
    // Association called functions for parent VASP accounts
    ///////////////////////////////////////////////////////////////////////////

    /// Renew's `parent_vasp`'s certification
    public fun recertify_vasp(parent_vasp: &mut ParentVASP) {
        parent_vasp.expiration_date = LibraTimestamp::now_microseconds() + cert_lifetime();
    }

    /// Non-destructively decertify `parent_vasp`. Can be
    /// recertified later on via `recertify_vasp`.
    public fun decertify_vasp(parent_vasp: &mut ParentVASP) {
        // Expire the parent credential.
        parent_vasp.expiration_date = 0;
    }

    // A year in microseconds
    fun cert_lifetime(): u64 {
        31540000000000
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be parent-vasp called functions
    ///////////////////////////////////////////////////////////////////////////

    /// Create a new `ParentVASP` resource under `vasp`
    /// Aborts if `association` is not an Association account
    public fun publish_parent_vasp_credential(
        vasp: &signer,
        _: &Capability<AssociationRootRole>,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>
    ) {
        assert(Signature::ed25519_validate_pubkey(copy compliance_public_key), 7004);
        move_to(
            vasp,
            ParentVASP {
                // For testnet and V1, so it should never expire. So set to u64::MAX
                expiration_date: 18446744073709551615,
                human_name,
                base_url,
                compliance_public_key,
                num_children: 0
            }
        );
    }

    /// Create a child VASP resource for the `parent`
    /// Aborts if `parent` is not a ParentVASP
    public fun publish_child_vasp_credential(
        parent: &signer,
        child: &signer,
        _: &Capability<ParentVASPRole>,
    ) acquires ParentVASP {
        let parent_vasp_addr = Signer::address_of(parent);
        assert(exists<ParentVASP>(parent_vasp_addr), 7000);
        let num_children = &mut borrow_global_mut<ParentVASP>(parent_vasp_addr).num_children;
        *num_children = *num_children + 1;
        move_to(child, ChildVASP { parent_vasp_addr });
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs
    ///////////////////////////////////////////////////////////////////////////

    /// Return `addr` if `addr` is a `ParentVASP` or its parent's address if it is a `ChildVASP`
    /// Aborts otherwise
    public fun parent_address(addr: address): address acquires ChildVASP {
        if (exists<ParentVASP>(addr)) {
            addr
        } else if (exists<ChildVASP>(addr)) {
            borrow_global<ChildVASP>(addr).parent_vasp_addr
        } else { // wrong account type, abort
            abort(88)
        }
    }

    public fun is_parent(addr: address): bool {
        exists<ParentVASP>(addr)
    }

    public fun is_child(addr: address): bool {
        exists<ChildVASP>(addr)
    }

    public fun is_vasp(addr: address): bool {
        is_parent(addr) || is_child(addr)
    }

    /// Return the human-readable name for the VASP account
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun human_name(addr: address): vector<u8>  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).human_name
    }

    /// Return the base URL for the VASP account
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun base_url(addr: address): vector<u8>  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).base_url
    }

    /// Return the compliance public key for the VASP account
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun compliance_public_key(addr: address): vector<u8> acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).compliance_public_key
    }

    /// Return the expiration date for the VASP account
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun expiration_date(addr: address): u64  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).expiration_date
    }

    /// Return the number of child accounts for this VASP.
    /// The total number of accounts for this VASP is num_children() + 1
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun num_children(addr: address): u64  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).num_children
    }

    /// Rotate the base URL for the `parent_vasp` account to `new_url`
    public fun rotate_base_url(parent_vasp: &signer, new_url: vector<u8>) acquires ParentVASP {
        let parent_addr = Signer::address_of(parent_vasp);
        borrow_global_mut<ParentVASP>(parent_addr).base_url = new_url
    }

    /// Rotate the compliance public key for `parent_vasp` to `new_key`
    public fun rotate_compliance_public_key(
        parent_vasp: &signer,
        new_key: vector<u8>
    ) acquires ParentVASP {
        assert(Signature::ed25519_validate_pubkey(copy new_key), 7004);
        let parent_addr = Signer::address_of(parent_vasp);
        borrow_global_mut<ParentVASP>(parent_addr).compliance_public_key = new_key
    }

}

}
