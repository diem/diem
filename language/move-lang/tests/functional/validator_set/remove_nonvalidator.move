// Check that removing a non-existent validator aborts.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: alice
script {
use 0x0::LibraSystem;
fun main() {
    // alice cannot remove itself
    LibraSystem::remove_validator({{alice}});
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use 0x0::LibraSystem;
fun main() {
    // alice cannot remove bob
    LibraSystem::remove_validator({{bob}});
}
}

// check: ABORTED

//! new-transaction
//! sender: bob
script {
use 0x0::LibraSystem;
fun main() {
    // bob cannot remove alice
    LibraSystem::remove_validator({{alice}});
}
}

// check: ABORTED
