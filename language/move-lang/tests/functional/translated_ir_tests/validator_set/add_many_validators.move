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
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: bob
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: carrol
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: david
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: eve
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! new-transaction
//! sender: fedor
use 0x0::ValidatorConfig2;
fun main() {
    ValidatorConfig2::initialize(x"beefbeef", x"10", x"20", x"30", x"40", x"50");
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 2

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem2;
// add allice to validator set
fun main() {
    0x0::Transaction::assert(LibraSystem2::validator_set_size() == 0, 99);
    LibraSystem2::add_validator({{alice}});
    0x0::Transaction::assert(LibraSystem2::is_validator({{alice}}), 100);
    0x0::Transaction::assert(!LibraSystem2::is_validator({{bob}}), 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 3

// check: EXECUTED

//! new-transaction
//! sender: association
use 0x0::LibraSystem2;
// add bob to validator set
fun main() {
    LibraSystem2::add_validator({{bob}});
}

// check: EXECUTED

//! new-transaction
//! sender: association
// adding carrol to the validator set in the same block will fail
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::add_validator({{carrol}});
}

// check: ABORTED

//! block-prologue
//! proposer: bob
//! block-time: 4

// check: EXECUTED

//! new-transaction
//! sender: association
// test that adding two validators in the same transaction aborts the transaction
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::add_validator({{carrol}});
    LibraSystem2::add_validator({{david}});
}

// check: ABORTED

//! new-transaction
//! sender: association
// add carrol to the validator set
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::add_validator({{carrol}});
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 5

// check: EXECUTED

//! new-transaction
//! sender: association
// add david to the validator set
use 0x0::LibraSystem2;
fun main() {
    0x0::Transaction::assert(!LibraSystem2::is_validator({{david}}), 99);
    LibraSystem2::add_validator({{david}});
    0x0::Transaction::assert(LibraSystem2::is_validator({{david}}), 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 6

// check: EXECUTED

//! new-transaction
//! sender: association
// add eve to validator set
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::add_validator({{eve}});
}

// check: EXECUTED
//! block-prologue
//! proposer: bob
//! block-time: 7

// check: EXECUTED

//! new-transaction
//! sender: association
// add fedor to validator set
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::add_validator({{fedor}});
    0x0::Transaction::assert(LibraSystem2::validator_set_size() == 6, 99);
}

// check: EXECUTED
//! block-prologue
//! proposer: bob
//! block-time: 8

// check: EXECUTED

//! new-transaction
//! sender: association
// remove alice
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::remove_validator({{alice}});
    0x0::Transaction::assert(LibraSystem2::validator_set_size() == 5, 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 9

// check: EXECUTED

//! new-transaction
//! sender: association
// test deleting a non-existent (or rather already removed) validator aborts
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::remove_validator({{alice}});
}

// check: ABORTED

//! new-transaction
//! sender: association
// remove fedor
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::remove_validator({{fedor}});
    0x0::Transaction::assert(LibraSystem2::validator_set_size() == 4, 99);
}

// check: EXECUTED

//! block-prologue
//! proposer: bob
//! block-time: 10

// check: EXECUTED

//! new-transaction
//! sender: association
// make sure nobody else can be deleted
// the safe number of validators is 4
// this test makes sure the number of validators can not go lower
use 0x0::LibraSystem2;
fun main() {
    LibraSystem2::remove_validator({{david}});
}

// check: ABORTED
