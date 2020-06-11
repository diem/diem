//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: alice
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{alice}});
        assert(!LibraSystem::is_validator({{alice}}), 77);
        assert(LibraSystem::is_validator({{bob}}), 78);
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
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{bob}});
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
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::add_validator(account, {{alice}});

        assert(LibraSystem::is_validator({{alice}}), 77);
        assert(LibraSystem::is_validator({{bob}}), 78);
    }
}
// check: NewEpochEvent
// check: EXECUTED
