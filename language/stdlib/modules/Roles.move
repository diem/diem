address 0x1 {
/// This module describes two things:
/// 1. The relationship between roles, e.g. Role_A can creates accounts of Role_B
/// 2. The granting of privileges to an account with a specific role
/// It is important to note here that this module _does not_ describe the
/// privileges that a specific role can have. This is a property of each of
/// the modules that declares a privilege.
///
/// It also defines functions for extracting capabilities from an
/// account, and ensuring that they can only be "restored" back to the
/// account that they were extracted from.
///
/// Roles are defined to be completely opaque outside of this module --
/// all operations should be guarded by privilege checks, and not by role
/// checks. Each role comes with a default privilege.
///
/// Terminology:
/// There are three main types of resources that we deal with in this
/// module. These are:
/// 1. *Privilege Witnesses* `P`: are resources that are declared in other
///    modules (with the exception of the default role-based privileges
///    defined in this module). The declaring module is responsible for
///    guarding the creation of resources of this type.
/// 2. *Privileges* `Privilege<P>`: where `P` is a privilege witness is a
///    resource published under an account signifying that it can perform
///    operations that require `P` permissions.
/// 3. *Capabilities* `Capability<P>`: where `P` is a privilege witness is
///    an object that represents the authority to perform actions requiring
///    `P` permission. These can only be extracted from accounts that hold
///    a `Privilege<P>` resource.

module Roles {
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;

    ///////////////////////////////////////////////////////////////////////////
    // Role ID constants
    ///////////////////////////////////////////////////////////////////////////

    // TODO: Change these to constants once the source language has them
    fun ASSOCIATION_ROOT_ROLE_ID(): u64 { 0 }
    fun TREASURY_COMPLIANCE_ROLE_ID(): u64 { 1 }
    fun DESIGNATED_DEALER_ROLE_ID(): u64 { 2 }
    fun VALIDATOR_ROLE_ID(): u64 { 3 }
    fun VALIDATOR_OPERATOR_ROLE_ID(): u64 { 4 }
    fun PARENT_VASP_ROLE_ID(): u64 { 5 }
    fun CHILD_VASP_ROLE_ID(): u64 { 6 }
    fun UNHOSTED_ROLE_ID(): u64 { 7 }

    /// The roleId contains the role id for the account. This is only moved
    /// to an account as a top-level resource, and is otherwise immovable.
    resource struct RoleId {
        role_id: u64,
    }

    ///////////////////////////////////////////////////////////////////////////
    // Privileges & Capabilities
    ///////////////////////////////////////////////////////////////////////////

    /// Privileges are extracted in to capabilities. Capabilities hold /
    /// the account address that they were extracted from (i.e. tagged or
    /// "tainted"). Capabilities can then only be restored to the account
    /// from which they were extracted.
    resource struct Capability<Privilege: resource> {
        owner_address: address,
    }

    /// The internal representation of of a privilege. We wrap every
    /// privilege witness resource here to avoid having to write extractors/restorers
    /// for each privilege, but can instead write this generically.
    resource struct Privilege<Priv: resource>  {
        witness: Priv,
        is_extracted: bool,
    }

    ///////////////////////////////////////////////////////////////////////////
    // Role-specific Privileges
    ///////////////////////////////////////////////////////////////////////////

    /// Every role is granted a "default privilege" for that role. This can
    /// be seen as a base-permission for every account of that role type.
    /// INVARIANT: Every account has exactly one of these, and these
    ///            correspond precisely to the RoleId.
    resource struct AssociationRootRole {}
    resource struct TreasuryComplianceRole {}
    resource struct DesignatedDealerRole {}
    resource struct ValidatorRole {}
    resource struct ValidatorOperatorRole {}
    resource struct ParentVASPRole {}
    resource struct ChildVASPRole {}
    resource struct UnhostedRole {}

    ///////////////////////////////////////////////////////////////////////////
    // Privilege Granting
    ///////////////////////////////////////////////////////////////////////////

    /// The privilege `witness: Priv` is granted to `account` as long as
    /// `account` has a `role` with `role.role_id == role_id`.
    /// INVARIANT: Once a privilege witness `Priv` has been granted to an
    ///            account it remains at that account.
    fun add_privilege_to_account<Priv: resource>(
        account: &signer,
        witness: Priv,
        role_id: u64,
    ) acquires RoleId {
        let account_role = borrow_global<RoleId>(Signer::address_of(account));
        assert(account_role.role_id == role_id, 0);
        move_to(account, Privilege<Priv>{ witness, is_extracted: false })
    }

    /// Public wrappers to the `add_privilege_to_account` function that sets the
    /// correct role_id for the role. This way the role that a privilege is
    /// being assigned to outside of the module is statically determinable.
    public fun add_privilege_to_account_association_root_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, ASSOCIATION_ROOT_ROLE_ID());
    }

    public fun add_privilege_to_account_treasury_compliance_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, TREASURY_COMPLIANCE_ROLE_ID());
    }

    public fun add_privilege_to_account_designated_dealer_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, DESIGNATED_DEALER_ROLE_ID());
    }

    public fun add_privilege_to_account_validator_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, VALIDATOR_ROLE_ID());
    }

    public fun add_privilege_to_account_validator_operator_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, VALIDATOR_OPERATOR_ROLE_ID());
    }

    public fun add_privilege_to_account_parent_vasp_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, PARENT_VASP_ROLE_ID());
    }

    public fun add_privilege_to_account_child_vasp_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, CHILD_VASP_ROLE_ID());
    }

    public fun add_privilege_to_account_unhosted_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        add_privilege_to_account(account, witness, UNHOSTED_ROLE_ID());
    }

    ///////////////////////////////////////////////////////////////////////////
    // ROLE GRANTING
    ///////////////////////////////////////////////////////////////////////////

    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles. This is _not_ called from within LibraAccount -- these
    /// privileges need to be created before accounts can be made
    /// (specifically, initialization of currency)
    public fun grant_root_association_role(
        association: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), 0);
        let owner_address = Signer::address_of(association);
        assert(owner_address == CoreAddresses::ASSOCIATION_ROOT_ADDRESS(), 0);
        // Grant the role to the association root account
        move_to(association, RoleId { role_id: ASSOCIATION_ROOT_ROLE_ID() });
        move_to(association, Privilege<AssociationRootRole>{ witness: AssociationRootRole{}, is_extracted: false})
    }

    /// NB: currency-related privileges are defined in the `Libra` module.
    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles.
    public fun grant_treasury_compliance_role(
        treasury_compliance_account: &signer,
        _: &Capability<AssociationRootRole>,
    ) {
        assert(LibraTimestamp::is_genesis(), 0);
        let owner_address = Signer::address_of(treasury_compliance_account);
        assert(owner_address == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), 0);
        // Grant the TC role to the treasury_compliance_account
        move_to(treasury_compliance_account, RoleId { role_id: TREASURY_COMPLIANCE_ROLE_ID() });
        move_to(treasury_compliance_account, Privilege<TreasuryComplianceRole>{ witness: TreasuryComplianceRole{}, is_extracted: false});

        // > XXX/TODO/HACK/REMOVE (tzakian): This is a _HACK_ for right now
        // so that we can allow minting to create an account. THIS NEEDS TO BE REMOVED.
        move_to(treasury_compliance_account, Privilege<AssociationRootRole>{ witness: AssociationRootRole{}, is_extracted: false})
    }

    /// Generic new role creation (for role ids != ASSOCIATION_ROOT_ROLE_ID
    /// and TREASURY_COMPLIANCE_ROLE_ID).
    /// We take a `&signer` here and link it with the account address so
    /// that we link the `signer` and `owner_address` together in this
    /// module. This should hopefully make proofs easier.
    ///
    /// Additionally, a role comes with a default privilege for its role. This can allow
    /// extensibility later on if a new module introduces a privilege for a role `R`.
    /// The new module can use a capability for the role `R`;
    /// `&Capability<R>` to guard the  granting of the privilege in order
    /// to ensure that only those with appropriate permissions are granted
    /// the new permission. e.g.
    /// ```
    /// public fun publish_new_privilege(account: &signer, _: &Capability<R>) {
    ///    Roles::add_privilege_to_account(account, Roles::R_ROLE_ID());
    /// }
    ///```
    ///
    /// Publish a DesignatedDealer `RoleId` under `new_account`.
    /// The `creating_account` must be TreasuryCompliance
    public fun new_designated_dealer_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        let calling_role = borrow_global<RoleId>(Signer::address_of(creating_account));
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        //assert(calling_role.role_id == ASSOCIATION_ROOT_ROLE_ID(), 0);
        assert(calling_role.role_id == TREASURY_COMPLIANCE_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: DESIGNATED_DEALER_ROLE_ID() });
        move_to(new_account, Privilege<DesignatedDealerRole>{ witness: DesignatedDealerRole{}, is_extracted: false })
    }

    /// Publish a Validator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_role(
        creating_account: &signer,
        new_account: &signer
    ) acquires RoleId {
        let calling_role = borrow_global<RoleId>(Signer::address_of(creating_account));
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        assert(calling_role.role_id == ASSOCIATION_ROOT_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: VALIDATOR_ROLE_ID() });
        move_to(new_account, Privilege<ValidatorRole>{ witness: ValidatorRole{}, is_extracted: false })
    }

    /// Publish a ValidatorOperator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_operator_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        let calling_role = borrow_global<RoleId>(Signer::address_of(creating_account));
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        assert(calling_role.role_id == ASSOCIATION_ROOT_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: VALIDATOR_OPERATOR_ROLE_ID() });
        move_to(new_account, Privilege<ValidatorOperatorRole>{ witness: ValidatorOperatorRole{}, is_extracted: false })
    }

    /// Publish a ParentVASP `RoleId` under `new_account`.
    /// The `creating_account` must be TreasuryCompliance
    public fun new_parent_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        let calling_role = borrow_global<RoleId>(Signer::address_of(creating_account));
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        assert(
                calling_role.role_id == ASSOCIATION_ROOT_ROLE_ID()
                // XXX/HACK/REMOVE(tzakian): This is for testnet semantics
                // only. THIS NEEDS TO BE REMOVED.
                || calling_role.role_id == TREASURY_COMPLIANCE_ROLE_ID(),
                0
            );
            move_to(new_account, RoleId { role_id: PARENT_VASP_ROLE_ID() });
            move_to(new_account, Privilege<ParentVASPRole>{ witness: ParentVASPRole{}, is_extracted: false })
    }

    /// Publish a ChildVASP `RoleId` under `new_account`.
    /// The `creating_account` must be a ParentVASP
    public fun new_child_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        let calling_role = borrow_global<RoleId>(Signer::address_of(creating_account));
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        assert(calling_role.role_id == PARENT_VASP_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: CHILD_VASP_ROLE_ID() });
        move_to(new_account, Privilege<ChildVASPRole>{ witness: ChildVASPRole{}, is_extracted: false })
    }

    /// Publish an Unhosted `RoleId` under `new_account`.
    // TODO(tzakian): remove unhosted creation/guard so that only
    // assoc root can create.
    public fun new_unhosted_role(_creating_account: &signer, new_account: &signer) {
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        move_to(new_account, RoleId { role_id: UNHOSTED_ROLE_ID() });
        move_to(new_account, Privilege<UnhostedRole>{ witness: UnhostedRole{}, is_extracted: false })
    }

    ///////////////////////////////////////////////////////////////////////////
    // Capability Extraction from Privileges, and Restoration to Privileges
    ///////////////////////////////////////////////////////////////////////////

    /// Some specs we may want to say about privileges and roles:
    /// 1. For all roles `R = R1, ..., Rn` the privilege witness `P` is only
    /// granted to accounts with roles `Ri1, Ri2, ...` where `Rik \in R`.
    /// This is a property of the module in which the privilege witness
    /// resource `P` is declared. (should be provable on a per-module basis)
    /// 2. For all privilege witnesses `P`, and  instances `p: Privileges<P>`, the
    ///    account at address `A` can hold `p` iff `p.owner_address == A`. (should be provable)
    /// 3. Once a privilege is granted to an account `A`, that account
    ///    holds that permission for all time.
    /// 4. Every account has one, and only one, role. The role of the
    ///    account does not change after creation.

    /// We don't need to check for roles now, because the only way you can
    /// get the correct capability is if you had that privilege, which can
    /// only be granted if you have the correct role. When a capability
    /// leaves the module we tag it with the account where it was held. You
    /// can only put the capability back if the `account` address you are
    /// storing it back under and the `owner_address` of the incoming capability agree.
    /// INVARIANT: Once a privilege witness is created and stored under
    /// a Privilege<PrivWitness> resource at an address A there are only two states:
    /// 1. The resource Privilege<PrivWitness> is stored at A;
    /// 2. The privilege witness is held in a Capability<PrivWitness> and
    ///    the `owner_address == A`.
    public fun extract_privilege_to_capability<Priv: resource>(account: &signer): Capability<Priv>
    acquires Privilege {
        let owner_address = Signer::address_of(account);
        // Privilege doesn't exist
        assert(exists<Privilege<Priv>>(owner_address), 3);
        let priv = borrow_global_mut<Privilege<Priv>>(owner_address);
        // Make sure this privilege was not previously extracted
        assert(!priv.is_extracted, 4);
        // Set that the privilege is now extracted
        priv.is_extracted = true;
        Capability<Priv> { owner_address }
    }

    /// When the capability is restored back to a privilege, we make sure
    /// that the underlying privilege cannot be stored under a different
    /// account than it was extracted from. Once we ensure that we then
    /// store the privilege witness back under the account.
    /// INVARIANT: Only a capability extracted from an account A can be
    /// restored back to A. i.e. \forall (cap: Capability<P>),
    /// (cap.owner_address != B).  restore_capability_to_privilege<P>(B, cap) fails
    public fun restore_capability_to_privilege<Priv: resource>(account: &signer, cap: Capability<Priv>)
    acquires Privilege {
        let account_address = Signer::address_of(account);
        let Capability<Priv>{ owner_address } = cap;
        // Make sure the owner of the privilege when we extracted it is the
        // same as the address we're putting it back under.
        assert(owner_address == account_address, 4);
        // Set that the privilege is now put back
        borrow_global_mut<Privilege<Priv>>(owner_address).is_extracted = false;
    }

