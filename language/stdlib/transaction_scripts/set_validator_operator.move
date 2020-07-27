script {
    use 0x1::ValidatorConfig;
    use 0x1::ValidatorOperatorConfig;

    /// Set validator's operator
    fun set_validator_operator(
        account: &signer,
        operator_name: vector<u8>,
        operator_account: address
    ) {
        assert(ValidatorOperatorConfig::get_human_name(operator_account) == operator_name, 0);
        ValidatorConfig::set_operator(account, operator_account);
    }
}
