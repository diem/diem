// This code adds 6 validators, checks that 2 can be removed safely,
// but more than 2 cannot be removed.
// It also checks that only one change can be made per block, i.e.
// if a transaction adds two validators - this transaction will fail,
// if the first transaction adds a validator and the second transaction
// adds a validator in the same block, then the first transaction will succeed
// and the second will fail.
// Check that removing a non-existent validator aborts.

//! account: alice
//! account: bob, 1000000, 0, validator
//! account: carrol
//! account: david
//! account: eve
//! account: fedor

//! sender: alice
use 0x0::ValidatorConfig;
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: bob
use 0x0::ValidatorConfig;
// try to double-initialize a validator config should fail
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: CANNOT_WRITE_EXISTING_RESOURCE

//! new-transaction
//! sender: carrol
use 0x0::ValidatorConfig;
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: david
use 0x0::ValidatorConfig;
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: eve
use 0x0::ValidatorConfig;
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: fedor
use 0x0::ValidatorConfig;
// validators: bob
fun main() {
    ValidatorConfig::register_candidate_validator(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem;
// add allice to validator set
// validators: bob
fun main() {
    0x0::Transaction::assert(LibraSystem::validator_set_size() == 1, 99);
    LibraSystem::add_validator({{alice}});
    0x0::Transaction::assert(LibraSystem::is_validator({{alice}}), 100);
    0x0::Transaction::assert(LibraSystem::is_validator({{bob}}), 101);
    0x0::Transaction::assert(!LibraSystem::is_validator({{eve}}), 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem;
// add bob to validator set fails as it already exists
// validators: alice, bob
fun main() {
    LibraSystem::add_validator({{bob}});
}

// check: ABORTED

//! new-transaction
//! sender: association
// adding two validators in the same block fails for now
use 0x0::LibraSystem;
// validators: alice, bob
fun main() {
    LibraSystem::add_validator({{carrol}});
    LibraSystem::add_validator({{david}});
}

// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 4

// check: EXECUTED

//! new-transaction
//! sender: association
// test that adding two validators in the same transaction aborts the transaction
use 0x0::LibraSystem;
// validators: alice, bob
fun main() {
    LibraSystem::add_validator({{carrol}});
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 5

// check: EXECUTED

//! new-transaction
//! sender: association
// add david to the validator set
use 0x0::LibraSystem;
// validators: alice, bob, carrol
fun main() {
    0x0::Transaction::assert(!LibraSystem::is_validator({{david}}), 99);
    LibraSystem::add_validator({{david}});
    0x0::Transaction::assert(LibraSystem::is_validator({{david}}), 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 6

// check: EXECUTED

//! new-transaction
//! sender: association
// add eve to validator set
use 0x0::LibraSystem;
// validators: alice, bob, carrol, david
fun main() {
    LibraSystem::add_validator({{eve}});
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 7

// check: EXECUTED

//! new-transaction
//! sender: association
// add fedor to validator set
use 0x0::LibraSystem;
// validators: alice, bob, carrol, david, eve
fun main() {
    LibraSystem::add_validator({{fedor}});
    0x0::Transaction::assert(LibraSystem::validator_set_size() == 6, 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 8

// check: EXECUTED

//! new-transaction
//! sender: association
// remove alice
use 0x0::LibraSystem;
// validators: alice, bob, carrol, david, eve, fedor
fun main() {
    LibraSystem::remove_validator({{alice}});
    0x0::Transaction::assert(LibraSystem::validator_set_size() == 5, 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 9

// check: EXECUTED

//! new-transaction
//! sender: association
// test deleting a non-existent (or rather already removed) validator aborts
use 0x0::LibraSystem;
// validators: bob, carrol, david, eve, fedor
fun main() {
    LibraSystem::remove_validator({{alice}});
}

// check: ABORTED

//! new-transaction
//! sender: association
// remove fedor
use 0x0::LibraSystem;
// validators: bob, carrol, david, eve, fedor
fun main() {
    LibraSystem::remove_validator({{fedor}});
    0x0::Transaction::assert(LibraSystem::validator_set_size() == 4, 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 10

// check: EXECUTED
