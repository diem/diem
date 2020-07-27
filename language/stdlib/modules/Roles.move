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
    use 0x1::Signer::{Self, spec_address_of};
    use 0x1::CoreAddresses;
    use 0x1::LibraTimestamp;

    const ENOT_GENESIS: u64 = 0;
    const EINVALID_ROOT_ADDRESS: u64 = 1;
    const EINVALID_TC_ADDRESS: u64 = 2;
    const EINVALID_PARENT_ROLE: u64 = 3;
    const EROLE_ALREADY_ASSIGNED: u64 = 4;

    ///////////////////////////////////////////////////////////////////////////
    // Role ID constants
    ///////////////////////////////////////////////////////////////////////////

    const LIBRA_ROOT_ROLE_ID: u64 = 0;
    const TREASURY_COMPLIANCE_ROLE_ID: u64 = 1;
    const DESIGNATED_DEALER_ROLE_ID: u64 = 2;
    const VALIDATOR_ROLE_ID: u64 = 3;
    const VALIDATOR_OPERATOR_ROLE_ID: u64 = 4;
    const PARENT_VASP_ROLE_ID: u64 = 5;
    const CHILD_VASP_ROLE_ID: u64 = 6;

    /// The roleId contains the role id for the account. This is only moved
    /// to an account as a top-level resource, and is otherwise immovable.
    resource struct RoleId {
        role_id: u64,
    }

    ///////////////////////////////////////////////////////////////////////////
    // ROLE GRANTING
    ///////////////////////////////////////////////////////////////////////////

    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles. This is _not_ called from within LibraAccount -- these
    /// privileges need to be created before accounts can be made
    /// (specifically, initialization of currency)
    public fun grant_libra_root_role(
        lr_account: &signer,
    ) {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        let owner_address = Signer::address_of(lr_account);
        assert(owner_address == CoreAddresses::LIBRA_ROOT_ADDRESS(), EINVALID_ROOT_ADDRESS);
        // Grant the role to the libra root account
        move_to(lr_account, RoleId { role_id: LIBRA_ROOT_ROLE_ID });
    }
    spec fun grant_libra_root_role {
        aborts_if !LibraTimestamp::spec_is_genesis();
        aborts_if spec_address_of(lr_account) != CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();
        aborts_if exists<RoleId>(spec_address_of(lr_account));
        ensures exists<RoleId>(spec_address_of(lr_account));
        ensures global<RoleId>(spec_address_of(lr_account)).role_id == SPEC_LIBRA_ROOT_ROLE_ID();
    }

    /// NB: currency-related privileges are defined in the `Libra` module.
    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles.
    public fun grant_treasury_compliance_role(
        treasury_compliance_account: &signer,
        lr_account: &signer,
    ) acquires RoleId {
        assert(LibraTimestamp::is_genesis(), ENOT_GENESIS);
        assert(has_libra_root_role(lr_account), EINVALID_PARENT_ROLE);
        let owner_address = Signer::address_of(treasury_compliance_account);
        assert(owner_address == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS(), EINVALID_TC_ADDRESS);
        // Grant the TC role to the treasury_compliance_account
        move_to(treasury_compliance_account, RoleId { role_id: TREASURY_COMPLIANCE_ROLE_ID });
    }
    spec fun grant_treasury_compliance_role {
        aborts_if !LibraTimestamp::spec_is_genesis();
        aborts_if !spec_has_libra_root_role_addr(Signer::spec_address_of(lr_account));
        aborts_if spec_address_of(treasury_compliance_account) != CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS();
        aborts_if exists<RoleId>(spec_address_of(treasury_compliance_account));
        ensures exists<RoleId>(spec_address_of(treasury_compliance_account));
        ensures global<RoleId>(spec_address_of(treasury_compliance_account)).role_id == SPEC_TREASURY_COMPLIANCE_ROLE_ID();
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
        assert(!exists<RoleId>(Signer::address_of(new_account)), EROLE_ALREADY_ASSIGNED);
        assert(calling_role.role_id == TREASURY_COMPLIANCE_ROLE_ID, EINVALID_PARENT_ROLE);
        move_to(new_account, RoleId { role_id: DESIGNATED_DEALER_ROLE_ID });
    }
    spec fun new_designated_dealer_role {
        aborts_if !spec_has_treasury_compliance_role_addr(Signer::spec_address_of(creating_account));
        aborts_if exists<RoleId>(spec_address_of(new_account));
        ensures exists<RoleId>(spec_address_of(new_account));
        ensures global<RoleId>(spec_address_of(new_account)).role_id == SPEC_DESIGNATED_DEALER_ROLE_ID();
    }

    /// Publish a Validator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_role(
        creating_account: &signer,
        new_account: &signer
    ) acquires RoleId {
        assert(has_libra_root_role(creating_account), EINVALID_PARENT_ROLE);
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), EROLE_ALREADY_ASSIGNED);
        move_to(new_account, RoleId { role_id: VALIDATOR_ROLE_ID });
    }
    spec fun new_validator_role {
        aborts_if !spec_has_libra_root_role_addr(Signer::spec_address_of(creating_account));
        aborts_if exists<RoleId>(spec_address_of(new_account));
        ensures exists<RoleId>(spec_address_of(new_account));
        ensures global<RoleId>(spec_address_of(new_account)).role_id == SPEC_VALIDATOR_ROLE_ID();
    }

    /// Publish a ValidatorOperator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_operator_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        assert(has_libra_root_role(creating_account), EINVALID_PARENT_ROLE);
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), EROLE_ALREADY_ASSIGNED);
        move_to(new_account, RoleId { role_id: VALIDATOR_OPERATOR_ROLE_ID });
    }
    spec fun new_validator_operator_role {
        aborts_if !spec_has_libra_root_role_addr(Signer::spec_address_of(creating_account));
        aborts_if exists<RoleId>(spec_address_of(new_account));
        ensures exists<RoleId>(spec_address_of(new_account));
        ensures global<RoleId>(spec_address_of(new_account)).role_id == SPEC_VALIDATOR_OPERATOR_ROLE_ID();
    }

    /// Publish a ParentVASP `RoleId` under `new_account`.
    /// The `creating_account` must be TreasuryCompliance
    public fun new_parent_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        assert(has_libra_root_role(creating_account), EINVALID_PARENT_ROLE);
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), EROLE_ALREADY_ASSIGNED);
        move_to(new_account, RoleId { role_id: PARENT_VASP_ROLE_ID });
    }
    spec fun new_parent_vasp_role {
        aborts_if !spec_has_libra_root_role_addr(Signer::spec_address_of(creating_account));
        aborts_if exists<RoleId>(spec_address_of(new_account));
        ensures exists<RoleId>(spec_address_of(new_account));
        ensures global<RoleId>(spec_address_of(new_account)).role_id == SPEC_PARENT_VASP_ROLE_ID();
    }

    /// Publish a ChildVASP `RoleId` under `new_account`.
    /// The `creating_account` must be a ParentVASP
    public fun new_child_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        assert(has_parent_VASP_role(creating_account), EINVALID_PARENT_ROLE);
        // A role cannot have previously been assigned to `new_account`.
        assert(!exists<RoleId>(Signer::address_of(new_account)), EROLE_ALREADY_ASSIGNED);
        move_to(new_account, RoleId { role_id: CHILD_VASP_ROLE_ID });
    }
    spec fun new_child_vasp_role {
        aborts_if !spec_has_parent_VASP_role_addr(Signer::spec_address_of(creating_account));
        aborts_if exists<RoleId>(spec_address_of(new_account));
        ensures exists<RoleId>(spec_address_of(new_account));
        ensures global<RoleId>(spec_address_of(new_account)).role_id == SPEC_CHILD_VASP_ROLE_ID();
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

    fun has_role(account: &signer, role_id: u64): bool acquires RoleId {
       let addr = Signer::address_of(account);
       exists<RoleId>(addr)
           && borrow_global<RoleId>(addr).role_id == role_id
    }

    public fun has_libra_root_role(account: &signer): bool acquires RoleId {
        has_role(account, LIBRA_ROOT_ROLE_ID)
    }

    public fun has_treasury_compliance_role(account: &signer): bool acquires RoleId {
        has_role(account, TREASURY_COMPLIANCE_ROLE_ID)
    }

    public fun has_designated_dealer_role(account: &signer): bool acquires RoleId {
        has_role(account, DESIGNATED_DEALER_ROLE_ID)
    }

    public fun has_validator_role(account: &signer): bool acquires RoleId {
        has_role(account, VALIDATOR_ROLE_ID)
    }

    public fun has_validator_operator_role(account: &signer): bool acquires RoleId {
        has_role(account, VALIDATOR_OPERATOR_ROLE_ID)
    }

    public fun has_parent_VASP_role(account: &signer): bool acquires RoleId {
        has_role(account, PARENT_VASP_ROLE_ID)
    }

    public fun has_child_VASP_role(account: &signer): bool acquires RoleId {
        has_role(account, CHILD_VASP_ROLE_ID)
    }

    public fun has_register_new_currency_privilege(account: &signer): bool acquires RoleId {
         has_libra_root_role(account)
    }

    public fun has_update_dual_attestation_limit_privilege(account: &signer): bool acquires RoleId {
         has_treasury_compliance_role(account)
    }

    /// Return true if `addr` is allowed to receive and send `Libra<T>` for any T
    public fun can_hold_balance(account: &signer): bool acquires RoleId {
        // VASP accounts and designated_dealers can hold balances.
        // Administrative accounts (`Validator`, `ValidatorOperator`, `TreasuryCompliance`, and
        // `LibraRoot`) cannot.
        has_parent_VASP_role(account) ||
        has_child_VASP_role(account) ||
        has_designated_dealer_role(account)
    }

    /// Return true if `account` must have limits on sending/receiving/holding of funds
    public fun needs_account_limits(account: &signer): bool acquires RoleId {
        // All accounts that hold balances are subject to limits except designated dealers
        can_hold_balance(account) && !has_designated_dealer_role(account)
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
            let addr = spec_address_of(account);
            global<RoleId>(addr).role_id
        }

        define spec_has_role_id_addr(addr: address, role_id: u64): bool {
            exists<RoleId>(addr) && global<RoleId>(addr).role_id == role_id
        }

        define SPEC_LIBRA_ROOT_ROLE_ID(): u64 { 0 }
        define SPEC_TREASURY_COMPLIANCE_ROLE_ID(): u64 { 1 }
        define SPEC_DESIGNATED_DEALER_ROLE_ID(): u64 { 2 }
        define SPEC_VALIDATOR_ROLE_ID(): u64 { 3 }
        define SPEC_VALIDATOR_OPERATOR_ROLE_ID(): u64 { 4 }
        define SPEC_PARENT_VASP_ROLE_ID(): u64 { 5 }
        define SPEC_CHILD_VASP_ROLE_ID(): u64 { 6 }

        define spec_has_libra_root_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_LIBRA_ROOT_ROLE_ID())
        }

        define spec_has_treasury_compliance_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_TREASURY_COMPLIANCE_ROLE_ID())
        }

        define spec_has_designated_dealer_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_DESIGNATED_DEALER_ROLE_ID())
        }

        define spec_has_validator_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_VALIDATOR_ROLE_ID())
        }

        define spec_has_validator_operator_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_VALIDATOR_OPERATOR_ROLE_ID())
        }

        define spec_has_parent_VASP_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_PARENT_VASP_ROLE_ID())
        }

        define spec_has_child_VASP_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, SPEC_CHILD_VASP_ROLE_ID())
        }

        define spec_has_register_new_currency_privilege_addr(addr: address): bool {
            spec_has_libra_root_role_addr(addr)
        }

        define spec_has_update_dual_attestation_limit_privilege_addr(addr: address): bool  {
            spec_has_treasury_compliance_role_addr(addr)
        }

        define spec_can_hold_balance_addr(addr: address): bool {
            spec_has_parent_VASP_role_addr(addr) ||
                spec_has_child_VASP_role_addr(addr) ||
                spec_has_designated_dealer_role_addr(addr)
        }

        define spec_needs_account_limits_addr(addr: address): bool {
            spec_can_hold_balance_addr(addr) && !spec_has_designated_dealer_role_addr(addr)
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
        apply RoleIdPersists to *<T>, * except has*;
    }


    spec schema ThisRoleIsNotNewlyPublished {
        this: u64;
        ensures forall addr: address where exists<RoleId>(addr) && global<RoleId>(addr).role_id == this:
            old(exists<RoleId>(addr)) && old(global<RoleId>(addr).role_id) == this;
    }

    spec schema AbortsIfNotLibraRoot {
        account: signer;
        aborts_if !spec_has_libra_root_role_addr(Signer::spec_address_of(account));
    }

    spec schema AbortsIfNotTreasuryCompliance {
        account: signer;
        aborts_if !spec_has_treasury_compliance_role_addr(Signer::spec_address_of(account));
    }

    spec schema AbortsIfNotParentVASP {
        account: signer;
        aborts_if !spec_has_parent_VASP_role_addr(Signer::spec_address_of(account));
    }

    spec module {
        /// Validator roles are only granted by LibraRoot [B4]. A new `RoldId` with `VALIDATOR_ROLE_ID()` is only
        /// published through `new_validator_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: SPEC_VALIDATOR_ROLE_ID()} to * except new_validator_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_validator_role;

        /// ValidatorOperator roles are only granted by LibraRoot [B5]. A new `RoldId` with `VALIDATOR_OPERATOR_ROLE_ID()` is only
        /// published through `new_validator_operator_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: SPEC_VALIDATOR_OPERATOR_ROLE_ID()} to * except new_validator_operator_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_validator_operator_role;

        /// DesignatedDealer roles are only granted by TreasuryCompliance [B6](TODO: resolve the discrepancy). A new `RoldId` with `DESIGNATED_DEALER_ROLE_ID()` is only
        /// published through `new_designated_dealer_role` which aborts if `creating_account` does not have the TreasuryCompliance role.
        apply ThisRoleIsNotNewlyPublished{this: SPEC_DESIGNATED_DEALER_ROLE_ID()} to * except new_designated_dealer_role;
        apply AbortsIfNotTreasuryCompliance{account: creating_account} to new_designated_dealer_role;

        /// ParentVASP roles are only granted by LibraRoot [B7]. A new `RoldId` with `PARENT_VASP_ROLE_ID()` is only
        /// published through `new_parent_vasp_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: SPEC_PARENT_VASP_ROLE_ID()} to * except new_parent_vasp_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_parent_vasp_role;

        /// ChildVASP roles are only granted by ParentVASP [B8]. A new `RoldId` with `CHILD_VASP_ROLE_ID()` is only
        /// published through `new_child_vasp_role` which aborts if `creating_account` does not have the ParentVASP role.
        apply ThisRoleIsNotNewlyPublished{this: SPEC_CHILD_VASP_ROLE_ID()} to * except new_child_vasp_role;
        apply AbortsIfNotParentVASP{account: creating_account} to new_child_vasp_role;

        /// The LibraRoot role is globally unique [C2]. A `RoldId` with `LIBRA_ROOT_ROLE_ID()` can only exists in the
        /// `LIBRA_ROOT_ADDRESS()`. TODO: Verify that `LIBRA_ROOT_ADDRESS()` has a LibraRoot role after `Genesis::initialize`.
        invariant [global] forall addr: address where spec_has_libra_root_role_addr(addr):
          addr == CoreAddresses::SPEC_LIBRA_ROOT_ADDRESS();

        /// The TreasuryCompliance role is globally unique [C3]. A `RoldId` with `TREASURY_COMPLIANCE_ROLE_ID()` can only exists in the
        /// `TREASURY_COMPLIANCE_ADDRESS()`. TODO: Verify that `TREASURY_COMPLIANCE_ADDRESS()` has a TreasuryCompliance role after `Genesis::initialize`.
        invariant [global] forall addr: address where spec_has_treasury_compliance_role_addr(addr):
          addr == CoreAddresses::SPEC_TREASURY_COMPLIANCE_ADDRESS();

        /// LibraRoot cannot have balances [E2].
        invariant [global] forall addr: address where spec_has_libra_root_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// TreasuryCompliance cannot have balances [E3].
        invariant [global] forall addr: address where spec_has_treasury_compliance_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// Validator cannot have balances [E4].
        invariant [global] forall addr: address where spec_has_validator_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// ValidatorOperator cannot have balances [E5].
        invariant [global] forall addr: address where spec_has_validator_operator_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// DesignatedDealer have balances [E6].
        invariant [global] forall addr: address where spec_has_designated_dealer_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// ParentVASP have balances [E7].
        invariant [global] forall addr: address where spec_has_parent_VASP_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// ChildVASP have balances [E8].
        invariant [global] forall addr: address where spec_has_child_VASP_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// DesignatedDealer does not need account limits [F6].
        invariant [global] forall addr: address where spec_has_designated_dealer_role_addr(addr):
            !spec_needs_account_limits_addr(addr);

        /// ParentVASP needs account limits [F7].
        invariant [global] forall addr: address where spec_has_parent_VASP_role_addr(addr):
            spec_needs_account_limits_addr(addr);

        /// ChildVASP needs account limits [F8].
        invariant [global] forall addr: address where spec_has_child_VASP_role_addr(addr):
            spec_needs_account_limits_addr(addr);

        /// update_dual_attestation_limit_privilege is granted to TreasuryCompliance [B16].
        invariant [global] forall addr: address where spec_has_update_dual_attestation_limit_privilege_addr(addr):
            spec_has_treasury_compliance_role_addr(addr);

        /// register_new_currency_privilege is granted to LibraRoot [B18].
        invariant [global] forall addr: address where spec_has_register_new_currency_privilege_addr(addr):
            spec_has_libra_root_role_addr(addr);
    }

    // TODO: Role is supposed to be set by end of genesis?

    // TODO: role-specific privileges persist, and role_ids never change?

    // ## Capabilities
    //
    // TODO: Capability is stored a owner_address unless is_extract == true??
    // TODO: Capability always returned to owner_address

}
}
