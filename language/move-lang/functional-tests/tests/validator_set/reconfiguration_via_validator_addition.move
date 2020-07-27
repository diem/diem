//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: invalidvalidator

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: libraroot
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
//! sender: bob
// bob cannot remove itself, only the libra root account can remove validators from the set
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::remove_validator(account, {{bob}});
    }
}
// check: "ABORTED { code: 2"

//! block-prologue
//! proposer: bob
//! block-time: 4

// check: EXECUTED

//! new-transaction
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::add_validator(account, {{alice}});
    }
}
// check: "ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::add_validator(account, {{invalidvalidator}});
    }
}
// check: "ABORTED { code: 263,"

//! new-transaction
//! sender: libraroot
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

//! new-transaction
//! sender: libraroot
script{
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::add_validator(account, {{alice}});
    }
}
// check: "ABORTED { code: 519,"
