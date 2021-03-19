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
//! sender: diemroot
script{
    use 0x1::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, {{vivian}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: viola
//! block-time: 4

//! new-transaction
// check that Vivian is no longer a validator, Alice is not, but Viola is still a
// validator
script{
    use 0x1::DiemSystem;
    fun main() {
        assert(!DiemSystem::is_validator({{vivian}}), 70);
        assert(!DiemSystem::is_validator({{alice}}), 71);
        assert(DiemSystem::is_validator({{viola}}), 72);
    }
}

// check: "Keep(EXECUTED)"