//**************** Specifications ****************

    /// >**Note:** Just started, only a few specs.
    ///

    spec module {
        pragma verify = true;
    }

    /// Helper functions
    spec module {
        define spec_has_role_id(addr: address): bool {
            exists<RoleId>(addr)
        }

        define spec_get_role_id(addr: address): u64 {
            global<RoleId>(addr).role_id
        }

        define SPEC_ASSOCIATION_ROOT_ROLE_ID(): u64 { 0 }
        define SPEC_TREASURY_COMPLIANCE_ROLE_ID(): u64 { 1 }
        define SPEC_DESIGNATED_DEALER_ROLE_ID(): u64 { 2 }
        define SPEC_VALIDATOR_ROLE_ID(): u64 { 3 }
        define SPEC_VALIDATOR_OPERATOR_ROLE_ID(): u64 { 4 }
        define SPEC_PARENT_VASP_ROLE_ID(): u64 { 5 }
        define SPEC_CHILD_VASP_ROLE_ID(): u64 { 6 }
        define SPEC_UNHOSTED_ROLE_ID(): u64 { 7 }
    }

    /// ## Matching of role_id's and role-specific privileges.

    /// To prove some of the invariants, it is necessary to add a precondition that
    /// the "witness" argument of add_privilege_to_* functions cannot be one
    /// of the "default privileges" declared above.  The reason is that there
    /// is no way an external module calling one of these functions to obtain
    /// an instance of these types to pass as an the witness argument.
    /// Ideally, the prover would have a way to figure this out,
    /// but it can't (yet).
    ///
    /// Instead, we add the precondition.  That assumption is sufficient to prove
    /// the invariant in this module.  When the function is called from another
    /// module, the precondition becomes a proof obligation (which, I hope, the
    /// prover can prove).

    spec module {
        /// Helper function that expresses the precondition predicate
        define is_not_internal_privilege(t: type): bool {
            type<t>() != type<AssociationRootRole>()
            && type<t>() != type<TreasuryComplianceRole>()
            && type<t>() != type<DesignatedDealerRole>()
            && type<t>() != type<ValidatorRole>()
            && type<t>() != type<ValidatorOperatorRole>()
            && type<t>() != type<ParentVASPRole>()
            && type<t>() != type<ChildVASPRole>()
            && type<t>() != type<UnhostedRole>()
        }
    }

    spec schema IsNotInternalPrivilege<Priv> {
        requires is_not_internal_privilege(type<Priv>());
    }
    spec module {
        apply IsNotInternalPrivilege<Priv> to public add_privilege_to_*<Priv>;
    }

    /// The properties below all assert the invariant that, for each address, there
    /// is a RoleId resource with role_id field == SOMETHING_ROLE_ID() iff that
    /// address has a Privilege<SomethingRole> resource . The property is split into
    /// separate "if" and "only if" parts for ease of debugging. There is a temporary
    /// "except" in the apply for AssociationRoot because of a temporary testnet hack
    /// that violates it.

    spec schema AssociationRootRoleMatchesRoleId {
        /// **Informally:** For every address, if the address has an
        /// AssociationRootRole Privilege, then it has a RoleId resource
        /// with role_id == ASSOCIATION_ROOT_ROLE_ID. (The remaining Role
        /// privileges are similar, so this explanation is not repeated.)
        invariant module forall addr: address where exists<Privilege<AssociationRootRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_ASSOCIATION_ROOT_ROLE_ID());

        /// **Informally:** For every address, if the address has a RootRole resource,
        /// and the role_id == ASSOCIATION_ROOT_ROLE_ID, then that address also has
        /// an AssociationRootRole Privilege (also similar to the following properties,
        /// so also not repeated).
        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_ASSOCIATION_ROOT_ROLE_ID())
                ==> exists<Privilege<AssociationRootRole>>(addr);
    }
    spec module {
        /// TODO (dd) "except" is a work-around for the testnet weirdness in
        /// `grant_treasury_compliance_role`
        /// BUG? (dd) If you delete "public" in front of "*", the "except" doesn't
        /// seem to work. I haven't been able to reproduce this in a simple test.
        apply AssociationRootRoleMatchesRoleId to public *, public *<T> except grant_treasury_compliance_role;
    }

    spec schema TreasuryComplianceRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<TreasuryComplianceRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_TREASURY_COMPLIANCE_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_TREASURY_COMPLIANCE_ROLE_ID())
                ==> exists<Privilege<TreasuryComplianceRole>>(addr);
    }
    spec module {
        apply TreasuryComplianceRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema DesignatedDealerRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<DesignatedDealerRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_DESIGNATED_DEALER_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_DESIGNATED_DEALER_ROLE_ID())
                ==> exists<Privilege<DesignatedDealerRole>>(addr);
    }
    spec module {
        apply DesignatedDealerRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema ValidatorRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<ValidatorRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_VALIDATOR_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_VALIDATOR_ROLE_ID())
                ==> exists<Privilege<ValidatorRole>>(addr);
    }
    spec module {
        apply ValidatorRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema ValidatorOperatorRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<ValidatorOperatorRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_VALIDATOR_OPERATOR_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_VALIDATOR_OPERATOR_ROLE_ID())
                ==> exists<Privilege<ValidatorOperatorRole>>(addr);
    }
    spec module {
        apply ValidatorOperatorRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema ParentVASPRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<ParentVASPRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_PARENT_VASP_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_PARENT_VASP_ROLE_ID())
                ==> exists<Privilege<ParentVASPRole>>(addr);
    }
    spec module {
        apply ParentVASPRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema ChildVASPRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<ChildVASPRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_CHILD_VASP_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_CHILD_VASP_ROLE_ID())
                ==> exists<Privilege<ChildVASPRole>>(addr);
    }
    spec module {
        apply ChildVASPRoleMatchesRoleId to public *, public *<T>;
    }

    spec schema UnhostedRoleMatchesRoleId {
        invariant module forall addr: address where exists<Privilege<UnhostedRole>>(addr):
             spec_has_role_id(addr)
                && (spec_get_role_id(addr) == SPEC_UNHOSTED_ROLE_ID());

        invariant module forall addr: address where spec_has_role_id(addr):
            (spec_get_role_id(addr) == SPEC_UNHOSTED_ROLE_ID())
                ==> exists<Privilege<UnhostedRole>>(addr);
    }
    spec module {
        apply UnhostedRoleMatchesRoleId to public *, public *<T>;
    }

    /// ## Role persistence

    /// **Informally:** Once an account at address `A` is granted a role `R` it
    /// will remain an account with role `R` for all time.
    spec schema RoleIdPersists {
        ensures forall addr: address where old(spec_has_role_id(addr)) :
            spec_has_role_id(addr) && (old(spec_get_role_id(addr)) == spec_get_role_id(addr));
    }

    spec module {
        apply RoleIdPersists to *<T>, *;
    }

    /// ## Privilege extraction

