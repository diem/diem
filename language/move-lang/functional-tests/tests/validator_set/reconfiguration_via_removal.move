//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: viola, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: viola
//! block-time: 2

//! new-transaction
//! sender: vivian
script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::remove_validator({{vivian}});
}
}

// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: viola
//! block-time: 4

//! new-transaction
// check that Vivian is no longer a validator, Alice is not, but Viola is still a
// validator
script{
use 0x0::LibraSystem;
fun main() {
    0x0::Transaction::assert(!LibraSystem::is_validator({{vivian}}), 70);
    0x0::Transaction::assert(!LibraSystem::is_validator({{alice}}), 71);
    0x0::Transaction::assert(LibraSystem::is_validator({{viola}}), 72);
}
}

// check: EXECUTED
