script {
    use 0x0::LibraSystem;

    // Add Validator to the set, called by the validator's operator
    fun main(validator_address: address) {
        LibraSystem::add_validator(validator_address);
    }
}
