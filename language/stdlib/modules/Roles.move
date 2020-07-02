address 0x1 {
/// This module describes two things:
///
/// 1. The relationship between roles, e.g. Role_A can creates accounts of Role_B
/// It is important to note here that this module _does not_ describe the
/// privileges that a specific role can have. This is a property of each of
/// the modules that declares a privilege.
///
/// Roles are defined to be completely opaque outside of this module --
/// all operations should be guarded by privilege checks, and not by role
/// checks. Each role comes with a default privilege.
///

module Roles {
    use 0x1::Signer;
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;

    ///////////////////////////////////////////////////////////////////////////
    // Role ID constants
    ///////////////////////////////////////////////////////////////////////////

    // TODO: Change these to constants once the source language has them
    fun LIBRA_ROOT_ROLE_ID(): u64 { 0 }
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
    // Privileges
    ///////////////////////////////////////////////////////////////////////////


    /// TODO: This is a legacy that will disappear soon when Tim removes a dependency
    /// of the VM on the ModulePublish privilege.
    /// The internal representation of of a privilege. We wrap every
    /// privilege witness resource here to avoid having to write extractors/restorers
    /// for each privilege, but can instead write this generically.
    resource struct Privilege<Priv: resource>  {
        witness: Priv,
        is_extracted: bool,
    }

    ///////////////////////////////////////////////////////////////////////////
    // Privilege Granting
    ///////////////////////////////////////////////////////////////////////////

    /// TODO: This is here because the VM expects to find a ModulePublish privilege
    /// published.
    public fun add_privilege_to_account_association_root_role<Priv: resource>(account: &signer, witness: Priv)
    acquires RoleId {
        let account_role = borrow_global<RoleId>(Signer::address_of(account));
        assert(account_role.role_id == LIBRA_ROOT_ROLE_ID(), 0);
        move_to(account, Privilege<Priv>{ witness, is_extracted: false })
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
        LibraTimestamp::assert_is_genesis();
        let owner_address = Signer::address_of(association);
        assert(owner_address == CoreAddresses::LIBRA_ROOT_ADDRESS(), 0);
        // Grant the role to the association root account
        move_to(association, RoleId { role_id: LIBRA_ROOT_ROLE_ID() });
    }

    /// NB: currency-related privileges are defined in the `Libra` module.
    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles.
    public fun grant_treasury_compliance_role(
        treasury_compliance_account: &signer,
        lr_account: &signer,
    ) acquires RoleId {
        LibraTimestamp::assert_is_genesis();
        assert(has_libra_root_role(lr_account), 999);
        let owner_address = Signer::address_of(treasury_compliance_account);
        assert(owner_address == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), 0);
        // Grant the TC role to the treasury_compliance_account
        move_to(treasury_compliance_account, RoleId { role_id: TREASURY_COMPLIANCE_ROLE_ID() });
    }

    /// Generic new role creation (for role ids != LIBRA_ROOT_ROLE_ID
    /// and TREASURY_COMPLIANCE_ROLE_ID).
    ///
    /// TODO: There is some common code here that can be factored out.
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
        assert(calling_role.role_id == TREASURY_COMPLIANCE_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: DESIGNATED_DEALER_ROLE_ID() });
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
        assert(calling_role.role_id == LIBRA_ROOT_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: VALIDATOR_ROLE_ID() });
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
        assert(calling_role.role_id == LIBRA_ROOT_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: VALIDATOR_OPERATOR_ROLE_ID() });
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
        assert(calling_role.role_id == LIBRA_ROOT_ROLE_ID(), 0);
        move_to(new_account, RoleId { role_id: PARENT_VASP_ROLE_ID() });
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
    }

    /// Publish an Unhosted `RoleId` under `new_account`.
    // TODO(tzakian): remove unhosted creation/guard so that only
    // assoc root can create.
    public fun new_unhosted_role(_creating_account: &signer, new_account: &signer) {
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), 1);
        move_to(new_account, RoleId { role_id: UNHOSTED_ROLE_ID() });
    }

    ///  ## privilege-checking functions for roles ##
    ///
    /// Naming conventions: Many of the "has_*_privilege" functions do have the same body
    /// because the spreadsheet grants all such privileges to addresses (usually a single
    /// address) with that role. In effect, having the privilege is equivalent to having the
    /// role, but the function names document the specific privilege involved.  Also, modules
    /// that use these functions as a privilege check can hide specific roles, so that a change
    /// in the privilege/role relationship can be implemented by changing Roles and not the
    /// module that uses it.

    public fun has_role(account: &signer, role_id: u64): bool acquires RoleId {
       let addr = Signer::address_of(account);
       exists<RoleId>(addr)
           && borrow_global<RoleId>(addr).role_id == role_id
    }

    public fun has_libra_root_role(account: &signer): bool acquires RoleId {
        has_role(account, LIBRA_ROOT_ROLE_ID())
    }

    public fun has_treasury_compliance_role(account: &signer): bool acquires RoleId {
        has_role(account, TREASURY_COMPLIANCE_ROLE_ID())
    }

    public fun has_designated_dealer_role(account: &signer): bool acquires RoleId {
        has_role(account, DESIGNATED_DEALER_ROLE_ID())
    }

    public fun has_validator_role(account: &signer): bool acquires RoleId {
        has_role(account, VALIDATOR_ROLE_ID())
    }

    public fun has_validator_operator_role(account: &signer): bool acquires RoleId {
        has_role(account, VALIDATOR_OPERATOR_ROLE_ID())
    }

    public fun has_parent_VASP_role(account: &signer): bool acquires RoleId {
        has_role(account, PARENT_VASP_ROLE_ID())
    }

    public fun has_child_VASP_role(account: &signer): bool acquires RoleId {
        has_role(account, CHILD_VASP_ROLE_ID())
    }

    public fun has_unhosted_role(account: &signer): bool acquires RoleId {
        has_role(account, UNHOSTED_ROLE_ID())
    }

    public fun has_register_new_currency_privilege(account: &signer): bool acquires RoleId {
         has_treasury_compliance_role(account)
    }

    public fun has_update_dual_attestation_threshold_privilege(account: &signer): bool acquires RoleId {
         has_treasury_compliance_role(account)
    }

    public fun has_on_chain_config_privilege(account: &signer): bool acquires RoleId {
         has_libra_root_role(account)
    }

