address 0x1 {

module ValidatorOperatorConfig {
    use 0x1::Errors;
    use 0x1::Signer;
    use 0x1::Roles;

    resource struct ValidatorOperatorConfig {
        /// The human readable name of this entity. Immutable.
        human_name: vector<u8>,
    }

    const EVALIDATOR_OPERATOR_CONFIG: u64 = 0;

    public fun publish(
        account: &signer,
        lr_account: &signer,
        human_name: vector<u8>,
    ) {
        Roles::assert_libra_root(lr_account);
        Roles::assert_validator_operator(account);
        assert(
            !has_validator_operator_config(Signer::address_of(account)),
            Errors::already_published(EVALIDATOR_OPERATOR_CONFIG)
        );

        move_to(account, ValidatorOperatorConfig {
            human_name,
        });
    }
    spec fun publish {
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include Roles::AbortsIfNotValidatorOperator;
        aborts_if has_validator_operator_config(Signer::spec_address_of(account)) with Errors::ALREADY_PUBLISHED;
        ensures has_validator_operator_config(Signer::spec_address_of(account));
    }


    /// Get validator's account human name
    /// Aborts if there is no ValidatorOperatorConfig resource
    public fun get_human_name(addr: address): vector<u8> acquires ValidatorOperatorConfig {
        assert(has_validator_operator_config(addr), Errors::not_published(EVALIDATOR_OPERATOR_CONFIG));
        *&borrow_global<ValidatorOperatorConfig>(addr).human_name
    }
    spec fun get_human_name {
        pragma opaque;
        aborts_if !has_validator_operator_config(addr) with Errors::NOT_PUBLISHED;
        ensures result == get_human_name(addr);
    }
    public fun has_validator_operator_config(addr: address): bool {
        exists<ValidatorOperatorConfig>(addr)
    }
    spec fun has_validator_operator_config {
        ensures result == has_validator_operator_config(addr);
    }
}
}
