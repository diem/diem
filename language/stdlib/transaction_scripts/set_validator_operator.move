script {
    use 0x1::ValidatorConfig;

    /// Set validator's operator
    fun set_validator_operator(account: &signer, operator_account: address) {
        ValidatorConfig::set_operator(account, operator_account);
     }
}