//**************** Specifications ****************

    /// >**Note:** Just started, only a few specs.
    ///
    /// ## Role persistence

    spec module {
        pragma verify = true;
    }

    /// Helper functions
    spec module {
        define spec_get_role_id(account: signer): u64 {
            let addr = Signer::spec_address_of(account);
            global<RoleId>(addr).role_id
        }

        define spec_has_role_id(account: signer, role_id: u64): bool {
            let addr = Signer::spec_address_of(account);
            exists<RoleId>(addr) && global<RoleId>(addr).role_id == role_id
        }

        define SPEC_LIBRA_ROOT_ROLE_ID(): u64 { 0 }
        define SPEC_TREASURY_COMPLIANCE_ROLE_ID(): u64 { 1 }
        define SPEC_DESIGNATED_DEALER_ROLE_ID(): u64 { 2 }
        define SPEC_VALIDATOR_ROLE_ID(): u64 { 3 }
        define SPEC_VALIDATOR_OPERATOR_ROLE_ID(): u64 { 4 }
        define SPEC_PARENT_VASP_ROLE_ID(): u64 { 5 }
        define SPEC_CHILD_VASP_ROLE_ID(): u64 { 6 }
        define SPEC_UNHOSTED_ROLE_ID(): u64 { 7 }

        define spec_has_libra_root_role(account: signer): bool {
            spec_has_role_id(account, SPEC_LIBRA_ROOT_ROLE_ID())
        }

        define spec_has_treasury_compliance_role(account: signer): bool {
            spec_has_role_id(account, SPEC_TREASURY_COMPLIANCE_ROLE_ID())
        }

        define spec_has_designated_dealer_role(account: signer): bool {
            spec_has_role_id(account, SPEC_DESIGNATED_DEALER_ROLE_ID())
        }

        define spec_has_validator_role(account: signer): bool {
            spec_has_role_id(account, SPEC_VALIDATOR_ROLE_ID())
        }

        define spec_has_validator_operator_role(account: signer): bool {
            spec_has_role_id(account, SPEC_VALIDATOR_OPERATOR_ROLE_ID())
        }

        define spec_has_parent_VASP_role(account: signer): bool {
            spec_has_role_id(account, SPEC_PARENT_VASP_ROLE_ID())
        }

        define spec_has_child_VASP_role(account: signer): bool {
            spec_has_role_id(account, SPEC_CHILD_VASP_ROLE_ID())
        }

        define spec_has_unhosted_role(account: signer): bool {
            spec_has_role_id(account, SPEC_UNHOSTED_ROLE_ID())
        }

        define spec_has_register_new_currency_privilege(account: signer): bool {
            spec_has_treasury_compliance_role(account)
        }

        define spec_has_update_dual_attestation_threshold_privilege(account: signer): bool  {
            spec_has_treasury_compliance_role(account)
        }

        define spec_has_on_chain_config_privilege(account: signer): bool {
            spec_has_treasury_compliance_role(account)
        }
    }

    /// **Informally:** Once an account at address `A` is granted a role `R` it
    /// will remain an account with role `R` for all time.
    spec schema RoleIdPersists {
        ensures forall addr: address where old(exists<RoleId>(addr)):
            exists<RoleId>(addr)
                && old(global<RoleId>(addr).role_id) == global<RoleId>(addr).role_id;
    }

    spec module {
        apply RoleIdPersists to *<T>, *;
    }

    // TODO: Role is supposed to be set by end of genesis?

    // TODO: role-specific privileges persist, and role_ids never change?

    // ## Capabilities
    //
    // TODO: Capability is stored a owner_address unless is_extract == true??
    // TODO: Capability always returned to owner_address

}
}
