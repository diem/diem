//! account: alice
//! account: bob
//! account: vivian, 1000000, 0, validator

//! sender: alice
// register Alice as a validator candidate
script{
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::register_candidate_validator(x"10", x"20", x"30", x"40", x"50", x"60");
}
}
//! new-transaction
//! sender: bob
// register Bob as a validator candidate
script{
use 0x0::ValidatorConfig;
fun main() {
    ValidatorConfig::register_candidate_validator(x"11", x"22", x"33", x"44", x"55", x"66");
}
}

//! block-prologue
//! proposer: vivian
//! block-time: 2

//! new-transaction
//! sender: association
// run a tx from the association that adds Alice and Bob as validators, this will fail as only one reconfiguration is
// allowed per block
script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::add_validator({{alice}});
    LibraSystem::add_validator({{bob}});

    // this will not take affect until the next epoch
    0x0::Transaction::assert(!LibraSystem::is_validator({{alice}}), 77);
    0x0::Transaction::assert(!LibraSystem::is_validator({{bob}}), 78);
}
}
// check: ABORT
// check: 23


//! new-transaction
//! sender: association
// run a tx from the association that adds Alice as a validator
script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::add_validator({{alice}});

    0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 77);
    0x0::Transaction::assert(!LibraSystem::is_validator({{bob}}), 78);
}
}
// check: NewEpochEvent
// check: EXECUTED

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
// run a tx from the association that adds Alice as a validator
script{
use 0x0::LibraSystem;
fun main() {
    LibraSystem::add_validator({{bob}});

    0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 77);
    0x0::Transaction::assert(LibraSystem::is_validator({{bob}}), 78);
}
}
// check: NewEpochEvent
// check: EXECUTED

//! new-transaction
// check that Alice and Bob are now validators
script{
use 0x0::LibraSystem;
fun main() {
    0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 79);
    0x0::Transaction::assert(LibraSystem::is_validator({{bob}}), 80);

    // check that Vivian is also still a validator
    0x0::Transaction::assert(LibraSystem::is_validator({{vivian}}), 81);
}
}

// check: EXECUTED
