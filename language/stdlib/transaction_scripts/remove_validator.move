script {
    use 0x0::LibraSystem;

    // Callable by Validator's operator
    fun main(validator_address: address) {
        LibraSystem::remove_validator(validator_address);
    }
}
