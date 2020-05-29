//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
// remove_validator cannot be called on a non-validator
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{alice}});
    }
}

// check: ABORTED
// check: 1106

// remove_validator can only be called by the Association
//! new-transaction
//! sender: alice
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{vivian}});
    }
}

// check: ABORTED
// check: 22

//! new-transaction
//! sender: vivian
// should work because Vivian is a validator
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{vivian}});
    }
}

// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
//! sender: vivian
// double-removing Vivian should fail
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{vivian}});
    }
}

// check: ABORTED
