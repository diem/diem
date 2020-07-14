// Check that removing a non-existent validator aborts.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: alice
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        // alice cannot remove herself
        LibraSystem::remove_validator(account, {{alice}});
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED

//! new-transaction
//! sender: alice
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        // alice cannot remove bob
        LibraSystem::remove_validator(account, {{bob}});
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        // bob cannot remove alice
        LibraSystem::remove_validator(account, {{alice}});
    }
}

// TODO(status_migration) remove duplicate check
// check: ABORTED
// check: ABORTED
