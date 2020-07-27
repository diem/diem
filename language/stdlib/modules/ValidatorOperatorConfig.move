address 0x1 {

module ValidatorOperatorConfig {
    use 0x1::Signer;
    use 0x1::Roles;

    resource struct ValidatorOperatorConfig {
        /// The human readable name of this entity. Immutable.
        human_name: vector<u8>,
    }

    const ENOT_LIBRA_ROOT: u64 = 0;
    const EVALIDATOR_OPERATOR_RESOURCE_DOES_NOT_EXIST: u64 = 1;

    public fun publish(
        account: &signer,
        lr_account: &signer,
        human_name: vector<u8>,
        ) {
        assert(Roles::has_libra_root_role(lr_account), ENOT_LIBRA_ROOT);
        move_to(account, ValidatorOperatorConfig {
            human_name,
        });
    }

    spec fun publish {
        aborts_if !Roles::spec_has_libra_root_role_addr(Signer::spec_address_of(lr_account));
        aborts_if spec_exists_config(Signer::spec_address_of(account));
        ensures spec_exists_config(Signer::spec_address_of(account));
    }

    spec module {
        /// Returns true if a ValidatorOperatorConfig resource exists under addr.
        define spec_exists_config(addr: address): bool {
            exists<ValidatorOperatorConfig>(addr)
        }

        /// Returns the human name of the validator
        define spec_get_human_name(addr: address): vector<u8> {
            global<ValidatorOperatorConfig>(addr).human_name
        }
    }


    /// Get validator's account human name
    /// Aborts if there is no ValidatorOperatorConfig resource
    public fun get_human_name(addr: address): vector<u8> acquires ValidatorOperatorConfig {
        assert(exists<ValidatorOperatorConfig>(addr), EVALIDATOR_OPERATOR_RESOURCE_DOES_NOT_EXIST);
        let t_ref = borrow_global<ValidatorOperatorConfig>(addr);
        *&t_ref.human_name
    }

    spec fun get_human_name {
        pragma opaque = true;
        aborts_if !spec_exists_config(addr);
        ensures result == spec_get_human_name(addr);
    }
}
}
