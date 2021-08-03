//! account: alice, 0, 0, address
//! account: bob, 1000000, 0, validator
//! account: carrol, 1000000, 0, validator

//! new-transaction
//! sender: diemroot
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice"
stdlib_script::AccountCreationScripts::create_validator_operator_account
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer-address: 0x0
//! block-time: 0
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        assert(DiemTimestamp::now_microseconds() == 0, 999);
        // register alice as bob's delegate
        ValidatorConfig::set_operator(account, @{{alice}});
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // set a new config locally
        ValidatorConfig::set_config(account, @{{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
    }
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // update is too soon, will fail
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: "Keep(ABORTED { code: 1544,"

//! block-prologue
//! proposer: bob
//! block-time: 300000000

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // update is too soon, will not trigger the reconfiguration
        assert(DiemTimestamp::now_microseconds() == 300000000, 999);
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: "Keep(ABORTED { code: 1544,"

//! block-prologue
//! proposer: bob
//! block-time: 300000001

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // update is in exactly 5 minutes and 1 microsecond, so will succeed
        assert(DiemTimestamp::now_microseconds() == 300000001, 999);
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 600000000

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // too soon to reconfig, but validator have not changed, should succeed but not reconfigure
        assert(DiemTimestamp::now_microseconds() == 600000000, 999);
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// not: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 600000002

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // good to reconfig
        assert(DiemTimestamp::now_microseconds() == 600000002, 999);
        ValidatorConfig::set_config(account, @{{bob}},
                                    x"d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a",
                                    x"", x"");
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 600000003

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script{
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        DiemSystem::remove_validator(account, @{{bob}});
        assert(!DiemSystem::is_validator(@{{bob}}), 77);
        assert(DiemSystem::is_validator(@{{carrol}}), 78);
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: carrol
//! block-time: 600000004

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: diemroot
script{
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    fun main(account: signer) {
    let account = &account;
        // add validator back
        assert(DiemTimestamp::now_microseconds() == 600000004, 999);
        DiemSystem::add_validator(account, @{{bob}});
        assert(DiemSystem::is_validator(@{{bob}}), 79);
        assert(DiemSystem::is_validator(@{{carrol}}), 80);
    }
}
// check: NewEpochEvent
// check: "Keep(EXECUTED)"

//! block-prologue
//! proposer: bob
//! block-time: 900000004

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // update too soon
        assert(DiemTimestamp::now_microseconds() == 900000004, 999);
        ValidatorConfig::set_config(account, @{{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: "Keep(ABORTED { code: 1544,"

//! block-prologue
//! proposer: bob
//! block-time: 900000005

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
    use DiemFramework::DiemTimestamp;
    use DiemFramework::DiemSystem;
    use DiemFramework::ValidatorConfig;
    fun main(account: signer) {
    let account = &account;
        // good to reconfigure
        assert(DiemTimestamp::now_microseconds() == 900000005, 999);
        ValidatorConfig::set_config(account, @{{bob}},
                                    x"3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c",
                                    x"", x"");
        DiemSystem::update_config_and_reconfigure(account, @{{bob}});
    }
}

// check: NewEpochEvent
// check: "Keep(EXECUTED)"
