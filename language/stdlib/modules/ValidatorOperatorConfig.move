address 0x1 {

module ValidatorOperatorConfig {
    use 0x1::Errors;
    use 0x1::Signer;
    use 0x1::Roles;
    use 0x1::LibraTimestamp;

    resource struct ValidatorOperatorConfig {
        /// The human readable name of this entity. Immutable.
        human_name: vector<u8>,
    }

    /// The `ValidatorOperatorConfig` was not in the required state
    const EVALIDATOR_OPERATOR_CONFIG: u64 = 0;

    public fun publish(
        validator_operator_account: &signer,
        lr_account: &signer,
        human_name: vector<u8>,
    ) {
        LibraTimestamp::assert_operating();
        Roles::assert_libra_root(lr_account);
        Roles::assert_validator_operator(validator_operator_account);
        assert(
            !has_validator_operator_config(Signer::address_of(validator_operator_account)),
            Errors::already_published(EVALIDATOR_OPERATOR_CONFIG)
        );

        move_to(validator_operator_account, ValidatorOperatorConfig {
            human_name,
        });
    }
    spec fun publish {
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include Roles::AbortsIfNotValidatorOperator{validator_operator_addr: Signer::address_of(validator_operator_account)};
        include PublishAbortsIf {validator_operator_addr: Signer::spec_address_of(validator_operator_account)};
        ensures has_validator_operator_config(Signer::spec_address_of(validator_operator_account));
    }

    spec schema PublishAbortsIf {
        validator_operator_addr: address;
        lr_account: signer;
        include LibraTimestamp::AbortsIfNotOperating;
        include Roles::AbortsIfNotLibraRoot{account: lr_account};
        include Roles::AbortsIfNotValidatorOperator;
        aborts_if has_validator_operator_config(validator_operator_addr)
            with Errors::ALREADY_PUBLISHED;
    }

    /// Get validator's account human name
    /// Aborts if there is no ValidatorOperatorConfig resource
    public fun get_human_name(validator_operator_addr: address): vector<u8> acquires ValidatorOperatorConfig {
        assert(has_validator_operator_config(validator_operator_addr), Errors::not_published(EVALIDATOR_OPERATOR_CONFIG));
        *&borrow_global<ValidatorOperatorConfig>(validator_operator_addr).human_name
    }
    spec fun get_human_name {
        pragma opaque;
        aborts_if !has_validator_operator_config(validator_operator_addr) with Errors::NOT_PUBLISHED;
        ensures result == get_human_name(validator_operator_addr);
    }
    public fun has_validator_operator_config(validator_operator_addr: address): bool {
        exists<ValidatorOperatorConfig>(validator_operator_addr)
    }
    spec fun has_validator_operator_config {
        ensures result == has_validator_operator_config(validator_operator_addr);
    }

    /// If address has a ValidatorOperatorConfig, it has a validator operator role.
    /// This invariant is useful in LibraSystem so we don't have to check whether
    /// every validator address has a validator role.
    spec module {
        invariant [global] forall addr1:address where has_validator_operator_config(addr1):
            Roles::spec_has_validator_operator_role_addr(addr1);
    }

}
}
