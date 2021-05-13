// Check that removing a non-existent validator aborts.

//! account: alice
//! account: bob, 1000000, 0, validator

//! sender: alice
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // alice cannot remove herself
        DiemSystem::remove_validator(account, @{{alice}});
    }
}

// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // alice cannot remove bob
        DiemSystem::remove_validator(account, @{{bob}});
    }
}

// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // bob cannot remove alice
        DiemSystem::remove_validator(account, @{{alice}});
    }
}

// check: "Keep(ABORTED { code: 2,"
