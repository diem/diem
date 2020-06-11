//! account: validatorvivian, 10000000, 0, validator
//! account: bob, 100000000, 0, unhosted
//! account: alice, 100000000, 0, unhosted

//! new-transaction
//! sender: association
script {
    use 0x1::Testnet;
    // Unset testnet
    fun main(account: &signer) {
        Testnet::remove_testnet(account)
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 10048

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::mint_lbr_to_address(account, {{bob}}, 1);
}
}
// TODO: fix
// chec: ABORTED
// chec: 10047

//! new-transaction
//! sender: association
script {
    use 0x1::Testnet;
    // Reset testnet
    fun main(account: &signer) {
        Testnet::initialize(account)
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::publish_unrestricted_limits(account)
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::certify_limits_definition(account, {{bob}});
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::decertify_limits_definition(account, {{bob}});
    }
}
// check: ABORTED
// check: 1002

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::decertify_limits_definition(account, {{bob}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::unpublish_limits_definition(account);
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x1::AccountLimits;
    fun main() {
        assert(AccountLimits::default_limits_addr() == {{association}}, 0);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    // Since we directly wrote into this account using fake data store, we
    // don't actually know that the balance is greater than 0 in the
    // account limits code, but it is.
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::mint_lbr_to_address(account, {{bob}}, 2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main(account: &signer) {
        AccountLimits::unpublish_limits_definition(account);
        AccountLimits::publish_unrestricted_limits(account);
        AccountLimits::certify_limits_definition(account, {{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
use 0x1::LibraAccount;
fun main(account: &signer) {
    LibraAccount::mint_lbr_to_address(account, {{bob}}, 2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main(account: &signer) {
        AccountLimits::decertify_limits_definition(account, {{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    // Publish our own limits definition for testing!
    fun main(account: &signer) {
        AccountLimits::unpublish_limits_definition(account);
        AccountLimits::publish_limits_definition(
            account,
            100,
            100,
            200,
            40000,
        );
        AccountLimits::certify_limits_definition(account, {{association}});
    }
}
// check: EXECUTED

//! block-prologue
//! proposer: validatorvivian
//! block-time: 40001

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::mint_lbr_to_address(account, {{bob}}, 100);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::mint_lbr_to_address(account, {{bob}}, 1);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 9

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 101);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// chec: ABORTED
// chec: 11

//! new-transaction
//! sender: association
script {
    use 0x1::AccountLimits;
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main(account: &signer) {
        AccountLimits::decertify_limits_definition(account, {{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: bob
script {
    use 0x1::LibraAccount;
    use 0x1::LBR::LBR;
    fun main(account: &signer) {
        let with_cap = LibraAccount::extract_withdraw_capability(account);
        LibraAccount::pay_from<LBR>(&with_cap, {{alice}}, 1);
        LibraAccount::restore_withdraw_capability(with_cap);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::mint_lbr_to_address(account, {{bob}}, 1);
    }
}
// TODO: fix
// chec: ABORTED
// chec: 1
