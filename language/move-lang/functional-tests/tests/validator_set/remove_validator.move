//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: libraroot
// remove_validator cannot be called on a non-validator
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{alice}});
    }
}

// check: "Keep(ABORTED { code: 775,"

// remove_validator can only be called by the Association
//! new-transaction
//! sender: alice
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{vivian}});
    }
}

// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
// should work because Vivian is a validator
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{vivian}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
// double-removing Vivian should fail
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{vivian}});
    }
}

// check: "Keep(ABORTED { code: 775,"
