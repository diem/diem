script {
    use 0x1::ValidatorConfig;

    /// Set validator's operator callable by validator's owner or by libra_root
    fun set_validator_operator(sender: &signer, validator_account: address, operator_account: address) {
        ValidatorConfig::set_operator(sender, validator_account, operator_account);
     }
}
