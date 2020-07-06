address 0x1 {

module VASP {
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;
    use 0x1::Signer;
    use 0x1::Roles;  // alias import does not play well with spec functions in Roles.
    use 0x1::Libra;
    use 0x1::AccountLimits::{Self, AccountLimitMutationCapability};

    /// Each VASP has a unique root account that holds a `ParentVASP` resource. This resource holds
    /// the VASP's globally unique name and all of the metadata that other VASPs need to perform
    /// off-chain protocols with this one.
    resource struct ParentVASP {
        /// Number of child accounts this parent has created.
        num_children: u64
    }

    /// A resource that represents a child account of the parent VASP account at `parent_vasp_addr`
    resource struct ChildVASP { parent_vasp_addr: address }

    /// A singleton resource allowing this module to publish limits definitions and accounting windows
    resource struct VASPOperationsResource { limits_cap: AccountLimitMutationCapability }

    const ENOT_GENESIS: u64 = 0;
    const ENOT_A_REGISTERED_CURRENCY: u64 = 1;
    const EINVALID_SINGLETON_ADDRESS: u64 = 2;
    const ENOT_LIBRA_ROOT: u64 = 3;
    const ENOT_A_PARENT_VASP: u64 = 4;
    const ENOT_A_VASP: u64 = 5;
    const EALREADY_A_VASP: u64 = 7;
    const ETOO_MANY_CHILDREN: u64 = 8;

    /// Maximum number of child accounts that can be created by a single ParentVASP
    const MAX_CHILD_ACCOUNTS: u64 = 256;

    public fun initialize(lr_account: &signer) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        assert(Signer::address_of(lr_account) == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_SINGLETON_ADDRESS);
        move_to(lr_account, VASPOperationsResource {
            limits_cap: AccountLimits::grant_mutation_capability(lr_account),
        })
    }

    ///////////////////////////////////////////////////////////////////////////
    // To-be parent-vasp called functions
    ///////////////////////////////////////////////////////////////////////////

    /// Create a new `ParentVASP` resource under `vasp`
    /// Aborts if `lr_account` is not the libra root account,
    /// or if there is already a VASP (child or parent) at this account.
    public fun publish_parent_vasp_credential(vasp: &signer, lr_account: &signer) {
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        let vasp_addr = Signer::address_of(vasp);
        assert(!is_vasp(vasp_addr), ENOT_A_VASP);
        move_to(vasp, ParentVASP { num_children: 0 });
    }
    spec fun publish_parent_vasp_credential {
        aborts_if !Roles::spec_has_libra_root_role(lr_account);
        aborts_if spec_is_vasp(Signer::spec_address_of(vasp));
        ensures spec_is_parent_vasp(Signer::spec_address_of(vasp));
        ensures spec_get_num_children(Signer::spec_address_of(vasp)) == 0;
    }

    /// Create a child VASP resource for the `parent`
    /// Aborts if `parent` is not a ParentVASP
    public fun publish_child_vasp_credential(
        parent: &signer,
        child: &signer,
    ) acquires ParentVASP {
        // DD: The spreadsheet does not have a "privilege" for creating
        // child VASPs. All logic in the code is based on the parent VASP role.
        // DD: Since it checks for a ParentVASP property, anyway, checking
        // for role might be a bit redundant (would need invariant that only
        // Parent Role has ParentVASP)
        assert(Roles::has_parent_VASP_role(parent), ENOT_A_PARENT_VASP);
        let child_vasp_addr = Signer::address_of(child);
        assert(!is_vasp(child_vasp_addr), EALREADY_A_VASP);
        let parent_vasp_addr = Signer::address_of(parent);
        assert(is_parent(parent_vasp_addr), ENOT_A_PARENT_VASP);
        let num_children = &mut borrow_global_mut<ParentVASP>(parent_vasp_addr).num_children;
        // Abort if creating this child account would put the parent VASP over the limit
        assert(*num_children < MAX_CHILD_ACCOUNTS, ETOO_MANY_CHILDREN);
        *num_children = *num_children + 1;
        move_to(child, ChildVASP { parent_vasp_addr });
    }
    spec fun publish_child_vasp_credential {
        aborts_if !Roles::spec_has_parent_VASP_role(parent);
        aborts_if spec_is_vasp(Signer::spec_address_of(child));
        aborts_if !spec_is_parent_vasp(Signer::spec_address_of(parent));
        aborts_if spec_get_num_children(Signer::spec_address_of(parent)) + 1 > 256; // MAX_CHILD_ACCOUNTS
        ensures spec_get_num_children(Signer::spec_address_of(parent))
             == old(spec_get_num_children(Signer::spec_address_of(parent))) + 1;
        ensures spec_is_child_vasp(Signer::spec_address_of(child));
        ensures TRACE(spec_parent_address(Signer::spec_address_of(child)))
             == TRACE(Signer::spec_address_of(parent));
    }

    /// If the account passed in is not a VASP account, this returns true since
    /// we don't need to ensure account limits exist for those accounts.
    /// If the account is a child VASP account, this returns true only if a `Window<CoinType>` is
    /// published in the parent's account.
    /// If the account is a child VASP account, this will always return true;
    /// either a `LimitsDefinition`/`Window` exist for `CoinType`, or these
    /// will be published under the account.
    public fun try_allow_currency<CoinType>(account: &signer): bool
    acquires ChildVASP, VASPOperationsResource {
        assert(Libra::is_currency<CoinType>(), ENOT_A_REGISTERED_CURRENCY);
        let account_address = Signer::address_of(account);
        if (!is_vasp(account_address)) return true;
        let parent_address = parent_address(account_address);
        if (AccountLimits::has_window_published<CoinType>(parent_address)) {
            true
        } else if (is_parent(account_address)) {
            let cap = &borrow_global<VASPOperationsResource>(CoreAddresses::LIBRA_ROOT_ADDRESS()).limits_cap;
            AccountLimits::publish_window<CoinType>(account, cap, CoreAddresses::LIBRA_ROOT_ADDRESS());
            true
        } else {
            // it's a child vasp, and we can't publish the limits definition under it.
            false
        }
    }

    ///////////////////////////////////////////////////////////////////////////
    // Publicly callable APIs
    ///////////////////////////////////////////////////////////////////////////

    /// Return `addr` if `addr` is a `ParentVASP` or its parent's address if it is a `ChildVASP`
    /// Aborts otherwise
    public fun parent_address(addr: address): address acquires ChildVASP {
        if (is_parent(addr)) {
            addr
        } else if (is_child(addr)) {
            borrow_global<ChildVASP>(addr).parent_vasp_addr
        } else { // wrong account type, abort
            abort(88)
        }
    }
    spec fun parent_address {
        pragma opaque = true;
        ensures result == spec_parent_address(addr);
    }
    spec module {
        /// Spec version of `Self::parent_address`.
        define spec_parent_address(addr: address): address {
            if (spec_is_parent_vasp(addr)) {
                addr
            } else {
                global<ChildVASP>(addr).parent_vasp_addr
            }
        }
    }

    /// Returns true if `addr` is a parent VASP.
    public fun is_parent(addr: address): bool {
        exists<ParentVASP>(addr)
    }
    spec fun is_parent {
        pragma opaque = true;
        ensures result == spec_is_parent_vasp(addr);
    }
    spec module {
        /// Spec version of `Self::is_parent`.
        define spec_is_parent_vasp(addr: address): bool {
            exists<ParentVASP>(addr)
        }
    }

    /// Returns true if `addr` is a child VASP.
    public fun is_child(addr: address): bool {
        exists<ChildVASP>(addr)
    }
    spec fun is_child {
        /// TODO(wrwg): Because the `ChildHasParent` invariant currently lets the prover hang,
        /// we make this function opaque and specify the *expected* result. We know its true
        /// because of the way ChildVASP is published, but can't verify this right now. This
        /// enables verification of code which checks is_child or is_vasp.
        pragma opaque = true;
        ensures result == spec_is_child_vasp(addr);
    }
    spec module {
        /// Spec version `Self::is_child`.
        define spec_is_child_vasp(addr: address): bool {
            exists<ChildVASP>(addr)
        }
    }

    /// Returns true if `addr` is a VASP.
    public fun is_vasp(addr: address): bool {
        is_parent(addr) || is_child(addr)
    }
    spec fun is_vasp {
        pragma opaque = true;
        ensures result == spec_is_vasp(addr);
    }
    spec module {
        /// Spec version of `Self::is_vasp`.
        define spec_is_vasp(addr: address): bool {
            spec_is_parent_vasp(addr) || spec_is_child_vasp(addr)
        }

        /// This is a stronger version of `Self::is_vasp` which also handles the potential abortion
        /// if the VASP under `addr` would not be well-formed (i.e. a child which has not parent).
        /// This is used in specification of callers for e.g. `Self::parent_address`.
        ///
        /// > TODO(wrwg): We would really like to have the fact that each child has a parent encoded
        /// > as a data invariant. We currently have formulated it as a module invariant, but this is
        /// > not clean because this invariant is only enforced after program points we have made a
        /// > a call to VASP functions.
        define spec_is_valid_vasp(addr: address): bool {
            spec_is_parent_vasp(addr)
                || spec_is_child_vasp(addr) && spec_is_parent_vasp(global<ChildVASP>(addr).parent_vasp_addr)
        }
    }

    /// Returns true if both addresses are VASPs and they have the same parent address.
    public fun is_same_vasp(addr1: address, addr2: address): bool acquires ChildVASP {
        is_vasp(addr1) && is_vasp(addr2) && parent_address(addr1) == parent_address(addr2)
    }
    spec fun is_same_vasp {
        pragma opaque = true;
        ensures result == spec_is_same_vasp(addr1, addr2);
    }
    spec module {
        /// Spec version of `Self::is_same_vasp`.
        define spec_is_same_vasp(addr1: address, addr2: address): bool {
            spec_is_vasp(addr1) && spec_is_vasp(addr2) && spec_parent_address(addr1) == spec_parent_address(addr2)
        }
    }

    /// Return the number of child accounts for this VASP.
    /// The total number of accounts for this VASP is num_children() + 1
    /// Aborts if `addr` is not a ParentVASP or ChildVASP account
    public fun num_children(addr: address): u64  acquires ChildVASP, ParentVASP {
        *&borrow_global<ParentVASP>(parent_address(addr)).num_children
    }

    // **************** SPECIFICATIONS ****************

    /// # Module specifications

    spec module {
        pragma verify = true;
    }

    /// # Each children has a parent

    spec schema ChildHasParent {
        /// > TODO(wrwg): this property currently causes timeouts/long running verification for multiple
        /// > functions in this module. Investigate why.
        invariant module true /*forall a: address: spec_child_has_parent(a)*/;
    }
    spec module {
        apply ChildHasParent to *, *<CoinType>;

        /// Returns true if the `addr`, when a ChildVASP, has a ParentVASP.
        define spec_child_has_parent(addr: address): bool {
            spec_is_child_vasp(addr) ==> spec_is_parent_vasp(global<ChildVASP>(addr).parent_vasp_addr)
        }
    }


    /// ## Privileges

    /// Only a parent VASP calling publish_child_vast_credential can create
    /// child VASP.
    spec schema ChildVASPsDontChange {
        /// **Informally:** A child is at an address iff it was there in the
        /// previous state.
        ensures forall a: address : exists<ChildVASP>(a) == old(exists<ChildVASP>(a));
    }
    spec module {
        apply ChildVASPsDontChange to *<T>, * except
            publish_child_vasp_credential;
    }

    /// ## Number of children is consistent

    /// > PROVER TODO(emmazzz): implement the features that allows users
    /// > to reason about number of resources with certain property,
    /// > such as "number of ChildVASPs whose parent address is 0xDD".
    /// > See issue #4665.

    /// ## Number of children does not change

    spec schema NumChildrenRemainsSame {
        ensures forall parent: address
            where old(spec_is_parent_vasp(parent)):
                old(spec_get_num_children(parent)) == spec_get_num_children(parent);
    }

    spec module {
        apply NumChildrenRemainsSame to * except publish_child_vasp_credential;

        /// Returns the number of children under `parent`.
        define spec_get_num_children(parent: address): u64 {
            global<ParentVASP>(parent).num_children
        }
    }

    /// ## Parent does not change

    spec schema ParentRemainsSame {
        ensures forall child_addr: address
            where old(spec_is_child_vasp(child_addr)):
                old(spec_parent_address(child_addr)) == spec_parent_address(child_addr);
    }

    spec module {
        apply ParentRemainsSame to *;
    }

    /// ## Aborts conditions shared between functions.

    spec schema AbortsIfNotVASP {
        addr: address;
        aborts_if [export] !spec_is_vasp(addr);
    }

    spec module {
        apply AbortsIfNotVASP to parent_address, num_children;
    }

    spec schema AbortsIfParentIsNotParentVASP {
        addr: address;
        aborts_if [export] !spec_is_parent_vasp(spec_parent_address(addr));
    }

    spec module {
        apply AbortsIfParentIsNotParentVASP to num_children;
    }
}

}
