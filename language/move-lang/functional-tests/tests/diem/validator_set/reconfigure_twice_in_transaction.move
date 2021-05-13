// Checks that only two reconfigurations can be done within the same transaction and will only emit one reconfiguration
// event.

//! account: alice, 1000000, 0, validator
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! block-prologue
//! proposer: bob
//! block-time: 2

//! new-transaction
//! sender: diemroot
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{alice}});
        DiemSystem::remove_validator(account, @{{bob}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: carrol
//! block-time: 3

//! new-transaction
//! sender: diemroot
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{bob}});
    }
}

// check: "Keep(ABORTED { code: 775"
