// Error codes:
// 7000 -> INSUFFICIENT_PRIVILEGES
// 7001 -> INVALID_PARENT_VASP_ACCOUNT
// 7002 -> INVALID_CHILD_VASP_ACCOUNT
// 7003 -> CHILD_ACCOUNT_STILL_PARENT
// 7004 -> INVALID_PUBLIC_KEY
address 0x1 {

module VASP {
    use 0x1::Association;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Signature;

    // A ParentVASP is held only by the root VASP account and holds the
    // VASP-related metadata for the account. It is subject to a time
    // limitation, and needs to be re-certified by an association
    // account before the certification expires.
    resource struct ParentVASP {
        // The human readable name of this VASP.
        human_name: vector<u8>,
        // The base_url holds the URL to be used for off-chain
        // communication. This contains the whole URL (e.g. https://...).
        base_url: vector<u8>,
        // Expiration date in microseconds from unix epoch. Mutable, but only by Association
        expiration_date: u64,
        // 32 byte Ed25519 public key whose counterpart must be used to sign
        // (1) the payment metadata for on-chain travel rule transactions
        // (2) the KYC information exchanged in the off-chain travel rule protocol.
        // Note that this is different than `authentication_key` used in LibraAccount::T, which is
        // a hash of a public key + signature scheme identifier, not a public key. Mutable
        compliance_public_key: vector<u8>,
    }

    // A ChildVASP type for representing `AccountType<ChildVASP>` accounts.
    // The parent VASP can create as many of these as they wish, and them
    // being a VASP account is dependent on the continuing certification of
    // the parent VASP account by the association.
    resource struct ChildVASP { parent_vasp_addr: address }

    ///////////////////////////////////////////////////////////////////////////
    // Association called functions for parent VASP accounts
    ///////////////////////////////////////////////////////////////////////////

    // Renew's `parent_vasp`'s certification
    public fun recertify_vasp(parent_vasp: &mut ParentVASP) {
        parent_vasp.expiration_date = LibraTimestamp::now_microseconds() + cert_lifetime();
    }

    // Non-destructively decertify `parent_vasp`. Can be
    // recertified later on via `recertify_vasp`.
    public fun decertify_vasp(parent_vasp: &mut ParentVASP) {
        // Expire the parent credential.
        parent_vasp.expiration_date = 0;
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be parent-vasp called functions
    ///////////////////////////////////////////////////////////////////////////

    public fun publish_parent_vasp_credential(
        association: &signer,
        vasp: &signer,
        human_name: vector<u8>,
        base_url: vector<u8>,
        compliance_public_key: vector<u8>
    ) {
        Association::assert_is_association(association);
        assert(Signature::ed25519_validate_pubkey(copy compliance_public_key), 7004);
        move_to(
            vasp,
            ParentVASP {
                // For testnet, so it should never expire. So set to u64::MAX
                expiration_date: 18446744073709551615,
                human_name,
                base_url,
                compliance_public_key,
            }
        )
    }

    /// Create a child VASP resource for the `parent`
    /// Aborts if `parent` is not a ParentVASP
    public fun publish_child_vasp_credential(parent: &signer, child: &signer) {
        let parent_vasp_addr = Signer::address_of(parent);
        assert(exists<ParentVASP>(parent_vasp_addr), 7000);
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
    public fun human_name(addr: address): vector<u8>  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).human_name
    }

    /// Return the base URL for the VASP account
    public fun base_url(addr: address): vector<u8>  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).base_url
    }

    /// Return the compliance public key for the VASP account
    public fun compliance_public_key(addr: address): vector<u8> acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).compliance_public_key
    }

    /// Return the expiration date for the VASP account
    public fun expiration_date(addr: address): u64  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).expiration_date
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

    // A year in microseconds
    fun cert_lifetime(): u64 {
        31540000000000
    }
}

}
