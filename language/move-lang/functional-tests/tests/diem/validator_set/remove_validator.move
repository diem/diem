//! account: alice
//! account: vivian, 1000000, 0, validator
//! account: v1, 1000000, 0, validator
//! account: v2, 1000000, 0, validator
//! account: v3, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: diemroot
// remove_validator cannot be called on a non-validator
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{alice}});
    }
}

// check: "Keep(ABORTED { code: 775,"

// remove_validator can only be called by the Association
//! new-transaction
//! sender: alice
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{vivian}});
    }
}

// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: diemroot
// should work because Vivian is a validator
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{vivian}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
// double-removing Vivian should fail
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{vivian}});
    }
}

// check: "Keep(ABORTED { code: 775,"
