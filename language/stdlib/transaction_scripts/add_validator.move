script {
    use 0x1::LibraSystem;

    // Add Validator to the set, called by the validator's operator
    fun add_validator(account: &signer, validator_address: address) {
        LibraSystem::add_validator(account, validator_address);
    }
}
