module 0x2::CreateValidatorAccount {
    struct CreateValidatorAccount has store, drop, copy {
        id: u64,
    }

    public fun create_validator_account(id: u64): CreateValidatorAccount {
        CreateValidatorAccount {
            id
        }
    }
}