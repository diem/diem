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
    use 0x1::Errors;
    use 0x1::LibraTimestamp;

    const EROLE_ID: u64 = 0;
    const ELIBRA_ROOT: u64 = 1;
    const ETREASURY_COMPLIANCE: u64 = 2;
    const EPARENT_VASP: u64 = 3;
    const ELIBRA_ROOT_OR_TREASURY_COMPLIANCE: u64 = 4;
    const EPARENT_VASP_OR_DESIGNATED_DEALER: u64 = 5;
    const EDESIGNATED_DEALER: u64 = 6;
    const EVALIDATOR: u64 = 7;
    const EVALIDATOR_OPERATOR: u64 = 8;

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
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        // Grant the role to the libra root account
        grant_role(lr_account, LIBRA_ROOT_ROLE_ID);
    }
    spec fun grant_libra_root_role {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotLibraRoot{account: lr_account};
        include GrantRole{account: lr_account, role_id: LIBRA_ROOT_ROLE_ID};
    }

    /// NB: currency-related privileges are defined in the `Libra` module.
    /// Granted in genesis. So there cannot be any pre-existing privileges
    /// and roles.
    public fun grant_treasury_compliance_role(
        treasury_compliance_account: &signer,
        lr_account: &signer,
    ) acquires RoleId {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_treasury_compliance(treasury_compliance_account);
        assert_libra_root(lr_account);
        // Grant the TC role to the treasury_compliance_account
        grant_role(treasury_compliance_account, TREASURY_COMPLIANCE_ROLE_ID);
    }
    spec fun grant_treasury_compliance_role {
        include LibraTimestamp::AbortsIfNotGenesis;
        include CoreAddresses::AbortsIfNotTreasuryCompliance{account: treasury_compliance_account};
        include AbortsIfNotLibraRoot{account: lr_account};
        include GrantRole{account: treasury_compliance_account, role_id: TREASURY_COMPLIANCE_ROLE_ID};
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
        assert_treasury_compliance(creating_account);
        grant_role(new_account, DESIGNATED_DEALER_ROLE_ID);
    }
    spec fun new_designated_dealer_role {
        include AbortsIfNotTreasuryCompliance{account: creating_account};
        include GrantRole{account: new_account, role_id: DESIGNATED_DEALER_ROLE_ID};
    }

    /// Publish a Validator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_role(
        creating_account: &signer,
        new_account: &signer
    ) acquires RoleId {
        assert_libra_root(creating_account);
        grant_role(new_account, VALIDATOR_ROLE_ID);
    }
    spec fun new_validator_role {
        include AbortsIfNotLibraRoot{account: creating_account};
        include GrantRole{account: new_account, role_id: VALIDATOR_ROLE_ID};
    }

    /// Publish a ValidatorOperator `RoleId` under `new_account`.
    /// The `creating_account` must be LibraRoot
    public fun new_validator_operator_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        assert_libra_root(creating_account);
        grant_role(new_account, VALIDATOR_OPERATOR_ROLE_ID);
    }
    spec fun new_validator_operator_role {
        include AbortsIfNotLibraRoot{account: creating_account};
        include GrantRole{account: new_account, role_id: VALIDATOR_OPERATOR_ROLE_ID};
    }

    /// Publish a ParentVASP `RoleId` under `new_account`.
    /// The `creating_account` must be TreasuryCompliance
    public fun new_parent_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        // TODO(wrwg): this is implemented different than the doc. Which of them is the truth?
        assert_libra_root(creating_account);
        grant_role(new_account, PARENT_VASP_ROLE_ID);
    }
    spec fun new_parent_vasp_role {
        include AbortsIfNotLibraRoot{account: creating_account};
        include GrantRole{account: new_account, role_id: PARENT_VASP_ROLE_ID};
    }

    /// Publish a ChildVASP `RoleId` under `new_account`.
    /// The `creating_account` must be a ParentVASP
    public fun new_child_vasp_role(
        creating_account: &signer,
        new_account: &signer,
    ) acquires RoleId {
        assert_parent_vasp_role(creating_account);
        grant_role(new_account, CHILD_VASP_ROLE_ID);
    }
    spec fun new_child_vasp_role {
        include AbortsIfNotParentVasp{account: creating_account};
        include GrantRole{account: new_account, role_id: CHILD_VASP_ROLE_ID};
    }

    /// Helper function to grant a role.
    fun grant_role(account: &signer, role_id: u64) {
        assert(!exists<RoleId>(Signer::address_of(account)), Errors::already_published(EROLE_ID));
        move_to(account, RoleId { role_id });
    }
    spec fun grant_role {
        pragma opaque;
        include GrantRole;
    }
    spec schema GrantRole {
        account: signer;
        role_id: num;
        let addr = Signer::spec_address_of(account);
        // Requires to satisfy global invariants.
        requires role_id == LIBRA_ROOT_ROLE_ID ==> addr == CoreAddresses::LIBRA_ROOT_ADDRESS();
        requires role_id == TREASURY_COMPLIANCE_ROLE_ID ==> addr == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();
        aborts_if exists<RoleId>(addr) with Errors::ALREADY_PUBLISHED;
        ensures exists<RoleId>(addr);
        ensures global<RoleId>(addr).role_id == role_id;
        modifies global<RoleId>(addr);
    }

    //  ## privilege-checking functions for roles ##

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

    //  ## role assertions ##

    /// Assert that the account is libra root.
    ///
    /// TODO(wrwg): previously throughout the framework, we had functions which only check for the role, and
    ///   functions which check both for role and address. This is now unified via this function to always
    ///   check for both. However, the address check might be considered redundant, as we already have a global
    ///   invariant that the role of libra root and TC can only be at a specific address.
    public fun assert_libra_root(account: &signer) acquires RoleId {
        CoreAddresses::assert_libra_root(account);
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(borrow_global<RoleId>(addr).role_id == LIBRA_ROOT_ROLE_ID, Errors::requires_role(ELIBRA_ROOT));
    }
    spec fun assert_libra_root {
        pragma opaque;
        include CoreAddresses::AbortsIfNotLibraRoot;
        include AbortsIfNotLibraRoot;
    }

    /// Assert that the account is treasury compliance.
    ///
    /// TODO(wrwg): see discussion for `assert_libra_root`
    public fun assert_treasury_compliance(account: &signer) acquires RoleId {
        CoreAddresses::assert_treasury_compliance(account);
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(
            borrow_global<RoleId>(addr).role_id == TREASURY_COMPLIANCE_ROLE_ID,
            Errors::requires_role(ETREASURY_COMPLIANCE)
        )
    }
    spec fun assert_treasury_compliance {
        pragma opaque;
        include AbortsIfNotTreasuryCompliance;
    }

    /// Assert that the account has the parent vasp role.
    public fun assert_parent_vasp_role(account: &signer) acquires RoleId {
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(
            borrow_global<RoleId>(addr).role_id == PARENT_VASP_ROLE_ID,
            Errors::requires_role(EPARENT_VASP)
        )
    }
    spec fun assert_parent_vasp_role {
        pragma opaque;
        include AbortsIfNotParentVasp;
    }

    /// Assert that the account has the designated dealer role.
    public fun assert_designated_dealer(account: &signer) acquires RoleId {
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(
            borrow_global<RoleId>(addr).role_id == DESIGNATED_DEALER_ROLE_ID,
            Errors::requires_role(EDESIGNATED_DEALER)
        )
    }
    spec fun assert_designated_dealer {
        pragma opaque;
        include AbortsIfNotDesignatedDealer;
    }

    /// Assert that the account has the validator role.
    public fun assert_validator(account: &signer) acquires RoleId {
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(
            borrow_global<RoleId>(addr).role_id == VALIDATOR_ROLE_ID,
            Errors::requires_role(EVALIDATOR)
        )
    }
    spec fun assert_validator {
        pragma opaque;
        include AbortsIfNotValidator;
    }

    /// Assert that the account has the validator operator role.
    public fun assert_validator_operator(account: &signer) acquires RoleId {
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        assert(
            borrow_global<RoleId>(addr).role_id == VALIDATOR_OPERATOR_ROLE_ID,
            Errors::requires_role(EVALIDATOR_OPERATOR)
        )
    }
    spec fun assert_validator_operator {
        pragma opaque;
        include AbortsIfNotValidatorOperator;
    }


    /// Assert that the account has either the libra root or treasury compliance role.
    public fun assert_libra_root_or_treasury_compliance(account: &signer) acquires RoleId {
        CoreAddresses::assert_libra_root_or_treasury_compliance(account);
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        let role_id = borrow_global<RoleId>(addr).role_id;
        assert(
            role_id == LIBRA_ROOT_ROLE_ID || role_id == TREASURY_COMPLIANCE_ROLE_ID,
            Errors::requires_role(ELIBRA_ROOT_OR_TREASURY_COMPLIANCE)
        )
    }
    spec fun assert_libra_root_or_treasury_compliance {
        pragma opaque;
        include AbortsIfNotLibraRootOrTreasuryCompliance;
    }

    /// Assert that the account has either the parent vasp or designated dealer role.
    public fun assert_parent_vasp_or_designated_dealer(account: &signer) acquires RoleId {
        let addr = Signer::address_of(account);
        assert(exists<RoleId>(addr), Errors::not_published(EROLE_ID));
        let role_id = borrow_global<RoleId>(addr).role_id;
        assert(
            role_id == PARENT_VASP_ROLE_ID || role_id == DESIGNATED_DEALER_ROLE_ID,
            Errors::requires_role(EPARENT_VASP_OR_DESIGNATED_DEALER)
        );
    }
    spec fun assert_parent_vasp_or_designated_dealer {
        pragma opaque;
        include AbortsIfNotParentVaspOrDesignatedDealer;
    }


    //**************** Specifications ****************

    spec module {
        pragma verify = true;
    }

    /// ## Helper Functions and Schemas

    spec module {
        define spec_get_role_id(account: signer): u64 {
            let addr = Signer::spec_address_of(account);
            global<RoleId>(addr).role_id
        }

        define spec_has_role_id_addr(addr: address, role_id: u64): bool {
            exists<RoleId>(addr) && global<RoleId>(addr).role_id == role_id
        }

        define spec_has_libra_root_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, LIBRA_ROOT_ROLE_ID)
        }

        define spec_has_treasury_compliance_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, TREASURY_COMPLIANCE_ROLE_ID)
        }

        define spec_has_designated_dealer_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, DESIGNATED_DEALER_ROLE_ID)
        }

        define spec_has_validator_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, VALIDATOR_ROLE_ID)
        }

        define spec_has_validator_operator_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, VALIDATOR_OPERATOR_ROLE_ID)
        }

        define spec_has_parent_VASP_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, PARENT_VASP_ROLE_ID)
        }

        define spec_has_child_VASP_role_addr(addr: address): bool {
            spec_has_role_id_addr(addr, CHILD_VASP_ROLE_ID)
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

    spec schema ThisRoleIsNotNewlyPublished {
        this: u64;
        ensures forall addr: address where exists<RoleId>(addr) && global<RoleId>(addr).role_id == this:
            old(exists<RoleId>(addr)) && old(global<RoleId>(addr).role_id) == this;
    }

    spec schema AbortsIfNotLibraRoot {
        account: signer;
        // TODO(wrwg): potentially remove the address check, as it follows from invariant.
        include CoreAddresses::AbortsIfNotLibraRoot;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != LIBRA_ROOT_ROLE_ID with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotTreasuryCompliance {
        account: signer;
        // TODO(wrwg): potentially remove this address check, as it follows from invariant.
        include CoreAddresses::AbortsIfNotTreasuryCompliance;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != TREASURY_COMPLIANCE_ROLE_ID with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotLibraRootOrTreasuryCompliance {
        account: signer;
        // TODO(wrwg): potentially remove this address check, as it follows from invariant.
        include CoreAddresses::AbortsIfNotLibraRootOrTreasuryCompliance;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        let role_id = global<RoleId>(addr).role_id;
        aborts_if role_id != LIBRA_ROOT_ROLE_ID && role_id != TREASURY_COMPLIANCE_ROLE_ID
            with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotParentVasp {
        account: signer;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != PARENT_VASP_ROLE_ID with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotDesignatedDealer {
        account: signer;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != DESIGNATED_DEALER_ROLE_ID with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotParentVaspOrDesignatedDealer {
        account: signer;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        let role_id = global<RoleId>(addr).role_id;
        aborts_if role_id != PARENT_VASP_ROLE_ID && role_id != DESIGNATED_DEALER_ROLE_ID
            with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotValidator {
        account: signer;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != VALIDATOR_ROLE_ID with Errors::REQUIRES_ROLE;
    }

    spec schema AbortsIfNotValidatorOperator {
        account: signer;
        let addr = Signer::spec_address_of(account);
        aborts_if !exists<RoleId>(addr) with Errors::NOT_PUBLISHED;
        aborts_if global<RoleId>(addr).role_id != VALIDATOR_OPERATOR_ROLE_ID with Errors::REQUIRES_ROLE;
    }


    /// ## Persistence of Roles

    /// **Informally:** Once an account at address `A` is granted a role `R` it
    /// will remain an account with role `R` for all time.
    spec module {
        invariant update [global]
            forall addr: address where old(exists<RoleId>(addr)):
                exists<RoleId>(addr) && old(global<RoleId>(addr).role_id) == global<RoleId>(addr).role_id;
    }

    /// ## Conditions from Requirements

    /// In this section, the conditions from the requirements for access control are systematically
    /// applied to the functions in this module. While some of those conditions have already been
    /// included in individual function specifications, listing them here again gives additional
    /// assurance that that all requirements are covered.

    /// TODO(wrwg): link to requirements

    spec module {
        /// Validator roles are only granted by LibraRoot [B4]. A new `RoleId` with `VALIDATOR_ROLE_ID()` is only
        /// published through `new_validator_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: VALIDATOR_ROLE_ID} to * except new_validator_role, grant_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_validator_role;

        /// ValidatorOperator roles are only granted by LibraRoot [B5]. A new `RoleId` with `VALIDATOR_OPERATOR_ROLE_ID()` is only
        /// published through `new_validator_operator_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: VALIDATOR_OPERATOR_ROLE_ID} to * except new_validator_operator_role, grant_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_validator_operator_role;

        /// DesignatedDealer roles are only granted by TreasuryCompliance [B6](TODO: resolve the discrepancy). A new `RoleId` with `DESIGNATED_DEALER_ROLE_ID()` is only
        /// published through `new_designated_dealer_role` which aborts if `creating_account` does not have the TreasuryCompliance role.
        apply ThisRoleIsNotNewlyPublished{this: DESIGNATED_DEALER_ROLE_ID} to * except new_designated_dealer_role, grant_role;
        apply AbortsIfNotTreasuryCompliance{account: creating_account} to new_designated_dealer_role;

        /// ParentVASP roles are only granted by LibraRoot [B7]. A new `RoleId` with `PARENT_VASP_ROLE_ID()` is only
        /// published through `new_parent_vasp_role` which aborts if `creating_account` does not have the LibraRoot role.
        apply ThisRoleIsNotNewlyPublished{this: PARENT_VASP_ROLE_ID} to * except new_parent_vasp_role, grant_role;
        apply AbortsIfNotLibraRoot{account: creating_account} to new_parent_vasp_role;

        /// ChildVASP roles are only granted by ParentVASP [B8]. A new `RoleId` with `CHILD_VASP_ROLE_ID()` is only
        /// published through `new_child_vasp_role` which aborts if `creating_account` does not have the ParentVASP role.
        apply ThisRoleIsNotNewlyPublished{this: CHILD_VASP_ROLE_ID} to * except new_child_vasp_role, grant_role;
        apply AbortsIfNotParentVasp{account: creating_account} to new_child_vasp_role;

        /// The LibraRoot role is globally unique [C2]. A `RoleId` with `LIBRA_ROOT_ROLE_ID()` can only exists in the
        /// `LIBRA_ROOT_ADDRESS()`. TODO: Verify that `LIBRA_ROOT_ADDRESS()` has a LibraRoot role after `Genesis::initialize`.
        invariant [global, on_update] forall addr: address where spec_has_libra_root_role_addr(addr):
          addr == CoreAddresses::LIBRA_ROOT_ADDRESS();

        /// The TreasuryCompliance role is globally unique [C3]. A `RoleId` with `TREASURY_COMPLIANCE_ROLE_ID()` can only exists in the
        /// `TREASURY_COMPLIANCE_ADDRESS()`. TODO: Verify that `TREASURY_COMPLIANCE_ADDRESS()` has a TreasuryCompliance role after `Genesis::initialize`.
        invariant [global, on_update] forall addr: address where spec_has_treasury_compliance_role_addr(addr):
          addr == CoreAddresses::TREASURY_COMPLIANCE_ADDRESS();

        /// LibraRoot cannot have balances [E2].
        invariant [global, on_update] forall addr: address where spec_has_libra_root_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// TreasuryCompliance cannot have balances [E3].
        invariant [global, on_update] forall addr: address where spec_has_treasury_compliance_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// Validator cannot have balances [E4].
        invariant [global, on_update] forall addr: address where spec_has_validator_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// ValidatorOperator cannot have balances [E5].
        invariant [global, on_update] forall addr: address where spec_has_validator_operator_role_addr(addr):
            !spec_can_hold_balance_addr(addr);

        /// DesignatedDealer have balances [E6].
        invariant [global, on_update] forall addr: address where spec_has_designated_dealer_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// ParentVASP have balances [E7].
        invariant [global, on_update] forall addr: address where spec_has_parent_VASP_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// ChildVASP have balances [E8].
        invariant [global, on_update] forall addr: address where spec_has_child_VASP_role_addr(addr):
            spec_can_hold_balance_addr(addr);

        /// DesignatedDealer does not need account limits [F6].
        invariant [global, on_update] forall addr: address where spec_has_designated_dealer_role_addr(addr):
            !spec_needs_account_limits_addr(addr);

        /// ParentVASP needs account limits [F7].
        invariant [global, on_update] forall addr: address where spec_has_parent_VASP_role_addr(addr):
            spec_needs_account_limits_addr(addr);

        /// ChildVASP needs account limits [F8].
        invariant [global, on_update] forall addr: address where spec_has_child_VASP_role_addr(addr):
            spec_needs_account_limits_addr(addr);

        /// update_dual_attestation_limit_privilege is granted to TreasuryCompliance [B16].
        invariant [global, on_update] forall addr: address where spec_has_update_dual_attestation_limit_privilege_addr(addr):
            spec_has_treasury_compliance_role_addr(addr);

        /// register_new_currency_privilege is granted to LibraRoot [B18].
        invariant [global, on_update] forall addr: address where spec_has_register_new_currency_privilege_addr(addr):
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
