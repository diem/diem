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
    LibraAccount::mint_lbr_to_address(account, {{blessed}}, 1);
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

// ----- Blessed updates max_inflow for unhosted wallet

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let new_max_total_flow = 2;
        AccountLimits::update_limits_definition(&tc_capability, new_max_total_flow, 0);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}

// check: EXECUTED

// ------ try and mint to unhosted bob, but inflow is higher than total flow

//! new-transaction
//! sender: blessed
script {
    use 0x1::LibraAccount;
    fun main(account: &signer) {
        LibraAccount::mint_lbr_to_address(account, {{bob}}, 3);
    }
    }

// TODO fix (should ABORT) - update unhosted //! account directive, and flow/balance updates for accounts
// check: EXECUTED


// --- increase limits limits

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(tc_account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(tc_account);
        let new_max_total_flow = 1000;
        AccountLimits::update_limits_definition(&tc_capability, new_max_total_flow, 1000);
        Roles::restore_capability_to_privilege(tc_account, tc_capability);
    }
}

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::certify_limits_definition(&tc_capability, {{bob}});
        Roles::restore_capability_to_privilege(account, tc_capability);
    }
}
// check: EXECUTED

//! new-transaction
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::decertify_limits_definition(&tc_capability, {{bob}});
        Roles::restore_capability_to_privilege(account, tc_capability);
    }
}
// check: ABORTED

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::decertify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    fun main(account: &signer) {
        AccountLimits::unpublish_limits_definition(account);
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
    LibraAccount::mint_lbr_to_address(account, {{blessed}}, 2);
}
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
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
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    // Publish our own limits definition for testing! Make sure we are
    // exercising the unrestricted limits check.
    fun main(account: &signer) {
        AccountLimits::publish_unrestricted_limits(account);
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::certify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
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
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::decertify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
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
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    // Publish our own limits definition for testing!
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::unpublish_limits_definition(account);
        AccountLimits::publish_limits_definition(
            account,
            100,
            200,
            40000,
        );
        AccountLimits::certify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
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
// check: 9

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
// check: 11

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::decertify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
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

//! new-transaction
//! sender: blessed
script {
    use 0x1::AccountLimits;
    use 0x1::Roles::{Self, TreasuryComplianceRole};
    fun main(account: &signer) {
        let tc_capability = Roles::extract_privilege_to_capability<TreasuryComplianceRole>(account);
        AccountLimits::update_limits_definition(
            &tc_capability,
            100,
            200,
        );
        AccountLimits::certify_limits_definition(&tc_capability, {{blessed}});
        Roles::restore_capability_to_privilege(account, tc_capability);
    }
}
// check: EXECUTED