/// >TODO: This has not been proved yet and may be wrong, or need further
/// strengthening.  Or, it may just be too hard for the prover.
/// The capability returned from extract_privilege_to_capability is free-floating,
/// so we have to quantify over members of the type, not over addresses or something.

    spec schema IsExtractedIffCapabilityExists {
        /// **Informally:** No capabilities for a privilege witness at a given
        /// address exist until the privilege exists.
        /// > NOTE: This verifies by itself when applied to everything.
        invariant forall priv: type, addr: address where !exists<Privilege<priv>>(addr):
            !(exists c: Capability<priv>: c.owner_address == addr);

        // Without additional properties, this appears to be violated by
        // restore_capability because the prover thinks
        // there could be >1 capability when is_extracted == true.  restore_capability
        // destroys one capability, but it doesn't know that it destroyed the ONLY one.
        // Proof sketch of the following: If there are no capabilities, it holds vacuously.
        // When we create one capability (with no capabilities is precondition),
        // it holds (not sure how prover knows).
        // We can't create any more capabilities until that is destroyed.
        // **Informally:** for each type `priv` and address `addr`,
        // there is at most one instance of Capability<priv>{ owner_address: addr}
        //
        // > NOTE: This verifies when applied to extract_privilege_to_capability,
        // and might verify when applied to everything.
        // invariant
        //     forall priv: type, addr: address where exists<Privilege<priv>>(addr):
        //         forall c1: Capability<priv>, c2: Capability<priv>
        //             where c1.owner_address == c2.owner_address:
        //                 c1 == c2;

        // **Informally:** For every address that has a privilege with witness Priv,
        // if the `is_extracted` field of the privilege is true, then there is a
        // capability for that privilege whose `owner_address` field is address.
        //
        // > NOTE: This verifies when applied to extract_privilege_to_capability,
        // and might verify when applied to everything.
        // invariant module
        //      forall priv: type, addr: address where exists<Privilege<priv>>(addr):
        //          !global<Privilege<priv>>(addr).is_extracted
        //              ==> !(exists c: Capability<priv>: c.owner_address == addr);

        // **Informally:** For every address that has a privilege with witness Priv,
        // if the `is_extracted` field of the privilege is true, then there is a
        // capability for that privilege whose `owner_address` field is address.
        //
        // > TODO: This property causes the prover to hang, even when applied to
        // just one function.  Perhaps the proeprty is wrong.
        // invariant module
        //     forall priv: type, addr: address where exists<Privilege<priv>>(addr):
        //         global<Privilege<priv>>(addr).is_extracted
        //              ==> (exists c: Capability<priv>: c.owner_address == addr);
    }
    spec module {
        apply IsExtractedIffCapabilityExists to *<T>, *;
    //        apply IsExtractedIffCapabilityExists to extract_privilege_to_capability<Priv>;
    }

    // TODO: Role is supposed to be set by end of genesis?

    // TODO: role-specific privileges persist, and role_ids never change?

    // ## Capabilities
    //
    // TODO: Capability always returned to owner_address

}
}
