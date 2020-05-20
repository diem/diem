//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{alice}});
        0x0::Transaction::assert(!LibraSystem::is_validator({{alice}}), 77);
        0x0::Transaction::assert(LibraSystem::is_validator({{bob}}), 78);
    }
}
// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::remove_validator({{bob}});
    }
}
// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 4

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::add_validator({{alice}});

        0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 77);
        0x0::Transaction::assert(LibraSystem::is_validator({{bob}}), 78);
    }
}
// check: NewEpochEvent
// check: EXECUTED
